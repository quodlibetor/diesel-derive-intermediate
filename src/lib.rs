//! # Derive intermediate structs
//!
//! An intermediate struct is a struct that does not have all the data that a
//! "full" struct has. For example, you might receive requests that do not have
//! an ID until they are inserted into your database (because of an e.g.
//! AUTOINCREMENT primary key).
//!
//! Since having `Option<id>` fields is sort of wrong and definitely
//! unergonomic most of the time, it's reasonable to have a `NewStruct` that is
//! exactly the same as `Struct`, but without the `id` field.
//!
//! The goal of this crate is to provide nice ergonomics around supporting
//! intermediate structs, and to provide nice integration with
//! [`Diesel`](https://diesel.rs/).
//!
//! `diesel-derive-intermediate` provides several attributes or targets. See
//! the example below if the prose isn't clear:
//!
//! * The `DieselIntermediate` derive target which primarily generates
//!   *structs* (not Traits, which is what `#[derive]` is supposed to generate)
//!   works with a few attributes to provide field-exclusions on the generated
//!   structs.
//! * The `#[intermediate_exclude]` field-level attribute which comes in two
//!   forms:
//!   * `#[intermediate_exclude]` by itself, which marks the field as being
//!     excluded from the `NewStruct` struct, and for inclusion in the
//!     `from_new_struct` static method.
//!   * `#[intermediate_exclude(SomePrefix)]` excludes from the `NewStruct`
//!     generated struct, but causes a `SomePrefixStruct` to be generated,
//!     which *will* have this field.
//! * The `#[intermediate_derive(Traits...)]` struct-level attribute applies
//!   its contained traits to all the intermediate structs generated.
//! * `DieselIntermediate` will apply diesel's `#[table_name = "..."]`
//!   struct-level attribute to all generated structs, if you need to use a
//!   different table name you can use `#[intermediate_table_name = "..."]` to
//!   override the default.
//!
//! # Example
//!
//! Given:
//!
//! ```rust
//! # #[macro_use] extern crate diesel_derive_intermediate;
//! #[derive(DieselIntermediate)]
//! #[intermediate_derive(Debug)]
//! pub struct Mycologist {
//!     #[intermediate_exclude]
//!     id: i32,
//!     rust_count: i32,
//! }
//!
//! #[derive(DieselIntermediate)]
//! #[intermediate_derive(Debug, PartialEq)]
//! pub struct Rust {
//!     #[intermediate_exclude]
//!     id: i32,
//!     #[intermediate_exclude(Captured)]
//!     mycologist_id: i32,
//!     life_cycle_stage: i32,
//! }
//! # fn main() {}
//! ```
//!
//! The result will be:
//!
//! ```rust
//! pub struct Mycologist {
//!     id: i32,
//!     rust_count: i32,
//! }
//!
//! #[derive(Debug)]
//! pub struct NewMycologist {
//!     rust_count: i32,
//! }
//!
//! impl Mycologist {
//!     // The `pub` comes from the `pub` on `Mycologist`
//!     pub fn from_new_mycologist(id: i32, base: NewMycologist) -> Mycologist {
//!         Mycologist {
//!             id,
//!             rust_count: base.rust_count,
//!         }
//!     }
//! }
//!
//! pub struct Rust {
//!     id: i32,
//!     mycologist_id: i32,
//!     life_cycle_stage: i32,
//! }
//! #[derive(Debug, PartialEq)]
//! pub struct CapturedRust {
//!     mycologist_id: i32,
//!     life_cycle_stage: i32,
//! }
//!
//! #[derive(Debug, PartialEq)]
//! pub struct NewRust {
//!     life_cycle_stage: i32,
//! }
//!
//! // Convenience constructors that take just the parameters that exist in
//! // this intermediate and not the intermediate it came from.
//! impl Rust {
//!     pub fn from_captured_rust(id: i32, base: CapturedRust) -> Rust {
//!         Rust {
//!             id,
//!             mycologist_id: base.mycologist_id,
//!             life_cycle_stage: base.life_cycle_stage,
//!         }
//!     }
//!
//!     pub fn from_new_rust(id: i32, mycologist_id: i32, base: NewRust) -> Rust {
//!         Rust {
//!             id,
//!             mycologist_id,
//!             life_cycle_stage: base.life_cycle_stage,
//!         }
//!     }
//! }
//! ```
//!
//! see [`tests/diesel-interaction.rs`](tests/diesel-interaction.rs) for a
//! couple fully-worked examples, including using with `Insertable` and the
//! purpose of the `intermediate_exclude(NAME)` form.
//!
//! Interestingly, since this is abusing the derive proc-macro infrastructure,
//! if you have no `#[intermediate_derive(...)]` attributes, you will get
//! "empty trait list in \`derive\`" warnings.
//!
//! ## Limitations
//!
//! * It's not possible to derive multiple `Associations` for the same pair of
//!   tables, I think. This means that we can't derive `Associations` for the
//!   intermediate types. This seems basically fine, you really only want to be
//!   able to join on complete types that have actually been inserted into the
//!   DB, not partials that are in the process of getting built to be inserted.

extern crate proc_macro;

extern crate syn;

extern crate heck;
#[macro_use]
extern crate quote;

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use heck::SnakeCase;
use proc_macro::TokenStream;
use quote::Tokens;
use syn::{Attribute, Body, DeriveInput, Field, Ident, MetaItem, NestedMetaItem, Visibility};

const EXCLUDE: &str = "intermediate_exclude";
const DERIVE: &str = "intermediate_derive";
const OVERRIDE_TABLE_NAME: &str = "intermediate_table_name";
const DIESEL_TABLE_NAME: &str = "table_name";

#[doc(hidden)]
#[proc_macro_derive(
    DieselIntermediate,
    attributes(intermediate_exclude, intermediate_derive, intermediate_table_name)
)]
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
    let intermediates = extract_intermediates(fields);

    let base_name = ast.ident.to_string();

    let (impl_generics, _ty_generics, where_clause) = ast.generics.split_for_impl();

    build_items(
        &ast.vis,
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
            }
            MetaItem::NameValue(ref ident, _) if ident == DIESEL_TABLE_NAME => {
                found = Some(attr.clone());
            }
            MetaItem::List(ref ident, _) if ident == OVERRIDE_TABLE_NAME => {
                panic!(r#"expected [.. = "<table-name>"], not: {}"#, quote!(#attr));
            }
            _ => {}
        }
    }

    found
}

fn build_items(
    vis: &syn::Visibility,
    intermediates: &IntermediateFields,
    derive_attr: &Attribute,
    table_name_attr: &Option<Attribute>,
    // The name of the full struct that everything else is an intermediate for
    base_name: &str,
    impl_generics: &syn::ImplGenerics,
    where_clause: &syn::WhereClause,
) -> quote::Tokens {
    let new_name = Ident::new("New".to_owned() + base_name);
    let common_fields = &intermediates.common_fields;

    // accumulator for all the gnerated code
    let mut new_structs = quote!();

    // add the impl <type> { from_<intermediates>... }
    let field_difs = intermediates.field_differences_full();
    new_structs = add_from_impls(
        &Ident::new(base_name),
        &base_name,
        &intermediates,
        vis,
        field_difs,
        &new_structs,
    );

    // add the New<type> struct
    new_structs = quote! {
        #new_structs

        #derive_attr
        #table_name_attr
        #vis struct #new_name #impl_generics #where_clause {
            #(#common_fields),*
        }
    };

    // add the same as above but for every extra intermediate
    for (prefix, extra_fields) in &intermediates.prefix_excluded {
        let this_name = Ident::new(prefix.clone() + base_name);

        new_structs = quote! {
            #new_structs

            #derive_attr
            #table_name_attr
            #vis struct #this_name #impl_generics #where_clause {
                #(#extra_fields),* ,
                #(#common_fields),*
            }
        };

        let field_difs = intermediates.field_differences(prefix);

        new_structs = add_from_impls(
            &this_name,
            &base_name,
            &intermediates,
            vis,
            field_difs,
            &new_structs,
        );
    }

    new_structs
}

fn add_from_impls(
    this_name: &Ident,
    base_name: &str,
    intermediates: &IntermediateFields,
    vis: &Visibility,
    field_differences: Vec<(String, Vec<&Field>, Vec<&Field>)>,
    new_structs: &quote::Tokens,
) -> quote::Tokens {
    let base_snake = base_name.to_snake_case();
    let base_field_idents = &to_struct_assignment_form(&intermediates.common_fields);

    let mut from_fns = quote!();
    for (other_prefix, different_fields, same_fields) in field_differences {
        let new_field_params: Vec<Field> = different_fields
            .iter()
            .cloned()
            .map(|f| strip_vis_and_attrs(f.clone()))
            .collect();
        let new_field_names: Vec<Ident> = different_fields
            .iter()
            .flat_map(|f| f.ident.clone())
            .collect();
        let same_field_idents = to_struct_assignment_form_ref(&same_fields);
        let from_ident = Ident::new(format!("{}{}", other_prefix, base_name));
        let from_fn_ident = Ident::new(format!(
            "from_{}_{}",
            other_prefix.to_snake_case(),
            base_snake,
        ));

        from_fns = quote! {
            #from_fns

            #vis fn #from_fn_ident(#(#new_field_params),* , base: #from_ident) -> #this_name {
                #this_name {
                    #(#new_field_names),* ,
                    #(#base_field_idents),* ,
                    #(#same_field_idents),*
                }
            }
        };
    }

    quote! {
        #new_structs

        impl #this_name {
            #from_fns
        }
    }
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

fn strip_vis_and_attrs(field: Field) -> Field {
    Field {
        ident: field.ident,
        vis: syn::Visibility::Inherited,
        attrs: vec![],
        ty: field.ty,
    }
}

fn to_struct_assignment_form(fields: &[Field]) -> Vec<Tokens> {
    fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            quote! { #ident: base.#ident }
        })
        .collect()
}

fn to_struct_assignment_form_ref(fields: &[&Field]) -> Vec<Tokens> {
    fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            quote! { #ident: base.#ident }
        })
        .collect()
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
                if let NestedMetaItem::MetaItem(MetaItem::Word(ref val)) = *item {
                    val.to_string()
                } else {
                    panic!("Unexpected format for item: {} ", quote!(#item));
                }
            })
        })
        .collect::<Vec<_>>()
}

#[cfg_attr(feature = "cargo-clippy", allow(large_enum_variant))]
enum ExcludeAttr<'a> {
    /// A field that is excluded from the `New` item
    Excluded(Field),
    /// A field that is excluded from a named item
    Intermediate(&'a str, Field),
    Included,
}

/// Parse the attributes on fields to get a list fields that should be excluded
fn extract_intermediates(fields: &[Field]) -> IntermediateFields {
    let mut intermediates = IntermediateFields::default();
    // Collect the fields that aren't decorated with "exclude"
    let common_fields = fields
        .iter()
        .filter(|f| {
            use ExcludeAttr::*;
            // If any of this fields attrs are "exclude" then we want to strip the entire field
            match field_status(f) {
                Excluded(field) => {
                    intermediates.excluded_at_least_once.push(field);
                    false
                }
                Intermediate(intermediate_prefix, field) => {
                    intermediates.excluded_at_least_once.push(field.clone());
                    intermediates
                        .prefix_excluded
                        .entry(intermediate_prefix.to_string())
                        .or_insert_with(Vec::new)
                        .push(field);
                    false
                }
                Included => true,
            }
        })
        .cloned()
        .collect::<Vec<_>>();
    intermediates.common_fields = common_fields;
    intermediates
}

/// A list of all the fields on an original struct, grouped by their status
#[derive(Default)]
struct IntermediateFields {
    /// The fields that never have an `#[intermediate_exclude]1 field on them
    common_fields: Vec<Field>,
    /// Every exclude annotation (either `#[intermediate_exclude]` or
    /// `#[intermediate_exclude(Prefix)]`) will add to this list
    excluded_at_least_once: Vec<Field>,
    /// Fields that are excluded with a prefix are grouped by prefix here
    prefix_excluded: HashMap<String, Vec<Field>>,
}

impl IntermediateFields {
    /// All groups of items that are field subsets of the current prefix
    ///
    /// So given a struct like:
    ///
    /// ```rust,ignore
    /// #[derive(DieselIntermediate)]
    /// struct Big {
    ///     #[intermediate_exclude],
    ///     id: i32,
    ///     #[intermediate_exclude],
    ///     meta: i32,
    ///     #[intermediate_exclude(Outer)],
    ///     outer: i32,
    ///     #[intermediate_exclude(Outer, Inner)],
    ///     outer_inner: i32,
    ///     #[intermediate_exclude(Inner)],
    ///     inner: i32,
    ///
    ///     common: i32,
    /// }
    /// ```
    ///
    /// (after diesel-derive-intermediate supports multiple intermediates)
    ///
    /// This would yield the following items:
    ///
    /// * field_differences("Outer")
    ///   * `New, [outer, outer_inner]`
    ///   * `Inner, [outer]`
    /// * field_differences("Inner")
    ///   * `New, [outer_inner, inner]`
    ///
    /// See also `field_difference_for_full_iter`
    fn field_differences(&self, current_prefix: &str) -> Vec<(String, Vec<&Field>, Vec<&Field>)> {
        // except for the current fields and the extra filter, this is
        // identical to the function below
        let current_fields = &self.prefix_excluded[current_prefix];

        self._field_differences_inner(current_prefix, current_fields)
    }

    /// All groups of items that are field subsets of the complete item
    ///
    /// So given a struct like:
    ///
    /// ```rust,ignore
    /// #[derive(DieselIntermediate)]
    /// struct Big {
    ///     #[intermediate_exclude],
    ///     id: i32,
    ///     #[intermediate_exclude],
    ///     meta: i32,
    ///     #[intermediate_exclude(Outer)],
    ///     outer: i32,
    ///     #[intermediate_exclude(Outer, Inner)],
    ///     outer_inner: i32,
    ///     #[intermediate_exclude(Inner)],
    ///     inner: i32,
    ///
    ///     common: i32,
    /// }
    /// ```
    ///
    /// (after diesel-derive-intermediate supports multiple intermediates)
    ///
    /// This would yield the following items:
    ///
    /// * field_difference_for_full_iter()
    ///   * `New, [id, meta, outer, outer_inner, inner]`
    ///   * `Outer, [id, meta, inner]`
    ///   * `Inner, [id, meta, outer]`
    ///
    /// See also `field_differences`
    fn field_differences_full(&self) -> Vec<(String, Vec<&Field>, Vec<&Field>)> {
        let current_fields = &self.excluded_at_least_once;

        self._field_differences_inner("__", current_fields)
    }

    fn _field_differences_inner<'s>(
        &'s self,
        current_prefix: &str,
        current_fields: &'s [Field],
    ) -> Vec<(String, Vec<&'s Field>, Vec<&'s Field>)> {
        self.prefix_excluded
            .iter()
            .chain(vec![(&"New".to_string(), &self.common_fields)].into_iter())
            .filter(|&(prefix, _)| prefix != current_prefix)
            .filter_map(|(prefix, other_excluded_fields)| {
                let prefix = prefix.clone();
                let other_fields: HashSet<&Field> =
                    HashSet::from_iter(other_excluded_fields.iter());
                let field_difference = current_fields
                    .iter()
                    .filter(|f| !other_fields.contains(f))
                    .collect::<Vec<_>>();
                let field_sames = current_fields
                    .iter()
                    .filter(|f| other_fields.contains(f))
                    .collect::<Vec<_>>();

                if !field_difference.is_empty() {
                    Some((prefix, field_difference, field_sames))
                } else {
                    None
                }
            })
            .collect()
    }
}

fn field_status(field: &Field) -> ExcludeAttr {
    use ExcludeAttr::*;
    for a in &field.attrs {
        match a.value {
            MetaItem::Word(ref ident) if ident == EXCLUDE => {
                return Excluded(field.clone());
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
    Included
}
