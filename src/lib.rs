extern crate proc_macro;

extern crate syn;

#[macro_use]
extern crate quote;

use std::collections::HashMap;

use proc_macro::TokenStream;
use syn::{Attribute, Body, DeriveInput, Field, Ident, MetaItem, NestedMetaItem};

const ATTR: &str = "diesel_intermediate_exclude";

#[proc_macro_derive(
    DieselIntermediate, attributes(diesel_intermediate_exclude))]
pub fn diesel_intermediate_fields(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    let ast = syn::parse_derive_input(&source).unwrap();

    let expanded = expand_diesel_intermediate_fields(&ast);

    expanded.parse().unwrap()
}

fn expand_diesel_intermediate_fields(ast: &DeriveInput) -> quote::Tokens {
    let fields = match ast.body {
        Body::Struct(ref data) => data.fields(),
        Body::Enum(_) => panic!("#[derive(DieselIntermediate)] can only be used with structs"),
    };

    let (new_fields, kinds) = extract_subtypes(fields);

    let base_name = ast.ident.to_string();

    let hidden_name = Ident::new("New".to_owned() + &base_name);

    let (impl_generics, _ty_generics, where_clause) = ast.generics.split_for_impl();

    let these_fields = new_fields.clone();
    let mut new_structs = quote! {
        struct #hidden_name #impl_generics #where_clause {
            #(#these_fields),*
        }
    };

    for (prefix, extra_fields) in &kinds {
        let this_name = Ident::new(prefix.clone() + &base_name);
        let these_fields = new_fields.clone();
        new_structs = quote! {
            #new_structs

            struct #this_name #impl_generics #where_clause {
                #(#extra_fields),* ,
                #(#these_fields),*
            }
        }
    }
    println!("new_structs: {}", quote!(#new_structs));
    new_structs
}

fn strip_this_attr(field: &Field) -> Vec<Attribute> {
    field.attrs.iter().cloned().filter(|a| match a.value {
        MetaItem::Word(ref ident) if ident == ATTR => false,
        MetaItem::List(ref ident, ..) if ident == ATTR => false,
        _ => true,
    }).collect::<Vec<_>>()
}

fn extract_subtypes(fields: &[Field]) -> (Vec<&Field>, HashMap<String, Vec<Field>>) {
    let mut subtypes = HashMap::new();
    let new_fields = fields
        .iter()
        .filter(|f| {
            f.attrs.iter().any(|a| match a.value {
                MetaItem::Word(ref ident) if ident == ATTR => false,
                MetaItem::List(ref ident, ref vals) if ident == ATTR && vals.len() == 1 => {
                    if let Some(&NestedMetaItem::MetaItem(MetaItem::Word(ref val))) = vals.get(0) {
                        let mut field_without_attr = (*f).clone();
                        println!("field with attr: {}", quote!(#field_without_attr));
                        field_without_attr.attrs = strip_this_attr(f);
                        println!("field WITHOUT attr: {}", quote!(#field_without_attr));
                        subtypes
                            .entry(val.to_string())
                            .or_insert_with(Vec::new)
                            .push(field_without_attr);
                        false
                    } else {
                        panic!("Unexected type in attribute: {}", quote!(vals));
                    }
                }
                MetaItem::List(ref ident, ref vals) if ident == ATTR => {
                    panic!("Cannot handle more than one intermediate type yet: {}",
                           quote! { #ident(#(#vals),*) })
                }
                ref raw => {
                    println!("in raw: {}", quote!(#raw));
                    true
                }
            })
        })
        .collect::<Vec<_>>();
    (new_fields, subtypes)
}
