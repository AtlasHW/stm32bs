# `generate by stm32bs default template`

> A project for applications for stm32 MCU

This project is developed and maintained by the [atlasHW <atlas.songk@gmail.com>].

## Dependencies

If you got the information, you have build the stm32 application project
To build the programs using this template you'll need:

- Rust 1.31, 1.30-beta, nightly-2018-09-13 or a newer toolchain. e.g. `rustup
  default beta`

- `rust-std` components (pre-compiled `core` crate) for the ARM Cortex-M
  targets. Run:

``` console
$ rustup target add thumbv6m-none-eabi
```

- `probe-rs` tool for download firmware and debug. Run:

``` console
$ cargo install binstall
$ cargo binstall probe-rs-tools
```

## Using this template

**NOTE**: This is the very short version that only covers building programs. For
the long version, which additionally covers flashing, running and debugging
programs, check [the embedded Rust book][book].

[book]: https://rust-embedded.github.io/book


4. Build the template application or one of the examples.

``` console
$ cargo build
```
or
``` console
$ cargo run
```

# License

Cargo-stm32bs has not license or open source requirement, you can add your license by your opinion.

## Contribution

To edit it by your opinion

## Code of Conduct

To edit it by your opinion
