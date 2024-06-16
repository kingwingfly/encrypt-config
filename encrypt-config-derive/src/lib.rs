#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(all(not(feature = "persist"), feature = "default_config_dir"))]
compile_error!("Feature `default_config_dir` only works with feature `persist` on.");

use proc_macro::TokenStream;
use quote::quote;
#[cfg(feature = "persist")]
use syn::LitStr;
use syn::{parenthesized, parse_macro_input, DeriveInput, Expr, Ident};

/// A derive macro helping implemente `Source` trait.
#[proc_macro_derive(Source, attributes(source))]
pub fn derive_normal_source(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut default_expr: Expr = syn::parse_str("[]").unwrap();
    let mut value: Ident = syn::parse_str("String").unwrap();

    if let Some(attr) = input
        .attrs
        .iter()
        .find(|&attr| attr.path().is_ident("source"))
    {
        attr.parse_nested_meta(|meta| {
            if let Some(i) = meta.path.get_ident() {
                let content;
                parenthesized!(content in meta.input);
                match i.to_string().as_str() {
                    "default" => {
                        default_expr = content.parse()?;
                    }
                    "value" => {
                        value = content.parse()?;
                    }
                    attr => {
                        panic!("This macro only supports the `default` and `value` attributes, but got {}", attr);
                    }
                }
            }
            Ok(())
        })
        .expect("");
    };

    let expanded = quote! {
        impl #impl_generics encrypt_config::Source for #name #ty_generics #where_clause {
            type Value = #value;
            type Map = ::std::collections::HashMap<String, Self::Value>;

            fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
                Ok(#default_expr.into_iter().collect())
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

/// A derive macro helping implemente `PersistSource` trait.
#[cfg(feature = "persist")]
#[proc_macro_derive(PersistSource, attributes(source))]
pub fn derive_persist_source(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut default_expr: Expr = syn::parse_str("[]").unwrap();
    let mut value: Ident = syn::parse_str("String").unwrap();
    #[cfg(not(feature = "default_config_dir"))]
    let mut path: Option<LitStr> = None;
    #[cfg(feature = "default_config_dir")]
    let mut source_name: Option<LitStr> = None;

    if let Some(attr) = input
        .attrs
        .iter()
        .find(|&attr| attr.path().is_ident("source"))
    {
        attr.parse_nested_meta(|meta| {
            if let Some(i) = meta.path.get_ident() {
                let content;
                parenthesized!(content in meta.input);
                match i.to_string().as_str() {
                    "default" => {
                        default_expr = content.parse()?;
                    }
                    "value" => {
                        value = content.parse()?;
                    }
                    #[cfg(not(feature = "default_config_dir"))]
                    "path" => {
                        path = content.parse().ok();
                    }
                    #[cfg(feature = "default_config_dir")]
                    "source_name" => {
                        source_name = content.parse().ok();
                    }
                    attr => {
                        #[cfg(not(feature = "default_config_dir"))]
                        panic!(
                            "This macro only support `default`, `value` and `path` attribute, but got {}", attr
                        );
                        #[cfg(feature = "default_config_dir")]
                        panic!(
                            "This macro only support `default`, `value` and `source_name` attribute, but got {}", attr
                        );
                    }
                }
            }
            Ok(())
        })
        .expect("");
    };

    #[cfg(not(feature = "default_config_dir"))]
    let func = quote!(
        fn path(&self) -> ::std::path::PathBuf {
            ::std::path::PathBuf::from(#path)
        }
    );
    #[cfg(feature = "default_config_dir")]
    let func = quote!(
        fn source_name(&self) -> String {
            #source_name.to_owned()
        }
    );

    let expanded = quote! {
        impl #impl_generics encrypt_config::PersistSource for #name #ty_generics #where_clause {
            type Value = #value;
            type Map = ::std::collections::HashMap<String, Self::Value>;

            #func

            fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
                Ok(#default_expr.into_iter().collect())
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

/// A derive macro helping implemente `SecretSource` trait.
#[cfg(feature = "secret")]
#[proc_macro_derive(SecretSource, attributes(source))]
pub fn derive_secret_source(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut default_expr: Expr = syn::parse_str("[]").unwrap();
    let mut value: Ident = syn::parse_str("String").unwrap();
    #[cfg(not(feature = "default_config_dir"))]
    let mut path: Option<LitStr> = None;
    #[cfg(feature = "default_config_dir")]
    let mut source_name: Option<LitStr> = None;

    if let Some(attr) = input
        .attrs
        .iter()
        .find(|&attr| attr.path().is_ident("source"))
    {
        attr.parse_nested_meta(|meta| {
            if let Some(i) = meta.path.get_ident() {
                let content;
                parenthesized!(content in meta.input);
                match i.to_string().as_str() {
                    "default" => {
                        default_expr = content.parse()?;
                    }
                    "value" => {
                        value = content.parse()?;
                    }
                    #[cfg(not(feature = "default_config_dir"))]
                    "path" => {
                        path = content.parse().ok();
                    }
                    #[cfg(feature = "default_config_dir")]
                    "source_name" => {
                        source_name = content.parse().ok();
                    }
                    attr => {
                        #[cfg(not(feature = "default_config_dir"))]
                        panic!(
                            "This macro only support `default`, `value` and `path` attribute, but got {}", attr
                        );
                        #[cfg(feature = "default_config_dir")]
                        panic!(
                            "This macro only support `default`, `value` and `source_name` attribute, but got {}", attr
                        );
                    }
                }
            }
            Ok(())
        })
        .expect("");
    };

    #[cfg(not(feature = "default_config_dir"))]
    let func = quote!(
        fn path(&self) -> ::std::path::PathBuf {
            ::std::path::PathBuf::from(#path)
        }
    );
    #[cfg(feature = "default_config_dir")]
    let func = quote!(
        fn source_name(&self) -> String {
            #source_name.to_owned()
        }
    );

    let expanded = quote! {
        impl #impl_generics encrypt_config::SecretSource for #name #ty_generics #where_clause {
            type Value = #value;
            type Map = ::std::collections::HashMap<String, Self::Value>;

            #func

            fn default(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
                Ok(#default_expr.into_iter().collect())
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
