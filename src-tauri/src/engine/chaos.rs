use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum ChaosToken {
    Numeric(i8),
    Skull,
    Cultist,
    Tablet,
    ElderThing,
    AutoFail,
    ElderSign,
    Bless,
    Curse,
}

impl ChaosToken {
    /// The numeric modifier this token applies to a skill test.
    ///
    /// Scenario-specific symbols (skull/cultist/tablet/elder thing) return 0
    /// here; their real modifier depends on the active scenario reference card.
    pub fn modifier(self) -> i8 {
        match self {
            ChaosToken::Numeric(value) => value,
            ChaosToken::Skull
            | ChaosToken::Cultist
            | ChaosToken::Tablet
            | ChaosToken::ElderThing => 0,
            ChaosToken::AutoFail => -100,
            ChaosToken::ElderSign => 0,
            ChaosToken::Bless => 2,
            ChaosToken::Curse => -2,
        }
    }

    pub fn is_symbol(self) -> bool {
        matches!(
            self,
            ChaosToken::Skull
                | ChaosToken::Cultist
                | ChaosToken::Tablet
                | ChaosToken::ElderThing
                | ChaosToken::AutoFail
                | ChaosToken::ElderSign
        )
    }
}

/// A deterministic chaos bag.
///
/// Draw order is a pure function of the `seed` and the number of prior draws,
/// so replaying a seed reproduces the exact sequence of tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosBag {
    pub tokens: Vec<ChaosToken>,
    pub seed: u64,
    pub draws: u64,
}

impl ChaosBag {
    pub fn new(tokens: Vec<ChaosToken>, seed: u64) -> Self {
        Self {
            tokens,
            seed,
            draws: 0,
        }
    }

    /// The standard 16-token bag used by the Night of the Zealot campaign.
    pub fn standard(seed: u64) -> Self {
        Self::new(Self::standard_tokens(), seed)
    }

    pub fn standard_tokens() -> Vec<ChaosToken> {
        vec![
            ChaosToken::Numeric(1),
            ChaosToken::Numeric(0),
            ChaosToken::Numeric(0),
            ChaosToken::Numeric(-1),
            ChaosToken::Numeric(-1),
            ChaosToken::Numeric(-1),
            ChaosToken::Numeric(-2),
            ChaosToken::Numeric(-2),
            ChaosToken::Numeric(-3),
            ChaosToken::Numeric(-4),
            ChaosToken::Skull,
            ChaosToken::Cultist,
            ChaosToken::Tablet,
            ChaosToken::ElderThing,
            ChaosToken::AutoFail,
            ChaosToken::ElderSign,
        ]
    }

    /// Draw a token. The bag is treated as "with replacement": each draw is
    /// independent and derived from `seed + draws`.
    pub fn draw(&mut self) -> ChaosToken {
        let token = self.peek_draw(self.draws);
        self.draws += 1;
        token
    }

    fn peek_draw(&self, draw_index: u64) -> ChaosToken {
        debug_assert!(!self.tokens.is_empty(), "chaos bag must not be empty");
        let mut rng = ChaCha20Rng::seed_from_u64(self.seed.wrapping_add(draw_index));
        let index = rng.gen_range(0..self.tokens.len());
        self.tokens[index]
    }

    pub fn reset(&mut self) {
        self.draws = 0;
    }
}
