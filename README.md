# Codewars TUI

#### ➡ An interactive codewars terminal user iterface (TUI), to search, explore and download kata in a graphical way.

<img src="./assets/CodewarsCLI_demo.gif" alt="Basic demo of Codewars CLI"/>

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

## Improvement roadmap

- message to user (e.g: "download successfully", "failed: reason"...)
- stop using "scapper" and only use "headless_chrome", more use of the API
- store the latest download path locally for future uses (setting page+ editor terminal command)
- `DONE` <s>search kata by id</s>
- `DONE` <s>fix rust bug when init cargo (output on screen)</s>
- `DONE` <s>dynamic text input with cursor</s>
- `DONE` <s>path input auto-completion on the path (with tab or idk) (>/< to change suggestion, Right to confirm)</s>
- `DONE` <s>in downloading windows, display the kata name to download</s>
- `DONE` <s>trim all special chars when created the kata folder</s>
- `DONE` <s>automatic projects init (e.g: for rust, run `cargo init` then delete `.git` folder)</s>

## Credit zone

- [Codewars API](https://dev.codewars.com/#users-api)
- [Codewars Seach Page](https://www.codewars.com/kata/search)
