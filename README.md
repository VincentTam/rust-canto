# rust-canto

A Rust library for segmenting Cantonese text and converting Chinese characters
to Jyutping (粵拼) romanization. Compiles to WebAssembly for use as a
[Typst](https://typst.app) plugin.

## Features

- **Word segmentation** — splits Cantonese text into natural word units using a
  trie + dynamic programming algorithm
- **Jyutping annotation** — converts each word to its Jyutping romanization
- **Mixed input** — handles mixed Chinese/English/punctuation input gracefully
- **WASM output** — compiles to `.wasm` for use as a Typst plugin via
  [`wasm-minimal-protocol`](https://github.com/astrale-sharp/wasm-minimal-protocol)

## Usage as a Typst Plugin

### Prerequisites

Install the WebAssembly build target:

```bash
rustup target add wasm32-unknown-unknown
```

### Build

```bash
cargo build --release --target wasm32-unknown-unknown
```

The compiled plugin will be at:

```
target/wasm32-unknown-unknown/release/rust_canto.wasm
```

### In Typst

Load the plugin and call `annotate()` with your input text:

```typst
#let canto = plugin("rust_canto.wasm")

#let to-jyutping-words(txt) = {
  let arr = json(canto.annotate(bytes(txt)))
  arr.map(p => ("word": p.at(0), "jyutping": p.at(1)))
}

#let data = to-jyutping-words("今日我要上堂")
```

The `annotate` function returns a JSON array of `[word, jyutping]` pairs:

```json
[
  ["今日", "gam1 jat6"],
  ["我",   "ngo5"],
  ["要",   "jiu3"],
  ["上堂", "soeng5 tong4"]
]
```

English words and punctuation are returned with `null` as the Jyutping:

```json
[
  ["今日", "gam1 jat6"],
  ["chem", null],
  ["？",   null]
]
```

## Algorithm

Text is segmented using a **trie + dynamic programming** approach:

1. A trie is built at startup from the bundled `words.tsv` (103,000+ entries)
   and `chars.tsv` (34,000+ characters) datasets, derived from
   [rime-cantonese](https://github.com/rime/rime-cantonese).
2. For each position in the input, all possible word matches are found by
   walking the trie left-to-right.
3. Dynamic programming selects the segmentation that minimises token count,
   using word frequency from `freq.txt` as a tiebreaker — so `學生` (freq
   71,278) beats `好學` (freq 2,847) when both produce the same token count.

## Data Sources

The bundled dictionary data is derived from
[rime-cantonese](https://github.com/rime/rime-cantonese), licensed under
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

## Related Projects

- [PyCantonese](https://pycantonese.org) — the Python library that inspired
  this project
- [to-jyutping](https://github.com/CanCLID/to-jyutping) — to NodeJS package
  that inspired the trie structure in this project

## License

MIT

Data bundled from rime-cantonese is licensed under CC BY 4.0 — see
[`data/README.md`](data/README.md) for details.
