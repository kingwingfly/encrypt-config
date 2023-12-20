use proc_macro::TokenStream;
use quote::quote;
use syn::{parenthesized, parse_macro_input, DeriveInput, Expr, Ident, LitStr};

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
                    _ => {}
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
                    _ => {}
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
                    _ => {}
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
