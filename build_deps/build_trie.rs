use super::trie::Trie;

const CHAR_DATA: &str = include_str!("../data/chars.tsv");
const WORD_DATA: &str = include_str!("../data/words.tsv");
const FREQ_DATA: &str = include_str!("../data/freq.txt");
const LETTERED_DATA: &str = include_str!("../data/lettered.tsv");

pub fn build_trie() -> Trie {
    let mut trie = Trie::new();

    for line in CHAR_DATA.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            if let Some(ch) = parts[0].chars().next() {
                // parse "5%" → 5, missing → 100 (highest priority)
                let weight = parts
                    .get(2)
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
