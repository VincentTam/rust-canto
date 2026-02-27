mod trie;
mod token;
mod yale;
use yale::jyutping_to_yale;
use yale::jyutping_to_yale_vec;

use trie::Trie;
use token::Token;
use once_cell::sync::Lazy;
use wasm_minimal_protocol::*;

const CHAR_DATA: &str = include_str!("../data/chars.tsv");
const WORD_DATA: &str = include_str!("../data/words.tsv");
const FREQ_DATA: &str = include_str!("../data/freq.txt");

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

        let cases = vec![
            (
                "都會大學",
                vec![("都會大學", Some("dou1 wui6 daai6 hok6"))],
            ),
            (
                "我會番教會",
                vec![
                    ("我", Some("ngo5")),
                    ("會", Some("wui5")),
                    ("番", Some("faan1")),
                    ("教會", Some("gaau3 wui2")),
                ],
            ),
            (
                "佢係好學生",
                vec![
                    ("佢", Some("keoi5")),
                    ("係", Some("hai6")),
                    ("好", Some("hou2")),
                    ("學生", Some("hok6 saang1")),
                ],
            ),
        ];

        for (input, expected) in cases {
            println!("Testing: {}", input);
            let result = trie.segment(input);
            assert_eq!(result.len(), expected.len(),
                "token count mismatch for '{}': got {:?}", input,
                result.iter().map(|t| &t.word).collect::<Vec<_>>()
            );
            for (i, token) in result.iter().enumerate() {
                assert_eq!(token.word, expected[i].0,
                    "word mismatch at index {} for '{}'", i, input);
                assert_eq!(token.reading.as_deref(), expected[i].1,
                    "reading mismatch at index {} for '{}'", i, input);
            }
        }
    }
}
