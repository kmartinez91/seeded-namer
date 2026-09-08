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

const ONSETS: &[&str] = &[
    "b", "br", "c", "ch", "cr", "d", "dr", "f", "fr", "g", "gr", "h", "j", "k", "kr", "l", "m",
    "n", "p", "pr", "qu", "r", "s", "sh", "sk", "st", "str", "t", "th", "tr", "v", "w", "z",
];

const VOWELS: &[&str] = &["a", "e", "i", "o", "u", "ae", "ai", "ia", "io", "ou"];

const CODAS: &[&str] = &[
    "", "", "", "n", "r", "s", "l", "th", "nd", "rk", "ll", "ss", "x", "m", "sh",
];

/// Builds one syllable from a state, returning the syllable and the next state.
fn syllable(state: u64) -> (String, u64) {
    let (onset_i, state) = pick_index(state, ONSETS.len());
    let (vowel_i, state) = pick_index(state, VOWELS.len());
    let (coda_i, state) = pick_index(state, CODAS.len());
    let text = format!("{}{}{}", ONSETS[onset_i], VOWELS[vowel_i], CODAS[coda_i]);
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

/// Generates one name from a seed. Two or three syllables, first letter
/// capitalized, everything else lowercase.
pub fn generate_name(seed: u64) -> String {
    let (syllable_count, state) = pick_index(seed, 2);
    let syllable_count = 2 + syllable_count; // 2 or 3

    let mut name = String::new();
    let mut state = state;
    for _ in 0..syllable_count {
        let (part, next_state) = syllable(state);
        name.push_str(&part);
        state = next_state;
    }
    capitalize(&name)
}

/// Generates `count` names starting from `seed`. Each name in the sequence
/// is derived from the previous name's ending state, so the whole sequence
/// is a pure function of `(seed, count)`: calling it twice with the same
/// arguments always yields the same `Vec`, and `name_sequence(seed, n)` is
/// always a prefix of `name_sequence(seed, n + 1)`.
pub fn name_sequence(seed: u64, count: usize) -> Vec<String> {
    let mut names = Vec::with_capacity(count);
    let mut state = seed;
    for _ in 0..count {
        names.push(generate_name(state));
        let (_, next_state) = next_u64(state);
        state = next_state;
    }
    names
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
    fn capitalize_handles_empty_string() {
        assert_eq!(capitalize(""), "");
    }
}
