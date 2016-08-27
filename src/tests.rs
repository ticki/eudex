use super::*;
use test::Bencher;

#[test]
fn test_exact() {
    assert_eq!(Hash::new("JAva"), Hash::new("jAva"));
    assert_eq!(Hash::new("co!mputer"), Hash::new("computer"));
    assert_eq!(Hash::new("comp-uter"), Hash::new("computer"));
    assert_eq!(Hash::new("comp@u#te?r"), Hash::new("computer"));
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
    assert!(Hash::new("jiva") != Hash::new("java"));
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
    assert!((Hash::new("goth") - Hash::new("god")).similar());
    assert!((Hash::new("maier") - Hash::new("meyer")).similar());
    assert!((Hash::new("java") - Hash::new("jiva")).similar());
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
