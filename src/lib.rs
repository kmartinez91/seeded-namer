//! Turns a u64 seed into a name. Same seed in, same name out, always.
//!
//! Everything public here is a pure function: no globals, no I/O, no clocks.
//! State is threaded explicitly (seed in, next-seed out) instead of hidden
//! behind a `&mut` RNG struct, so every step can be called and checked in
//! isolation without constructing anything first.

/// One step of the splitmix64 generator: given a state, returns a pseudo-random
/// value derived from it and the state to use for the next step.
///
/// splitmix64 was picked over something simpler (e.g. a linear congruential
/// generator) because it has good bit dispersion even for seeds that are
/// small or sequential (0, 1, 2, ...), which matters here since callers often
/// pass in small counters or test ids as seeds.
pub fn next_u64(state: u64) -> (u64, u64) {
    let next_state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = next_state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    let value = z ^ (z >> 31);
    (value, next_state)
}

/// Picks an index in `0..len` from a state, returning the index and the next state.
///
/// `len` must be nonzero; every call site here passes a fixed non-empty
/// slice length, so that's an invariant rather than a runtime condition.
pub fn pick_index(state: u64, len: usize) -> (usize, u64) {
    debug_assert!(len > 0, "pick_index called with an empty range");
    let (value, next_state) = next_u64(state);
    ((value % len as u64) as usize, next_state)
}

/// Picks an index into `weights` with probability proportional to each weight,
/// returning the index and the next state.
///
/// The total weight must be nonzero. Entries with weight 0 are never picked.
/// For a table where every weight is 1 this is the same draw as
/// [`pick_index`], and repeating an entry `n` times is the same as giving it
/// weight `n`, so tables that used repeats to bias a pick can switch to
/// explicit weights without changing any seed's output.
pub fn pick_weighted(state: u64, weights: &[u32]) -> (usize, u64) {
    let total: u64 = weights.iter().map(|&w| w as u64).sum();
    debug_assert!(total > 0, "pick_weighted called with no weight to pick from");
    let (value, next_state) = next_u64(state);
    let mut roll = value % total;
    for (i, &w) in weights.iter().enumerate() {
        if roll < w as u64 {
            return (i, next_state);
        }
        roll -= w as u64;
    }
    unreachable!("roll is below the total weight, so some entry must claim it")
}

const ONSETS: &[&str] = &[
    "b", "br", "c", "ch", "cr", "d", "dr", "f", "fr", "g", "gr", "h", "j", "k", "kr", "l", "m",
    "n", "p", "pr", "qu", "r", "s", "sh", "sk", "st", "str", "t", "th", "tr", "v", "w", "z",
];

const VOWELS: &[&str] = &["a", "e", "i", "o", "u", "ae", "ai", "ia", "io", "ou"];

// Codas are (text, weight) pairs. The empty coda is weighted up so open
// syllables stay common.
type Coda = (&'static str, u32);

const CODAS: &[Coda] = &[
    ("", 3), ("n", 1), ("r", 1), ("s", 1), ("l", 1), ("th", 1), ("nd", 1), ("rk", 1),
    ("ll", 1), ("ss", 1), ("x", 1), ("m", 1), ("sh", 1),
];

const HARSH_ONSETS: &[&str] = &[
    "b", "br", "d", "dr", "g", "gr", "k", "kr", "kh", "z", "zg", "x", "thr", "vr", "grk",
];
const HARSH_VOWELS: &[&str] = &["a", "o", "u", "au", "ou", "ak", "ug"];
const HARSH_CODAS: &[Coda] = &[
    ("k", 1), ("g", 1), ("z", 1), ("rk", 1), ("zg", 1), ("gg", 1), ("kk", 1), ("th", 1),
    ("x", 1), ("grn", 1),
];

const SOFT_ONSETS: &[&str] = &["l", "m", "n", "s", "sh", "f", "v", "th", "w", "wh", "y"];
const SOFT_VOWELS: &[&str] = &["a", "e", "i", "o", "u", "ae", "ea", "ia", "io", "ui"];
const SOFT_CODAS: &[Coda] = &[
    ("", 3), ("n", 1), ("m", 1), ("l", 1), ("s", 1), ("th", 1), ("ne", 1), ("le", 1),
    ("se", 1),
];

const SCIFI_ONSETS: &[&str] = &[
    "x", "z", "zy", "qu", "vor", "kry", "xen", "jy", "nyx", "zir", "vex", "quy",
];
const SCIFI_VOWELS: &[&str] = &["a", "e", "i", "o", "u", "ax", "ex", "ix", "ox", "yx"];
const SCIFI_CODAS: &[Coda] = &[
    ("", 1), ("x", 1), ("z", 1), ("on", 1), ("ax", 1), ("ex", 1), ("ix", 1), ("tron", 1),
    ("nix", 1), ("zar", 1), ("vex", 1),
];

/// A syllable set to build names from. `Default` is the original set this
/// crate shipped with; the others exist so callers can match a name's
/// texture to their setting without forking the generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Default,
    Harsh,
    Soft,
    SciFi,
}

impl Style {
    fn syllable_parts(self) -> (&'static [&'static str], &'static [&'static str], &'static [Coda]) {
        match self {
            Style::Default => (ONSETS, VOWELS, CODAS),
            Style::Harsh => (HARSH_ONSETS, HARSH_VOWELS, HARSH_CODAS),
            Style::Soft => (SOFT_ONSETS, SOFT_VOWELS, SOFT_CODAS),
            Style::SciFi => (SCIFI_ONSETS, SCIFI_VOWELS, SCIFI_CODAS),
        }
    }
}

impl std::str::FromStr for Style {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "default" => Ok(Style::Default),
            "harsh" => Ok(Style::Harsh),
            "soft" => Ok(Style::Soft),
            "sci-fi" | "scifi" => Ok(Style::SciFi),
            other => Err(format!(
                "unknown style {:?}, expected one of: default, harsh, soft, sci-fi",
                other
            )),
        }
    }
}

/// Builds one syllable from a state, returning the syllable and the next state.
fn syllable(state: u64, style: Style) -> (String, u64) {
    let (onsets, vowels, codas) = style.syllable_parts();
    let (onset_i, state) = pick_index(state, onsets.len());
    let (vowel_i, state) = pick_index(state, vowels.len());
    let weights: Vec<u32> = codas.iter().map(|&(_, w)| w).collect();
    let (coda_i, state) = pick_weighted(state, &weights);
    let text = format!("{}{}{}", onsets[onset_i], vowels[vowel_i], codas[coda_i].0);
    (text, state)
}

/// Capitalizes the first character of `s`, leaving the rest untouched.
/// Returns `s` unchanged if it's empty.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Generates one name from a seed and style. Two or three syllables, first
/// letter capitalized, everything else lowercase.
pub fn generate_name_with_style(seed: u64, style: Style) -> String {
    let (syllable_count, state) = pick_index(seed, 2);
    let syllable_count = 2 + syllable_count; // 2 or 3

    let mut name = String::new();
    let mut state = state;
    for _ in 0..syllable_count {
        let (part, next_state) = syllable(state, style);
        name.push_str(&part);
        state = next_state;
    }
    capitalize(&name)
}

/// Generates one name from a seed using the default style. See
/// [`generate_name_with_style`] to pick a different syllable set.
pub fn generate_name(seed: u64) -> String {
    generate_name_with_style(seed, Style::Default)
}

/// Generates `count` names starting from `seed`, using `style`. Each name in
/// the sequence is derived from the previous name's ending state, so the
/// whole sequence is a pure function of `(seed, count, style)`: calling it
/// twice with the same arguments always yields the same `Vec`, and
/// `name_sequence_with_style(seed, n, style)` is always a prefix of
/// `name_sequence_with_style(seed, n + 1, style)`.
pub fn name_sequence_with_style(seed: u64, count: usize, style: Style) -> Vec<String> {
    let mut names = Vec::with_capacity(count);
    let mut state = seed;
    for _ in 0..count {
        names.push(generate_name_with_style(state, style));
        let (_, next_state) = next_u64(state);
        state = next_state;
    }
    names
}

/// Generates `count` names starting from `seed`, using the default style.
/// See [`name_sequence_with_style`] to pick a different syllable set.
pub fn name_sequence(seed: u64, count: usize) -> Vec<String> {
    name_sequence_with_style(seed, count, Style::Default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_same_name() {
        assert_eq!(generate_name(42), generate_name(42));
    }

    #[test]
    fn different_seeds_usually_give_different_names() {
        // Not a guarantee for every pair, but true for this specific pair,
        // which is enough to catch a generator that ignores the seed.
        assert_ne!(generate_name(1), generate_name(2));
    }

    #[test]
    fn name_is_capitalized_and_otherwise_lowercase() {
        let name = generate_name(7);
        let first = name.chars().next().unwrap();
        assert!(first.is_uppercase());
        assert!(name.chars().skip(1).all(|c| c.is_lowercase() || !c.is_alphabetic()));
    }

    #[test]
    fn name_is_never_empty() {
        for seed in 0..100u64 {
            assert!(!generate_name(seed).is_empty());
        }
    }

    #[test]
    fn sequence_is_a_prefix_of_a_longer_sequence() {
        let short = name_sequence(9, 3);
        let long = name_sequence(9, 6);
        assert_eq!(short, &long[..3]);
    }

    #[test]
    fn pick_index_stays_in_range() {
        for state in 0..50u64 {
            let (i, _) = pick_index(state, 7);
            assert!(i < 7);
        }
    }

    #[test]
    fn pick_weighted_with_equal_weights_matches_pick_index() {
        for state in 0..50u64 {
            assert_eq!(pick_weighted(state, &[1, 1, 1, 1, 1]), pick_index(state, 5));
        }
    }

    #[test]
    fn pick_weighted_matches_repeated_entries() {
        // [3, 1, 1] is the same table as [a, a, a, b, c].
        for state in 0..50u64 {
            let (weighted, _) = pick_weighted(state, &[3, 1, 1]);
            let (repeated, _) = pick_index(state, 5);
            let expected = match repeated {
                0..=2 => 0,
                3 => 1,
                _ => 2,
            };
            assert_eq!(weighted, expected);
        }
    }

    #[test]
    fn pick_weighted_never_picks_zero_weight() {
        for state in 0..200u64 {
            let (i, _) = pick_weighted(state, &[0, 2, 0, 1]);
            assert!(i == 1 || i == 3);
        }
    }

    #[test]
    fn pick_weighted_favors_heavier_entries() {
        let mut counts = [0u32; 2];
        for state in 0..1000u64 {
            counts[pick_weighted(state, &[9, 1]).0] += 1;
        }
        assert!(counts[0] > counts[1] * 3);
    }

    #[test]
    fn capitalize_handles_empty_string() {
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn style_parses_known_names() {
        assert_eq!("default".parse::<Style>(), Ok(Style::Default));
        assert_eq!("harsh".parse::<Style>(), Ok(Style::Harsh));
        assert_eq!("soft".parse::<Style>(), Ok(Style::Soft));
        assert_eq!("sci-fi".parse::<Style>(), Ok(Style::SciFi));
        assert_eq!("SCIFI".parse::<Style>(), Ok(Style::SciFi));
    }

    #[test]
    fn style_rejects_unknown_name() {
        assert!("robotic".parse::<Style>().is_err());
    }

    #[test]
    fn generate_name_matches_default_style() {
        assert_eq!(generate_name(42), generate_name_with_style(42, Style::Default));
    }

    #[test]
    fn different_styles_are_deterministic_per_style() {
        for style in [Style::Default, Style::Harsh, Style::Soft, Style::SciFi] {
            assert_eq!(
                generate_name_with_style(11, style),
                generate_name_with_style(11, style)
            );
        }
    }

    #[test]
    fn every_style_produces_nonempty_names() {
        for style in [Style::Default, Style::Harsh, Style::Soft, Style::SciFi] {
            for seed in 0..20u64 {
                assert!(!generate_name_with_style(seed, style).is_empty());
            }
        }
    }
}
