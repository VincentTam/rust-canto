use flate2::read::GzDecoder;
use std::io::Read;
use crate::trie::Trie;

/// Decompresses Gzip bytes into a String
pub fn decompress(bytes: &[u8]) -> String {
    let mut decoder = GzDecoder::new(bytes);
    let mut s = String::new();
    decoder.read_to_string(&mut s).expect("Failed to decompress embedded data");
    s
}

/// Building the Trie from raw string data
pub fn build_trie_from_raw(chars: &str, words: &str, freq: &str, lettered: &str) -> Trie {
    let mut trie = Trie::new();

    for line in chars.lines() {
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

    for line in words.lines() {
        let Some((left, right)) = line.split_once('\t') else {
            continue;
        };
        trie.insert_word(left, right);
    }

    for line in freq.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            if let Ok(freq) = parts[1].parse::<i64>() {
                trie.insert_freq(parts[0], freq);
            }
        }
    }

    for line in lettered.lines() {
        let Some((left, right)) = line.split_once('\t') else {
            continue;
        };
        trie.insert_lettered(left, right);
    }

    trie
}

/// True for CJK ideographs, including extension blocks needed for
/// rare Cantonese characters like 𠮩 (U+20BA9) and 𠹌 (U+20E4C).
pub fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}'     // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}'   // CJK Extension A
        | '\u{20000}'..='\u{2A6DF}' // CJK Extension B
        | '\u{2A700}'..='\u{2B73F}' // CJK Extension C
        | '\u{2B740}'..='\u{2B81F}' // CJK Extension D
        | '\u{2B820}'..='\u{2CEAF}' // CJK Extension E
        | '\u{F900}'..='\u{FAFF}'   // CJK Compatibility Ideographs
    )
}

/// True if `ch` is a letter or digit but not a CJK ideograph.
/// These are the characters that form the body of an alpha run
/// (e.g. ASCII letters, digits, accented letters like é).
pub fn is_alpha_char(ch: char) -> bool {
    ch.is_alphanumeric() && !is_cjk(ch)
}

/// True if `ch` is an intra-word connector: hyphen, underscore, or apostrophe.
/// Connectors are allowed *inside* an alpha run but not at the start or end.
/// Examples: "part-time", "rust_canto", "i'm"
/// Non-examples: "-abc" (leading), "abc-" (trailing), "3%" (% is not a connector)
pub fn is_connector(ch: char) -> bool {
    matches!(ch, '-' | '_' | '\'')
}
