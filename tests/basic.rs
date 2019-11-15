extern crate packer;

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

const STATIC_FILES: [(&'static str, &'static str); 2] = [
    (
        "tests/basic/first/kermit.jpg",
        "9a2c63b0f308c3c98021e810b8852c3f6ebe3983b7d17571cb1ebb848ceb0529",
    ),
    (
        "tests/basic/second/LICENSE",
        "11abea45320df7723b156cbd4994d61da79f5e67e2eebba63c370f84196d621e",
    ),
];

#[test]
fn does_it_work() {
    use packer::Packer;

    #[derive(Packer)]
    #[packer(source = "tests/basic/first")]
    #[packer(source = "tests/basic/second", ignore = "**/*.x")]
    struct Assets;

    let static_files = STATIC_FILES.iter().cloned().collect::<BTreeMap<_, _>>();
    assert_eq!(
        Assets::list().collect::<BTreeSet<_>>(),
        static_files.keys().cloned().collect::<BTreeSet<_>>()
    );
    eprintln!("list: {:?}", Assets::list());

    let mut hasher;
    for (file, hash) in static_files {
        hasher = Sha256::default();
        let data = Assets::get(file).unwrap();
        hasher.input(data);
        assert_eq!(hash, format!("{:x}", hasher.result()), "for file {}", file);
    }

    // test if get_str works
    assert!(
        Assets::get_str("tests/basic/second/LICENSE").unwrap()
            == include_str!("../tests/basic/second/LICENSE")
    );
}

#[test]
fn does_it_work_with_generics() {
    use packer::Packer;

    #[derive(Packer)]
    #[packer(source = "tests/basic/first")]
    #[packer(source = "tests/basic/second", ignore = "**/*.x")]
    struct Assets<'a, S, T: 'a>
    where
        S: Sized,
    {
        _f: ::std::marker::PhantomData<&'a T>,
        _g: ::std::marker::PhantomData<S>,
    }

    let static_files = STATIC_FILES.iter().cloned().collect::<BTreeMap<_, _>>();
    assert_eq!(
        Assets::<(), ()>::list().collect::<BTreeSet<_>>(),
        static_files.keys().cloned().collect::<BTreeSet<_>>()
    );
    eprintln!("list: {:?}", Assets::<(), ()>::list());

    let mut hasher;
    for (file, hash) in static_files {
        hasher = Sha256::default();
        let data = Assets::<(), ()>::get(file).unwrap();
        hasher.input(data);
        assert_eq!(hash, format!("{:x}", hasher.result()), "for file {}", file);
    }

    // test if get_str works
    assert!(
        Assets::<(), ()>::get_str("tests/basic/second/LICENSE").unwrap()
            == include_str!("../tests/basic/second/LICENSE")
    );
}
