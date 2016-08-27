//! The raw Eudex API.

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
const PHONES: [u8; LETTERS as usize] = [
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
const PHONES_C1: [u8; LETTERS_C1 as usize] = [
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
const INJECTIVE_PHONES: [u8; LETTERS as usize] = [
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
const INJECTIVE_PHONES_C1: [u8; LETTERS_C1 as usize] = [
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

/// Map the first character in a word.
#[inline(always)]
pub fn map_first(mut x: u8) -> u8 {
    x |= 32;
    x = x.wrapping_sub(b'a');

    if x < LETTERS {
        INJECTIVE_PHONES[x as usize]
    } else if x >= 0xDF && x < 0xFF {
        INJECTIVE_PHONES_C1[(x - 0xDF) as usize]
    } else {
        0
    }
}

/// Filter a non-head character.
///
/// `None` means "skip this character", whereas `Some(x)` means "push x".
///
/// Eudex works by building up a hash by this filter and then XORing to get the difference.
#[inline(always)]
pub fn filter(prev: u8, mut x: u8) -> Option<u8> {
    x |= 32;
    x = x.wrapping_sub(b'a');

    x = if x < LETTERS {
        PHONES[x as usize]
    } else if x >= 0xDF && x < 0xFF {
        PHONES_C1[(x - 0xDF) as usize]
    } else {
        return None;
    };

    if x & 1 != prev & 1 {
        Some(x)
    } else { None }
}
