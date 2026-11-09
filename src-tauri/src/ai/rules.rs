use crate::actions::{ActionSpec, Confirm, Level as Required};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Zeroize,
)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    #[default]
    Off,
    Look,
    Edit,
    Full,
}

pub fn allowed(level: Level, action: &ActionSpec) -> bool {
    match action.level {
        Required::Look => level >= Level::Look,
        Required::Edit => level >= Level::Edit,
        Required::Full => level >= Level::Full,
        Required::Never => false,
    }
}

pub fn confirmation(action: &ActionSpec, items: usize, enabled: bool, threshold: usize) -> Confirm {
    match action.confirm {
        Confirm::AskIfBig if enabled && items > threshold => Confirm::Ask,
        Confirm::AskIfBig => Confirm::None,
        other => other,
    }
}

pub fn strongest(a: Confirm, b: Confirm) -> Confirm {
    let rank = |c| match c {
        Confirm::Strong => 2,
        Confirm::Ask => 1,
        _ => 0,
    };
    if rank(a) >= rank(b) {
        a
    } else {
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::registry::ACTIONS;
    #[test]
    fn permission_matrix_and_big_change_switch() {
        for a in ACTIONS {
            for level in [Level::Off, Level::Look, Level::Edit, Level::Full] {
                let expected = match a.level {
                    Required::Look => level >= Level::Look,
                    Required::Edit => level >= Level::Edit,
                    Required::Full => level == Level::Full,
                    Required::Never => false,
                };
                assert_eq!(allowed(level, a), expected, "{} {level:?}", a.name);
            }
        }
        let write = ACTIONS.iter().find(|a| a.name == "page.write").unwrap();
        let trash = ACTIONS.iter().find(|a| a.name == "page.trash").unwrap();
        for enabled in [true, false] {
            assert_eq!(confirmation(write, 20, enabled, 20), Confirm::None);
            assert_eq!(
                confirmation(write, 21, enabled, 20),
                if enabled { Confirm::Ask } else { Confirm::None }
            );
            assert_eq!(confirmation(trash, 1, enabled, 20), Confirm::Ask);
        }
    }
}
