# rust-canto

A Rust library for segmenting Cantonese text and converting Chinese characters
to Jyutping (粵拼)/Yale romanization (耶魯拼音). Compiles to WebAssembly for use as a
[Typst](https://typst.app) plugin.

## Features

- **Word segmentation** — splits Cantonese text into natural word units using a
  trie + dynamic programming algorithm
- **Jyutping annotation** — converts each word to its Jyutping romanization
- **Yale annotation** — converts each word to its Yale romanization
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
  json(canto.annotate(bytes(txt)))
}

#let data = to-jyutping-words("今日我要上堂")
```

The `annotate` function returns a JSON array of `{word, jyutping, yale}` objects,
so that my Typst package
[pycantonese-parser](https://github.com/VincentTam/pycantonese-parser) can
process it.

```json
[
  {
    word: "今日",
    jyutping: "gam1 jat6",
    yale: [
      "gām",
      "yaht",
    ],
  },
  {
    word: "我",
    jyutping: "ngo5",
    yale: [
      "ngóh",
    ],
  },
  {
    word: "要",
    jyutping: "jiu3",
    yale: [
      "yiu",
    ],
  },
  {
    word: "上堂",
    jyutping: "soeng5 tong4",
    yale: [
      "séuhng",
      "tòhng",
    ],
  },
]
```

English words and punctuation are returned with `null` as the Jyutping:

```json
[
  {
    word: "今日",
    jyutping: "gam1 jat6",
    yale: [
      "gām",
      "yaht",
    ],
  },
  {
    word: "c",
    jyutping: none,
    yale: none,
  },
  {
    word: "h",
    jyutping: none,
    yale: none,
  },
  {
    word: "e",
    jyutping: none,
    yale: none
  },
  {
    word: "m",
    jyutping: none,
    yale: none
  },
  {
    word: "？",
    jyutping: none,
    yale: none
  },
]
```

## Algorithm

Text is segmented using a **trie + dynamic programming** approach:

### 1. Building the trie

A trie is built at startup from three bundled data files derived from
[rime-cantonese](https://github.com/rime/rime-cantonese):

- **`chars.tsv`** (34,000+ entries) — single-character readings with optional
  frequency weights (e.g. `佢 keoi5` and `佢 heoi5 3%`). Each character's
  readings are inserted in descending weight order so that `readings[0]` always
  holds the most common pronunciation. Entries with no percentage are treated as
  the primary reading (weight 100) and take precedence over those with an
  explicit percentage.
- **`words.tsv`** (103,000+ entries) — multi-character word readings. These
  build full paths through the trie and are loaded after `chars.tsv` so that
  single-character nodes are already in place.
- **`freq.txt`** (266,000+ entries) — word frequencies used as a tiebreaker
  during segmentation (see below).

### 2. Segmentation

For each position in the input, all possible word matches are found by walking
the trie left-to-right from that position. Dynamic programming then selects the
segmentation that minimises the token count. When two segmentations produce the
same number of tokens, the one with the higher total word frequency wins — so
`學生` (freq 71,278) beats `好學` (freq 2,847) when both yield a two-token
result for `好學生`.

### 3. Romanization

Each segmented token's Jyutping reading is taken directly from the trie.
Yale romanization is then derived from the Jyutping by converting initials
(`z`→`j`, `c`→`ch`, `j`→`y`), finals (`eoi`→`eui`, `eo`/`oe`→`eu`, etc.),
and applying tone diacritics (macron for tone 1, acute for tone 2, grave for
tone 4, acute for tone 5; tones 3 and 6 are unmarked). Low-register tones
(4–6) additionally insert `h` after the vowel nucleus and before any stop coda
(`-p`, `-t`, `-k`, `-m`, `-n`, `-ng`).

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
