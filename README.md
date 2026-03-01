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

It is a standalone binary file that can be copied to your project.

### In Typst

You can use my Typst package
[`auto-canto`](https://github.com/VincentTam/auto-canto) to retrieve this
crate's output conveniently.

If you wish you process this crate's output yourself, you may load the plugin
and call `annotate()` with your input text:

```typst
// replace with the relative path
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
    yale: ["gām", "yaht"],
  },
  {
    word: "我",
    jyutping: "ngo5",
    yale: ["ngóh"],
  },
  {
    word: "要",
    jyutping: "jiu3",
    yale: ["yiu"],
  },
  {
    word: "上堂",
    jyutping: "soeng5 tong4",
    yale: ["séuhng","tòhng"],
  },
]
```

English words and punctuation are returned with `null` as the Jyutping:

```json
[
  {
    word: "佢",
    jyutping: "keoi",
    yale: ["kéuih"],
  },
  {
    word: "有",
    jyutping: "jau6",
    yale: ["yauh"],
  },
  {
    word: "chem",
    jyutping: kem1,
    yale: ["kēm"],
  },
  {
    word: "堂",
    jyutping: "tong4",
    yale: ["tòhng"],
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
- **lettered.tsv** (1,000+ entries) – Latin+CJK word readings.  They are loaded after `words.tsv`.
- **`freq.txt`** (266,000+ entries) — word frequencies used as a tiebreaker
  during segmentation (see below).

### 2. Segmentation

Input text is tokenised in a single left-to-right pass using dynamic
programming over the trie. `dp[i]` holds the best `(token_count, total_freq)`
for the first `i` characters; the goal is to minimise `token_count` and, on a
tie, maximise `total_freq`. For example, `好學生` can split as `好學 + 生` or
`好 + 學生`; both yield two tokens, but `學生` (freq 71,278) beats `好學`
(freq 2,847), so `好 + 學生` wins.

Each character position is resolved by three rules applied in priority order:

**Trie walk.** For every possible start position, the trie is walked
left-to-right to find all matching words. A match contributes one token and
carries the word's Jyutping reading and frequency. Mixed Latin+CJK entries such
as `AB膠` and `做part-time`, as well as hyphenated entries like `chok-cheat`,
are stored in the trie and matched here.

**Alpha-run fallback.** If the trie finds no reading for a span, the span may
still be merged into one token if it is a contiguous run of non-CJK
alphanumeric characters. Hyphens (`-`), underscores (`_`), and apostrophes
(`'`) are allowed as internal connectors but not at the start or end of the
span, so `part-time`, `rust_canto`, and `i'm` each become one token while a
bare `-` remains a single-character token. The resulting token has no Jyutping
reading. This rule only fires when the trie has no entry for the span, so a
word like `ge` that appears in the lettered dictionary correctly receives its
reading `ge3` rather than `None`.

**Single-character fallback.** Any character not covered by the above —
whitespace, punctuation, symbols — becomes its own token. The trie is still
consulted for a reading, which is how single-character lettered entries such as
`%` → `pat6 sen1` are handled. In particular, `%` is never absorbed into an
alpha run, so `3%` always splits into two tokens `3` and `%`, allowing the
Cantonese reading of `%` to be displayed independently.

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

- [auto-canto](https://github.com/VincentTam/auto-canto) — the Typst package
  that processes the output of this crate for automatic Catonese annotation
- [PyCantonese](https://pycantonese.org) — the Python library that inspired
  this project
- [to-jyutping](https://github.com/CanCLID/to-jyutping) — to NodeJS package
  that inspired the trie structure in this project

## License

MIT

Data bundled from rime-cantonese is licensed under CC BY 4.0 — see
[`data/README.md`](data/README.md) for details.
