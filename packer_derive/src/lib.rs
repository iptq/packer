#![recursion_limit = "1024"]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::path::{Path, PathBuf};
use syn::{parse_macro_input, AttrStyle, Data, DeriveInput, Lit, Meta, MetaNameValue};
use walkdir::WalkDir;

fn generate_file_list<P>(item: &syn::DeriveInput, folder_path: P) -> TokenStream2
where
    P: AsRef<Path>,
{
    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let values = WalkDir::new(&folder_path)
        .into_iter()
        .map(|entry| entry.unwrap().path().to_path_buf())
        .filter(|path| path.is_file())
        .map(|path| {
            path.strip_prefix(&folder_path)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        });

    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            pub fn list() -> impl std::iter::Iterator<Item = &'static str> {
                const FILES: &[&str] = &[#(#values),*];
                FILES.into_iter().cloned()
            }

            pub fn get_str(file_path: &str) -> Option<&'static str> {
                Self::get(file_path).and_then(|s| ::std::str::from_utf8(s).ok())
            }
        }
    }
}

#[cfg(debug_assertions)]
fn generate_assets<P>(item: &syn::DeriveInput, folder_path: P) -> TokenStream2
where
    P: AsRef<Path>,
{
    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let folder_path = folder_path.as_ref().to_str().unwrap();
    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            pub fn get(file_path: &str) -> Option<&'static [u8]> {
                use std::collections::HashSet;
                use std::fs::read;
                use std::path::{PathBuf, Path};
                use std::sync::Mutex;

                packer::lazy_static! {
                    static ref CACHE: Mutex<HashSet<&'static [u8]>> = Mutex::new(HashSet::new());
                }

                let mut path = PathBuf::from(#folder_path);
                let fpath = PathBuf::from(file_path);
                path.push(fpath);

                let file = read(path).ok()?;

                let mut cache = CACHE.lock().unwrap();
                if !cache.contains(&file as &[_]) {
                    cache.insert(Box::leak(file.clone().into_boxed_slice()));
                }
                Some(cache.get(&file as &[_]).unwrap())
            }
        }
    }
}

#[cfg(not(debug_assertions))]
fn generate_assets<P>(item: &syn::DeriveInput, folder_path: P) -> TokenStream2
where
    P: AsRef<Path>,
{
    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let values = WalkDir::new(&folder_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|entry| {
            let base = folder_path.as_ref();
            let key = String::from(
                entry
                    .path()
                    .strip_prefix(base)
                    .unwrap()
                    .to_str()
                    .expect("Path does not have a string representation"),
            );
            let canonical_path =
                std::fs::canonicalize(entry.path()).expect("Could not get canonical path");
            let canonical_path_str = canonical_path.to_str();
            quote! { #key => Some(include_bytes!(#canonical_path_str)) }
        })
        .collect::<Vec<_>>();

    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            pub fn get(file_path: &str) -> Option<&'static [u8]> {
                match file_path {
                    #(#values,)*
                    _ => None,
                }
            }
        }
    }
}

fn help() {
    panic!(
        "#[derive(Packer)] should contain one attribute like this #[folder = \"examples/public/\"]"
    );
}

fn impl_packer(ast: &syn::DeriveInput) -> TokenStream2 {
    match ast.data {
        Data::Enum(_) => help(),
        _ => (),
    };

    if ast.attrs.len() < 1 {
        panic!("Missing #[folder = \"\"] attribute.");
    }

    let attr = &ast.attrs[0];

    match attr.style {
        AttrStyle::Outer => (),
        _ => panic!("Attribute must be an outer attribute."),
    }

    let meta = attr.parse_meta().expect("Failed to parse meta");
    let (name, value) = match meta {
        Meta::NameValue(MetaNameValue { ident, lit, .. }) => (ident, lit),
        _ => panic!("u dum but"),
    };

    if name != "folder" {
        panic!("Attribute name must be 'folder'")
    }

    let folder_path = match value {
        Lit::Str(s) => PathBuf::from(s.value()),
        _ => panic!("Attribute value must be a string."),
    };

    if !Path::new(&folder_path).exists() {
        panic!(
            "#[derive(Packer)] folder '{}' does not exist. cwd: '{}'",
            folder_path.to_str().unwrap(),
            std::env::current_dir().unwrap().to_str().unwrap()
        );
    };

    let generate_file_list_fn = generate_file_list(ast, &folder_path);
    let generate_assets_fn = generate_assets(ast, &folder_path);

    quote! {
        #generate_file_list_fn
        #generate_assets_fn
    }
}

#[proc_macro_derive(Packer, attributes(folder))]
pub fn derive_input_object(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = impl_packer(&ast);
    TokenStream::from(gen)
}
