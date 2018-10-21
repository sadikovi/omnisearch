//
// Murmur3 is successor to Murmur2 fast non-crytographic hash algorithms.
//
// This is a public domain code with no copyrights.
// From homepage of MurmurHash (https://code.google.com/p/smhasher/),
// "All MurmurHash versions are public domain software, and the author disclaims all
// copyright to their code."
//
use std::num::Wrapping;

// Rotate left for a u32 with wrapping semantics.
macro_rules! rotl {
  ($value:expr, $amount:expr) => {
    ($value << $amount) | ($value >> (32 - $amount))
  }
}

// Converts byte slice into u32 (little endian).
macro_rules! read_u32_le {
  ($b0:expr, $b1:expr, $b2:expr, $b3:expr) => {
    Wrapping(($b0 as u32) | ($b1 as u32) << 8 | ($b2 as u32) << 16 | ($b3 as u32) << 24)
  }
}

pub fn murmur3_32(bytes: &[u8], seed: u32) -> u32 {
  let c1 = Wrapping::<u32>(0xCC9E2D51);
  let c2 = Wrapping::<u32>(0x1B873593);
  let r1 = 15;
  let r2 = 13;
  let m = Wrapping::<u32>(5);
  let n = Wrapping::<u32>(0xE6546B64);
  let length = Wrapping(bytes.len() as u32);
  let tail_len = bytes.len() & 0x3;
  let exact_len = bytes.len() - tail_len;

  let mut hash = Wrapping(seed);


  // Iterate four bytes at a time.
  for chunk in bytes[..exact_len].chunks(4) {
    let mut k = read_u32_le!(chunk[0], chunk[1], chunk[2], chunk[3]);
    k = k * c1;
    k = rotl!(k, r1);
    k = k * c2;
    hash = hash ^ k;

    hash = rotl!(hash, r2);
    hash = hash * m + n;
  }

  // Process tail, if required.
  if tail_len > 0 {
    let chunk = &bytes[exact_len..];
    let mut k = match tail_len {
      3 => read_u32_le!(chunk[0], chunk[1], chunk[2], 0),
      2 => read_u32_le!(chunk[0], chunk[1], 0, 0),
      1 => read_u32_le!(chunk[0], 0, 0, 0),
      _ => unreachable!()
    };
    k = k * c1;
    k = rotl!(k, r1);
    k = k * c2;
    hash = hash ^ k;
  }

  // Finalization.
  hash = hash ^ length;
  hash = hash ^ (hash >> 16);
  hash = hash * Wrapping::<u32>(0x85EBCA6B);
  hash = hash ^ (hash >> 13);
  hash = hash * Wrapping::<u32>(0xC2B2AE35);
  hash = hash ^ (hash >> 16);

  hash.0
}

#[cfg(test)]
mod tests {
  // Converts u32 into a byte array (little endian).
  macro_rules! write_u32_le {
    ($num:expr) => {
      [
        ($num & 0xff) as u8,
        (($num >> 8) & 0xff) as u8,
        (($num >> 16) & 0xff) as u8,
        (($num >> 24) & 0xff) as u8
      ]
    }
  }

  use super::*;

  #[test]
  fn test_murmur3() {
    // MurmurHash3 x86 32 0xB0F57EE3
    // MurmurHash3 x86 128 0xB3ECE62A
    // MurmurHash3 x64 128 0x6384BA69
    let mut buffer = Vec::new();
    for i in 0..256 {
      let hash = murmur3_32(&vec![i as u8], 256 - i);
      buffer.extend_from_slice(&write_u32_le!(hash));
    }
    let result = murmur3_32(&buffer[..], 0);
    // assert_eq!(result, 0xB0F57EE3);
  }

  #[test]
  fn test_murmur3_bytes() {
    let hash = murmur3_32("asdfljasdfljasdfljkasdfljasdf".as_bytes(), 123);
    assert_eq!(hash, 4252461677u32);
    let hash = murmur3_32("1234".as_bytes(), 123);
    assert_eq!(hash, 785304072u32);
    let hash = murmur3_32("123412341234123412341234".as_bytes(), 123);
    assert_eq!(hash, 3956594597u32);
  }
}
