# amproj

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/amproj.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/amproj.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: Active Development

**amproj** - Data extraction layer for `.amproj` project files with C-ABI support.

| English         | Simplified Chinese                 |
|-----------------|---------------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`amproj` is a pure Rust data extraction crate for `.amproj` project files.
It parses XML-based animation project data, providing schema types, keyframe interpolation, coordinate conversion, an effects registry, and C-ABI exports for non-Rust engine integration.

With `amproj`, you can load and inspect animation project data in any language that supports C FFI, or integrate it directly into Rust applications.

## Features

* XML schema types with serde serialization for all project data
* Keyframe interpolation with cubic-bezier, bounce, elastic, and step easing
* Coordinate system conversion with presets for Bevy, Unity, Godot, and CSS
* AM project loading from `.amproj` zip archives and raw directories
* Comprehensive validation reporting for effects and layer types
* Effects registry with 34+ built-in effect definitions
* Auto-generated documentation for effects and builtins
* Optional C-ABI (`cdylib`) support for Unity, Godot, and Unreal integration
* Zero Bevy dependency

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Add to your Cargo.toml**:
   ```toml
   [dependencies]
   amproj = "0.5"
   ```

3. **Load a project**:
   ```rust
   use amproj::loader::load_project_from_path;
   let project = load_project_from_path("path/to/project.amproj")?;
   ```

4. **FFI usage**: Build with `--features ffi` to produce a C-compatible shared library.
   ```bash
   cargo build --features ffi --release
   ```

## Dependencies

This project uses the following crates:

| Crate                                             | Version | Description                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [chrono](https://crates.io/crates/chrono)         | 0.4     | Date and time library       |
| [glam](https://crates.io/crates/glam)             | 0.30    | Linear algebra types        |
| [quick-xml](https://crates.io/crates/quick-xml)   | 0.39    | XML serialization/deserialization |
| [serde](https://crates.io/crates/serde)           | 1.0     | Serialization framework     |
| [serde_json](https://crates.io/crates/serde_json) | 1.0     | JSON serialization          |
| [thiserror](https://crates.io/crates/thiserror)   | 2.0     | Error derive macros         |
| [zip](https://crates.io/crates/zip)               | 8.1     | ZIP archive support         |

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.

## License

This project is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
