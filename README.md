# #[derive(DieselIntermediate)]

[![Build Status](https://travis-ci.org/quodlibetor/diesel-derive-intermediate.svg?branch=master)](https://travis-ci.org/quodlibetor/diesel-derive-intermediate)

This is still in the prototype phase!

Given:

```rust
#[derive(DieselIntermediate)]
#[intermediate_derive(Debug)]
pub struct Mycologist {
    #[intermediate_exclude]
    id: i32,
    rust_count: i32,
}

#[derive(DieselIntermediate)]
#[intermediate_derive(Debug)]
pub struct Rust {
    #[intermediate_exclude]
    id: i32,
    #[intermediate_exclude(Captured)]
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
you have no `#[intermediate_derive(...)]` attributes, you will get
"empty trait list in \`derive\`" warnings.

### Limitations

* It's not possible to derive multiple `Associations` for the same pair of
  tables, I think. This means that we can't derive `Associations` for the
  intermediate types. This seems basically fine, you really only want to be
  able to join on complete types that have actually been inserted into the DB,
  not partials that are in the process of getting built to be inserted.

## License

diesel-newtype is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

Patches and bug reports welcome!
