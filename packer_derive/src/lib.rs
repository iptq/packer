#![recursion_limit = "1024"]

extern crate proc_macro;

use std::env;
use std::path::{Path, PathBuf};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, AttrStyle, Data, DeriveInput, Lit, Meta, MetaNameValue};
use walkdir::WalkDir;

fn generate_file_list(file_list: &Vec<PathBuf>) -> TokenStream2 {
    let values = file_list
        .iter()
        .map(|path| path.to_str().unwrap().to_string());

    quote! {
        fn list() -> Self::Item {
            const FILES: &[&str] = &[#(#values),*];
            FILES.into_iter().cloned()
        }

        fn get_str(file_path: &str) -> Option<&'static str> {
            Self::get(file_path).and_then(|s| ::std::str::from_utf8(s).ok())
        }
    }
}

#[cfg(debug_assertions)]
fn generate_assets(file_list: &Vec<PathBuf>) -> TokenStream2 {
    quote! {
        fn get(file_path: &str) -> Option<&'static [u8]> {
            use std::collections::HashSet;
            use std::fs::read;
            use std::path::{PathBuf, Path};
            use std::sync::Mutex;

            packer::lazy_static! {
                static ref CACHE: Mutex<HashSet<&'static [u8]>> = Mutex::new(HashSet::new());
            }

            let path = PathBuf::from(file_path);
            let file = read(path).ok()?;

            let mut cache = CACHE.lock().unwrap();
            if !cache.contains(&file as &[_]) {
                cache.insert(Box::leak(file.clone().into_boxed_slice()));
            }
            Some(cache.get(&file as &[_]).unwrap())
        }
    }
}

#[cfg(not(debug_assertions))]
fn generate_assets(file_list: &Vec<PathBuf>) -> TokenStream2 {
    let values = file_list
        .iter()
        .map(|path| {
            // let base = folder_path.as_ref();
            let key = String::from(
                path.to_str()
                    .expect("Path does not have a string representation"),
            );
            let canonical_path =
                std::fs::canonicalize(&path).expect("Could not get canonical path");
            let canonical_path_str = canonical_path.to_str();
            quote! { #key => Some(include_bytes!(#canonical_path_str)) }
        })
        .collect::<Vec<_>>();

    quote! {
        fn get(file_path: &str) -> Option<&'static [u8]> {
            match file_path {
                #(#values,)*
                _ => None,
            }
        }
    }
}

fn impl_packer(ast: &syn::DeriveInput) -> TokenStream2 {
    match ast.data {
        Data::Enum(_) => panic!("#[derive(Packer)] must be used on structs."),
        _ => (),
    };

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut file_list = Vec::new();

    // look for #[folder = ""] attributes
    for attr in &ast.attrs {
        let meta = attr.parse_meta().expect("Failed to parse meta.");
        let (name, value) = match meta {
            Meta::NameValue(MetaNameValue { ident, lit, .. }) => (ident, lit),
            _ => panic!("rtfm"),
        };

        if name == "source" {
            let path = match value {
                Lit::Str(s) => PathBuf::from(s.value()),
                _ => panic!("Attribute value must be a string."),
            };

            if !Path::new(&path).exists() {
                panic!(
                    "Directory '{}' does not exist. cwd: '{}'",
                    path.to_str().unwrap(),
                    std::env::current_dir().unwrap().to_str().unwrap()
                );
            };

            WalkDir::new(&path).into_iter().for_each(|dir_entry| {
                let dir_entry = dir_entry.unwrap_or_else(|err| {
                    panic!("WalkDir error: {}", err);
                });
                let file_path = dir_entry.path();

                if !file_path.is_file() {
                    // ignore directories
                    return;
                }

                if !file_path.exists() {
                    panic!("Path doesn't exist: {:?}", &file_path);
                }

                file_list.push(file_path.to_path_buf());
            })
        }
    }

    let generate_file_list_fn = generate_file_list(&file_list);
    let generate_assets_fn = generate_assets(&file_list);

    quote! {
        impl #impl_generics ::packer::Packer for #ident #ty_generics #where_clause {
            type Item = ::std::iter::Cloned<::std::slice::Iter<'static, &'static str>>;

            #generate_file_list_fn
            #generate_assets_fn
        }
    }
}

#[proc_macro_derive(Packer, attributes(source))]
pub fn derive_input_object(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = impl_packer(&ast);
    TokenStream::from(gen)
}
