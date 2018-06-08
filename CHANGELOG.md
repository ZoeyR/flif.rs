# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2018-06-07
### Added
- grayscale support

### Fixed
- reduced memory footprint on complex files
- improved speed
- decoding image header and then image caused an error

## [0.1.0] - 2017-12-23
### Added
- reading of maniac trees
- reading certain transformations
    - YCoCg
    - Bounds
    - Channel Compact
- pixel data decoding

### Fixed
- near zero decoding for negative numbers


## [0.0.2] - 2017-10-14
### Added
- reading of second header (minus transformations)
- RAC chance decoding+encoding

### Fixed
- RAC no longer returns Err on end of file
    - previously the RAC could not read streams within 4 bytes of the end

### Changed
- reodered cli commands

## 0.0.1 - 2017-08-26
### Added
- ability to decode main flif header
- decoding metadata chunks
- framework for reading secondary flif header
- example that decodes and prints the header and metadata of a flif file
- started a changelog
- readme file
- license

[Unreleased]: https://github.com/dgriffen/flif.rs/compare/v0.2.0...HEAD
[0.0.2]: https://github.com/dgriffen/flif.rs/compare/v0.0.1...v0.0.2
[0.1.0]: https://github.com/dgriffen/flif.rs/compare/v0.0.2...v0.1.0
[0.2.0]: https://github.com/dgriffen/flif.rs/compare/v0.1.0...v0.2.0