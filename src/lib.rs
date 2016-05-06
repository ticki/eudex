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
const PHONES_C1: [u64; 33] = [
    PHONES[(b's' - b'a') as usize] ^ 1, // ß
    PHONES[(b'a' - b'a') as usize], // à
    PHONES[(b'a' - b'a') as usize], // á
    PHONES[(b'a' - b'a') as usize], // â
    PHONES[(b'a' - b'a') as usize], // ã
    PHONES[(b'e' - b'a') as usize], // ä [æ]
    PHONES[(b'o' - b'a') as usize], // å [oː]
    PHONES[(b'e' - b'a') as usize], // æ [æ]
    PHONES[(b'z' - b'a') as usize] ^ 1, // ç [t͡ʃ]
    PHONES[(b'e' - b'a') as usize], // è
    PHONES[(b'e' - b'a') as usize], // é
    PHONES[(b'e' - b'a') as usize], // ê
    PHONES[(b'e' - b'a') as usize], // ë
    PHONES[(b'i' - b'a') as usize], // ì
    PHONES[(b'i' - b'a') as usize], // í
    PHONES[(b'i' - b'a') as usize], // î
    PHONES[(b'i' - b'a') as usize], // ï
    PHONES[(b't' - b'a') as usize] ^ 8, // ð [ð̠] (represented as a non-plosive T)
    PHONES[(b'n' - b'a') as usize] | PHONES[(b'j' - b'a') as usize], // ñ [nj] (represented as a combination of n and j)
    PHONES[(b'o' - b'a') as usize], // ò
    PHONES[(b'o' - b'a') as usize], // ó
    PHONES[(b'o' - b'a') as usize], // ô
    PHONES[(b'o' - b'a') as usize], // õ
    PHONES[(b'o' - b'a') as usize], //) ö
    !0, // ÷
    PHONES[(b'o' - b'a') as usize], // ø
    PHONES[(b'u' - b'a') as usize], // ù
    PHONES[(b'u' - b'a') as usize], // ú
    PHONES[(b'u' - b'a') as usize], // û
    PHONES[(b'y' - b'a') as usize], // ü
    PHONES[(b'y' - b'a') as usize], // ý
    PHONES[(b't' - b'a') as usize] ^ 8, // þ [ð̠] (represented as a non-plosive T)
    PHONES[(b'i' - b'a') as usize], // ÿ
];

/// An _injective_ phone table.
///
/// The first bit (MSD) is set if it is a vowel. If so, the second bit represent, if it is close
/// or not, and the third is set, if it is a front vowel. The rest of the digits are used as
/// discriminants.
///
/// If it is a consonant, the rest of the bits are simply a right truncated version of the
/// [`PHONES`](./const.PHONES.hmtl) table, with the LSD used as discriminant.
///
/// The discriminants of the vowels are chosen in the following way:
///
/// 1. 00000001 (double each time)
/// 2. 00000010
/// 3. 00000100
/// 3. 00001000
/// 4. 00010000
/// 5. 00000011 (set the LSD and use the same pattern as above, skip the first step)
/// 6. 00000101
/// 7. 00001001
/// 8. 00010001
/// 9. 00000110 (use 2 to mark the next section)
/// 10. 00001010
/// 11. 00010010
const INJECTIVE_PHONES: [u64; LETTERS as usize] = [
    0b10100000, // a
    0b00100100, // b
    0b00000110, // c
    0b00001100, // d
    0b11100000, // e
    0b00100010, // f
    0b00000100, // g
    0b00000010, // h
    0b11100001, // i
    0b00000011, // j
    0b00000101, // k
    0b01010000, // l
    0b00000001, // m
    0b00001001, // n
    0b10000000, // o
    0b00100101, // p
    0b01010100, // q
    0b01010001, // r
    0b00001010, // s
    0b00001110, // t
    0b11000001, // u
    0b00100011, // v
    0b00000000, // w
    0b01000010, // x
    0b11100010, // y
    0b01001010, // z
];

/// Non-ASCII injective phone table.
///
/// Starting at C1.
const INJECTIVE_PHONES_C1: [u64; 33] = [
    INJECTIVE_PHONES[(b's' - b'a') as usize] ^ 1, // ß
    INJECTIVE_PHONES[(b'a' - b'a') as usize] ^ 2, // à
    INJECTIVE_PHONES[(b'a' - b'a') as usize] ^ 4, // á
    INJECTIVE_PHONES[(b'a' - b'a') as usize] ^ 8, // â
    INJECTIVE_PHONES[(b'a' - b'a') as usize] ^ 16, // ã
    INJECTIVE_PHONES[(b'e' - b'a') as usize] ^ 4, // ä [æ]
    INJECTIVE_PHONES[(b'o' - b'a') as usize] ^ 1, // å [oː]
    INJECTIVE_PHONES[(b'e' - b'a') as usize] ^ 8, // æ [æ]
    INJECTIVE_PHONES[(b'z' - b'a') as usize] ^ 1, // ç [t͡ʃ]
    INJECTIVE_PHONES[(b'e' - b'a') as usize] ^ 16, // è
    INJECTIVE_PHONES[(b'e' - b'a') as usize] ^ 3, // é
    INJECTIVE_PHONES[(b'e' - b'a') as usize] ^ 5, // ê
    INJECTIVE_PHONES[(b'e' - b'a') as usize] ^ 7, // ë
    INJECTIVE_PHONES[(b'i' - b'a') as usize] ^ 17, // ì
    INJECTIVE_PHONES[(b'i' - b'a') as usize] ^ 6, // í
    INJECTIVE_PHONES[(b'i' - b'a') as usize] ^ 10, // î
    INJECTIVE_PHONES[(b'i' - b'a') as usize] ^ 18, // ï
    INJECTIVE_PHONES[(b't' - b'a') as usize] ^ 4, // ð [ð̠] (represented as a non-plosive T)
    INJECTIVE_PHONES[(b'n' - b'a') as usize] | INJECTIVE_PHONES[(b'j' - b'a') as usize], // ñ [nj] (represented as a combination of n and j)
    INJECTIVE_PHONES[(b'o' - b'a') as usize] ^ 1, // ò
    INJECTIVE_PHONES[(b'o' - b'a') as usize] ^ 2, // ó
    INJECTIVE_PHONES[(b'o' - b'a') as usize] ^ 4, // ô
    INJECTIVE_PHONES[(b'o' - b'a') as usize] ^ 8, // õ
    INJECTIVE_PHONES[(b'o' - b'a') as usize] ^ 16, //) ö
    !0, // ÷
    INJECTIVE_PHONES[(b'o' - b'a') as usize] ^ 3, // ø
    INJECTIVE_PHONES[(b'u' - b'a') as usize] ^ 1, // ù
    INJECTIVE_PHONES[(b'u' - b'a') as usize] ^ 2, // ú
    INJECTIVE_PHONES[(b'u' - b'a') as usize] ^ 4, // û
    INJECTIVE_PHONES[(b'y' - b'a') as usize] ^ 12, // ü
    INJECTIVE_PHONES[(b'y' - b'a') as usize] ^ 20, // ý
    INJECTIVE_PHONES[(b't' - b'a') as usize] ^ 4, // þ [ð̠] (represented as a non-plosive T)
    INJECTIVE_PHONES[(b'i' - b'a') as usize] ^ 24, // ÿ
];


/// Number of letters in our phone map.
const LETTERS: u8 =  26;

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
    let first_byte = {
        let entry = (string.as_bytes().get(0).map_or(0, |&x| x) | 32).wrapping_sub(b'a');
        if entry < LETTERS {
            INJECTIVE_PHONES[entry as usize]
        } else if entry >= 0xDF && entry < 0xFF {
            INJECTIVE_PHONES_C1[(entry - 0xDF) as usize]
        } else { return 0 }
    };
    let mut res = 0;
    let mut n = 1u8;
    let mut b = 0;

    loop {
        b += 1;
        // Detect overflows into the first slot.
        if (n == 0) | (b == string.len()) {
            break;
        }

        let entry = (string.as_bytes()[b] | 32).wrapping_sub(b'a');
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
    distance(a, b) < 670000
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
        assert!(similar("goth", "god"));
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
