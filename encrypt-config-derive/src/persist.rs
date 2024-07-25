use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr};

pub(crate) fn derive_persist_source(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut path_or_name: Option<Expr> = None;

    if let Some(attr) = input
        .attrs
        .iter()
        .find(|&attr| attr.path().is_ident("source"))
    {
        attr.parse_nested_meta(|meta| {
            match &meta.path {
                #[cfg(not(feature = "default_config_dir"))]
                path if path.is_ident("path") => {
                    let value = meta.value()?; // this parses the `=`
                    path_or_name = value.parse().ok();
                }
                #[cfg(feature = "default_config_dir")]
                path if path.is_ident("name") => {
                    let value = meta.value()?; // this parses the `=`
                    path_or_name = value.parse().ok();
                }
                _ => Err(meta.error("unsupported attribute"))?,
            }
            Ok(())
        })
        .unwrap();
    };

    if path_or_name.is_none() {
        #[cfg(feature = "default_config_dir")]
        panic!("`#[source(name = \"...\")]` is required.");
        #[cfg(not(feature = "default_config_dir"))]
        panic!("`#[source(path = \"...\")]` is required.");
    }

    #[cfg(not(feature = "default_config_dir"))]
    let persist_source_impl = quote! {
        impl #impl_generics ::encrypt_config::source::PersistSource for #name #ty_generics #where_clause {
            const PATH: &'static str = #path_or_name;
        }
    };

    #[cfg(feature = "default_config_dir")]
    let persist_source_impl = quote! {
        impl #impl_generics ::encrypt_config::source::PersistSource for #name #ty_generics #where_clause {
            const NAME: &'static str = #path_or_name;
        }
    };

    let expanded = quote! {
        #persist_source_impl

        impl #impl_generics ::encrypt_config::source::Source for #name #ty_generics #where_clause {
            fn load() -> ::encrypt_config::error::ConfigResult<Self>
            where
                Self: Sized,
            {
                <Self as ::encrypt_config::PersistSource>::load()
            }

            fn save(&self) -> ::encrypt_config::error::ConfigResult<()> {
                <Self as ::encrypt_config::PersistSource>::save(self)
            }
        }
    };

    TokenStream::from(expanded)
}
