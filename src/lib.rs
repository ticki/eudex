#![deny(missing_docs)]

//! Eudex is a Soundex-esque phonetic reduction/hashing algorithm, providing locality sensitive
//! "hashes" of words, based on the spelling and pronunciation.

#![cfg_attr(test, feature(test))]
#[cfg(test)]
extern crate test;

use std::ops;

/// The sound table.
///
/// The first bit each describes a certain property of the phone:
///
/// | Position | Modifier | Property     | Phones                   |
/// |----------|---------:|--------------|:------------------------:|
/// | 1        | 1        | Discriminant | (for tagging duplicates) |
/// | 2        | 2        | Nasal        | mn                       |
/// | 3        | 4        | Fricative    | fvsjxzhct                |
/// | 4        | 8        | Plosive      | pbtdcgqk                 |
/// | 5        | 16       | Dental       | tdnzs                    |
/// | 6        | 32       | Liquid       | lr                       |
/// | 7        | 64       | Labial       | bfpv                     |
/// | 8        | 128      | Confident¹   | lrxzq                    |
///
/// ¹hard to misspell.
///
/// Vowels are, to maxize the XOR distance, represented by 0 and 1 (open and close, respectively).
const PHONES: [u64; LETTERS as usize] = [
    0, // a
//    +--------- Confident
//    |+-------- Labial
//    ||+------- Liquid
//    |||+------ Dental
//    ||||+----- Plosive
//    |||||+---- Fricative
//    ||||||+--- Nasal
//    |||||||+-- Discriminant
//    ||||||||
    0b01001000, // b
    0b00001100, // c
    0b00011000, // d
    0, // e
    0b01000100, // f
    0b00001000, // g
    0b00000100, // h
    1, // i
    0b00000101, // j
    0b00001001, // k
    0b10100000, // l
    0b00000010, // m
    0b00010010, // n
    0, // o
    0b01001001, // p
    0b10101000, // q
    0b10100001, // r
    0b00010100, // s
    0b00011101, // t
    1, // u
    0b01000101, // v
    0b00000000, // w
    0b10000100, // x
    1, // y
    0b10010100, // z
];

// Non ASCII-phones.
//
// Starts 0xDF (ß). These are all aproixmated sounds, since they can vary a lot between languages.
const PHONES_C1: [u64; LETTERS_C1 as usize] = [
    PHONES[(b's' - b'a') as usize] ^ 1, // ß
    0, // à
    0, // á
    0, // â
    0, // ã
    0, // ä [æ]
    1, // å [oː]
    0, // æ [æ]
    PHONES[(b'z' - b'a') as usize] ^ 1, // ç [t͡ʃ]
    1, // è
    1, // é
    1, // ê
    1, // ë
    1, // ì
    1, // í
    1, // î
    1, // ï
    0b00010101, // ð [ð̠] (represented as a non-plosive T)
    0b00010111, // ñ [nj] (represented as a combination of n and j)
    0, // ò
    0, // ó
    0, // ô
    0, // õ
    1, // ö [ø]
    !0, // ÷
    1, // ø [ø]
    1, // ù
    1, // ú
    1, // û
    1, // ü
    1, // ý
    0b00010101, // þ [ð̠] (represented as a non-plosive T)
    1, // ÿ
];

/// An _injective_ phone table.
///
/// The table is derived the following way:
///
/// | Position | Modifier | Property (vowel)    | Property (consonant)                              |
/// |----------|---------:|---------------------|---------------------------------------------------|
/// | 1        | 1        | Discriminant        | (property 2 from the phone table) or discriminant |
/// | 2        | 2        | Is it open-mid?     | (property 3 from the phone table)                 |
/// | 3        | 4        | Is it central?      | (property 4 from the phone table)                 |
/// | 4        | 8        | Is it close-mid?    | (property 5 from the phone table)                 |
/// | 5        | 16       | Is it front?        | (property 6 from the phone table)                 |
/// | 6        | 32       | Is it close?        | (property 7 from the phone table)                 |
/// | 7        | 64       | More close than [ɜ] | (property 8 from the phone table)                 |
/// | 8        | 128      | Vowel?                                                                  |
///
/// If it is a consonant, the rest of the bits are simply a right truncated version of the
/// [`PHONES`](./const.PHONES.html) table, with the LSD used as discriminant.
const INJECTIVE_PHONES: [u64; LETTERS as usize] = [
//    +--------- Vowel
//    |+-------- Closer than ɜ
//    ||+------- Close
//    |||+------ Front
//    ||||+----- Close-mid
//    |||||+---- Central
//    ||||||+--- Open-mid
//    |||||||+-- Discriminant
//    ||||||||   (*=vowel)
    0b10000100, // a*
    0b00100100, // b
    0b00000110, // c
    0b00001100, // d
    0b11011000, // e*
    0b00100010, // f
    0b00000100, // g
    0b00000010, // h
    0b11111000, // i*
    0b00000011, // j
    0b00000101, // k
    0b01010000, // l
    0b00000001, // m
    0b00001001, // n
    0b10010100, // o*
    0b00100101, // p
    0b01010100, // q
    0b01010001, // r
    0b00001010, // s
    0b00001110, // t
    0b11100000, // u*
    0b00100011, // v
    0b00000000, // w
    0b01000010, // x
    0b11100100, // y*
    0b01001010, // z
];

/// Non-ASCII injective phone table.
///
/// Starting at C1.
const INJECTIVE_PHONES_C1: [u64; LETTERS_C1 as usize] = [
    INJECTIVE_PHONES[(b's' - b'a') as usize] ^ 1, // ß
    INJECTIVE_PHONES[(b'a' - b'a') as usize] ^ 1, // à
    INJECTIVE_PHONES[(b'a' - b'a') as usize] ^ 1, // á
//    +--------- Vowel
//    |+-------- Closer than ɜ
//    ||+------- Close
//    |||+------ Front
//    ||||+----- Close-mid
//    |||||+---- Central
//    ||||||+--- Open-mid
//    |||||||+-- Discriminant
//    ||||||||
    0b10000000, // â
    0b10000110, // ã
    0b10100110, // ä [æ]
    0b11000010, // å [oː]
    0b10100111, // æ [æ]
    0b01010100, // ç [t͡ʃ]
    INJECTIVE_PHONES[(b'e' - b'a') as usize] ^ 1, // è
    INJECTIVE_PHONES[(b'e' - b'a') as usize] ^ 1, // é
    INJECTIVE_PHONES[(b'e' - b'a') as usize] ^ 1, // ê
    0b11000110, // ë [ə] or [œ]
    INJECTIVE_PHONES[(b'i' - b'a') as usize] ^ 1, // ì
    INJECTIVE_PHONES[(b'i' - b'a') as usize] ^ 1, // í
    INJECTIVE_PHONES[(b'i' - b'a') as usize] ^ 1, // î
    INJECTIVE_PHONES[(b'i' - b'a') as usize] ^ 1, // ï
    0b00001011, // ð [ð̠] (represented as a non-plosive T)
    0b00001011, // ñ [nj] (represented as a combination of n and j)
    INJECTIVE_PHONES[(b'o' - b'a') as usize] ^ 1, // ò
    INJECTIVE_PHONES[(b'o' - b'a') as usize] ^ 1, // ó
    INJECTIVE_PHONES[(b'o' - b'a') as usize] ^ 1, // ô
    INJECTIVE_PHONES[(b'o' - b'a') as usize] ^ 1, // õ
    0b11011100, // ö [œ] or [ø]
    !0, // ÷
    0b11011101,// ø [œ] or [ø]
    INJECTIVE_PHONES[(b'u' - b'a') as usize] ^ 1, // ù
    INJECTIVE_PHONES[(b'u' - b'a') as usize] ^ 1, // ú
    INJECTIVE_PHONES[(b'u' - b'a') as usize] ^ 1, // û
    INJECTIVE_PHONES[(b'y' - b'a') as usize] ^ 1, // ü
    INJECTIVE_PHONES[(b'y' - b'a') as usize] ^ 1, // ý
    0b00001011, // þ [ð̠] (represented as a non-plosive T)
    INJECTIVE_PHONES[(b'y' - b'a') as usize] ^ 1, // ÿ
];


/// Number of letters in our phone map.
const LETTERS: u8 =  26;
/// Number of letters in our C1 phone map.
const LETTERS_C1: u8 =  33;

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
        let first_byte = {
            let entry = (string.get(0).map_or(0, |&x| x) | 32).wrapping_sub(b'a');
            if entry < LETTERS {
                INJECTIVE_PHONES[entry as usize]
            } else if entry >= 0xDF && entry < 0xFF {
                INJECTIVE_PHONES_C1[(entry - 0xDF) as usize]
            } else {
                0
            }
        };
        let mut res = 0;
        let mut n = 1u8;

        loop {
            b += 1;
            // Detect overflows into the first slot.
            if n == 0 || b >= string.len() {
                break;
            }

            let entry = (string[b] | 32).wrapping_sub(b'a');
            if entry <= 26 {
                let x = if entry < LETTERS {
                    PHONES[entry as usize]
                } else if entry >= 0xDF && entry < 0xFF {
                    PHONES_C1[(entry - 0xDF) as usize]
                } else { continue };

                // Collapse consecutive vowels and similar sounding consonants into one.
                if res & 254 != x & 254 {
                    res <<= 8;
                    res |= x;
                    // Bit shifting is slightly faster than addition on certain (especially older)
                    // microprocessors.  Is this premature optimization? Yes, yes it is.
                    n <<= 1;
                }
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
            + ((self.xor >> 8 ) as u8).count_ones() as u32 * 2
            + ((self.xor >> 16) as u8).count_ones() as u32 * 4
            + ((self.xor >> 24) as u8).count_ones() as u32 * 8
            + ((self.xor >> 32) as u8).count_ones() as u32 * 16
            + ((self.xor >> 40) as u8).count_ones() as u32 * 32
            + ((self.xor >> 48) as u8).count_ones() as u32 * 64
            + ((self.xor >> 56) as u8).count_ones() as u32 * 128
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
        self.dist() < 10
    }
}

/// Deprecated, do not use.
#[deprecated]
pub fn similar(a: &str, b: &str) -> bool {
    (Hash::new(a) - Hash::new(b)).similar()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_exact() {
        assert_eq!(Hash::new("JAva"), Hash::new("jAva"));
        assert_eq!(Hash::new("co!mputer"), Hash::new("computer"));
        assert_eq!(Hash::new("comp-uter"), Hash::new("computer"));
        assert_eq!(Hash::new("comp@u#te?r"), Hash::new("computer"));
        assert_eq!(Hash::new("java"), Hash::new("jiva"));
        assert_eq!(Hash::new("lal"), Hash::new("lel"));
        assert_eq!(Hash::new("rindom"), Hash::new("ryndom"));
        assert_eq!(Hash::new("riiiindom"), Hash::new("ryyyyyndom"));
        assert_eq!(Hash::new("riyiyiiindom"), Hash::new("ryyyyyndom"));
        assert_eq!(Hash::new("triggered"), Hash::new("TRIGGERED"));
        assert_eq!(Hash::new("repert"), Hash::new("ropert"));
    }

    #[test]
    fn test_mismatch() {
        assert!(Hash::new("reddit") != Hash::new("eddit"));
        assert!(Hash::new("lol") != Hash::new("lulz"));
        assert!(Hash::new("ijava") != Hash::new("java"));
        assert!(Hash::new("jesus") != Hash::new("iesus"));
        assert!(Hash::new("aesus") != Hash::new("iesus"));
        assert!(Hash::new("iesus") != Hash::new("yesus"));
        assert!(Hash::new("rupirt") != Hash::new("ropert"));
        assert!(Hash::new("ripert") != Hash::new("ropyrt"));
        assert!(Hash::new("rrr") != Hash::new("rraaaa"));
        assert!(Hash::new("randomal") != Hash::new("randomai"));
    }

    #[test]
    fn test_distance() {
        assert!((Hash::new("lizzard") - Hash::new("wizzard")).dist() > (Hash::new("rick") - Hash::new("rolled")).dist());
        assert!((Hash::new("bannana") - Hash::new("panana")).dist() >= (Hash::new("apple") - Hash::new("abple")).dist());
        //assert!((Hash::new("franco") - Hash::new("sranco")).dist() < (Hash::new("unicode") - Hash::new("ASCII")).dist());
        assert!((Hash::new("trump") - Hash::new("drumpf")).dist() < (Hash::new("gangam") - Hash::new("style")).dist());
    }

    #[test]
    fn test_reflexivity() {
        assert_eq!((Hash::new("a") - Hash::new("b")).dist(), (Hash::new("b") - Hash::new("a")).dist());
        assert_eq!((Hash::new("youtube") - Hash::new("facebook")).dist(), (Hash::new("facebook") - Hash::new("youtube")).dist());
        assert_eq!((Hash::new("Rust") - Hash::new("Go")).dist(), (Hash::new("Go") - Hash::new("Rust")).dist());
        assert_eq!((Hash::new("rick") - Hash::new("rolled")).dist(), (Hash::new("rolled") - Hash::new("rick")).dist());
    }

    #[test]
    fn test_similar() {
        // Similar.
        assert!((Hash::new("yay") - Hash::new("yuy")).similar());
        assert!((Hash::new("crack") - Hash::new("crakk")).dist().count_ones() < 10);
        assert!((Hash::new("what") - Hash::new("wat")).similar());
        assert!((Hash::new("jesus") - Hash::new("jeuses")).similar());
        assert!((Hash::new("") - Hash::new("")).similar());
        assert!((Hash::new("jumpo") - Hash::new("jumbo")).similar());
        assert!((Hash::new("lol") - Hash::new("lulz")).similar());
        //assert!((Hash::new("goth") - Hash::new("god")).similar());
        assert!((Hash::new("maier") - Hash::new("meyer")).similar());
        assert!((Hash::new("möier") - Hash::new("meyer")).similar());
        assert!((Hash::new("fümlaut") - Hash::new("fymlaut")).similar());
        //assert!((Hash::new("ümlaut") - Hash::new("ymlaut")).similar());
        assert!((Hash::new("schmid") - Hash::new("schmidt")).dist().count_ones() < 14);

        // Not similar.
        assert!(!(Hash::new("youtube") - Hash::new("reddit")).similar());
        assert!(!(Hash::new("yet") - Hash::new("vet")).similar());
        assert!(!(Hash::new("hacker") - Hash::new("4chan")).similar());
        assert!(!(Hash::new("awesome") - Hash::new("me")).similar());
        assert!(!(Hash::new("prisco") - Hash::new("vkisco")).similar());
        assert!(!(Hash::new("no") - Hash::new("go")).similar());
        assert!(!(Hash::new("horse") - Hash::new("norse")).similar());
        assert!(!(Hash::new("nice") - Hash::new("mice")).similar());
    }

    #[bench]
    fn bench_dict(b: &mut Bencher) {
        use std::fs;
        use std::io::{BufRead, BufReader};

        b.iter(|| {
            let dict = fs::File::open("/usr/share/dict/american-english").unwrap_or_else(|_| {
                fs::File::open("/usr/share/dict/words").unwrap()
            });
            let mut vec = Vec::new();

            for i in BufReader::new(dict).lines() {
                vec.push(Hash::new(&i.unwrap()));
            }

            vec
        });
    }
}
