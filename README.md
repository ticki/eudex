# Eudex: A blazingly fast phonetic reduction/hashing algorithm.

Eudex is a Soundex-esque phonetic reduction/hashing algorithm, providing
locality sensitive "hashes" of words, based on the spelling and pronunciation.

It is derived from the classification of the pulmonic consonants.

Eudex is about two orders of magnitude faster than Soundex, and several orders
of magnitude faster than Levenshtein distance, making it feasible to run on
large sets of strings in very short time.

It is tested on the English, Catalan, German, Spanish, and Italian
dictionaries, and has relatively good quality.

It is **not** a replacement for Levenshtein distance, it is a replacement for
Levenshtein distance in certain use cases, e.g. searching for spellcheck
suggestions.

## Example

```rust
extern crate eudex;

fn main() {
    assert!(eudex::similar("jumpo", "jumbo"));
    assert!(!eudex::similar("horse", "norse"));
    println!("{}", eudex::hash("hello"));
}
```

## Cargo

Add this to your `Cargo.toml`:

```toml
[dependencies.eudex]
git = "https://github.com/ticki/eudex.git"
```
