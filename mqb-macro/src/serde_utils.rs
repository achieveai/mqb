use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma};

pub enum SerdeFieldRenameStrategy {
    CamelCase,
    FieldSpecific,
    PascalCase,
    ScreamingSnakeCase,
}

pub fn get_ser_field_names(
    fields: &Punctuated<syn::Field, Comma>,
    rename_strategy: SerdeFieldRenameStrategy,
) -> Vec<syn::Ident> {
    fields
        .iter()
        .map(|field| {
            let ident = field.ident.clone().unwrap();

            let mut final_field_ident = syn::Ident::new(
                match rename_strategy {
                    SerdeFieldRenameStrategy::PascalCase => {
                        ident.to_string().to_case(Case::Pascal)
                    }
                    SerdeFieldRenameStrategy::ScreamingSnakeCase => {
                        ident.to_string().to_case(Case::ScreamingSnake)
                    }
                    SerdeFieldRenameStrategy::CamelCase => {
                        ident.to_string().to_case(Case::Camel)
                    }
                    SerdeFieldRenameStrategy::FieldSpecific => {
                        ident.to_string()
                    }
                }
                .as_str(),
                ident.span(),
            );

            for attr in &field.attrs {
                if attr.path().is_ident("serde")
                    && attr.path().segments.len() == 1
                {
                    _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("rename") {
                            let v = meta.value()?;
                            let t: syn::LitStr = v.parse()?;
                            final_field_ident = syn::Ident::new(
                                t.value().as_str(),
                                ident.clone().span(),
                            );
                        }

                        Ok(())
                    });
                }
            }

            final_field_ident
        })
        .collect()
}

pub fn get_struct_field_rename_strategy(
    input: &syn::DeriveInput,
) -> syn::Result<SerdeFieldRenameStrategy> {
    let mut rv = Ok(SerdeFieldRenameStrategy::FieldSpecific);

    for attr in &input.attrs {
        if !attr.path().is_ident("serde") || attr.path().segments.len() != 1 {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let v = meta.value()?;
                let t: syn::LitStr = v.parse()?;
                rv = match t.value().as_str() {
                    "PascalCase" => Ok(SerdeFieldRenameStrategy::PascalCase),
                    "SCREAMING_SNAKE_CASE" => {
                        Ok(SerdeFieldRenameStrategy::ScreamingSnakeCase)
                    }
                    "camelCase" => Ok(SerdeFieldRenameStrategy::CamelCase),
                    x => Err(syn::Error::new(
                        attr.span(),
                        format!("Unknown sede field rename strategy for {}", x),
                    )),
                }
            }

            Ok(())
        })
        .unwrap();
    }

    rv
}

pub fn get_serializers(
    fields: &Punctuated<syn::Field, Comma>,
) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let mut serialize_with: std::option::Option<String> = None;

            for attr in &field.attrs {
                if attr.path().is_ident("serde")
                    && attr.path().segments.len() == 1
                {
                    _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("serialize_with") {
                            let v = meta.value()?;
                            let t: syn::LitStr = v.parse()?;
                            serialize_with = Some(t.value().to_string());
                        } else if meta.path.is_ident("with") {
                            let v = meta.value()?;
                            let t: syn::LitStr = v.parse()?;
                            serialize_with = Some(format!(
                                "{}::serialize",
                                t.value().as_str()
                            ));
                        }

                        Ok(())
                    });
                }
            }

            if let Some(f) = serialize_with {
                let path = syn::parse_str::<syn::Path>(&f).unwrap();
                quote! {
                    {
                        |v| #path(v, bson::Serializer::new())
                    }
                }
            } else {
                quote! {
                    {
                        bson::to_bson
                    }
                }
            }
        })
        .collect()
}

pub fn get_underlying_types(
    fields: &Punctuated<syn::Field, Comma>,
) -> Vec<syn::Type> {
    fields
        .iter()
        .zip(get_serialize_with_paths(fields))
        .map(|(field, serialize_fn)| match serialize_fn {
            None => field.ty.clone(),
            Some(serialize_with) => match serialize_with.as_str() {
                "bson::serde_helpers::chrono_datetime_as_bson_datetime" => {
                    syn::parse_str::<syn::Type>("bson::DateTime").unwrap()
                }
                _ => syn::parse_str::<syn::Type>(
                    "mqb_core::kp::UnknownUnderlyingType",
                )
                .unwrap(),
            },
        })
        .collect()
}

pub fn get_serialize_with_paths(
    fields: &Punctuated<syn::Field, Comma>,
) -> Vec<Option<String>> {
    fields
        .iter()
        .map(|field| {
            let mut serialize_with: std::option::Option<String> = None;

            for attr in &field.attrs {
                if attr.path().is_ident("serde")
                    && attr.path().segments.len() == 1
                {
                    _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("serialize_with") {
                            let v = meta.value()?;
                            let t: syn::LitStr = v.parse()?;
                            serialize_with = Some(t.value().to_string());
                        } else if meta.path.is_ident("with") {
                            let v = meta.value()?;
                            let t: syn::LitStr = v.parse()?;
                            serialize_with =
                                Some(t.value().as_str().to_string());
                        }

                        Ok(())
                    });
                }
            }

            serialize_with
        })
        .collect()
}
