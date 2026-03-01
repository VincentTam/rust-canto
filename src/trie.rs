use crate::token::Token;
use crate::utils::{is_alpha_char, is_connector};
use std::collections::HashMap;

pub struct TrieNode {
    pub children: HashMap<char, TrieNode>,
    pub readings: Vec<String>,
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

    /// Segment text into tokens using trie + dynamic programming.
    ///
    /// dp[i] = (token_count, total_freq) for the best segmentation of the
    /// first i characters. We minimise token_count; on a tie we maximise
    /// total_freq so that high-frequency words are preferred over rare ones.
    ///
    /// Example for "好學生":
    ///   dp[0] = (0, 0)              ← base: empty string costs 0 tokens
    ///   dp[1] = (1, freq(好))       ← "好" as one token
    ///   dp[2] = (1, freq(好學))     ← "好學" in trie: 1 token from dp[0]
    ///   dp[3] = (2, freq(好學)+freq(生))  ← "好學" + "生"
    ///         vs (2, freq(好)+freq(學生)) ← "好" + "學生"
    ///         freq(學生)=71278 >> freq(好學)=2847 → "好"+"學生" wins
    ///   → reconstruct: track[3]=(1,"學生"), track[1]=(0,"好") → ["好","學生"]
    ///
    /// Tokenisation rules for non-CJK characters:
    ///
    /// 1. ALPHA RUNS — a contiguous span where every character is either:
    ///    - a non-CJK alphanumeric (letter or digit, including accented letters
    ///      like é since Rust's `is_alphanumeric()` covers all Unicode letters), or
    ///    - an intra-word connector (hyphen `-`, underscore `_`, apostrophe `'`)
    ///      that is surrounded by alphanumeric chars on both sides
    ///    is merged into one token. This handles:
    ///      "package"    → one token (no dict entry needed)
    ///      "café"       → one token (é is alphanumeric)
    ///      "part-time"  → one token if in lettered dict; otherwise hyphen splits it
    ///      "rust_canto" → one token
    ///      "i'm"        → one token
    ///    The trie walk always runs first. If the trie finds a reading for the span
    ///    (e.g. "ge" → "ge3", "café" → "kat6 fei1"), that reading is used. The
    ///    alpha-run fallback only fires when the trie has no entry, giving reading=None.
    ///
    /// 2. STANDALONE TOKENS — characters that are never part of an alpha run:
    ///    - Whitespace (space, tab, newline) → each becomes its own token, no reading
    ///    - Punctuation and symbols, including `%` → each becomes its own token;
    ///      the trie is checked for a reading (e.g. "%" → "pat6 sen1")
    ///    This ensures "3%" splits into "3" (alpha run) + "%" (standalone), so that
    ///    the Cantonese reading of "%" can be displayed independently.
    pub fn segment(&self, text: &str) -> Vec<Token> {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();

        let mut dp: Vec<(usize, i64)> = vec![(usize::MAX, 0); n + 1];
        let mut track: Vec<(usize, Option<String>)> = vec![(0, None); n + 1];
        dp[0] = (0, 0);

        for end in 1..=n {
            // --- single-character fallback ---
            // Covers whitespace, punctuation, symbols, and any character with no
            // better multi-char match. Checks the trie for a reading so that
            // single-char lettered entries like "%" → "pat6 sen1" are not lost.
            if dp[end - 1].0 != usize::MAX {
                let single_reading = self
                    .root
                    .children
                    .get(&chars[end - 1])
                    .and_then(|n| n.readings.first().cloned());
                let cost = (dp[end - 1].0 + 1, dp[end - 1].1);
                if Self::better(&cost, &dp[end]) {
                    dp[end] = cost;
                    track[end] = (end - 1, single_reading);
                }
            }

            // --- multi-character spans ---
            for start in (0..end).rev() {
                if dp[start].0 == usize::MAX {
                    continue;
                }

                // TRIE WALK: look up chars[start..end] in the trie.
                // Matches CJK words (words.tsv), mixed Latin+CJK entries (AB膠,
                // Hap唔Happy呀), hyphenated entries (chok-cheat, part-time), and
                // any other lettered dict entries that carry a Jyutping reading.
                // trie_matched is set as soon as a reading is found at end-1,
                // regardless of whether that reading wins dp[end], so that the
                // alpha-run fallback below stays silent for known words.
                let mut node = &self.root;
                let mut trie_matched = false;
                for j in start..end {
                    let ch = chars[j];
                    match node.children.get(&ch) {
                        None => break,
                        Some(child) => {
                            node = child;
                            if j == end - 1 && !node.readings.is_empty() {
                                trie_matched = true;
                                let cost = (dp[start].0 + 1, dp[start].1 + node.freq);
                                if Self::better(&cost, &dp[end]) {
                                    dp[end] = cost;
                                    track[end] = (start, Some(node.readings[0].clone()));
                                }
                            }
                        }
                    }
                }

                // Determine whether chars[start..end] qualifies as an alpha run:
                // every character must be a non-CJK alphanumeric or a connector,
                // and the first and last characters must be alphanumeric (no leading
                // or trailing connectors).
                let span_is_alpha_run = {
                    let span = &chars[start..end];
                    span.iter().all(|&c| is_alpha_char(c) || is_connector(c))
                        && span.first().map(|&c| is_alpha_char(c)).unwrap_or(false)
                        && span.last().map(|&c| is_alpha_char(c)).unwrap_or(false)
                };

                // ALPHA RUN fallback — fires only when the trie has no entry for
                // this span, ensuring that words with dict readings (e.g. "ge" → "ge3")
                // are never silently downgraded to reading=None.
                if !trie_matched && span_is_alpha_run {
                    let cost = (dp[start].0 + 1, dp[start].1);
                    if Self::better(&cost, &dp[end]) {
                        dp[end] = cost;
                        track[end] = (start, None);
                    }
                }
            }
        }

        // reconstruct token sequence by following track[] backwards
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

    /// Fewer tokens wins; on a tie, higher total frequency wins.
    fn better(candidate: &(usize, i64), current: &(usize, i64)) -> bool {
        if candidate.0 != current.0 {
            candidate.0 < current.0
        } else {
            candidate.1 > current.1
        }
    }
}
