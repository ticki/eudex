# eudex
A blazingly fast phonetic reduction/hashing algorithm.

This is about two orders of magnitude faster than Soundex, and several orders
of magnitude faster than Levenshtein distance, making it feasible to run on
large sets of strings in very short time.

## Example

```rust
extern crate eudex;

fn main() {
    assert!(eudex::similar("horse", "norse"));
    assert!(eudex::similar("jumpo", "jumbo"));
    println!("{}", eudex::hash("hello"));
}
```

## Cargo

Add this to your `Cargo.toml`:

```toml
[dependency.eudex]
git = "https://github.com/ticki/eudex.git"
```
