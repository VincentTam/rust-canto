mod trie;
mod token;
mod utils;
mod yale;
use yale::{jyutping_to_yale, jyutping_to_yale_vec};

use trie::Trie;
use token::Token;
use once_cell::sync::Lazy;
use wasm_minimal_protocol::*;

const CHAR_DATA: &str = include_str!("../data/chars.tsv");
const WORD_DATA: &str = include_str!("../data/words.tsv");
const FREQ_DATA: &str = include_str!("../data/freq.txt");
const LETTERED_DATA: &str = include_str!("../data/lettered.tsv");

initiate_protocol!();

static TRIE: Lazy<Trie> = Lazy::new(|| build_trie());

fn build_trie() -> Trie {
    let mut trie = Trie::new();

    for line in CHAR_DATA.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            if let Some(ch) = parts[0].chars().next() {
                // parse "5%" → 5, missing → 100 (highest priority)
                let weight = parts.get(2)
                    .map(|s| s.replace('%', "").trim().parse::<u32>().unwrap_or(0))
                    .unwrap_or(100);
                trie.insert_char(ch, parts[1], weight);
            }
        }
    }

    for line in WORD_DATA.lines() {
        let Some((left, right)) = line.split_once('\t') else {
            continue;
        };
        trie.insert_word(left, right);
    }

    for line in FREQ_DATA.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            if let Ok(freq) = parts[1].parse::<i64>() {
                trie.insert_freq(parts[0], freq);
            }
        }
    }

    for line in LETTERED_DATA.lines() {
        let Some((left, right)) = line.split_once('\t') else {
            continue;
        };
        trie.insert_lettered(left, right);
    }

    trie
}

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
        let trie = build_trie();

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
                    ("入面",     Some("jap6 min6")),
                    ("3",        None),               // digit: alpha run, no dict entry
                    ("%",        Some("pat6 sen1")),   // single-char lettered entry
                    ("人",       Some("jan4")),
                    ("識",       Some("sik1")),
                    ("用",       Some("jung6")),
                    ("AB膠",     Some("ei1 bi1 gaau1")), // mixed lettered dict entry
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
                    ("我",        Some("ngo5")),
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
                    ("你", Some("nei5")),
                    ("好", Some("hou2")),
                    ("\n",   None),
                    ("世界", Some("sai3 gaai3")),
                ],
            ),
        ];

        for (input, expected) in &cases {
            println!("Testing: {}", input);
            let result = trie.segment(input);
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
