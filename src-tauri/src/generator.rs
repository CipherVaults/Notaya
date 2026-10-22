//! Password and passphrase generation, using only the OS secure RNG.

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::crypto::random_below;
use crate::error::{Error, Result};

const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()-_=+[]{};:,.?/~";
const AMBIGUOUS: &str = "Il1O0o";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordOptions {
    pub length: u32,
    pub lower: bool,
    pub upper: bool,
    pub digits: bool,
    pub symbols: bool,
    pub avoid_ambiguous: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassphraseOptions {
    pub words: u32,
    pub separator: String,
    pub capitalize: bool,
    pub include_number: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Generated {
    pub value: String,
    /// Bits of entropy: each bit doubles the guesses needed.
    pub entropy: f64,
}

pub fn password(opts: &PasswordOptions) -> Result<Generated> {
    if !(4..=256).contains(&opts.length) {
        return Err(Error::invalid("Length must be between 4 and 256."));
    }
    let filter = |set: &str| -> Vec<char> {
        set.chars().filter(|c| !opts.avoid_ambiguous || !AMBIGUOUS.contains(*c)).collect()
    };
    let classes: Vec<Vec<char>> = [
        (opts.lower, LOWER),
        (opts.upper, UPPER),
        (opts.digits, DIGITS),
        (opts.symbols, SYMBOLS),
    ]
    .iter()
    .filter(|(on, _)| *on)
    .map(|(_, set)| filter(set))
    .collect();
    if classes.is_empty() {
        return Err(Error::invalid("Choose at least one character type."));
    }
    if opts.length < classes.len() as u32 {
        return Err(Error::invalid("Too short to include every selected character type."));
    }
    let pool: Vec<char> = classes.iter().flatten().copied().collect();

    // Draw every character uniformly from the pool, then reject results that
    // miss a selected class. Rejection (instead of forcing characters into
    // positions) keeps every valid password exactly equally likely.
    loop {
        let candidate: Zeroizing<String> = Zeroizing::new(
            (0..opts.length).map(|_| pool[random_below(pool.len() as u32) as usize]).collect(),
        );
        if classes.iter().all(|cls| candidate.chars().any(|c| cls.contains(&c))) {
            return Ok(Generated {
                value: candidate.to_string(),
                entropy: opts.length as f64 * (pool.len() as f64).log2(),
            });
        }
    }
}

pub fn passphrase(opts: &PassphraseOptions) -> Result<Generated> {
    if !(3..=20).contains(&opts.words) {
        return Err(Error::invalid("Use between 3 and 20 words."));
    }
    if opts.separator.chars().count() > 3 {
        return Err(Error::invalid("The separator can be at most 3 characters."));
    }
    let list = eff_wordlist::large::LIST;
    let mut words: Vec<String> = (0..opts.words)
        .map(|_| {
            let w = list[random_below(list.len() as u32) as usize].1;
            if opts.capitalize {
                let mut c = w.chars();
                c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
            } else {
                w.to_string()
            }
        })
        .collect();
    let mut entropy = opts.words as f64 * (list.len() as f64).log2();
    if opts.include_number {
        let idx = random_below(opts.words) as usize;
        words[idx].push_str(&random_below(10).to_string());
        entropy += (10f64).log2() + (opts.words as f64).log2();
    }
    let value = words.join(&opts.separator);
    words.iter_mut().for_each(zeroize::Zeroize::zeroize);
    Ok(Generated { value, entropy })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Strength {
    /// 0 (very weak) to 4 (very strong)
    pub score: u8,
    pub guesses_log10: f64,
    pub crack_time: String,
    pub warning: Option<String>,
    pub suggestions: Vec<String>,
}

pub fn strength(password: &str, user_inputs: &[&str]) -> Strength {
    if password.is_empty() {
        return Strength {
            score: 0,
            guesses_log10: 0.0,
            crack_time: "instantly".into(),
            warning: None,
            suggestions: vec![],
        };
    }
    let e = zxcvbn::zxcvbn(password, user_inputs);
    let feedback = e.feedback();
    Strength {
        score: u8::from(e.score()),
        guesses_log10: e.guesses_log10(),
        crack_time: e.crack_times().offline_slow_hashing_1e4_per_second().to_string(),
        warning: feedback.and_then(|f| f.warning()).map(|w| w.to_string()),
        suggestions: feedback
            .map(|f| f.suggestions().iter().map(|s| s.to_string()).collect())
            .unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(length: u32) -> PasswordOptions {
        PasswordOptions { length, lower: true, upper: true, digits: true, symbols: true, avoid_ambiguous: false }
    }

    #[test]
    fn passwords_contain_every_selected_class() {
        for _ in 0..500 {
            let p = password(&opts(8)).unwrap().value;
            assert_eq!(p.chars().count(), 8);
            assert!(p.chars().any(|c| c.is_ascii_lowercase()));
            assert!(p.chars().any(|c| c.is_ascii_uppercase()));
            assert!(p.chars().any(|c| c.is_ascii_digit()));
            assert!(p.chars().any(|c| SYMBOLS.contains(c)));
        }
    }

    #[test]
    fn ambiguous_characters_can_be_excluded() {
        let o = PasswordOptions { avoid_ambiguous: true, ..opts(200) };
        let p = password(&o).unwrap().value;
        assert!(!p.chars().any(|c| AMBIGUOUS.contains(c)));
    }

    #[test]
    fn digits_are_roughly_uniform() {
        let o = PasswordOptions { length: 100, lower: false, upper: false, digits: true, symbols: false, avoid_ambiguous: false };
        let mut counts = [0u32; 10];
        for _ in 0..200 {
            for c in password(&o).unwrap().value.chars() {
                counts[c.to_digit(10).unwrap() as usize] += 1;
            }
        }
        // 20,000 samples: expect 2,000 each. Allow a generous ±15%.
        for (d, n) in counts.iter().enumerate() {
            assert!((1700..=2300).contains(n), "digit {d} appeared {n} times");
        }
    }

    #[test]
    fn passphrase_shape_and_entropy() {
        let g = passphrase(&PassphraseOptions { words: 6, separator: "-".into(), capitalize: true, include_number: false }).unwrap();
        assert_eq!(g.value.split('-').count(), 6);
        assert!(g.value.split('-').all(|w| w.chars().next().unwrap().is_uppercase()));
        assert!((g.entropy - 6.0 * 7776f64.log2()).abs() < 0.01);
        assert_eq!(eff_wordlist::large::LIST.len(), 7776);
    }

    #[test]
    fn invalid_options_are_rejected() {
        assert!(password(&PasswordOptions { lower: false, upper: false, digits: false, symbols: false, ..opts(10) }).is_err());
        assert!(password(&opts(2)).is_err());
    }

    #[test]
    fn strength_scores() {
        assert!(strength("password", &[]).score <= 1);
        assert!(strength("t7#Vq9!rLp2@zW4m", &[]).score >= 3);
    }
}
