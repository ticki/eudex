#![feature(static_recursion)]

//! Eudex is a Soundex-esque phonetic reduction/hashing algorithm, providing locality sensitive
//! "hashes" of words, based on the spelling and pronunciation.

#![cfg_attr(test, feature(test))]
#[cfg(test)]
extern crate test;

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

/// Phonetically hash this string.
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
/// Each byte in the string will be mapped to a value from a table, such that similarly sounding
/// characters have many overlapping bits. This way you ensure that strings sounding alike will be
/// mapped nearby each other. Vowels and duplicates will be skipped.
///
/// The first byte, however, is skipped and left in the most significant place, making it most
/// influential to the hash.
///
/// Case has no effect.
pub fn hash(string: &str) -> u64 {
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
        if entry <= b'z' {
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

    res | (first_byte << 56)
}

/// Calculate the Eudex distance between two words.
///
/// This metric will only measure the bitwise distance between two strings, which is a XOR-like
/// metric, making it unfit for certain purposes.  Generally speaking, the lower this is, the more
/// similar are the two words, although each byte carries different weight (the first one carries
/// the most, and the following's weights are simply half the weight of the previous byte).
///
/// # Example
///
/// ```rust
/// let distance = eudex::distance("write", "right").count_ones();
/// // Hamming weight of the Eudex distance gives a "smoother" word metric.
///
/// assert_eq!(distance, 15);
/// ```
pub fn distance(a: &str, b: &str) -> u64 {
    hash(a) ^ hash(b)
}

/// Check if two sentences sound "similar".
pub fn similar(a: &str, b: &str) -> bool {
    let dist = distance(a, b);

    (dist as u8).count_ones() as u32
        + ((dist >> 8 ) as u8).count_ones() as u32 * 2
        + ((dist >> 16) as u8).count_ones() as u32 * 4
        + ((dist >> 24) as u8).count_ones() as u32 * 8
        + ((dist >> 32) as u8).count_ones() as u32 * 16
        + ((dist >> 40) as u8).count_ones() as u32 * 32
        + ((dist >> 48) as u8).count_ones() as u32 * 64
        + ((dist >> 56) as u8).count_ones() as u32 * 128 < 10
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn test_exact() {
        assert_eq!(hash("JAva"), hash("jAva"));
        assert_eq!(hash("co!mputer"), hash("computer"));
        assert_eq!(hash("comp-uter"), hash("computer"));
        assert_eq!(hash("comp@u#te?r"), hash("computer"));
        assert_eq!(hash("java"), hash("jiva"));
        assert_eq!(hash("lal"), hash("lel"));
        assert_eq!(hash("rindom"), hash("ryndom"));
        assert_eq!(hash("riiiindom"), hash("ryyyyyndom"));
        assert_eq!(hash("riyiyiiindom"), hash("ryyyyyndom"));
        assert_eq!(hash("triggered"), hash("TRIGGERED"));
        assert_eq!(hash("repert"), hash("ropert"));
    }

    #[test]
    fn test_mismatch() {
        assert!(hash("reddit") != hash("eddit"));
        assert!(hash("lol") != hash("lulz"));
        assert!(hash("ijava") != hash("java"));
        assert!(hash("jesus") != hash("iesus"));
        assert!(hash("aesus") != hash("iesus"));
        assert!(hash("iesus") != hash("yesus"));
        assert!(hash("rupirt") != hash("ropert"));
        assert!(hash("ripert") != hash("ropyrt"));
        assert!(hash("rrr") != hash("rraaaa"));
        assert!(hash("randomal") != hash("randomai"));
    }

    #[test]
    fn test_distance() {
        assert!(distance("lizzard", "wizzard") > distance("rick", "rolled"));
        assert!(distance("bannana", "panana") >= distance("apple", "abple"));
        //assert!(distance("franco", "sranco") < distance("unicode", "ASCII"));
        assert!(distance("trump", "drumpf") < distance("gangam", "style"));
        assert!(distance("right", "write").count_zeros() > distance("write", "abdominohysterotomy").count_zeros());
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
        //assert!(similar("goth", "god"));
        assert!(similar("maier", "meyer"));
        assert!(similar("möier", "meyer"));
        assert!(similar("fümlaut", "fymlaut"));
        //assert!(similar("ümlaut", "ymlaut"));
        assert!(distance("schmid", "schmidt").count_ones() < 14);

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
            let dict = fs::File::open("/usr/share/dict/american-english").unwrap_or_else(|_| {
                fs::File::open("/usr/share/dict/words").unwrap()
            });
            let mut vec = Vec::new();

            for i in BufReader::new(dict).lines() {
                vec.push(hash(&i.unwrap()));
            }

            vec
        });
    }
}
