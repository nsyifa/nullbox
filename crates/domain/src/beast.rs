use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Mood of the Backlog Beast, derived purely from the number of active tickets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum BeastMood {
    Asleep,
    Peckish,
    Grumpy,
    Kaiju,
}

impl BeastMood {
    pub fn from_open_count(open: u64) -> Self {
        match open {
            0 => Self::Asleep,
            1..=5 => Self::Peckish,
            6..=20 => Self::Grumpy,
            _ => Self::Kaiju,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mood_thresholds() {
        let cases = [(0, BeastMood::Asleep), (1, BeastMood::Peckish), (5, BeastMood::Peckish)];
        let more = [(6, BeastMood::Grumpy), (20, BeastMood::Grumpy), (21, BeastMood::Kaiju)];
        for (open, mood) in cases.into_iter().chain(more) {
            assert_eq!(BeastMood::from_open_count(open), mood, "{open} open tickets");
        }
    }
}
