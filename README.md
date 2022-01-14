# Sanuli

[![Sponsored](https://img.shields.io/badge/chilicorn-sponsored-brightgreen.svg?logo=data%3Aimage%2Fpng%3Bbase64%2CiVBORw0KGgoAAAANSUhEUgAAAA4AAAAPCAMAAADjyg5GAAABqlBMVEUAAAAzmTM3pEn%2FSTGhVSY4ZD43STdOXk5lSGAyhz41iz8xkz2HUCWFFhTFFRUzZDvbIB00Zzoyfj9zlHY0ZzmMfY0ydT0zjj92l3qjeR3dNSkoZp4ykEAzjT8ylUBlgj0yiT0ymECkwKjWqAyjuqcghpUykD%2BUQCKoQyAHb%2BgylkAyl0EynkEzmkA0mUA3mj86oUg7oUo8n0k%2FS%2Bw%2Fo0xBnE5BpU9Br0ZKo1ZLmFZOjEhesGljuzllqW50tH14aS14qm17mX9%2Bx4GAgUCEx02JySqOvpSXvI%2BYvp2orqmpzeGrQh%2Bsr6yssa2ttK6v0bKxMBy01bm4zLu5yry7yb29x77BzMPCxsLEzMXFxsXGx8fI3PLJ08vKysrKy8rL2s3MzczOH8LR0dHW19bX19fZ2dna2trc3Nzd3d3d3t3f39%2FgtZTg4ODi4uLj4%2BPlGxLl5eXm5ubnRzPn5%2Bfo6Ojp6enqfmzq6urr6%2Bvt7e3t7u3uDwvugwbu7u7v6Obv8fDz8%2FP09PT2igP29vb4%2BPj6y376%2Bu%2F7%2Bfv9%2Ff39%2Fv3%2BkAH%2FAwf%2FtwD%2F9wCyh1KfAAAAKXRSTlMABQ4VGykqLjVCTVNgdXuHj5Kaq62vt77ExNPX2%2Bju8vX6%2Bvr7%2FP7%2B%2FiiUMfUAAADTSURBVAjXBcFRTsIwHAfgX%2FtvOyjdYDUsRkFjTIwkPvjiOTyX9%2FAIJt7BF570BopEdHOOstHS%2BX0s439RGwnfuB5gSFOZAgDqjQOBivtGkCc7j%2B2e8XNzefWSu%2BsZUD1QfoTq0y6mZsUSvIkRoGYnHu6Yc63pDCjiSNE2kYLdCUAWVmK4zsxzO%2BQQFxNs5b479NHXopkbWX9U3PAwWAVSY%2FpZf1udQ7rfUpQ1CzurDPpwo16Ff2cMWjuFHX9qCV0Y0Ok4Jvh63IABUNnktl%2B6sgP%2BARIxSrT%2FMhLlAAAAAElFTkSuQmCC)](http://spiceprogram.org/oss-sponsorship)
[![Netlify Status](https://api.netlify.com/api/v1/badges/d1dbf5f4-e4f4-4aed-9664-63200637ad12/deploy-status)](https://app.netlify.com/sites/sanuli/deploys)

![Sanuli](/static/sanuli-1200x630.png)

A finnish version of the word guessing game [Wordle](https://www.powerlanguage.co.uk/wordle/) implemented in [Rust](https://www.rust-lang.org).

Live version running at [sanuli.fi](https://sanuli.fi).

## Quick start

Follow [Rust](https://www.rust-lang.org/en-US/install.html) installation instructions.

To build the WASM based [yew](https://yew.rs/) UI, further wasm tooling is required

```
$ rustup target add wasm32-unknown-unknown
$ cargo install --locked trunk
$ cargo install wasm-bindgen-cli
```

Create word list files and populate them with uppercase words, one per line

```
$ touch common-words.txt
$ touch daily-words.txt
$ touch full-words.txt
```

Start the UI in development mode
```
$ trunk serve
```

## Word lists

Three separate word list files in the root of this project containing all the words are required. The lists are not included in this repository.

The lists are:
- `full-words.txt` - Full list of all accepted 5 and 6 character words. The checks if a word real or not is done against this list
- `daily-words.txt` - List of daily words. The daily word is taken from row equal to the days from 2022-01-07.
- `common-words.txt` - Subset of the full words list, intended for easier game mode. Note that all these words _must_ exist on the `full-words.txt`

Beware that these are _included in the release binary_, and anyone can obtain the lists!

## Generating base word lists

To create a word list, a dictionary like the "nykysuomen sanalista" by [Kotus](https://kaino.kotus.fi/sanat/nykysuomi/), licensed with [CC BY 3.0](https://creativecommons.org/licenses/by/3.0/deed.fi), can be used as a baseline.

A parser for parsing `kotus-sanalista_v1.xml` file from [Kotus](https://kaino.kotus.fi/sanat/nykysuomi/) is included:

```bash
$ cargo run --bin parse-kotus-word-list your/path/to/kotus-sanalista_v1.xml
```

which creates a `full-words-generated.txt` file in the working directory.

## Development

For development, start the web server with

```
$ trunk serve
```

This should make the UI available at 0.0.0.0:8080 with hot reload on code changes.

To change the default port, use

```
$ trunk serve --port=9090
```

## Release build

```
$ trunk build --release
```

and copy the produced `dist` directory to your target server.
