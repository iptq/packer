//! packer is a library that helps you pack static files into binaries using macro magic. Here's
//! how it's done:
//!
//! ### Step 1: Include
//!
//! Include the crate in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! packer = "0.2"
//! ```
//!
//! and in your `lib.rs` or `main.rs`:
//!
//! ```rs
//! #[macro_use]
//! extern crate packer;
//! ```
//!
//! ### Step 2: Derive
//!
//! Start deriving `Packer` from your structs. You need to provide a `folder` attribute to indicate
//! the folder from which it should be pulling. Paths are relative to the crate root.
//!
//! ```rs
//! # use packer::Packer;
//! #[derive(Packer)]
//! #[folder = "static"]
//! struct Assets;
//! ```
//!
//! ### Step 3: Use it!
//!
//! You can now access any file using the `get` function:
//!
//! ```
//! # use packer::Packer;
//! # #[derive(packer::Packer)]
//! # #[folder = "static"]
//! # struct Assets;
//! let data: Option<&'static [u8]> = Assets::get("kermit.jpg");
//! ```
//!
//! You may also choose to list all the files that have been stored.
//!
//! ```
//! # use packer::Packer;
//! # #[derive(packer::Packer)]
//! # #[folder = "static"]
//! # struct Assets;
//! let files /*: impl Iterator<Item = &'static str>*/ = Assets::list();
//! ```
//!
//! When you build in dev mode, it will fetch off your filesystem as usual, but when you build with
//! `--release`, it will pack the assets into your binary!

#[doc(hidden)]
pub use lazy_static::*;
#[doc(hidden)]
pub use packer_derive::*;

pub trait Packer {
    // used for iterator, will be changed when impl Trait is stable in trait def
    #[doc(hidden)]
    type Item: Iterator<Item = &'static str> + Sized;

    /// Lists the 
    fn list() -> Self::Item;

    /// Returns the contents of the file named `file_name` as a &'static [u8] if
    /// it exists, `None` otherwise.
    fn get(file_name: &str) -> Option<&'static [u8]>;

    /// Returns the contents of the file named `file_name` as a &'static str if
    /// it exists, `None` otherwise.
    fn get_str(file_name: &str) -> Option<&'static str>;
}
