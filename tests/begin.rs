#![allow(dead_code)] // this is a compile-pass test
#![cfg_attr(feature = "cargo-clippy", allow(no_effect))]

#[macro_use]
extern crate diesel_derive_intermediate;

#[derive(DieselIntermediate)]
#[diesel_intermediate_derive(Debug)]
struct Val {
    #[diesel_intermediate_exclude]
    id: i32,
    /// has a docstring
    other: &'static str,
}

fn builds() {
    Val { id: 0, other: "" };
    NewVal { other: "new" };
}

#[derive(DieselIntermediate)]
#[diesel_intermediate_derive(Debug)]
struct Complex {
    #[diesel_intermediate_exclude]
    id: i32,
    #[diesel_intermediate_exclude(MyPrefix)]
    oid: i32,
    /// has a docstring
    other: &'static str,
}

fn builds_complex() {
    Complex {
        id: 0,
        oid: 1,
        other: "",
    };
    NewComplex { other: "" };
    MyPrefixComplex { oid: 1, other: "" };
}
