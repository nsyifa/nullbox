use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Animal keys for anonymous customers. The frontend translates each key,
/// so adding one here needs a matching entry in `copy/en.ts` and `copy/ja.ts`.
pub const ANON_ANIMALS: &[&str] = &[
    "platypus",
    "capybara",
    "axolotl",
    "narwhal",
    "quokka",
    "pangolin",
    "tardigrade",
    "red_panda",
    "otter",
    "tanuki",
    "alpaca",
    "sloth",
];

/// "Anonymous Platypus #4471". Stored as structured data, rendered by the frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnonIdentity {
    pub animal: String,
    pub number: u16,
}

impl AnonIdentity {
    /// Deterministic from caller-supplied randomness, so this stays IO-free.
    pub fn from_random(random: u32) -> Self {
        let animal = ANON_ANIMALS[random as usize % ANON_ANIMALS.len()];
        let number = 1000 + ((random / ANON_ANIMALS.len() as u32) % 9000) as u16;
        Self { animal: animal.to_owned(), number }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_always_four_digits() {
        for random in [0, 1, 4471, u32::MAX] {
            let id = AnonIdentity::from_random(random);
            assert!((1000..=9999).contains(&id.number));
            assert!(ANON_ANIMALS.contains(&id.animal.as_str()));
        }
    }
}
