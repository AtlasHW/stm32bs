# cargo-stm32bs

This tools is stm32 cargo project generation tool, base on
[cargo-generate].

[cargo-generate]: https://github.com/cargo-generate/cargo-generate/


> cargo-stm322bs, to create a stm32 project!

## Documentation

proccess...

## Templates

The default template is [stm32bs-template-default] in github.

[stm32bs-template-default]: https://github.com/AtlasHW/stm32bs-template-default 

## Quickstart

### Installation

```sh
cargo install cargo-stm32bs
```

If you haven't embedded target you rust tools, your need install them.
for "STM32F0", "STM32G0", "STM32L0", "STM32C0", "STM32U0", "STM32WL3", "STM32WB0"
family, you should install target "thumbv6m-none-eabi".
```sh
rustup target add thumbv6m-none-eabi
``` 

for "STM32F1", "STM32F2", "STM32L1" you should install target "thumbv7m-none-eabi".
```sh
rustup target add thumbv7m-none-eabi
``` 

for "STM32F3", "STM32F4", "STM32F7", "STM32G4", "STM32H7", "STM32L4", "STM32L4+",
"STM32WB", "STM32WL"  you should install target "thumbv7em-none-eabi".
```sh
rustup target add thumbv7em-none-eabi
``` 

for "STM32L5", "STM32U5", "STM32H5", "STM32WBA", "STM32N6", "STM32U3" you should install 
target "thumbv8m.main-none-eabihf".
```sh
rustup target add thumbv8m.main-none-eabihf
``` 

We recommend debug tools [probe-rs] as debug and download tools, you can download binstall first

[probe-rs]: https://probe.rs/

```sh
cargo install binstall
``` 
and using binstall to install [probe-rs]
```sh
cargo binstall probe-rs-tools
``` 


### Usage

if you want to use default template, you can change directory to you 
workspace or a directory to store rust code
```sh
cd ./rust
cargo stm32bs
🤷 Project Name: blink
🤷 Chip Part Number (eg. stm32g071cbt6): stm32g071cbt6tr
✔ 🤷 Choose a project type · Demo
Create a STM32 Demo project...
✔ 🤷 Choose a demo · blink
🔧 Destination: /home/atlassong-k/rust/cargo-stm32bs/blink ...
🔧 project-name: blink ...
🔧 username: "atlasHW" (placeholder provided by cli argument)
🔧 Generating template ...
🤷 Port of GPIO is used to LED, eg. B: B
🤷 Pin of GPIO is used to LED, eg. 5: 5
[1/6]   Done: Cargo.toml
[2/6]   Done: src/main.rs
[3/6]   Done: build.rs
[4/6]   Done: .cargo/config.toml
[5/6]   Done: memory.x
[6/6]   Done: README.md
✨ Done! New project created /home/atlassong-k/rust/cargo-stm32bs/blink
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE)
  or [apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))

at your option.

### Contributions

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.




