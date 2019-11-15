use std::collections::BTreeSet;

#[test]
fn prefixed() {
    use packer::Packer;

    // prefixed by default
    #[derive(Packer)]
    #[packer(source = "tests/prefix")]
    struct Assets;

    assert_eq!(
        Assets::list().collect::<BTreeSet<_>>(),
        vec![
            "tests/prefix/bar/baz",
            "tests/prefix/baz",
            "tests/prefix/xyzzy",
        ]
        .into_iter()
        .collect::<BTreeSet<_>>()
    );

    assert_eq!(Assets::get("tests/prefix/bar/baz"), Some("abc".as_bytes()));
    assert_eq!(Assets::get("tests/prefix/baz"), Some("conflict".as_bytes()));
    assert_eq!(Assets::get("tests/prefix/xyzzy"), Some("xyz".as_bytes()));
}

#[test]
fn unprefixed() {
    use packer::Packer;

    #[derive(Packer)]
    #[packer(source = "tests/prefix", prefixed = false)]
    struct Assets;

    assert_eq!(
        Assets::list().collect::<BTreeSet<_>>(),
        vec!["bar/baz", "baz", "xyzzy",]
            .into_iter()
            .collect::<BTreeSet<_>>()
    );

    assert_eq!(Assets::get("bar/baz"), Some("abc".as_bytes()));
    assert_eq!(Assets::get("baz"), Some("conflict".as_bytes()));
    assert_eq!(Assets::get("xyzzy"), Some("xyz".as_bytes()));
}
