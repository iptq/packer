Packer
======

[![](https://api.travis-ci.org/iptq/packer.svg?branch=master)](https://travis-ci.org/iptq/packer) [![dependency status](https://deps.rs/repo/github/iptq/packer/status.svg)](https://deps.rs/repo/github/iptq/packer)

**NOTE** This project is based on [the original rust-embed](https://github.com/pyros2097/rust-embed). I made enough modifications to it that I decided to just publish it in a separate repository.

packer is a library that helps you pack static files into binaries using macro magic. Here's how it's done:

### Step 1: Include

Include the crate in your `Cargo.toml`:

```toml
[dependencies]
packer = "0.2"
```

and in your `lib.rs` or `main.rs`:

```rs
#[macro_use]
extern crate packer;
```

### Step 2: Derive

Start deriving `Packer` from your structs. You need to provide a `folder` attribute to indicate the folder from which it should be pulling. Paths are relative to the crate root.

```rs
#[derive(Packer)]
#[folder = "static"]
struct Assets;
```

### Step 3: Use it!

You can now access any file using the `get` function:

```rs
let data: Option<&'static [u8]> = Assets::get("kermit.jpg");
```

You may also choose to list all the files that have been stored.

```rs
let files /*: impl Iterator<Item = &'static str>*/ = Assets::list();
```

When you build in dev mode, it will fetch off your filesystem as usual, but when you build with `--release`, it will pack the assets into your binary!

Future Work
-----------

-	Possibly add options for excluding files?

Contact
-------

Author: Michael Zhang

License: MIT
