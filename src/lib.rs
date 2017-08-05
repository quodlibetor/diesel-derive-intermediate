extern crate proc_macro;

extern crate syn;

#[macro_use]
extern crate quote;

use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::{Attribute, Body, DeriveInput, Field, Ident, MetaItem, NestedMetaItem};

const EXCLUDE: &str = "diesel_intermediate_exclude";
const DERIVE: &str = "diesel_intermediate_derive";
const TABLE_NAME: &str = "intermediate_table_name";

#[proc_macro_derive(
    DieselIntermediate, attributes(
        diesel_intermediate_exclude,
        diesel_intermediate_derive,
        intermediate_table_name,
    ))]
pub fn diesel_intermediate_fields(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    let ast = syn::parse_derive_input(&source).unwrap();

    let expanded = expand_diesel_intermediate_fields(&ast);

    expanded.parse().unwrap()
}

fn expand_diesel_intermediate_fields(ast: &DeriveInput) -> Tokens {
    let fields = match ast.body {
        Body::Struct(ref data) => data.fields(),
        Body::Enum(_) => panic!("#[derive(DieselIntermediate)] can only be used with structs"),
    };

    // look, you gotta do what you gotta do.
    // I know that I don't gotta do this but it's easy and it works.
    let derives = extract_items(&ast.attrs, DERIVE);
    let derive_attr = format!("#[derive({})]", derives.join(","));
    let derive_attr = syn::parse_outer_attr(&derive_attr).unwrap();

    let insert_table_name = extract_items(&ast.attrs, TABLE_NAME);
    let table_name_attr = if insert_table_name.len() > 0 {
        let table_name_attr = format!(r#"#[table_name = "{}"]"#, insert_table_name.join(","));
        Some(syn::parse_outer_attr(&table_name_attr).unwrap())
    } else {
        None
    };
    let (common_fields, intermediates) = extract_intermediates(fields);

    let base_name = ast.ident.to_string();

    let (impl_generics, _ty_generics, where_clause) = ast.generics.split_for_impl();

    build_items(
        &common_fields, &intermediates, &derive_attr, &table_name_attr, &base_name,
        &impl_generics, where_clause)
}

fn build_items(
    common_fields: &[&Field],
    intermediates: &HashMap<String, Vec<Field>>,
    derive_attr: &Attribute,
    table_name_attr: &Option<Attribute>,
    base_name: &str,
    impl_generics: &syn::ImplGenerics,
    where_clause: &syn::WhereClause,
) -> quote::Tokens {
    let new_name = Ident::new("New".to_owned() + base_name);
    let mut new_structs = quote! {
        #derive_attr
        #table_name_attr
        struct #new_name #impl_generics #where_clause {
            #(#common_fields),*
        }
    };

    for (prefix, extra_fields) in intermediates {
        let this_name = Ident::new(prefix.clone() + &base_name);
        new_structs = quote! {
            #new_structs

            #derive_attr
            #table_name_attr
            struct #this_name #impl_generics #where_clause {
                #(#extra_fields),* ,
                #(#common_fields),*
            }
        }
    }
    new_structs
}

fn strip_attr(field: &Field, attr: &str) -> Vec<Attribute> {
    field.attrs.iter().cloned().filter(|a| match a.value {
        MetaItem::Word(ref ident) if ident == attr => false,
        MetaItem::List(ref ident, ..) if ident == attr => false,
        _ => true,
    }).collect::<Vec<_>>()
}

fn extract_items(attrs: &[Attribute], attr: &str) -> Vec<String> {
    attrs.iter().filter_map(|a| match a.value {
        MetaItem::List(ref ident, ref vals) if ident == attr => Some(vals),
        _ => None,
    }).flat_map(|list_items| {
        list_items.into_iter().map(|item| {
            if let &NestedMetaItem::MetaItem(MetaItem::Word(ref val)) = item {
                val.to_string()
            } else {
                panic!("Unexpected format for item: {} ", quote!(#item));
            }
        })
    }).collect::<Vec<_>>()
}

fn extract_intermediates(fields: &[Field]) -> (Vec<&Field>, HashMap<String, Vec<Field>>) {
    let mut subtypes = HashMap::new();

    // Collect the fields that aren't decorated with "exclude"
    let common_fields = fields
        .iter()
        .filter(|f| {
            if f.attrs.len() == 0 {
                return true;
            }
            // If any of this fields attrs are "exclude" then we want to strip the entire field
            f.attrs.iter().any(|a| match a.value {
                MetaItem::Word(ref ident) if ident == EXCLUDE => {
                    false
                },
                MetaItem::List(ref ident, ref vals) if ident == EXCLUDE && vals.len() == 1 => {
                    // but, if the field is marked with some prefix, then we
                    // want to store it to be used in the Prefix struct
                    if let Some(&NestedMetaItem::MetaItem(MetaItem::Word(ref val))) = vals.get(0) {
                        let mut field_without_attr = (*f).clone();
                        field_without_attr.attrs = strip_attr(f, EXCLUDE);
                        subtypes
                            .entry(val.to_string())
                            .or_insert_with(Vec::new)
                            .push(field_without_attr);
                        false
                    } else {
                        panic!("Unexpected shape for attribute: {} over {}",
                               quote!(#vals), quote!(#f));
                    }
                }
                MetaItem::List(ref ident, ref vals) if ident == EXCLUDE => {
                    panic!("Cannot handle more than one intermediate type yet: {}",
                           quote! { #ident(#(#vals),*) })
                }
                MetaItem::NameValue(..) | MetaItem::Word(_) | MetaItem::List(..) => true,
            })
        })
        .collect::<Vec<_>>();
    (common_fields, subtypes)
}
