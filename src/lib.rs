extern crate proc_macro;

extern crate syn;

#[macro_use]
extern crate quote;

use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::{Attribute, Body, DeriveInput, Field, Ident, MetaItem, NestedMetaItem};

const EXCLUDE: &str = "intermediate_exclude";
const DERIVE: &str = "intermediate_derive";
const OVERRIDE_TABLE_NAME: &str = "intermediate_table_name";
const DIESEL_TABLE_NAME: &str = "table_name";

#[proc_macro_derive(DieselIntermediate,
                    attributes(intermediate_exclude, intermediate_derive,
                                 intermediate_table_name))]
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

    let table_name_attr = extract_table_name_attr(&ast.attrs);
    let (common_fields, intermediates) = extract_intermediates(fields);

    let base_name = ast.ident.to_string();

    let (impl_generics, _ty_generics, where_clause) = ast.generics.split_for_impl();

    build_items(
        &common_fields,
        &intermediates,
        &derive_attr,
        &table_name_attr,
        &base_name,
        &impl_generics,
        where_clause,
    )
}

/// Extract the table name
///
/// set by either `#[intermediate_table_name]` or `#[table_name]`, with
/// intermediate... having higher priority
fn extract_table_name_attr(attrs: &[Attribute]) -> Option<Attribute> {
    let mut found = None;
    for attr in attrs {
        match attr.value {
            MetaItem::NameValue(ref ident, ref literal) if ident == OVERRIDE_TABLE_NAME => {
                let table_name_attr = format!(r#"#[table_name = {}]"#, quote!(#literal));

                return Some(syn::parse_outer_attr(&table_name_attr).unwrap());
            },
            MetaItem::NameValue(ref ident, _) if ident == DIESEL_TABLE_NAME => {
                found = Some(attr.clone());
            },
            MetaItem::List(ref ident, _) if ident == OVERRIDE_TABLE_NAME => {
                panic!(r#"expected [.. = "<table-name>"], not: {}"#, quote!(#attr));
            }
            _ => {}
        }
    }

    found
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

/// Return the attrs, without any that have the `to_strip` ident
fn strip_attr(attrs: &[Attribute], to_strip: &str) -> Vec<Attribute> {
    attrs
        .iter()
        .cloned()
        .filter(|a| match a.value {
            MetaItem::Word(ref ident) if ident == to_strip => false,
            MetaItem::List(ref ident, ..) if ident == to_strip => false,
            _ => true,
        })
        .collect::<Vec<_>>()
}

fn extract_items(attrs: &[Attribute], attr: &str) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|a| match a.value {
            MetaItem::List(ref ident, ref vals) if ident == attr => Some(vals),
            _ => None,
        })
        .flat_map(|list_items| {
            list_items.into_iter().map(|item| {
                if let &NestedMetaItem::MetaItem(MetaItem::Word(ref val)) = item {
                    val.to_string()
                } else {
                    panic!("Unexpected format for item: {} ", quote!(#item));
                }
            })
        })
        .collect::<Vec<_>>()
}

enum ExcludeAttr<'a> {
    Excluded,
    Intermediate(&'a str, Field),
    Included,
}

fn extract_intermediates(fields: &[Field]) -> (Vec<&Field>, HashMap<String, Vec<Field>>) {
    let mut subtypes = HashMap::new();

    // Collect the fields that aren't decorated with "exclude"
    let common_fields = fields
        .iter()
        .filter(|f| {
            use ExcludeAttr::*;
            // If any of this fields attrs are "exclude" then we want to strip the entire field
            match field_status(f) {
                Excluded => false,
                Intermediate(intermediate_prefix, field) => {
                    subtypes
                        .entry(intermediate_prefix.to_string())
                        .or_insert_with(Vec::new)
                        .push(field);
                    false
                }
                Included => true,
            }
        })
        .collect::<Vec<_>>();
    (common_fields, subtypes)
}

fn field_status(field: &Field) -> ExcludeAttr {
    use ExcludeAttr::*;
    for a in &field.attrs {
        match a.value {
            MetaItem::Word(ref ident) if ident == EXCLUDE => {
                return Excluded;
            }
            MetaItem::List(ref ident, ref vals) if ident == EXCLUDE && vals.len() == 1 => {
                // but, if the field is marked with some prefix, then we
                // want to store it to be used in the Prefix struct
                if let Some(&NestedMetaItem::MetaItem(MetaItem::Word(ref val))) = vals.get(0) {
                    let mut field_without_attr = (*field).clone();
                    field_without_attr.attrs = strip_attr(&field.attrs, EXCLUDE);
                    return Intermediate(val.as_ref(), field_without_attr);
                } else {
                    panic!(
                        "Unexpected shape for attribute: {} over {}",
                        quote!(#vals),
                        quote!(#field)
                    );
                }
            }
            MetaItem::List(ref ident, ref vals) if ident == EXCLUDE => panic!(
                "Cannot handle more than one intermediate type yet: {}",
                quote! { #ident(#(#vals),*) }
            ),
            MetaItem::NameValue(..) | MetaItem::Word(..) | MetaItem::List(..) => {
                // If it's not an EXCLUDE attr we don't need to do anything to it
            }
        }
    }
    // if we never encountered an EXCLUDE attr then it's still included
    return Included;
}
