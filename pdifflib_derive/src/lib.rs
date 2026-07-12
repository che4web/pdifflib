use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Field, attributes(field))]
pub fn derive_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let mut field_name = name.to_string();

    for attr in input.attrs {
        if attr.path().is_ident("field") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let value: syn::LitStr = value.parse()?;
                    field_name = value.value();
                }
                Ok(())
            });
        }
    }

    let expanded = quote! {
        impl Field for #name {
            fn name(&self) -> &'static str {
                #field_name
            }

            fn get_f(&self) -> &ndarray::Array2<f64> {
                &self.f
            }

            fn get_f_mut(&mut self) -> &mut ndarray::Array2<f64> {
                &mut self.f
            }
        }
    };

    TokenStream::from(expanded)
}
