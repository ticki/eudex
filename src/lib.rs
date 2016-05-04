/// The sound table.
const MAP: [u64; 26] = [0, 13, 24, 19, 0, 14, 27, 0, 0, 28, 26, 1, 6, 7, 0, 15, 33, 10, 22,
                        18, 0, 16, 32, 30, 0, 31];

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
/// the most, and the following's weights are simply half the weight of the previous byte). If you
/// want to strictly measure the similarity, check [`similarity`](./fn.similarity.html).
pub fn distance(a: &str, b: &str) -> u64 {
    hash(a) ^ hash(b)
}

/// Calculate the "similarity" of two strings.
///
/// That is, how similar they are in terms of their Eudex hash. In particular, each byte carries
/// equal weight, and the more "similar" (e.g., sound, pronunciation, and spelling) it is, the
/// higher integer will be returned.
///
/// This is _not_ the opposite of [`distance`](./fn.distance.html).
///
/// The returned value can be between 0 and 64, although the scale is not linear (in normal
/// circumstances, the value will be between 45 and 64).
pub fn similarity(a: &str, b: &str) -> u8 {
    distance(a, b).count_zeros() as u8
}

/// Does these two sentences sound "similar".
///
/// Depending on the purpose, this might be rather strict, thus you are encouraged to use
/// [`similarity`](./fn.similarity.html) directly.
pub fn similar(a: &str, b: &str) -> bool {
    similarity(a, b) > 59
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_exact() {
        assert_eq!(hash("lal"), hash("lel"));
        assert_eq!(hash("rupert"), hash("ropert"));
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
        assert!(hash("burn") != hash("brund"));
        assert!(hash("reddit") != hash("eddit"));
        assert!(hash("lol") != hash("lulz"));
        assert!(hash("ijava") != hash("java"));
        assert!(hash("jesus") != hash("iesus"));
    }

    #[test]
    fn test_similarity() {
        // Similar.
        assert!(similarity("trump", "drumpf") >= 56); // lulz
        assert!(similarity("horse", "norse") >= 60);
        assert!(similarity("lizzard", "wizzard") >= 60);
        assert!(similarity("endu", "pondu") >= 60);
        assert!(similarity("lol", "lulz") >= 50);
        assert!(similarity("trigger", "triggered") >= 55);
        assert!(similarity("bicycle", "cycle") >= 60);

        // Exact similarity.
        assert_eq!(similarity("CYCLE", "CYCLE"), 64);
        assert_eq!(similarity("cycle", "CYCLE"), 64);
        assert_eq!(similarity("youtube", "yuotybe"), 64);

        // Not similar.
        assert!(similarity("trump", "nice") <= 53);
        assert!(similarity("morse", "Greensleeves") < 52);
        assert!(similarity("night", "day") < 57);
        assert!(similarity("racist", "rational") < 58);
        assert!(similarity("4chan", "lulz") < 57);
    }

    #[test]
    fn test_reflexivity() {
        assert_eq!(similarity("a", "b"), similarity("b", "a"));
        assert_eq!(similarity("youtube", "facebook"), similarity("facebook", "youtube"));
        assert_eq!(similarity("Rust", "Go"), similarity("Go", "Rust"));
        assert_eq!(similarity("rick", "rolled"), similarity("rolled", "rick"));
    }

    #[test]
    fn test_similar() {
        // Similar.
        assert!(similar("yay", "yuy"));
        assert!(similar("horse", "norse"));
        assert!(similar("what", "wat"));
        assert!(similar("jesus", "jeuses"));
        assert!(similar("nice", "mice"));
        assert!(similar("", ""));

        // Not similar.
        assert!(!similar("youtube", "reddit"));
        assert!(!similar("hacker", "4chan"));
        assert!(!similar("awesome", "me"));
    }
}
