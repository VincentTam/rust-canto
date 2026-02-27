use unicode_normalization::UnicodeNormalization;

/// Convert a Jyutping string (may contain multiple syllables separated by spaces)
/// to Yale romanization with tone numbers (e.g. "keoi5" → "keui5")
/// or with Yale diacritics (e.g. "keoi5" → "kéuih")
pub fn jyutping_to_yale(jyutping: &str, diacritics: bool) -> Option<String> {
    let syllables: Vec<&str> = jyutping.split_whitespace().collect();
    if syllables.is_empty() {
        return None;
    }

    let converted: Vec<String> = syllables
        .iter()
        .filter_map(|s| convert_syllable(s, diacritics))
        .collect();

    if converted.is_empty() {
        None
    } else {
        // normalize to NFC so combining diacritics collapse to precomposed chars
        // e.g. "i" + U+0304 → "ī" as a single codepoint
        Some(converted.join(" ").nfc().collect())
    }
}

fn convert_syllable(syllable: &str, diacritics: bool) -> Option<String> {
    // split tone number off the end
    let (body, tone) = split_tone(syllable)?;

    // convert initial
    let (initial, rest) = convert_initial(body);

    // convert final (vowel + coda)
    let mut final_part = convert_final(rest);

    // bare "aa" (no coda) → "a" in Yale
    if final_part == "aa" {
        final_part = "a".to_string();
    }

    if diacritics {
        Some(apply_diacritic(initial, &final_part, tone))
    } else {
        Some(format!("{}{}{}", initial, final_part, tone))
    }
}

/// Returns (body_without_tone, tone_number)
fn split_tone(s: &str) -> Option<(&str, u8)> {
    let last = s.chars().last()?;
    if last.is_ascii_digit() {
        let tone = last.to_digit(10)? as u8;
        Some((&s[..s.len() - 1], tone))
    } else {
        None
    }
}

/// Returns (yale_initial, remaining_final)
fn convert_initial(body: &str) -> (&str, &str) {
    // order matters — check longer initials first
    if let Some(rest) = body.strip_prefix("gw") { return ("gw", rest); }
    if let Some(rest) = body.strip_prefix("kw") { return ("kw", rest); }
    if let Some(rest) = body.strip_prefix("ng") { return ("ng", rest); }
    if let Some(rest) = body.strip_prefix('z')  { return ("j",  rest); }
    if let Some(rest) = body.strip_prefix('c')  { return ("ch", rest); }
    if let Some(rest) = body.strip_prefix('j')  { return ("y",  rest); }
    // initials identical in both systems: b p m f d t n l g k h s w
    for i in ["b","p","m","f","d","t","n","l","g","k","h","s","w"] {
        if let Some(rest) = body.strip_prefix(i) {
            return (i, rest);
        }
    }
    ("", body)  // no initial (vowel-initial syllable)
}

/// Convert Jyutping final to Yale final
fn convert_final(fin: &str) -> String {
    fin
        .replace("eoi",  "eui")   // eoi  → eui
        .replace("oeng", "eung")  // oeng → eung
        .replace("oek",  "euk")   // oek  → euk
        .replace("oe",   "eu")    // oe   → eu
        .replace("eo",   "eu")    // eo   → eu
        // all aa finals (aam, aan, aang, aap, aat, aak, aai, aau) stay as-is
        // bare "aa" is handled separately in convert_syllable
}

/// Split final into (nucleus, coda)
/// coda = trailing consonant: ng, p, t, k, m, n
/// trailing glides i, u are part of the nucleus
fn split_nucleus_coda<'a>(fin: &'a str) -> (&'a str, &'a str) {
    for coda in &["ng", "p", "t", "k", "m", "n"] {
        if fin.ends_with(coda) {
            let nucleus = &fin[..fin.len() - coda.len()];
            return (nucleus, coda);
        }
    }
    (fin, "")
}

/// Apply Yale diacritic tones
/// High register (1-3): diacritic on first vowel, no h
/// Low register (4-6):  diacritic on first vowel + h after nucleus, before coda
/// Tone 1: macron ā   Tone 4: grave + h àh
/// Tone 2: acute á    Tone 5: acute + h áh
/// Tone 3: no mark    Tone 6: no mark + h
fn apply_diacritic(initial: &str, fin: &str, tone: u8) -> String {
    let vowels = ['a', 'e', 'i', 'o', 'u'];
    let low_register = tone >= 4;

    let diacritic: Option<char> = match tone {
        1 => Some('\u{0304}'),  // macron  ā
        2 => Some('\u{0301}'),  // acute   á
        3 => None,              // no mark — mid level tone
        4 => Some('\u{0300}'),  // grave   à (low falling)
        5 => Some('\u{0301}'),  // acute   á (low rising)
        6 => None,              // no mark (low level)
        _ => None,
    };

    let (nucleus, coda) = split_nucleus_coda(fin);

    // place diacritic on first vowel of nucleus
    let mut result = String::from(initial);
    let mut marked = false;
    for ch in nucleus.chars() {
        result.push(ch);
        if !marked && vowels.contains(&ch) {
            if let Some(d) = diacritic {
                result.push(d);
            }
            marked = true;
        }
    }

    // h goes after entire nucleus, before coda
    if low_register {
        result.push('h');
    }

    result.push_str(coda);
    result
}

/// Returns one Yale syllable per Jyutping syllable, matching pycantonese output.
/// e.g. "nei5 hou2 aa3" → ["néih", "hóu", "a"]
pub fn jyutping_to_yale_vec(jyutping: &str) -> Option<Vec<String>> {
    let syllables: Vec<&str> = jyutping.split_whitespace().collect();
    if syllables.is_empty() {
        return None;
    }

    let converted: Vec<String> = syllables
        .iter()
        .filter_map(|s| convert_syllable(s, true))
        .map(|s| s.nfc().collect())
        .collect();

    if converted.is_empty() { None } else { Some(converted) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yale_numeric() {
        // initials
        assert_eq!(jyutping_to_yale("zi1",  false), Some("ji1".into()));
        assert_eq!(jyutping_to_yale("ci1",  false), Some("chi1".into()));
        assert_eq!(jyutping_to_yale("ji1",  false), Some("yi1".into()));
        // finals
        assert_eq!(jyutping_to_yale("keoi5", false), Some("keui5".into()));
        assert_eq!(jyutping_to_yale("heoi3", false), Some("heui3".into()));
        // bare aa → a
        assert_eq!(jyutping_to_yale("aa3",  false), Some("a3".into()));
        // aa finals stay intact
        assert_eq!(jyutping_to_yale("saan1", false), Some("saan1".into()));
        assert_eq!(jyutping_to_yale("baak3", false), Some("baak3".into()));
        assert_eq!(jyutping_to_yale("haam4", false), Some("haam4".into()));
        // multi-syllable
        assert_eq!(
            jyutping_to_yale("gwong2 dung1 waa2", false),
            Some("gwong2 dung1 wa2".into())
        );
    }

    #[test]
    fn test_yale_diacritics() {
        // tone 3: no mark
        assert_eq!(jyutping_to_yale("si3",   true), Some("si".into()));
        assert_eq!(jyutping_to_yale("heoi3", true), Some("heui".into()));

        // tone 1: macron
        assert_eq!(jyutping_to_yale("si1",   true), Some("sī".into()));
        assert_eq!(jyutping_to_yale("jat1",  true), Some("yāt".into()));

        // tone 2: acute
        assert_eq!(jyutping_to_yale("hou2",  true), Some("hóu".into()));

        // tone 4: grave + h
        assert_eq!(jyutping_to_yale("haam4", true), Some("hàahm".into()));

        // tone 5: acute + h after nucleus
        assert_eq!(jyutping_to_yale("ngo5",  true), Some("ngóh".into()));

        // tone 6: no mark + h after nucleus
        assert_eq!(jyutping_to_yale("hai6",  true), Some("haih".into()));
        assert_eq!(jyutping_to_yale("hok6",  true), Some("hohk".into()));
        assert_eq!(jyutping_to_yale("sap6",  true), Some("sahp".into()));

        // aa finals with diacritics
        assert_eq!(jyutping_to_yale("saan1", true), Some("sāan".into()));
        assert_eq!(jyutping_to_yale("baak3", true), Some("baak".into()));
    }
}
