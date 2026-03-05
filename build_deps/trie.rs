use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct TrieNode {
    pub children: HashMap<char, TrieNode>,
    pub readings: Vec<String>,
    #[serde(skip_serializing)]
    pub char_weights: Vec<u32>, // parallel to readings, for sorting by weight
    pub freq: i64,
}

impl TrieNode {
    pub fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            readings: Vec::new(),
            char_weights: Vec::new(),
            freq: 0,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Trie {
    pub root: TrieNode,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            root: TrieNode::new(),
        }
    }

    /// Insert a single CJK character with a weighted reading.
    /// Higher weight = more common pronunciation = inserted earlier in readings[].
    /// Entries with no percentage in chars.tsv get weight=100 (highest priority).
    pub fn insert_char(&mut self, ch: char, reading: &str, weight: u32) {
        let node = self.root.children.entry(ch).or_insert_with(TrieNode::new);
        let r = reading.to_string();
        if !node.readings.contains(&r) {
            let pos = node
                .char_weights
                .iter()
                .position(|&w| w < weight)
                .unwrap_or(node.readings.len());
            node.readings.insert(pos, r);
            node.char_weights.insert(pos, weight);
        }
    }

    /// Insert a multi-character CJK word (words.tsv).
    /// Skips single-character entries — use insert_char for those.
    pub fn insert_word(&mut self, word: &str, reading: &str) {
        if word.chars().count() < 2 {
            return;
        }
        let mut node = &mut self.root;
        for ch in word.chars() {
            node = node.children.entry(ch).or_insert_with(TrieNode::new);
        }
        let r = reading.to_string();
        if !node.readings.contains(&r) {
            node.readings.push(r);
        }
    }

    /// Insert a word frequency for use as a DP tiebreaker.
    /// Only updates nodes already in the trie (from insert_char/insert_word).
    pub fn insert_freq(&mut self, word: &str, freq: i64) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            match node.children.get_mut(&ch) {
                None => return,
                Some(child) => node = child,
            }
        }
        node.freq = freq;
    }

    /// Insert an entry from the lettered dict (lettered.tsv).
    /// Unlike insert_word, allows single-character entries (%, D, K, ...)
    /// and mixed Latin+CJK entries (AB膠, chok-cheat, Hap唔Happy呀).
    pub fn insert_lettered(&mut self, word: &str, reading: &str) {
        if word.is_empty() {
            return;
        }
        let mut node = &mut self.root;
        for ch in word.chars() {
            node = node.children.entry(ch).or_insert_with(TrieNode::new);
        }
        let r = reading.to_string();
        if !node.readings.contains(&r) {
            node.readings.push(r);
        }
    }
}
