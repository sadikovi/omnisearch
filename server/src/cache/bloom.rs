use std;
use cache::murmur3::murmur3_32;

// m: the number of bits in the bit vector
// k: the number of hash functions to use (practically, a seed for the Murmur hash)
// bv: the bit vector which will be checked to determine set membership
// cardinality: the number of elements we've inserted so far
pub struct BloomFilter {
  m: u32,
  k: u32,
  bv: Vec<u8>,
  cardinality: u32
}

impl BloomFilter {
  pub fn new(num_elements: usize, num_hash_functions: u32) -> Self {
    let bv = vec![0u8; num_elements / 8 + 1];
    Self {
      m: num_elements as u32,
      k: num_hash_functions as u32,
      bv: bv,
      cardinality: 0
    }
  }

  pub fn insert(&mut self, item: &[u8]) {
    for x in 0..self.k {
      let hash = murmur3_32(item, x);
      let bloom_index = hash % self.m;
      self.set_bit(bloom_index as usize);
    }
    self.cardinality += 1;
  }

  pub fn is_member(&self, item: &[u8]) -> bool {
    for x in 0..self.k {
      let hash = murmur3_32(item, x);
      // We need to get our hash value from [0, 2^32 - 1] to [0, m - 1].
      // This is slightly biased because we're constraining the range with a modulus,
      // but this isn't an issue in practice.
      let bloom_index = hash % self.m;

      if !self.is_bit_set(bloom_index as usize) {
        return false;
      }
    }
    true
  }

  // Return an f32 representing the probability that a false positive result will be
  // returned for a random key.
  pub fn false_positive(&self) -> f32 {
    let n = self.cardinality as f32;
    let m = self.m as f32;
    let k = self.k as f32;
    (1.0 - std::f32::consts::E.powf(-k*n/m)).powf(k)
  }

  #[inline]
  fn set_bit(&mut self, idx: usize) {
    let byte_idx = idx / 8;
    let place = idx % 8;
    self.bv[byte_idx] |= 1u8 << place;
  }

  #[inline]
  fn is_bit_set(&self, idx: usize) -> bool {
    let byte_idx = idx / 8;
    let place = idx % 8;
    self.bv[byte_idx] & (1u8 << place) != 0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_bloom_filter() {
    let mut filter = BloomFilter::new(128, 3);
    filter.insert("abc".as_bytes());
    assert!(filter.is_member("abc".as_bytes()));
    assert!(!filter.is_member("def".as_bytes()));
  }

  #[test]
  fn test_bloom_filter_short() {
    let mut filter = BloomFilter::new(3, 3);
    filter.insert("1234567890".as_bytes());
    assert!(filter.is_member("1234567890".as_bytes()));
    assert!(!filter.is_member("abcdefg".as_bytes()));
  }
}
