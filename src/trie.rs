use crate::token::Token;
use std::collections::HashMap;

pub struct TrieNode {
    pub children: HashMap<char, TrieNode>,
    pub readings: Vec<String>,
    pub char_weights: Vec<u32>,  // parallel to readings, for sorting
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

pub struct Trie {
    pub root: TrieNode,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            root: TrieNode::new(),
        }
    }

    pub fn insert_char(&mut self, ch: char, reading: &str, weight: u32) {
        let node = self.root.children
            .entry(ch)
            .or_insert_with(TrieNode::new);
        let r = reading.to_string();
        if !node.readings.contains(&r) {
            // insert so that higher weight readings come first
            let pos = node.char_weights
                .iter()
                .position(|&w| w < weight)
                .unwrap_or(node.readings.len());
            node.readings.insert(pos, r);
            node.char_weights.insert(pos, weight);
        }
    }

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

    pub fn insert_freq(&mut self, word: &str, freq: i64) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            match node.children.get_mut(&ch) {
                None => return,  // word not in trie, skip
                Some(child) => node = child,
            }
        }
        node.freq = freq;
    }

    /// Like insert_word but allows single-character entries (for lettered dict).
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

    /// DP segmentation for a chunk of non-ASCII text.
    ///
    /// dp[i] = (token_count, total_freq) for the best segmentation of the
    /// first i characters. We minimise token_count; on a tie we maximise
    /// total_freq so that high-frequency words are preferred.
    ///
    /// Example for "好學生":
    ///   dp[0] = (0, 0)          ← base: empty prefix costs 0 tokens
    ///   dp[1] = (1, freq(好))   ← "好" is one token
    ///   dp[2] = (1, freq(好學)) ← "好學" found in trie: still 1 token from dp[0]
    ///   dp[3] = (2, freq(好學)+freq(生))  ← fallback: "好學"+"生"
    ///         vs (2, freq(好)+freq(學生)) ← "好"+"學生"; freq(學生)=71278 wins
    ///   → reconstruct: track[3]=(1,"學生"), track[1]=(0,"好") → ["好","學生"]
    pub fn segment(&self, text: &str) -> Vec<Token> {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();

        // dp[i] = (token_count, total_freq) — minimise tokens, maximise freq
        let mut dp: Vec<(usize, i64)> = vec![(usize::MAX, 0); n + 1];
        let mut track: Vec<(usize, Option<String>)> = vec![(0, None); n + 1];
        dp[0] = (0, 0);

        for end in 1..=n {
            // fallback: single character
            if dp[end - 1].0 != usize::MAX {
                // look up reading for this single char from the trie
                let single_reading = self.root
                    .children
                    .get(&chars[end - 1])
                    .and_then(|n| n.readings.first().cloned());
                let cost = (dp[end - 1].0 + 1, dp[end - 1].1);
                if Self::better(&cost, &dp[end]) {
                    dp[end] = cost;
                    track[end] = (end - 1, single_reading);  // ← was always None before
                }
            }

            // try all start positions, walk trie left-to-right
            for start in (0..end).rev() {
                if dp[start].0 == usize::MAX {
                    continue;
                }
                let mut node = &self.root;
                for j in start..end {
                    let ch = chars[j];
                    match node.children.get(&ch) {
                        None => break,
                        Some(child) => {
                            node = child;
                            if j == end - 1 && !node.readings.is_empty() {
                                let cost = (dp[start].0 + 1, dp[start].1 + node.freq);
                                if Self::better(&cost, &dp[end]) {
                                    dp[end] = cost;
                                    track[end] = (start, Some(node.readings[0].clone()));
                                }
                            }
                        }
                    }
                }
            }
        }

        // reconstruct
        let mut tokens = Vec::new();
        let mut curr = n;
        while curr > 0 {
            let (prev, reading) = &track[curr];
            let word: String = chars[*prev..curr].iter().collect();
            tokens.push(Token {
                word,
                reading: reading.clone(),
                yale: None,  // filled in by annotate() in lib.rs after segmentation
            });
            curr = *prev;
        }
        tokens.reverse();
        tokens
    }

    // fewer tokens wins; on tie, higher freq wins
    fn better(candidate: &(usize, i64), current: &(usize, i64)) -> bool {
        if candidate.0 != current.0 {
            candidate.0 < current.0 // fewer tokens
        } else {
            candidate.1 > current.1 // higher freq
        }
    }
}
