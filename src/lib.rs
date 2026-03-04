mod trie;
use trie::Trie;
mod token;
use token::Token;
mod utils;
use utils::{decompress, build_trie_from_raw};
mod yale;
use yale::{jyutping_to_yale, jyutping_to_yale_vec};

use std::sync::LazyLock;
use wasm_minimal_protocol::*;

// Use compressed data
const CHAR_DATA_GZ: &[u8] = include_bytes!("../data/chars.tsv.gz");
const WORD_DATA_GZ: &[u8] = include_bytes!("../data/words.tsv.gz");
const FREQ_DATA_GZ: &[u8] = include_bytes!("../data/freq.txt.gz");
const LETTERED_DATA_GZ: &[u8] = include_bytes!("../data/lettered.tsv.gz");

initiate_protocol!();

static TRIE: LazyLock<Trie> = LazyLock::new(|| {
    let char_s = decompress(CHAR_DATA_GZ);
    let word_s = decompress(WORD_DATA_GZ);
    let freq_s = decompress(FREQ_DATA_GZ);
    let lett_s = decompress(LETTERED_DATA_GZ);

    build_trie_from_raw(&char_s, &word_s, &freq_s, &lett_s)
});

#[wasm_func]
pub fn annotate(input: &[u8]) -> Vec<u8> {
    let text = std::str::from_utf8(input).unwrap_or("");
    let tokens = TRIE.segment(text);

    let output: Vec<Token> = tokens
        .into_iter()
        .map(|t| Token {
            word: t.word,
            yale: t.reading.as_deref().and_then(jyutping_to_yale_vec),
            reading: t.reading,
        })
        .collect();

    serde_json::to_string(&output)
        .unwrap_or_else(|_| "[]".to_string())
        .into_bytes()
}

/// Input: jyutping bytes, e.g. b"gwong2 dung1 waa2"
/// Output: Yale with tone numbers, e.g. b"gwong2 dung1 waa2"
#[wasm_func]
pub fn to_yale_numeric(input: &[u8]) -> Vec<u8> {
    let jp = std::str::from_utf8(input).unwrap_or("");
    jyutping_to_yale(jp, false)
        .unwrap_or_default()
        .into_bytes()
}

/// Input: jyutping bytes
/// Output: Yale with diacritics, e.g. b"gwóngdūngwá"
#[wasm_func]
pub fn to_yale_diacritics(input: &[u8]) -> Vec<u8> {
    let jp = std::str::from_utf8(input).unwrap_or("");
    jyutping_to_yale(jp, true)
        .unwrap_or_default()
        .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmentation() {
        let cases: Vec<(&str, Vec<(&str, Option<&str>)>)> = vec![
            // --- basic CJK ---
            (
                "佢係好學生",
                vec![
                    ("佢",   Some("keoi5")),
                    ("係",   Some("hai6")),
                    ("好",   Some("hou2")),
                    ("學生", Some("hok6 saang1")),
                ],
            ),
            // --- CJK + special chars + lettered dict (no space before AB膠) ---
            (
                "都會大學入面3%人識用AB膠",
                vec![
                    ("都會大學", Some("dou1 wui6 daai6 hok6")),
                    ("入面",    Some("jap6 min6")),
                    ("3",      None),               // digit: alpha run, no dict entry
                    ("%",      Some("pat6 sen1")),   // single-char lettered entry
                    ("人",     Some("jan4")),
                    ("識",     Some("sik1")),
                    ("用",     Some("jung6")),
                    ("AB膠",   Some("ei1 bi1 gaau1")), // mixed lettered dict entry
                ],
            ),
            // --- pure alpha non-lettered-word run at start ---
            (
                "abc",
                vec![
                    ("abc", None),
                ],
            ),
            // --- pure alpha lettered-word run at start ---
            (
                "ge",
                vec![
                    ("ge", Some("ge3")),
                ],
            ),
            // --- alpha run beside CJK, with space ---
            (
                "ABCD 一二",
                vec![
                    ("ABCD", None),
                    (" ",    None),
                    ("一",   Some("jat1")),
                    ("二",   Some("ji6")),
                ],
            ),
            // --- accented letter in alpha run ---
            (
                "café好",
                vec![
                    ("café", Some("kat6 fei1")),
                    ("好",   Some("hou2")),
                ],
            ),
            // --- hyphenated lettered dict entry ---
            (
                "我做part-time",
                vec![
                    ("我",          Some("ngo5")),
                    ("做part-time", Some("zou6 paat1 taai1")),
                ],
            ),
            // --- mixed CJK+Latin lettered entry ---
            (
                "Hap唔Happy呀",
                vec![
                    ("Hap唔Happy呀", Some("hep1 m4 hep1 pi2 aa3")),
                ],
            ),
            // --- newline becomes its own token ---
            (
                "你好\n世界",
                vec![
                    ("你",  Some("nei5")),
                    ("好",  Some("hou2")),
                    ("\n",  None),
                    ("世界", Some("sai3 gaai3")),
                ],
            ),
        ];

        for (input, expected) in &cases {
            println!("Testing: {}", input);
            let result = TRIE.segment(input);
            assert_eq!(
                result.len(), expected.len(),
                "token count mismatch for {:?}: got [{}]",
                input,
                result.iter().map(|t| format!("{:?}", t.word)).collect::<Vec<_>>().join(", ")
            );
            for (i, token) in result.iter().enumerate() {
                assert_eq!(
                    token.word, expected[i].0,
                    "word mismatch at index {} for {:?}", i, input
                );
                assert_eq!(
                    token.reading.as_deref(), expected[i].1,
                    "reading mismatch at index {} for {:?} (word={:?})", i, input, token.word
                );
            }
        }
    }
}
