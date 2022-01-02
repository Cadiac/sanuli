# Sanuli

![Sanuli](/static/sanuli-1200x630.png)

A finnish version of [Wordle](https://www.powerlanguage.co.uk/wordle/) implemented in [Rust](https://www.rust-lang.org).

Live version running at [sanuli.fi](https://sanuli.fi).

## Installing

Follow [Rust](https://www.rust-lang.org/en-US/install.html) installation instructions.

To build the WASM based [yew](https://yew.rs/) UI, further wasm tooling is required

```
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
cargo install wasm-bindgen-cli
```

## Generating word list

A `word-list.txt` file in the root of this project containing all uppercase 5 and 6 letter words is required.

To obtain one, a word list like the "nykysuomen sanalista" by [Kotus](https://kaino.kotus.fi/sanat/nykysuomi/), licensed with [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/deed.fi), can be used as a baseline.

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
