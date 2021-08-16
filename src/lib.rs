#![deny(missing_docs)]

//! Eudex is a Soundex-esque phonetic reduction/hashing algorithm, providing locality sensitive
//! "hashes" of words, based on the spelling and pronunciation.

#![cfg_attr(test, feature(test))]
#[cfg(test)]
extern crate test;

use std::ops;

pub mod raw;
#[cfg(test)]
mod tests;

/// A phonetic hash.
///
/// Using the `Sub` implementation of the hashes will give you the difference.
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Hash {
    hash: u64,
}

impl Hash {
    /// Phonetically hash this string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use eudex::Hash;
    ///
    /// println!("{:?}", Hash::new("lulz"));
    /// ```
    #[inline]
    pub fn new(string: &str) -> Hash {
        let string = string.as_bytes();

        let mut b = 0;
        let first_byte = raw::map_first(*string.get(0).unwrap_or(&0)) as u64;

        let mut res = 0;
        let mut n = 1u8;

        loop {
            b += 1;
            // Detect overflows into the first slot.
            if n == 0 || b >= string.len() {
                break;
            }

            if let Some(x) = raw::filter(res as u8, string[b]) {
                res <<= 8;
                res |= x as u64;
                // Bit shifting is slightly faster than addition on certain (especially older)
                // microprocessors.  Is this premature optimization? Yes, yes it is.
                n <<= 1;
            }
        }

        Hash {
            hash: res | (first_byte << 56),
        }
    }
}

/// Get the inner hash value.
impl Into<u64> for Hash {
    #[inline]
    fn into(self) -> u64 {
        self.hash
    }
}

/// Get a `Hash` value from its hash value.
impl From<u64> for Hash {
    #[inline]
    fn from(hash: u64) -> Self {
        Hash { hash }
    }
}

/// Calculate the difference of two hashes.
impl ops::Sub for Hash {
    type Output = Difference;

    #[inline]
    fn sub(self, rhs: Hash) -> Difference {
        Difference {
            xor: self.hash ^ rhs.hash,
        }
    }
}

/// The difference between two words.
#[derive(Copy, Clone)]
pub struct Difference {
    xor: u64,
}

impl Difference {
    /// The "graduated" distance.
    ///
    /// This will assign different weights to each of the bytes Hamming weight and simply add it.
    /// For most use cases, this metric is the preferred one.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use eudex::Hash;
    ///
    /// println!("{}", (Hash::new("lulz") - Hash::new("lol")).dist());
    /// ```
    #[inline]
    pub fn dist(self) -> u32 {
        (self.xor as u8).count_ones() as u32
            + ((self.xor >> 8) as u8).count_ones() as u32 * 2
            + ((self.xor >> 16) as u8).count_ones() as u32 * 3
            + ((self.xor >> 24) as u8).count_ones() as u32 * 5
            + ((self.xor >> 32) as u8).count_ones() as u32 * 8
            + ((self.xor >> 40) as u8).count_ones() as u32 * 13
            + ((self.xor >> 48) as u8).count_ones() as u32 * 21
            + ((self.xor >> 56) as u8).count_ones() as u32 * 34
    }

    /// The XOR distance.
    ///
    /// This is generally not recommend unless you have a very specific reason to prefer it over
    /// the other methods provided.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use eudex::Hash;
    ///
    /// println!("{}", (Hash::new("hello") - Hash::new("hellou")).xor())
    /// ```
    #[inline]
    pub fn xor(self) -> u64 {
        self.xor
    }

    /// The "flat" Hamming based distance.
    ///
    /// This will let every byte carry the same weight, such that mismatch in the early and later
    /// mismatch counts the same.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use eudex::Hash;
    ///
    /// println!("{}", (Hash::new("hello") - Hash::new("hellou")).hamming())
    /// ```
    #[inline]
    pub fn hamming(self) -> u32 {
        self.xor.count_ones()
    }

    /// Does this difference constitute similarity?
    ///
    /// # Examples
    ///
    /// ```rust
    /// use eudex::Hash;
    ///
    /// assert!((Hash::new("hello") - Hash::new("hellou")).similar())
    /// ```
    #[inline]
    pub fn similar(self) -> bool {
        self.dist() < 15
    }
}

/// Deprecated, do not use.
#[deprecated]
pub fn similar(a: &str, b: &str) -> bool {
    (Hash::new(a) - Hash::new(b)).similar()
}
