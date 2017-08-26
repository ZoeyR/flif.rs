# flif.rs
<img src="flif.rs.png" alt="logo" style="width: 10em;"/>

[![version][version-badge]][CHANGELOG] [![license][license-badge]][LICENSE]

flif.rs is a Rust implementation of the [flif16](http://flif.info/spec.html) image format. This project was inspired by the work on [flif-rs](https://github.com/panicbit/flif-rs). I have no plans to try and one-up the other Rust library; this project was simply created to be a learning experience.

## Development
### Prerequisites
- rustc (either via rustup or your distributions package manager)
- cargo (via the same method as above)

### Building
- `git clone https://github.com/dgriffen/flif.rs.git`
- `cd flif.rs`
- `cargo build`

## Usage
1. add this crate to your crates `Cargo.toml` like so:
```toml
[package]
name = "some_package"
version = "0.0.1"
authors = ["John Doe <you@example.com>"]

[dependencies]
flif = { git = "https://github.com/dgriffen/flif.rs" }
```
2. in the root of your project reference the crate:
```rust
extern crate flif;
```
3. the crate can now be used to decode flif headers :D
```rust
extern crate flif;

use flif::Decoder;

fn main() {
    let file = std::fs::File::open("some_image.flif").unwrap();
    let mut decoder = Decoder::new(file);
    let header = decoder.read_main_header().unwrap();
    println!("{:?}", header);
}
```

### Trademarks
The flif.rs logo is a combination of the official flif logo and Rust logo.

[CHANGELOG]: ./CHANGELOG.md
[LICENSE]: ./LICENSE
[version-badge]: https://img.shields.io/badge/version-0.0.1-blue.svg
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[logo]: ./flif.rs.png
