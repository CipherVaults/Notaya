//! Master-password helpers: passphrase suggestions and strength estimates.
//! Generating passwords for logins is KeePassXC's job (amendment A9).

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::crypto::random_below;
use crate::error::{Error, Result};

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

/// Picks words uniformly from the EFF large wordlist using the OS secure RNG.
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
    words.iter_mut().for_each(Zeroize::zeroize);
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

    #[test]
    fn passphrase_shape_and_entropy() {
        let g = passphrase(&PassphraseOptions { words: 6, separator: "-".into(), capitalize: true, include_number: false }).unwrap();
        assert_eq!(g.value.split('-').count(), 6);
        assert!(g.value.split('-').all(|w| w.chars().next().unwrap().is_uppercase()));
        assert!((g.entropy - 6.0 * 7776f64.log2()).abs() < 0.01);
        assert_eq!(eff_wordlist::large::LIST.len(), 7776);
    }

    #[test]
    fn passphrase_number_and_bounds() {
        let g = passphrase(&PassphraseOptions { words: 4, separator: " ".into(), capitalize: false, include_number: true }).unwrap();
        assert_eq!(g.value.chars().filter(|c| c.is_ascii_digit()).count(), 1);
        assert!(passphrase(&PassphraseOptions { words: 2, separator: "-".into(), capitalize: false, include_number: false }).is_err());
        assert!(passphrase(&PassphraseOptions { words: 5, separator: "----".into(), capitalize: false, include_number: false }).is_err());
    }

    #[test]
    fn strength_scores() {
        assert!(strength("password", &[]).score <= 1);
        assert!(strength("t7#Vq9!rLp2@zW4m", &[]).score >= 3);
    }
}
