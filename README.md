# Speed Typing

A fast and lightweight terminal-based typing speed test written in Rust.

Speed Typing provides a simple TUI for practicing your typing skills and measuring your typing speed directly from the terminal.

## Installation

### GitHub Releases

Download the latest pre-built binary for your platform from the [GitHub Releases](https://github.com/DanieloM83/Speed-Typing/releases) page.

Extract the archive and run the `speed-typing` executable.

### Manual Installation

Make sure you have [Rust](https://www.rust-lang.org/tools/install) and Cargo installed.

Clone the repository:

```bash
git clone https://github.com/DanieloM83/Speed-Typing.git
cd Speed-Typing
cargo run --release
```

Or install it directly with Cargo:

```
cargo install --path .
speed-typing
```

## Acknowledgements

This project is built with the following open-source Rust crates:

- [Ratatui](https://ratatui.rs/) — Terminal User Interface framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) — Cross-platform terminal manipulation
- [Rand](https://crates.io/crates/rand) — Random number generation
- [IndexMap](https://crates.io/crates/indexmap) — Ordered hash map implementation
- [Color-Eyre](https://github.com/eyre-rs/eyre) — Error handling and reporting
- [Unicode Width](https://crates.io/crates/unicode-width) — Unicode character width calculation