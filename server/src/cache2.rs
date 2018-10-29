use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::mem::size_of;
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::thread;
use std::time::Duration;

use errors;
use grep::searcher::Searcher;
use ext::{Extension, Extensions};
use ignore::WalkBuilder;
use result::{ContentItem, FileItem};
use search::MatcherSpec;

// Default hashmap capacity.
const DEFAULT_HASH_MAP_CAPACITY: usize = 64;
// Default thread pool size.
const DEFAULT_THREAD_POOL_SIZE: usize = 4;
// Min size in bytes to enable caching of the file.
const MIN_BYTES_TO_CACHE: u64 = 10_000;
// Number of seconds after which trigger cache refresh.
const CACHE_POLL_INTERVAL_SECS: u64 = 5;

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
// Memory size
///////////////////////////////////////////////////////////

trait MemoryUsed {
  // Returns size in bytes.
  fn memory_used(&self) -> usize;
}

impl MemoryUsed for String {
  fn memory_used(&self) -> usize {
    self.as_bytes().len()
  }
}

impl<T: MemoryUsed> MemoryUsed for Option<T> {
  fn memory_used(&self) -> usize {
    self.as_ref().map(|inner| inner.memory_used()).unwrap_or(0)
  }
}

impl MemoryUsed for FileIndex {
  fn memory_used(&self) -> usize {
    size_of::<FileIndex>() + size_of::<Vec<u8>>() + self.content.len()
  }
}

impl MemoryUsed for FileIndexTree {
  fn memory_used(&self) -> usize {
    size_of::<FileIndexTree>() + self.entries.iter().fold(0, |n, (key, value)| {
      n + key.memory_used() + value.memory_used()
    })
  }
}

impl MemoryUsed for Cache {
  fn memory_used(&self) -> usize {
    size_of::<Cache>() + self.index.iter().fold(0, |n, (key, value)| {
      n + key.memory_used() + value.memory_used()
    })
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileIndexTreeStatistics {
  txid: usize,
  memory_used: usize,
  num_entries: usize,
  indexed_fraction: f32
}

impl FileIndexTreeStatistics {
  // Creates new file index tree statistics.
  pub fn new(
    txid: usize,
    memory_used: usize,
    num_entries: usize,
    indexed_fraction: f32
  ) -> Self {
    Self { txid, memory_used, num_entries, indexed_fraction }
  }

  // Returns txid of the file index tree.
  pub fn txid(&self) -> usize {
    self.txid
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
      if file.metadata()?.len() >= MIN_BYTES_TO_CACHE {
        let mut content = Vec::with_capacity(MIN_BYTES_TO_CACHE as usize);
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
    let fraction = if total == 0 { 0f32 } else { indexed as f32 / total as f32 };
    FileIndexTreeStatistics::new(self.txid(), self.memory_used(), total, fraction)
  }

  // List of entries.
  pub fn entries(&self) -> &HashMap<String, Option<FileIndex>> {
    &self.entries
  }
}

///////////////////////////////////////////////////////////
// Cache
///////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
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

pub type SharedCache = Arc<Mutex<Cache>>;

// Global cache that keeps track of paths and their corresponding index trees.
pub struct Cache {
  // Map of path to index.
  index: HashMap<String, Arc<FileIndexTree>>
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
    self.upsert_index(path, Arc::new(FileIndexTree::new()))
  }

  // Adds new index, or updates existing one.
  // Update is based on index timestamp, we only keep the value with the latest timestamp.
  #[inline]
  fn upsert_index(
    &mut self,
    path: &Path,
    idx: Arc<FileIndexTree>
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
  pub fn get_index(&self, path: &Path) -> Option<Arc<FileIndexTree>> {
    match path.to_str() {
      Some(p) => self.index.get(p).map(|rf| rf.clone()),
      None => None
    }
  }

  // Returns true, if path is cached.
  pub fn contains(&self, path: &Path) -> bool {
    match path.to_str() {
      Some(p) => self.index.contains_key(p),
      None => false
    }
  }

  // Removes index if it exists, otherwise is no-op.
  pub fn remove_index(&mut self, path: &Path) -> Result<(), errors::Error> {
    self.upsert_index(path, Arc::new(FileIndexTree::new()))
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
    let mut stats = Vec::with_capacity(self.index.len());
    let mut iter = self.index.values();
    while let Some(value) = iter.next() {
      stats.push(value.stats());
    }
    CacheStatistics::new(self.memory_used(), stats)
  }
}

// Creates shared cache.
pub fn create_cache() -> SharedCache {
  Arc::new(Mutex::new(Cache::new()))
}

// Returns true if path contains in the cache.
pub fn contains_cache(cache: &SharedCache, path: &Path) -> Result<bool, errors::Error> {
  let cache = cache.lock()?;
  Ok(cache.contains(path))
}

// Returns cache statistics.
pub fn cache_stats(cache: &SharedCache) -> Result<CacheStatistics, errors::Error> {
  let cache = cache.lock()?;
  Ok(cache.stats())
}

// Adds entry to the cache.
pub fn update_cache(cache: &SharedCache, path: &Path) -> Result<(), errors::Error> {
  let mut cache = cache.lock()?;
  cache.add_index(path)
}

// Internal function to start search.
pub fn search(
  cache: &SharedCache,
  searcher: Searcher,
  content_matcher: MatcherSpec,
  path: &Path,
  ext_check: Extensions,
  file_counter: Arc<AtomicUsize>,
  content_counter: Arc<AtomicUsize>,
  fsx: &mpsc::Sender<FileItem>,
  csx: &mpsc::Sender<ContentItem>
) -> Result<(), errors::Error> {
  let index_opt = {
    let arc = cache.lock()?;
    arc.get_index(path)
  };

  if let Some(index) = index_opt {
    // Start search
    let mut iter = index.entries().iter();
    let tp = ThreadPool::new(DEFAULT_THREAD_POOL_SIZE);
    while let Some((path_str, _)) = iter.next() {
      let path_str = path_str.clone();
      tp.execute(move || {
        let path = Path::new(&path_str);
      });
    }
    println!("{:?}", index.stats());
  }
  Ok(())
}

pub fn periodic_refresh(cache: &SharedCache) -> ThreadPool {
  let thread_pool = ThreadPool::new(1);
  let arc = cache.clone();
  thread_pool.execute(move || {
    loop {
      let res = refresh_sync(&arc);
      if let Err(error) = res {
        eprintln!("# ERROR Error during periodic refresh: {}", error);
      }
      thread::sleep(Duration::from_secs(CACHE_POLL_INTERVAL_SECS));
    }
  });
  thread_pool
}

// Refresh cache entries.
pub fn refresh_sync(cache: &SharedCache) -> Result<(), errors::Error> {
  let paths = {
    let cache = cache.lock()?;
    cache.paths()
  };

  let thread_pool = ThreadPool::new(DEFAULT_THREAD_POOL_SIZE);
  for path in paths {
    let arc = cache.clone();
    thread_pool.execute(move || {
      let path_ref = Path::new(&path);
      if let Err(error) = refresh_func(arc, path_ref) {
        eprintln!("# ERROR Error during refresh: {}", error);
      }
    });
  }

  Ok(())
}

// Closure for refreshing cache entries.
fn refresh_func(arc: SharedCache, path: &Path) -> Result<(), errors::Error> {
  // This should match files similar to search module.
  let mut walk = WalkBuilder::new(path)
    .follow_links(false)
    .standard_filters(true)
    .same_file_system(true)
    .build();

  // Cache all extensions.
  let extensions = Extensions::all();

  let mut tree = FileIndexTree::new();
  while let Some(res) = walk.next() {
    if let Ok(entry) = res {
      if entry.path().is_file() {
        let ext = entry.path().extension()
          .and_then(|os| os.to_str())
          .unwrap_or("")
          .parse::<Extension>()
          .unwrap();
        if extensions.is_supported_extension(ext) {
          // build index
          tree.add(entry.path())?;
        }
      }
    }
  }

  // Update index for a path.
  let mut cache = arc.lock()?;
  cache.upsert_index(path, Arc::new(tree))
}
