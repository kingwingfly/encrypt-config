use proc_macro::TokenStream;
use quote::quote;
use syn::{parenthesized, parse_macro_input, DeriveInput, Expr, Ident, LitStr};

/// A derive macro helping implemente `Source` trait.
/// # Example
/// ```
/// # use encrypt_config_derive::Source;
/// #[derive(Source)]
/// #[source(default([("key".to_owned(), "value".to_owned())]))]
/// struct SourceArray;
/// ```
#[proc_macro_derive(Source, attributes(source))]
pub fn derive_normal_source(input: TokenStream) -> TokenStream {
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
                match i.to_string().as_str() {
                    "default" => {
                        let content;
                        parenthesized!(content in meta.input);
                        default_expr = content.parse()?;
                    }
                    "value" => {
                        let content;
                        parenthesized!(content in meta.input);
                        value = content.parse()?;
                    }
                    #[cfg(not(feature = "default_config_dir"))]
                    "path" => {
                        let content;
                        parenthesized!(content in meta.input);
                        path = content.parse().ok();
                    }
                    #[cfg(feature = "default_config_dir")]
                    "source_name" => {
                        let content;
                        parenthesized!(content in meta.input);
                        source_name = content.parse().ok();
                    }
                    attr => {
                        panic!("unknown attribute: {}", attr)
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

            fn collect(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
                Ok(#default_expr.into_iter().collect())
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

/// A derive macro helping implemente `PersistSource` trait.
/// # Example
/// ```
/// # use encrypt_config_derive::PersistSource;
/// # use serde::{Serialize, Deserialize};
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct Foo(String);
///
/// // If feature `default_config_dir` is off:
/// # #[cfg(not(feature = "default_config_dir"))]
/// #[derive(PersistSource)]
/// #[source(value(Foo), path("tests/persist.conf"), default([("key".to_owned(), Foo("value".to_owned()))]))]
/// struct SourceFoo;
///
/// // If feature `default_config_dir` is on:
/// # #[cfg(feature = "default_config_dir")]
/// #[derive(PersistSource)]
/// #[source(value(Foo), source_name("secret.conf"), default([("key".to_owned(), Foo("value".to_owned()))]))]
/// struct SourceFoo;
/// ```
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
                match i.to_string().as_str() {
                    "default" => {
                        let content;
                        parenthesized!(content in meta.input);
                        default_expr = content.parse()?;
                    }
                    "value" => {
                        let content;
                        parenthesized!(content in meta.input);
                        value = content.parse()?;
                    }
                    #[cfg(not(feature = "default_config_dir"))]
                    "path" => {
                        let content;
                        parenthesized!(content in meta.input);
                        path = content.parse().ok();
                    }
                    #[cfg(feature = "default_config_dir")]
                    "source_name" => {
                        let content;
                        parenthesized!(content in meta.input);
                        source_name = content.parse().ok();
                    }
                    attr => {
                        panic!("unknown attribute: {}", attr)
                    }
                }
            }
            Ok(())
        })
        .expect("");
    };
    #[cfg(not(feature = "default_config_dir"))]
    let expanded = quote! {
        impl #impl_generics encrypt_config::PersistSource for #name #ty_generics #where_clause {
            type Value = #value;

            fn path(&self) -> ::std::path::PathBuf {
                ::std::path::PathBuf::from(#path)
            }

            fn default(&self) -> ::std::collections::HashMap<String, Self::Value> {
                #default_expr.into_iter().collect()
            }
        }
    };
    #[cfg(feature = "default_config_dir")]
    let expanded = quote! {
        impl #impl_generics encrypt_config::PersistSource for #name #ty_generics #where_clause {
            type Value = #value;

            fn source_name(&self) -> String {
                #source_name.to_owned()
            }

            fn default(&self) -> ::std::collections::HashMap<String, Self::Value> {
                #default_expr.into_iter().collect()
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

/// A derive macro helping implemente `SecretSource` trait.
/// # Example
/// ```
/// # use encrypt_config_derive::SecretSource;
/// # use serde::{Serialize, Deserialize};
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct Foo(String);
///
/// // If feature `default_config_dir` is off:
/// # #[cfg(not(feature = "default_config_dir"))]
/// #[derive(SecretSource)]
/// #[source(value(Foo), path("tests/secret.conf"), default([("key".to_owned(), Foo("value".to_owned()))]))]
/// struct SourceFoo;
///
/// // If feature `default_config_dir` is on:
/// # #[cfg(feature = "default_config_dir")]
/// #[derive(SecretSource)]
/// #[source(value(Foo), source_name("secret.conf"), default([("key".to_owned(), Foo("value".to_owned()))]))]
/// struct SourceFoo;
/// ```
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
                match i.to_string().as_str() {
                    "default" => {
                        let content;
                        parenthesized!(content in meta.input);
                        default_expr = content.parse()?;
                    }
                    "value" => {
                        let content;
                        parenthesized!(content in meta.input);
                        value = content.parse()?;
                    }
                    #[cfg(not(feature = "default_config_dir"))]
                    "path" => {
                        let content;
                        parenthesized!(content in meta.input);
                        path = content.parse().ok();
                    }
                    #[cfg(feature = "default_config_dir")]
                    "source_name" => {
                        let content;
                        parenthesized!(content in meta.input);
                        source_name = content.parse().ok();
                    }
                    attr => {
                        panic!("unknown attribute: {}", attr)
                    }
                }
            }
            Ok(())
        })
        .expect("");
    };
    #[cfg(not(feature = "default_config_dir"))]
    let expanded = quote! {
        impl #impl_generics encrypt_config::SecretSource for #name #ty_generics #where_clause {
            type Value = #value;

            fn path(&self) -> ::std::path::PathBuf {
                ::std::path::PathBuf::from(#path)
            }

            fn default(&self) -> ::std::collections::HashMap<String, Self::Value> {
                #default_expr.into_iter().collect()
            }
        }
    };
    #[cfg(feature = "default_config_dir")]
    let expanded = quote! {
        impl #impl_generics encrypt_config::SecretSource for #name #ty_generics #where_clause {
            type Value = #value;

            fn source_name(&self) -> String {
                #source_name.to_owned()
            }

            fn default(&self) -> ::std::collections::HashMap<String, Self::Value> {
                #default_expr.into_iter().collect()
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
