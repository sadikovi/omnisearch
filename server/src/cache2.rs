use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::mem::size_of_val;
use std::path::Path;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::thread;

use errors;
use ignore::WalkBuilder;

// Default hashmap capacity.
const DEFAULT_HASH_MAP_CAPACITY: usize = 64;
// Default thread pool size.
const DEFAULT_THREAD_POOL_SIZE: usize = 4;
// Min size in bytes to enable caching of the file.
const MIN_SIZE_TO_CACHE: u64 = 20_000;

// Global txid sequence
static GLOBAL_INDEX_SEQ: AtomicUsize = ATOMIC_USIZE_INIT;


///////////////////////////////////////////////////////////
// Thread pool
///////////////////////////////////////////////////////////

pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: mpsc::Sender<Message>
}

impl ThreadPool {
  /// Create a new ThreadPool.
  ///
  /// The size is the number of threads in the pool.
  ///
  /// # Panics
  ///
  /// The `new` function will panic if the size is zero.
  pub fn new(size: usize) -> ThreadPool {
    assert!(size > 0);
    let (sender, receiver) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));

    let mut workers = Vec::with_capacity(size);

    for id in 0..size {
      workers.push(Worker::new(id, receiver.clone()));
    }

    ThreadPool { workers, sender }
  }

  pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
    let job = Box::new(f);
    self.sender.send(Message::NewJob(job)).unwrap();
  }
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    for _ in &mut self.workers {
        self.sender.send(Message::Terminate).unwrap();
    }

    for worker in &mut self.workers {
      if let Some(thread) = worker.thread.take() {
        thread.join().unwrap();
      }
    }
  }
}

trait FnBox {
  fn call_box(self: Box<Self>);
}

impl <F: FnOnce()> FnBox for F {
  fn call_box(self: Box<F>) {
    (*self)();
  }
}

type Job = Box<FnBox + Send + 'static>;

enum Message {
  NewJob(Job),
  Terminate,
}

#[allow(dead_code)]
struct Worker {
  id: usize,
  thread: Option<thread::JoinHandle<()>>
}

impl Worker {
  fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
    let thread = thread::spawn(move || {
      loop {
        let message = receiver.lock().unwrap().recv().unwrap();
        match message {
          Message::NewJob(job) => {
            job.call_box();
          },
          Message::Terminate => {
            break;
          },
        }
      }
    });
    Worker { id, thread: Some(thread) }
  }
}

///////////////////////////////////////////////////////////
// File Index
///////////////////////////////////////////////////////////

// Simple struct to keep the content of the file in memory.
pub struct FileIndex {
  content: Vec<u8>
}

impl FileIndex {
  // Creates new file index from content.
  pub fn new(content: Vec<u8>) -> Self {
    Self { content }
  }

  // Returns content as slice of bytes.
  pub fn content(&self) -> &[u8] {
    self.content.as_slice()
  }
}

///////////////////////////////////////////////////////////
// File Index Tree
///////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FileIndexTreeStatistics {
  memory_used: usize,
  num_entries: usize,
  indexed_fraction: f32
}

impl FileIndexTreeStatistics {
  // Creates new file index tree statistics.
  pub fn new(memory_used: usize, num_entries: usize, indexed_fraction: f32) -> Self {
    Self { memory_used, num_entries, indexed_fraction }
  }

  // Memory used by file index tree.
  pub fn memory_used(&self) -> usize {
    self.memory_used
  }

  // Number of entries in the file index tree.
  pub fn num_entries(&self) -> usize {
    self.num_entries
  }

  // The fraction of entries that have file index.
  pub fn indexed_fraction(&self) -> f32 {
    self.indexed_fraction
  }
}

// In-memory append-only index of the project.
// Keeps track of the list of files for the project and their corresponding file index,
// if available.
pub struct FileIndexTree {
  txid: usize,
  entries: HashMap<String, Option<FileIndex>>
}

impl FileIndexTree {
  // Creates new index tree.
  pub fn new() -> Self {
    Self {
      txid: GLOBAL_INDEX_SEQ.fetch_add(1, Ordering::SeqCst),
      entries: HashMap::with_capacity(DEFAULT_HASH_MAP_CAPACITY)
    }
  }

  // Returns true, if tree is empty.
  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  // Global monotonically inreasing index id.
  pub fn txid(&self) -> usize {
    self.txid
  }

  // Adds path to the file index.
  pub fn add(&mut self, path: &Path) -> Result<(), errors::Error> {
    if let Some(p) = path.to_str() {
      let mut file = File::open(path)?;
      if file.metadata()?.len() >= MIN_SIZE_TO_CACHE {
        let mut content = Vec::with_capacity(MIN_SIZE_TO_CACHE as usize);
        file.read_to_end(&mut content)?;
        self.entries.insert(p.to_owned(), Some(FileIndex::new(content)));
      } else {
        self.entries.insert(p.to_owned(), None);
      }
      Ok(())
    } else {
      err!("Failed to convert path {:?}", path)
    }
  }

  // Returns statistics for file index tree.
  pub fn stats(&self) -> FileIndexTreeStatistics {
    let indexed = self.entries.values().filter(|entry| entry.is_some()).count();
    let total = self.entries.len();
    let fraction = indexed as f32 / total as f32;
    FileIndexTreeStatistics::new(size_of_val(self), total, fraction)
  }

  // List of entries.
  pub fn entries(&self) -> &HashMap<String, Option<FileIndex>> {
    &self.entries
  }
}

///////////////////////////////////////////////////////////
// Cache
///////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CacheStatistics {
  memory_used: usize,
  trees: Vec<FileIndexTreeStatistics>
}

impl CacheStatistics {
  // Creates new cache statistics.
  pub fn new(memory_used: usize, trees: Vec<FileIndexTreeStatistics>) -> Self {
    Self { memory_used, trees }
  }

  // Returns total memory used by this cache.
  pub fn memory_used(&self) -> usize {
    self.memory_used
  }

  // Returns list of file index tree statistics for the cache.
  pub fn trees(&self) -> &[FileIndexTreeStatistics] {
    self.trees.as_ref()
  }
}

type SharedCache = Arc<Mutex<Cache>>;

// Global cache that keeps track of paths and their corresponding index trees.
pub struct Cache {
  // Map of path to index.
  index: HashMap<String, FileIndexTree>
}

impl Cache {
  // Creates new instance of cache.
  pub fn new() -> Self {
    Self {
      index: HashMap::with_capacity(DEFAULT_HASH_MAP_CAPACITY)
    }
  }

  // Adds index to the cache with deferred execution.
  // This does not block the thread to build an index.
  pub fn add_index(&mut self, path: &Path) -> Result<(), errors::Error> {
    self.upsert_index(path, FileIndexTree::new())
  }

  // Adds new index, or updates existing one.
  // Update is based on index timestamp, we only keep the value with the latest timestamp.
  #[inline]
  fn upsert_index(
    &mut self,
    path: &Path,
    idx: FileIndexTree
  ) -> Result<(), errors::Error> {
    if let Some(p) = path.to_str() {
      let txid = self.index.get(p).map(|prev| prev.txid());
      match txid {
        Some(txid) if txid >= idx.txid() => { },
        _ => { self.index.insert(p.to_owned(), idx); }
      }
      Ok(())
    } else {
      err!("Failed to convert path {:?}", path)
    }
  }

  // Returns optional index if available, otherwise None.
  pub fn get_index(&self, path: &Path) -> Option<&FileIndexTree> {
    match path.to_str() {
      Some(p) => self.index.get(p),
      None => None
    }
  }

  // Removes index if it exists, otherwise is no-op.
  pub fn remove_index(&mut self, path: &Path) -> Result<(), errors::Error> {
    self.upsert_index(path, FileIndexTree::new())
  }

  // Returns the amount of memory used by cache.
  pub fn memory_used(&self) -> usize {
    size_of_val(self)
  }

  // Returns list of paths in the cache.
  pub fn paths(&self) -> Vec<String> {
    let mut paths = Vec::with_capacity(self.index.len());
    let mut iter = self.index.keys();
    while let Some(key) = iter.next() {
      paths.push(key.clone());
    }
    paths
  }

  // Returns statistics of the cache
  pub fn stats(&self) -> CacheStatistics {
    let mut fits = Vec::with_capacity(self.index.len());
    let mut iter = self.index.values();
    while let Some(value) = iter.next() {
      fits.push(value.stats());
    }
    CacheStatistics::new(size_of_val(self), fits)
  }
}

// Refresh cache entries.
pub fn refresh(cache: SharedCache) -> Result<(), errors::Error> {
  let paths = {
    let cache = cache.lock()?;
    cache.paths()
  };

  let thread_pool = ThreadPool::new(DEFAULT_THREAD_POOL_SIZE);
  for path in paths {
    let arc = cache.clone();
    thread_pool.execute(move || {
      // This should match files similar to search module.
      let path = Path::new(&path);
      let mut walk = WalkBuilder::new(path)
        .follow_links(false)
        .standard_filters(true)
        .same_file_system(true)
        .build();

      let mut tree = FileIndexTree::new();
      while let Some(res) = walk.next() {
        if let Ok(entry) = res {
          if entry.path().is_file() {
            // build index
            tree.add(entry.path()).unwrap();
          }
        }
      }

      // Update index for a path.
      let mut cache = arc.lock().unwrap();
      let _ = cache.upsert_index(path, tree);
    });
  }

  Ok(())
}
