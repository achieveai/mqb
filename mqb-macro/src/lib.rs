use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

mod serde_utils;
use serde_utils::*;

#[proc_macro_derive(LeafKeyPathable)]
pub fn derive_leaf_keypathable(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let item_name = match input.data {
        Data::Struct(_) => input.ident,
        Data::Enum(_) => input.ident,
        _ => panic!("expected a struct or enum"),
    };

    quote! {
        impl mqb_core::kp::KeyPathable for #item_name {
            type KeyPathNode<Parent: mqb_core::kp::KeyPathNodeLike, UnderlyingType> = mqb_core::kp::TerminalKeyPathNode<Parent, UnderlyingType>;
        }
    }
    .into()
}

#[proc_macro_derive(KeyPathable)]
pub fn derive_keypathable(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand_derive_keypathable(input).into()
}

fn expand_derive_keypathable(input: DeriveInput) -> TokenStream {
    // 1. KeyPathNode for the struct.
    //    - Contains methods for each of the struct's fields.
    // 2. impl KeyPathNode for the constructed struct (method to return an instance of the struct)
    // 3. impl KeyPathable for the input struct, associating it with the constructed KeyPathNode.
    // 4. A method for each of the fields of the struct.

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    let field_name =
        fields.iter().map(|field| &field.ident).collect::<Vec<_>>();

    let root_rename_strategy =
        get_struct_field_rename_strategy(&input).unwrap();

    let serde_field_name_str =
        get_ser_field_names(fields, root_rename_strategy)
            .iter()
            .map(|field| format!("{}", field))
            .collect::<Vec<_>>();

    let field_type = fields.iter().map(|field| &field.ty).collect::<Vec<_>>();
    let underlying_types = get_underlying_types(fields);

    let serializers = get_serializers(fields);

    let struct_name = &input.ident;

    let key_path_node_name = syn::Ident::new(
        &format!("{}KeyPathNode", struct_name),
        proc_macro2::Span::call_site(),
    );

    let key_path_root_node_name = syn::Ident::new(
        &format!("{}KeyPathRootNode", struct_name),
        proc_macro2::Span::call_site(),
    );

    quote! {
        pub struct #key_path_node_name<Parent, UnderlyingType = #struct_name> where Parent: mqb_core::kp::KeyPathNodeLike {
            key: &'static str,
            parent: Parent,
            serializer: mqb_core::kp::SerializeFn<#struct_name>,
            marker: std::marker::PhantomData<UnderlyingType>,
        }

        impl<Parent: mqb_core::kp::KeyPathNodeLike, UnderlyingType> Clone for #key_path_node_name<Parent, UnderlyingType> {
            fn clone(&self) -> Self {
                #key_path_node_name {
                    key: self.key,
                    parent: self.parent.clone(),
                    serializer: self.serializer,
                    marker: std::marker::PhantomData,
                }
            }
        }

        impl<Parent: mqb_core::kp::KeyPathNodeLike, UnderlyingType> mqb_core::kp::KeyPathNodeLike for #key_path_node_name<Parent, UnderlyingType> {
            type Origin = Parent::Origin;
            type Current = #struct_name;

            fn render_path(&self) -> String {
                if Parent::IS_ROOT {
                    self.key.to_owned()
                } else {
                    format!("{}.{}", self.parent.render_path(), self.key)
                }
            }
        }

        impl<Parent: mqb_core::kp::KeyPathNodeLike, UnderlyingType> mqb_core::kp::KeyPathNonInitialNodeLike for #key_path_node_name<Parent, UnderlyingType> {

            type ParentNodeTy = Parent;
            type UnderlyingType = UnderlyingType;

            fn instance(key: &'static str, serializer: mqb_core::kp::SerializeFn<Self::Current>, parent: Self::ParentNodeTy) -> Self {
                #key_path_node_name {
                    key,
                    parent,
                    serializer,
                    marker: std::marker::PhantomData,
                }
            }

            fn key(&self) -> String {
                self.key.to_owned()
            }

            fn parent(&self) -> &Self::ParentNodeTy {
                &self.parent
            }

            fn serializer(&self) -> mqb_core::kp::SerializeFn<Self::Current> {
                self.serializer
            }
        }

        impl mqb_core::kp::KeyPathable for #struct_name {
            type KeyPathNode<Parent: mqb_core::kp::KeyPathNodeLike, UnderlyingType> = #key_path_node_name<Parent, UnderlyingType>;
        }

        #[derive(Clone)]
        pub struct #key_path_root_node_name {
        }

        impl mqb_core::kp::KeyPathNodeLike for #key_path_root_node_name {
            type Origin = #struct_name;
            type Current = #struct_name;

            const IS_ROOT: bool = true;

            fn render_path(&self) -> String {
                "".to_owned()
            }
        }

        impl mqb_core::kp::KeyPathInitialNodeLike for #key_path_root_node_name {
        }

        impl mqb_core::kp::KeyPathableAsRoot for #struct_name {
            type RootKeyPathNode = #key_path_root_node_name;

            fn kp() -> #key_path_root_node_name {
                #key_path_root_node_name {}
            }
        }

        impl<Parent: mqb_core::kp::KeyPathNodeLike> #key_path_node_name<Parent, #struct_name> {
            #(
                pub fn #field_name(self) -> <#field_type as mqb_core::kp::KeyPathable>::KeyPathNode<Self, #underlying_types> {
                    use mqb_core::kp::*;
                    <#field_type as mqb_core::kp::KeyPathable>::KeyPathNode::<Self, #underlying_types>::instance(#serde_field_name_str, #serializers, self)
                }
            )*
        }

        impl #key_path_root_node_name {
            #(
                pub fn #field_name(self) -> <#field_type as mqb_core::kp::KeyPathable>::KeyPathNode<Self, #underlying_types> {
                    use mqb_core::kp::*;
                    <#field_type as mqb_core::kp::KeyPathable>::KeyPathNode::<Self, #underlying_types>::instance(#serde_field_name_str, #serializers, self)
                }
            )*
        }
    }
}
