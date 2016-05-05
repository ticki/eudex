#![cfg_attr(test, feature(test))]
#[cfg(test)]
extern crate test;

/// The sound table.
///
/// The first bit each describes a certain property of the phone:
///
/// | Position | Modifier | Property    | Phones   |
/// |----------|----------|-------------|----------|
/// | 1        | 1        | Nasal       | mn       |
/// | 2        | 2        | Plosive     | pbtdcgqk |
/// | 3        | 4        | Fricative   | fvsjxzhc |
/// | 4        | 8        | Approximant | vrhjwc   |
/// | 5        | 16       | Trill       | r        |
/// | 6        | 32       | Lateral     | l        |
/// | 7        | 64       | Type²       | mpbfv    |
/// | 8        | 128      | Confident¹  | lrxzq    |
///
/// ¹hard to misspell.
/// ²1 means labial or dorsal, 0 means apical.
const MAP: [u64; 26] = [
    0, // a
    0b01000010, // b
    0b00001110, // c
    0b00000010, // d
    0, // e
    0b01000100, // f
    0b00000010, // g
    0b00001100, // h
    0, // i
    0b00001100, // j
    0b00000010, // k
    0b10100000, // l
    0b01000001, // m
    0b00000001, // n
    0, // o
    0b01000010, // p
    0b10000010, // q
    0b10011000, // r
    0b00000100, // s
    0b00000010, // t
    0, // u
    0b01001100, // v
    0b00001000, // w
    0b10000100, // x
    0, // y
    0b10000100, // z
];

/// Phonetically, hash this string.
///
/// This hashing function is based upon a phonetic reduction algorithm, and is locality sensitive.
///
/// This will map the string a Soundex-esque value, although to similar sounding strings will not
/// necessarily be mapped to the same, but will map to nearby values (nearby in this case means the
/// XOR having a low Hamming weight).
///
/// This is two orders of magnitude faster than Soundex, and several orders of magnitude faster
/// than Levenshtein distance, making it feasible to run on large sets of strings in very short
/// time.
///
/// It is optimized for European languages as well.
///
/// A word containing too many non-duplicate consonants will overflow and possibly cause
/// unspecified behavior, although observations shows that it affect the result relatively little.
///
/// Each byte in the string will be mapped to a value from a table, such that similarly sounding
/// characters have many overlapping bits. This way you ensure that strings sounding alike will be
/// mapped nearby each other. Vowels and duplicates will be skipped.
///
/// The first byte, however, is skipped and left in the most significant place, making it most
/// influential to the hash.
///
/// Case has no effect.
pub fn hash(s: &str) -> u64 {
    let mut bytes = s.bytes();
    let first_byte = bytes.next().map_or(0, |x| (x | 32) as u64);
    let mut res = 0;

    for i in bytes {
        if let Some(&x) =  MAP.get(((i | 32).wrapping_sub(b'a')) as usize) {
            if x != 0 && res & 255 != x {
                res <<= 8;
                res |= x;
            }
        }
    }

    res | (first_byte << 56)
}

/// Calculate the Eudex distance between two words.
///
/// This metric will only measure the bitwise distance between two strings, which is a XOR-like
/// metric, making it unfit for certain purposes.  Generally speaking, the lower this is, the more
/// similar are the two words, although each byte carries different weight (the first one carries
/// the most, and the following's weights are simply half the weight of the previous byte).
pub fn distance(a: &str, b: &str) -> u64 {
    hash(a) ^ hash(b)
}

/// Check if two sentences sound "similar".
pub fn similar(a: &str, b: &str) -> bool {
    distance(a, b) < 670000
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_exact() {
        assert_eq!(hash("lal"), hash("lel"));
        assert_eq!(hash("rupert"), hash("ropert"));
        assert_eq!(hash("rrr"), hash("rraaaa"));
        assert_eq!(hash("random"), hash("rondom"));
        assert_eq!(hash("java"), hash("jiva"));
        assert_eq!(hash("JAva"), hash("jAva"));
        assert_eq!(hash("triggered"), hash("TRIGGERED"));
        assert_eq!(hash("comp-uter"), hash("computer"));
        assert_eq!(hash("comp@u#te?r"), hash("computer"));
        assert_eq!(hash("c0mp^tər"), hash("computer"));
    }

    #[test]
    fn test_mismatch() {
        assert!(hash("reddit") != hash("eddit"));
        assert!(hash("lol") != hash("lulz"));
        assert!(hash("ijava") != hash("java"));
        assert!(hash("jesus") != hash("iesus"));
    }

    #[test]
    fn test_distance() {
        assert!(distance("lizzard", "wizzard") > distance("rick", "rolled"));
        assert!(distance("bannana", "panana") > distance("apple", "abple"));
        //assert!(distance("franco", "sranco") < distance("unicode", "ASCII"));
        assert!(distance("trump", "drumpf") < distance("gangam", "style"));
    }

    #[test]
    fn test_reflexivity() {
        assert_eq!(distance("a", "b"), distance("b", "a"));
        assert_eq!(distance("youtube", "facebook"), distance("facebook", "youtube"));
        assert_eq!(distance("Rust", "Go"), distance("Go", "Rust"));
        assert_eq!(distance("rick", "rolled"), distance("rolled", "rick"));
    }

    #[test]
    fn test_similar() {
        // Similar.
        assert!(similar("yay", "yuy"));
        assert!(distance("crack", "crakk").count_ones() < 10);
        assert!(similar("what", "wat"));
        assert!(similar("jesus", "jeuses"));
        assert!(similar("", ""));
        assert!(similar("jumpo", "jumbo"));
        assert!(similar("lol", "lulz"));
        assert!(similar("goth", "god"));

        // Not similar.
        assert!(!similar("youtube", "reddit"));
        assert!(!similar("yet", "vet"));
        assert!(!similar("hacker", "4chan"));
        assert!(!similar("awesome", "me"));
        assert!(!similar("prisco", "vkisco"));
        assert!(!similar("no", "go"));
        assert!(!similar("horse", "norse"));
        assert!(!similar("nice", "mice"));
    }

    #[bench]
    fn bench_dict(b: &mut Bencher) {
        use std::fs;
        use std::io::{BufRead, BufReader};

        b.iter(|| {
            let dict = fs::File::open("/usr/share/dict/american-english").unwrap();
            let mut vec = Vec::new();

            for i in BufReader::new(dict).lines() {
                vec.push(hash(&i.unwrap()));
            }

            vec
        });
    }
}
