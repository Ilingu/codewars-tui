# Codewars TUI

#### ➡ An interactive codewars terminal user iterface (TUI), to search, explore and download kata in a graphical way.

<img src="./assets/demo.gif" alt="Demo of Codewars CLI">

## Purpose

- Learn rust
- Learn TUI design pattern
- Having fun

## Installation

> This TUI might not works (at all) on OS other than linux.

Build from source with `cargo`, or download an executable from the `release page`

```bash
cargo build --release # will creates a single executable for your os in ./target/release
```

### Chromium must be installed in your pc!

- on linux just run:

```bash
sudo snap install chromium
# or
sudo apt update
sudo apt install chromium-browser
```

## Made with:

1. **Elegance** ✅
2. `RUST` ✨🦀
3. [tui-rs](https://github.com/fdehau/tui-rs) ♥ (awesome lib btw)
4. [headless_chrome](https://github.com/rust-headless-chrome/rust-headless-chrome), automated browser for scrapping page that require JS loading (like selenium..., it's the reason why `chromium must be installed`)
5. go see [Cargo.toml](/Cargo.toml)

## Credit zone

- [Codewars API](https://dev.codewars.com/#users-api)
- [Codewars Seach Page](https://www.codewars.com/kata/search)
