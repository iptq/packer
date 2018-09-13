Embed
=====

**NOTE** This project is based on [the original rust-embed](https://github.com/pyros2097/rust-embed). I made enough modifications to it that I decided to just publish it in a separate repositories.

Embed is a library that helps you pack static files into binaries using macro magic. Here's how it's done:

### Step 1: Include

Include the crate in your `Cargo.toml`:

```toml
# Use the git URL unless I decide to publish on crates.io
embed = { git = "https://github.com/iptq/embed.git" }
```

and in your `lib.rs` or `main.rs`:

```rs
#[macro_use]
extern crate embed;
```

### Step 2: Derive

Start deriving `Embed` from your structs. You need to provide a `folder` attribute to indicate the folder 
from which it should be pulling. Paths are relative to the crate root.

```rs
#[derive(Embed)]
#[folder = "static"]
struct Assets;
```

### Step 3: Use it!

You can now access any file using the `get` function:

```rs
Assets::get("kermit.jpg"); // this is a Option<Vec<u8>>
```

You may also choose to list all the files that have been stored.

```rs
Assets::list(); // this is a ::std::vec::IntoIter<&'static str>
```

When you build in dev mode, it will fetch off your filesystem as usual, but when you build with `--release`, it will pack the assets into your binary!

Future Work
-----------

- Possibly add options for excluding files?

Contact
-------

Author: Michael Zhang

License: MIT
