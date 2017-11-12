# `#[derive(DieselIntermediate)]`

[![Build Status](https://travis-ci.org/quodlibetor/diesel-derive-intermediate.svg?branch=master)](https://travis-ci.org/quodlibetor/diesel-derive-intermediate)

{{readme}}

# Contributing

## Compatibility policy

This project doesn't actually integrate with diesel particularly closely, so it
actually works pretty well against a wide range of diesel versions. It is only
explicitly tested against the newest Diesel version. It is currently known to
work at least with Diesel versions 0.14 - 0.16.

It will always support the lowest version of Rust that Diesel supports. Since
at least diesel 0.14 that's `1.18.0`. Requiring a new Diesel or Rust version
will always at least bump the minor version.

## License

diesel-newtype is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

Patches and bug reports welcome!
