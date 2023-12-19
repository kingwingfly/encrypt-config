use proc_macro::TokenStream;
use quote::quote;
use syn::{parenthesized, parse_macro_input, DeriveInput, Expr, LitStr};

enum SourceType {
    Normal,
    Persist,
    Secret,
}

#[proc_macro_derive(Source, attributes(source))]
pub fn derive_source(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut default_expr: Expr = syn::parse_str("Self::Map::default()").unwrap();
    let mut kind = SourceType::Normal;
    let mut path: Option<LitStr> = None;
    let mut source_name: Option<LitStr> = None;

    if let Some(attr) = input
        .attrs
        .iter()
        .find(|&attr| attr.path().is_ident("source"))
    {
        attr.parse_nested_meta(|meta| {
            match meta.path.get_ident() {
                Some(i) if i == "default" => {
                    let content;
                    parenthesized!(content in meta.input);
                    default_expr = content.parse()?;
                }
                Some(i) if i == "normal" => kind = SourceType::Normal,
                Some(i) if i == "persist" => kind = SourceType::Persist,
                Some(i) if i == "secret" => kind = SourceType::Secret,
                _ => {}
            };
            Ok(())
        })
        .expect("");
    };

    let expanded = match kind {
        SourceType::Normal => quote! {
            impl #impl_generics encrypt_config::Source for #name #ty_generics #where_clause {
                type Value = String;
                type Map = Vec<(String, Self::Value)>;

                fn collect(&self) -> Result<Self::Map, Box<dyn std::error::Error>> {
                    Ok(#default_expr.into_iter().collect())
                }
            }
        },
        SourceType::Persist => todo!(),
        SourceType::Secret => todo!(),
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
