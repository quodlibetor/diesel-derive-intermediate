# `#[derive(DieselIntermediate)]`

[![Build Status](https://travis-ci.org/quodlibetor/diesel-derive-intermediate.svg?branch=master)](https://travis-ci.org/quodlibetor/diesel-derive-intermediate)

## Derive intermediate structs

An intermediate struct is a struct that does not have all the data that a
"full" struct has. For example, you might receive requests that do not have
an ID until they are inserted into your database (because of an e.g.
AUTOINCREMENT primary key).

Since having `Option<id>` fields is sort of wrong and definitely
unergonomic most of the time, it's reasonable to have a `NewStruct` that is
exactly the same as `Struct`, but without the `id` field.

The goal of this crate is to provide nice ergonomics around supporting
intermediate structs, and to provide nice integration with
[`Diesel`](https://diesel.rs/).

`diesel-derive-intermediate` provides several attributes or targets. See
the example below if the prose isn't clear:

* The `DieselIntermediate` derive target which primarily generates
  *structs* (not Traits, which is what `#[derive]` is supposed to generate)
  works with a few attributes to provide field-exclusions on the generated
  structs.
* The `#[intermediate_exclude]` field-level attribute which comes in two
  forms:
  * `#[intermediate_exclude]` by itself, which marks the field as being
    excluded from the `NewStruct` struct, and for inclusion in the
    `from_new_struct` static method.
  * `#[intermediate_exclude(SomePrefix)]` excludes from the `NewStruct`
    generated struct, but causes a `SomePrefixStruct` to be generated,
    which *will* have this field.
* The `#[intermediate_derive(Traits...)]` struct-level attribute applies
  its contained traits to all the intermediate structs generated.
* `DieselIntermediate` will apply diesel's `#[table_name = "..."]`
  struct-level attribute to all generated structs, if you need to use a
  different table name you can use `#[intermediate_table_name = "..."]` to
  override the default.

## Example

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
    pub fn from_new_mycologist(id: i32, base: NewMycologist) -> Mycologist {
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

see [`tests/diesel-interaction.rs`](tests/diesel-interaction.rs) for a
couple fully-worked examples, including using with `Insertable` and the
purpose of the `intermediate_exclude(NAME)` form.

Interestingly, since this is abusing the derive proc-macro infrastructure,
if you have no `#[intermediate_derive(...)]` attributes, you will get
"empty trait list in \`derive\`" warnings.

### Limitations

* It's not possible to derive multiple `Associations` for the same pair of
  tables, I think. This means that we can't derive `Associations` for the
  intermediate types. This seems basically fine, you really only want to be
  able to join on complete types that have actually been inserted into the
  DB, not partials that are in the process of getting built to be inserted.

## License

diesel-newtype is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

Patches and bug reports welcome!
