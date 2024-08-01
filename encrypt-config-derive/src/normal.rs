use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub(crate) fn derive_normal_source(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics ::encrypt_config::source::NormalSource for #name #ty_generics #where_clause { }

        impl #impl_generics ::encrypt_config::source::Cacheable for #name #ty_generics #where_clause {
            fn load() -> ::std::io::Result<Self>
            where
                Self: Sized,
            {
                Ok(Self::default())
            }

            fn store(&self) -> ::std::io::Result<()> {
                Ok(())
            }

            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }
        }
    };

    TokenStream::from(expanded)
}
