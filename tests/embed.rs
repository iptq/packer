#[macro_use]
extern crate embed;
extern crate sha2;

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

const STATIC_FILES: [(&'static str, &'static str); 2] = [
    (
        "kermit.jpg",
        "9a2c63b0f308c3c98021e810b8852c3f6ebe3983b7d17571cb1ebb848ceb0529",
    ),
    (
        "LICENSE",
        "11abea45320df7723b156cbd4994d61da79f5e67e2eebba63c370f84196d621e",
    ),
];

#[test]
fn does_it_work() {
    #[derive(Embed)]
    #[folder = "static"]
    struct Assets;

    let static_files = STATIC_FILES.iter().cloned().collect::<BTreeMap<_, _>>();
    assert_eq!(
        Assets::list().collect::<BTreeSet<_>>(),
        static_files.keys().cloned().collect::<BTreeSet<_>>()
    );

    let mut hasher;
    for (file, hash) in static_files {
        hasher = Sha256::default();
        let data = Assets::get(file).unwrap();
        hasher.input(data.as_slice());
        assert_eq!(hash, format!("{:x}", hasher.result()));
    }
}
