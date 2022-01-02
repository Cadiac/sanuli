# Sanuli

A finnish version of [Wordle](https://www.powerlanguage.co.uk/wordle/) implemented in [Rust](https://www.rust-lang.org).

## Installing

Follow [Rust](https://www.rust-lang.org/en-US/install.html) installation instructions.

To build the WASM based [yew](https://yew.rs/) UI, further wasm tooling is required

```
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
cargo install wasm-bindgen-cli
```

## Generating word list

A `word-list.txt` file in the root of this project containing uppercase 5 letter words is required.

To obtain one, the [Kotus](https://kaino.kotus.fi/sanat/nykysuomi/) word list can be used, licensed under "[Attribution 3.0 Unported (CC BY 3.0)](https://creativecommons.org/licenses/by/3.0/deed.fi)".

A parser for parsing `kotus-sanalista_v1.xml` file from [Kotus](https://kaino.kotus.fi/sanat/nykysuomi/) is included:

```bash
$ cargo run --bin parse-kotus-word-list your/path/to/kotus-sanalista_v1.xml
```

which creates a `word-list.txt` file in the working directory.

## Development

For development, start the web server with

```
$ trunk serve
```

This should make the UI available at 0.0.0.0:8080 with hot reload on code changes.

## Release build

```
trunk build --release
```

and copy the produced `dist` directory to the target server.
