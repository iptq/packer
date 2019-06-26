#![recursion_limit = "1024"]

extern crate proc_macro;

use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Lit, LitBool, Meta, MetaNameValue, NestedMeta};
use walkdir::WalkDir;

fn generate_file_list(file_list: &BTreeMap<String, PathBuf>) -> TokenStream2 {
    let files = file_list
        .iter()
        .map(|(key, _)| key);

    quote! {
        fn list() -> Self::Item {
            const FILES: &[&str] = &[#(#files),*];
            FILES.into_iter().cloned()
        }

        fn get_str(file_path: &str) -> Option<&'static str> {
            Self::get(file_path).and_then(|s| ::std::str::from_utf8(s).ok())
        }
    }
}

#[cfg(all(debug_assertions, not(feature = "always_pack")))]
fn generate_assets(_file_list: &BTreeMap<String, PathBuf>) -> TokenStream2 {
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

#[cfg(any(not(debug_assertions), feature = "always_pack"))]
fn generate_assets(file_list: &BTreeMap<String, PathBuf>) -> TokenStream2 {
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
        Data::Struct(_) => (),
        _ => panic!("#[derive(Packer)] must be used on structs."),
    };

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut file_list = BTreeMap::new();

    // look for #[folder = ""] attributes
    for attr in &ast.attrs {
        let meta = attr.parse_meta().expect("Failed to parse meta.");
        let (ident, meta_list) = match meta {
            Meta::List(list) => (list.ident, list.nested),
            Meta::NameValue(_) => {
                panic!("The API has changed. Please see the docs for the updated syntax.")
            }
            _ => panic!("rtfm"),
        };

        if ident == "packer" {
            let mut source_path = None;
            let mut prefixed = true;
            let mut ignore_patterns = Vec::new();

            for meta_item in meta_list {
                let meta = match meta_item {
                    NestedMeta::Meta(meta) => meta,
                    _ => panic!("rtfm"),
                };

                let (name, value) = match meta {
                    Meta::NameValue(MetaNameValue { ident, lit, .. }) => (ident, lit),
                    _ => panic!("rtfm"),
                };

                match name.to_string().as_str() {
                    "source" => {
                        let path = match value {
                            Lit::Str(s) => PathBuf::from(s.value()),
                            _ => panic!("Attribute value must be a string."),
                        };

                        if let Some(_) = source_path {
                            panic!("Cannot put two sources in the same attribute. Please create a new attribute.");
                        }

                        if !path.exists() {
                            panic!(
                                "Directory '{}' does not exist. cwd: '{}'",
                                path.to_str().unwrap(),
                                env::current_dir().unwrap().to_str().unwrap()
                            );
                        };

                        source_path = Some(path);
                    }
                    "prefixed" => {
                        match value {
                            Lit::Bool(LitBool { value, .. }) => prefixed = value,
                            _ => panic!("The `prefixed` parameter must be a bool"),
                        };
                    }
                    #[cfg(feature = "ignore")]
                    "ignore" => {
                        let pattern = match value {
                            Lit::Str(s) => s.value(),
                            _ => panic!("Attribute value must be a string."),
                        };

                        let pattern =
                            glob::Pattern::new(&pattern).expect("Could not compile glob.");
                        ignore_patterns.push(pattern);
                    }
                    unsupported => {
                        panic!("unsupported parameter '{}'", unsupported);
                    }
                }
            }

            let source_path = match source_path {
                Some(path) => path,
                None => panic!("No source path provided."),
            };
            if source_path.is_file() {
                // check with the filter anyway
                let mut allowed = true;
                #[cfg(feature = "ignore")]
                {
                    for pattern in &ignore_patterns {
                        if pattern.matches_path(&source_path) {
                            allowed = false;
                            break;
                        }
                    }
                }
                if allowed {
                    // makes no difference if it's prefixed or not for single files
                    file_list.insert(source_path.to_str().unwrap().to_string(), source_path);
                }
            } else if source_path.is_dir() {
                WalkDir::new(&source_path)
                    .into_iter()
                    .for_each(|dir_entry| {
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

                        #[cfg(feature = "ignore")]
                        {
                            for pattern in &ignore_patterns {
                                if pattern.matches_path(&file_path) {
                                    return;
                                }
                            }
                        }

                        let file_name = if !prefixed {
                            file_path.strip_prefix(&source_path).unwrap()
                        } else {
                            file_path
                        };
                        let key = file_name.to_str().unwrap().to_string();
                        if file_list.contains_key(&key) {
                            panic!("collision for name '{}'", key);
                        }
                        file_list.insert(
                            key,
                            file_path.to_path_buf(),
                        );
                    });
            }
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

#[proc_macro_derive(Packer, attributes(packer))]
pub fn derive_input_object(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let gen = impl_packer(&ast);
    TokenStream::from(gen)
}
