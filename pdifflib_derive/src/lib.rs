#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, ExprArray, LitStr};

// Accept literals with an optional unary minus, but never arbitrary expressions.
// Check finiteness here so generated numeric schema entries remain valid JSON.
fn numeric_literal(value: Expr) -> syn::Result<Expr> {
    let literal = match &value {
        Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => &*unary.expr,
        value => value,
    };
    let finite = match literal {
        Expr::Lit(literal) => match &literal.lit {
            syn::Lit::Float(value) => value.base10_parse::<f64>()?.is_finite(),
            syn::Lit::Int(value) => value.base10_parse::<f64>()?.is_finite(),
            _ => false,
        },
        _ => false,
    };
    if !finite {
        return Err(syn::Error::new_spanned(
            value,
            "expected a finite numeric literal, optionally preceded by '-'",
        ));
    }
    Ok(value)
}

/// Implements `pdifflib::field::Field` for a struct with an `f: Array2<f64>` field.
#[proc_macro_derive(Field, attributes(field))]
pub fn derive_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let name = input.ident;
    let mut field_name = name.to_string();

    for attr in input.attrs {
        if attr.path().is_ident("field") {
            if let Err(error) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let value: syn::LitStr = value.parse()?;
                    field_name = value.value();
                    return Ok(());
                }
                Err(meta.error("unsupported field attribute"))
            }) {
                return error.to_compile_error().into();
            }
        }
    }

    let expanded = quote! {
        impl #impl_generics ::pdifflib::field::Field for #name #ty_generics #where_clause {
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

/// Describes named `f64`, `usize`, `bool`, and `String` parameters.
/// Numeric attributes accept integer or floating-point literals with an optional
/// unary minus. Every field requires `#[parameter(description = "...")]`.
#[proc_macro_derive(ParameterSchema, attributes(parameter))]
pub fn derive_parameter_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let name = input.ident;
    let syn::Data::Struct(data) = input.data else {
        return syn::Error::new_spanned(name, "ParameterSchema can only be derived for structs")
            .to_compile_error()
            .into();
    };
    let syn::Fields::Named(fields) = data.fields else {
        return syn::Error::new_spanned(name, "ParameterSchema requires named fields")
            .to_compile_error()
            .into();
    };

    let mut parameters = Vec::new();
    for field in fields.named {
        let Some(field_name) = field.ident else {
            continue;
        };
        let mut description = None;
        let mut minimum = None;
        let mut exclusive_minimum = None;
        let mut allowed_values = None;
        let mut default = None;

        for attribute in field.attrs {
            if !attribute.path().is_ident("parameter") {
                continue;
            }
            if let Err(error) = attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("description") {
                    let value: LitStr = meta.value()?.parse()?;
                    description = Some(value);
                    return Ok(());
                }
                if meta.path.is_ident("minimum") {
                    minimum = Some(numeric_literal(meta.value()?.parse()?)?);
                    return Ok(());
                }
                if meta.path.is_ident("exclusive_minimum") {
                    exclusive_minimum = Some(numeric_literal(meta.value()?.parse()?)?);
                    return Ok(());
                }
                if meta.path.is_ident("allowed_values") {
                    let values: ExprArray = meta.value()?.parse()?;
                    let mut parsed_values = Vec::new();
                    for value in values.elems {
                        parsed_values.push(numeric_literal(value)?);
                    }
                    allowed_values = Some(parsed_values);
                    return Ok(());
                }
                if meta.path.is_ident("default") {
                    default = Some(numeric_literal(meta.value()?.parse()?)?);
                    return Ok(());
                }
                Err(meta.error("unsupported parameter attribute"))
            }) {
                return error.to_compile_error().into();
            }
        }

        let Some(description) = description else {
            return syn::Error::new_spanned(
                field_name,
                "every parameter needs #[parameter(description = \"...\")]",
            )
            .to_compile_error()
            .into();
        };
        let value_type = match &field.ty {
            syn::Type::Path(path) if path.path.is_ident("f64") => "number",
            syn::Type::Path(path) if path.path.is_ident("usize") => "integer",
            syn::Type::Path(path) if path.path.is_ident("bool") => "boolean",
            syn::Type::Path(path) if path.path.is_ident("String") => "string",
            _ => {
                return syn::Error::new_spanned(
                    field.ty,
                    "ParameterSchema supports f64, usize, bool, and String fields",
                )
                .to_compile_error()
                .into()
            }
        };
        let field_name = field_name.to_string();
        let minimum = minimum
            .map(|value| quote!(Some((#value) as f64)))
            .unwrap_or_else(|| quote!(None));
        let exclusive_minimum = exclusive_minimum
            .map(|value| quote!(Some((#value) as f64)))
            .unwrap_or_else(|| quote!(None));
        let allowed_values = allowed_values
            .map(|values| quote!(Some(&[#((#values) as f64),*])))
            .unwrap_or_else(|| quote!(None));
        let default = default
            .map(|value| quote!(Some((#value) as f64)))
            .unwrap_or_else(|| quote!(None));
        parameters.push(quote! {
            ::pdifflib::params::Parameter {
                name: #field_name,
                value_type: #value_type,
                description: #description,
                minimum: #minimum,
                exclusive_minimum: #exclusive_minimum,
                allowed_values: #allowed_values,
                default: #default,
            }
        });
    }

    let expanded = quote! {
        impl #impl_generics ::pdifflib::params::ParameterSchema for #name #ty_generics #where_clause {
            fn parameters() -> &'static [::pdifflib::params::Parameter] {
                const PARAMETERS: &[::pdifflib::params::Parameter] = &[
                    #(#parameters),*
                ];
                PARAMETERS
            }
        }
    };

    TokenStream::from(expanded)
}

/// Describes named `f64`, `bool`, and `String` log columns.
/// Every field requires `#[logging(description = "...")]`.
#[proc_macro_derive(LoggingSchema, attributes(logging))]
pub fn derive_logging_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let name = input.ident;
    let syn::Data::Struct(data) = input.data else {
        return syn::Error::new_spanned(name, "LoggingSchema can only be derived for structs")
            .to_compile_error()
            .into();
    };
    let syn::Fields::Named(fields) = data.fields else {
        return syn::Error::new_spanned(name, "LoggingSchema requires named fields")
            .to_compile_error()
            .into();
    };

    let mut columns = Vec::new();
    for field in fields.named {
        let Some(field_name) = field.ident else {
            continue;
        };
        let mut description = None;

        for attribute in field.attrs {
            if !attribute.path().is_ident("logging") {
                continue;
            }
            if let Err(error) = attribute.parse_nested_meta(|meta| {
                if meta.path.is_ident("description") {
                    let value: LitStr = meta.value()?.parse()?;
                    description = Some(value);
                    return Ok(());
                }
                Err(meta.error("unsupported logging attribute"))
            }) {
                return error.to_compile_error().into();
            }
        }

        let Some(description) = description else {
            return syn::Error::new_spanned(
                field_name,
                "every log column needs #[logging(description = \"...\")]",
            )
            .to_compile_error()
            .into();
        };
        let value_type = match &field.ty {
            syn::Type::Path(path) if path.path.is_ident("f64") => "number",
            syn::Type::Path(path) if path.path.is_ident("bool") => "boolean",
            syn::Type::Path(path) if path.path.is_ident("String") => "string",
            _ => {
                return syn::Error::new_spanned(
                    field.ty,
                    "LoggingSchema supports f64, bool, and String fields",
                )
                .to_compile_error()
                .into()
            }
        };
        let field_name = field_name.to_string();
        columns.push(quote! {
            ::pdifflib::params::LogColumn {
                name: #field_name,
                value_type: #value_type,
                description: #description,
                minimum: None,
                exclusive_minimum: None,
                allowed_values: None,
                default: None,
            }
        });
    }

    let expanded = quote! {
        impl #impl_generics ::pdifflib::params::LoggingSchema for #name #ty_generics #where_clause {
            fn columns() -> &'static [::pdifflib::params::LogColumn] {
                const COLUMNS: &[::pdifflib::params::LogColumn] = &[
                    #(#columns),*
                ];
                COLUMNS
            }
        }
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::numeric_literal;

    #[test]
    fn accepts_signed_integer_and_float_literals() {
        for value in ["1", "-1", "0.25", "-0.25", "-1e-3", "2usize"] {
            assert!(
                numeric_literal(syn::parse_str(value).unwrap()).is_ok(),
                "{value}"
            );
        }
    }

    #[test]
    fn rejects_expressions_and_nonfinite_literals() {
        for value in ["true", "\"1\"", "f64::NAN", "1 + 2", "-value", "1e999"] {
            assert!(
                numeric_literal(syn::parse_str(value).unwrap()).is_err(),
                "{value}"
            );
        }
    }
}
