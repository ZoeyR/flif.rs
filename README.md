# flif.rs
<p align="center">
  <img src="https://cdn.rawgit.com/dgriffen/flif.rs/e9cb5c4a/flif.rs.svg" alt="logo" height=150 />
</p>

 [![Build Status](https://travis-ci.org/dgriffen/flif.rs.svg?branch=master)](https://travis-ci.org/dgriffen/flif.rs) [![version][version-badge]][CHANGELOG] [![license][license-badge]][LICENSE]

flif.rs is a Rust implementation of the [flif16](http://flif.info/spec.html) image format. This project was inspired by the work on [flif-rs](https://github.com/panicbit/flif-rs).
## Current Status

Currently this project in alpha stage. As of right now pixel data can be decoded but only for a limited subset of valid flif images. The most significant limitations are:
- Animations are not supported.
- Interlaced images are not supported.
- Certain transformations are not supported.

As this project progresses more and more missing features will end up being supported.

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
flif = "0.2"
```
2. in the root of your project reference the crate:
```rust
extern crate flif;
```
3. the crate can now be used to decode flif headers :D
```rust
extern crate flif;

use std::fs::File;
use std::io::BufReader;
use flif::Flif;

fn main() {
    let file = std::fs::File::open("/path/to/image.flif").unwrap();
    // use `BufReader` to improve performance
    let reader = BufReader::new(file);
    let image = Flif::decode(reader).unwrap();
    println!("image info: {:?}", image.info());
    let raw_pixels = image.get_raw_pixels();
}
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

## Trademarks
The flif.rs logo is a combination of the official flif logo and Rust logo.

[CHANGELOG]: ./CHANGELOG.md
[LICENSE]: ./LICENSE
[version-badge]: https://img.shields.io/badge/version-0.4.0-blue.svg
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[logo]: ./flif.rs.png
