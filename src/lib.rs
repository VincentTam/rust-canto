mod trie;
mod token;

use trie::Trie;
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
                trie.insert_char(ch, parts[1]);
            }
        }
    }

    for line in WORD_DATA.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            trie.insert_word(parts[0], parts[1]);
        }
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
    serde_json::to_string(&tokens)
        .unwrap_or_else(|_| "[]".to_string())
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
                "好學生",
                vec![
                    ("好", Some("hou2")),
                    ("學生", Some("hok6 saang1")),
                ],
            ),
            (
                "我係好學生",
                vec![
                    ("我", Some("ngo5")),
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
