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
