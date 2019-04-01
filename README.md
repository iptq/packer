Packer
======

[![](https://api.travis-ci.org/iptq/packer.svg?branch=master)](https://travis-ci.org/iptq/packer) [![dependency status](https://deps.rs/repo/github/iptq/packer/status.svg)](https://deps.rs/repo/github/iptq/packer)

**NOTE** This project is based on [the original rust-embed](https://github.com/pyros2097/rust-embed). I made enough modifications to it that I decided to just publish it in a separate repository.

**NOTE** This project requires a Rust 2018 (Rust 1.31+) compiler.

packer is a library that helps you pack static files into binaries using macro magic. When you build in dev mode, it will fetch off your filesystem as usual, but when you build with `--release`, it will pack the assets into your binary!

See the docs to see how to use it.

Future Work
-----------

- Possibly add options for excluding files?

Contact
-------

Author: Michael Zhang, Nathan Ringo

License: MIT
