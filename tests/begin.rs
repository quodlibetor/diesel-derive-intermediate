// this is a compile-pass test
#![allow(dead_code)]
#![cfg_attr(feature = "cargo-clippy", allow(no_effect))]

#[macro_use]
extern crate diesel_derive_intermediate;

#[derive(DieselIntermediate)]
#[intermediate_derive(Debug)]
struct Val {
    #[intermediate_exclude]
    id: i32,
    /// has a docstring
    other: &'static str,
}

fn builds() {
    Val { id: 0, other: "" };
    NewVal { other: "new" };
}

#[derive(DieselIntermediate)]
#[intermediate_derive(Debug)]
struct Complex {
    #[intermediate_exclude]
    id: i32,
    #[intermediate_exclude(MyPrefix)]
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
