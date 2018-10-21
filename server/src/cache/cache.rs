use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use cache::bloom::BloomFilter;
use errors::Error;

const MAX_BYTES: u64 = 1_000_000;

// Structure to store file cache.
pub struct FileCache {
  filter: BloomFilter
}

impl FileCache {
  pub fn build(path: &Path) -> Result<Self, Error> {
    let file = File::open(path)?;
    let total_bytes = file.metadata()?.len();
    let mut buf = BufReader::new(file);
    let mut line = String::with_capacity(1024);

    let num_elements = if total_bytes > 0 { MAX_BYTES } else { total_bytes };
    let mut filter = BloomFilter::new(num_elements as usize, 3);

    while buf.read_line(&mut line)? > 0 {
      index_substrings(&line, &mut filter);
      line.clear();
    }

    Ok(Self { filter })
  }

  // Returns true, if substring could exist in the cache.
  // If false is returned, then substring is definitely not in cache,
  // otherwise, the file content has to be checked to determine the final result.
  //
  // Empty pattern always returns false.
  pub fn exists(&self, pattern: &str) -> bool {
    self.filter.is_member(pattern.as_bytes())
  }

  // Returns size in bytes for this file cache.
  pub fn size(&self) -> u64 {
    self.filter.size()
  }

  // Number of items we have inserted into the cache.
  pub fn cardinality(&self) -> usize {
    self.filter.cardinality()
  }

  // Returns FPP metric from underlying bloom filter.
  pub fn fpp(&self) -> f32 {
    self.filter.false_positive()
  }
}

// Generates all substrings from the line and adds to bloom filter.
#[inline]
fn index_substrings(line: &str, filter: &mut BloomFilter) {
  for i in 0..line.len() - 1 {
    for j in i + 1..line.len() {
      filter.insert(&line[i..j].as_bytes());
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_file_cache() {
    let path = Path::new("/Users/sadikovi/developer/hadoop/hadoop-tools/hadoop-aws/src/main/java/org/apache/hadoop/fs/s3a/S3AFileSystem.java");
    let cache = FileCache::build(path).unwrap();
    println!("size: {}", cache.size());
    println!("fpp: {}", cache.fpp());
    println!("cardinality: {}", cache.cardinality());
  }
}
