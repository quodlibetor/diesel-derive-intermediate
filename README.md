# `#[derive(DieselIntermediate)]`

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
#[intermediate_derive(Debug, PartialEq)]
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

impl Mycologist {
    // The `pub` comes from the `pub` on `Mycologist`
    pub fn from_new_mycologist(id: i32, base: NewMycologist) {
        Mycologist {
            id,
            rust_count: base.rust_count,
        }
    }
}

pub struct Rust {
    id: i32,
    mycologist_id: i32,
    life_cycle_stage: i32,
}
#[derive(Debug, PartialEq)]
pub struct CapturedRust {
    mycologist_id: i32,
    life_cycle_stage: i32,
}

#[derive(Debug, PartialEq)]
pub struct NewRust {
    life_cycle_stage: i32,
}

// Convenience constructors that take just the parameters that exist in
// this intermediate and not the intermediate it came from.
impl Rust {
    pub fn from_captured_rust(id: i32, base: CapturedRust) -> Rust {
        Rust {
            id,
            mycologist_id: base.mycologist_id,
            life_cycle_stage: base.life_cycle_stage,
        }
    }

    pub fn from_new_rust(id: i32, mycologist_id: i32, base: NewRust) -> Rust {
        Rust {
            id,
            mycologist_id,
            life_cycle_stage: base.life_cycle_stage,
        }
    }
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
