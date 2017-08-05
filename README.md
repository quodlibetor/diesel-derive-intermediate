# #[derive(DieselIntermediate)]

This is still in the prototype phase!

Given:

```rust
#[derive(DieselIntermediate)]
#[diesel_intermediate_derive(Debug)]
pub struct Mycologist {
    #[diesel_intermediate_exclude]
    id: i32,
    rust_count: i32,
}

#[derive(DieselIntermediate)]
#[diesel_intermediate_derive(Debug)]
pub struct Rust {
    #[diesel_intermediate_exclude]
    id: i32,
    #[diesel_intermediate_exclude(Captured)]
    mycologist_id: i32,
    life_cycle_stage: i32,
}
```

The result will be:

```rust
pub struct Mycologist {
    id: i32,
    rust_count: i32,
}
#[derive(Debug)]
pub struct NewMycologist {
    rust_count: i32,
}

pub struct Rust {
    id: i32,
    mycologist_id: i32,
    life_cycle_stage: i32,
}
pub struct CapturedRust {
    mycologist_id: i32,
    life_cycle_stage: i32,
}
pub struct NewRust {
    life_cycle_stage: i32,
}
```

see [`tests/diesel-interaction.rs`](tests/diesel-interaction.rs) for a couple
fully-worked examples, including using with `Insertable` and the purpose of the
`intermediate_exclude(NAME)` form.

Interestingly, since this is abusing the derive proc-macro infrastructure, if
you have no `#[diesel_intermediate_derive(...)]` attributes, you will get
"empty trait list in \`derive\`" warnings.

## License

diesel-newtype is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

Patches and bug reports welcome!
