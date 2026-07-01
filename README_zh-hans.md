# amproj

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/amproj.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/amproj.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：活跃开发中

**amproj** - `.amproj` 项目文件的数据提取层，支持 C-ABI。

| English         | 简体中文                 |
|-----------------|---------------------------------|
| [English](./readme.md) | 简体中文 |

## 简介

`amproj` 是一个纯 Rust 的 `.amproj` 项目文件数据提取库。
它解析基于 XML 的动画项目数据，提供 schema 类型、关键帧插值、坐标系转换、特效注册表和 C-ABI 导出，用于非 Rust 引擎集成。

有了 `amproj`，你可以在任何支持 C FFI 的语言中加载和检视动画项目数据，或直接在 Rust 应用中集成它。

## 特性

* 所有项目数据的 XML schema 类型与 serde 序列化
* 支持 cubic-bezier、bounce、elastic、step 等缓动函数的关键帧插值
* 坐标系转换，内置 Bevy、Unity、Godot、CSS 预设
* 支持 `.amproj` zip 归档和原始目录的 AM 项目加载
* 特效与图层类型的全面验证报告
* 包含 34+ 个内置特效定义的特效注册表
* 自动生成的特效与内置项文档
* 可选的 C-ABI（`cdylib`）支持，用于 Unity、Godot、Unreal 集成
* 零 Bevy 依赖

## 使用方法

1. **安装 Rust**（如果尚未安装）：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **添加到 Cargo.toml**：
   ```toml
   [dependencies]
   amproj = "0.5"
   ```

3. **加载项目**：
   ```rust
   use amproj::loader::load_project_from_path;
   let project = load_project_from_path("path/to/project.amproj")?;
   ```

4. **FFI 使用**: 使用 `--features ffi` 构建以生成 C 兼容的共享库。
   ```bash
   cargo build --features ffi --release
   ```

## 依赖

本项目使用以下 crate：

| Crate                                             | Version | Description                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [chrono](https://crates.io/crates/chrono)         | 0.4     | Date and time library       |
| [glam](https://crates.io/crates/glam)             | 0.30    | Linear algebra types        |
| [quick-xml](https://crates.io/crates/quick-xml)   | 0.39    | XML serialization/deserialization |
| [serde](https://crates.io/crates/serde)           | 1.0     | Serialization framework     |
| [serde_json](https://crates.io/crates/serde_json) | 1.0     | JSON serialization          |
| [thiserror](https://crates.io/crates/thiserror)   | 2.0     | Error derive macros         |
| [zip](https://crates.io/crates/zip)               | 8.1     | ZIP archive support         |

## 贡献

欢迎贡献！无论是修复 bug、添加功能还是改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法并讨论设计或架构。

## 许可证

本项目采用以下任一许可证：

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) 或 [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) 或 [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

任选其一。
