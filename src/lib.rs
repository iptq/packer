#![recursion_limit = "1024"]
extern crate proc_macro;

extern crate walkdir;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use std::path::{Path, PathBuf};
use quote::quote;
use syn::{parse_macro_input, Meta,Lit, MetaNameValue,DeriveInput, AttrStyle,Data};

fn generate_file_list<P>(item: &syn::DeriveInput, folder_path: P) -> TokenStream2
where
    P: AsRef<Path>,
{
    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    use walkdir::WalkDir;
    let mut values = Vec::<TokenStream2>::new();
    for entry in WalkDir::new(&folder_path) {
        let path = entry.unwrap().path().to_path_buf();
        if path.is_file() {
            let pathstr = path.strip_prefix(&folder_path).unwrap().to_str();
            values.push(quote!(#pathstr,));
        }
    }
    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            pub fn list() -> ::std::vec::IntoIter<&'static str> {
                vec![
                    #(#values)*
                ].into_iter()
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
    quote!{
        impl #impl_generics #ident #ty_generics #where_clause {
            pub fn get(file_path: &str) -> Option<Vec<u8>> {
                use std::fs::File;
                use std::io::Read;
                use std::path::{PathBuf, Path};

                let mut path = PathBuf::from(#folder_path);
                let fpath = PathBuf::from(file_path);
                path.push(fpath);

                let mut file = match File::open(path) {
                    Ok(mut file) => file,
                    Err(_e) => {
                        return None
                    }
                };
                let mut data: Vec<u8> = Vec::new();
                match file.read_to_end(&mut data) {
                    Ok(_) => Some(data),
                    Err(_e) =>  {
                        return None
                    }
                }
            }
        }
    }
}

#[cfg(not(debug_assertions))]
fn generate_assets<P>(item: &syn::DeriveInput, folder_path: P) -> quote::Tokens
where
    P: AsRef<Path>,
{
    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    use walkdir::WalkDir;
    let mut values = Vec::<Tokens>::new();
    for entry in WalkDir::new(&folder_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
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
        let value = quote!{
          #key => Some(include_bytes!(#canonical_path_str).to_vec()),
        };
        values.push(value);
    }
    quote!{
        impl #impl_generics #ident #ty_generics #where_clause {
            pub fn get(file_path: &str) -> Option<Vec<u8>> {
                match file_path {
                    #(#values)*
                    _ => None,
                }
            }
        }
    }
}

fn help() {
    panic!("#[derive(Embed)] should contain one attribute like this #[folder = \"examples/public/\"]");
}

fn impl_embed(ast: &syn::DeriveInput) -> TokenStream2 {
    match ast.data {
        Data::Enum(_) => help(),
        _ => (),
    };

    // let value = &ast.attrs[0].value;
    // let literal_value = match value {
    //     &MetaItem::NameValue(ref attr_name, ref value) => {
    //         if attr_name == "folder" {
    //             value
    //         } else {
    //             panic!("#[derive(Embed)] attribute name must be folder");
    //         }
    //     }
    //     _ => {
    //         panic!("#[derive(Embed)] attribute name must be folder");
    //     }
    // };
    // let folder_path = match literal_value {
    //     &Lit::Str(ref val, _) => PathBuf::from(val),
    //     _ => {
    //         panic!("#[derive(Embed)] attribute value must be a string literal");
    //     }
    // };
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
        _ => panic!("u dum but")
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
            "#[derive(Embed)] folder '{}' does not exist. cwd: '{}'",
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
    let gen = impl_embed(&ast);
    TokenStream::from(gen)
}
