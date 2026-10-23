//! Master-password strength estimate.
//! Generating passwords for logins is KeePassXC's job (amendment A9).

use serde::Serialize;

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
    fn strength_scores() {
        assert!(strength("password", &[]).score <= 1);
        assert!(strength("t7#Vq9!rLp2@zW4m", &[]).score >= 3);
    }
}
