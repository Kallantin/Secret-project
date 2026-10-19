use serde::{Deserialize, Serialize};

/// The four phases of a round, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Mythos,
    Investigation,
    Enemy,
    Upkeep,
}

impl Phase {
    /// Returns the next phase, wrapping from `Upkeep` back to `Mythos`.
    pub fn next(self) -> Phase {
        match self {
            Phase::Mythos => Phase::Investigation,
            Phase::Investigation => Phase::Enemy,
            Phase::Enemy => Phase::Upkeep,
            Phase::Upkeep => Phase::Mythos,
        }
    }

    /// True when this phase is the last of the current round.
    pub fn is_round_end(self) -> bool {
        matches!(self, Phase::Upkeep)
    }
}

impl Default for Phase {
    fn default() -> Self {
        Phase::Mythos
    }
}
