use serde::Serialize;

/// Errors produced by the game engine and the Tauri command layer.
///
/// Variants are serialized as plain strings so the frontend receives a stable,
/// human-readable message regardless of internal representation.
#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("card not found: {0}")]
    CardNotFound(String),

    #[error("investigator not found: {0}")]
    InvestigatorNotFound(String),

    #[error("illegal phase transition from {from:?} to {to:?}")]
    IllegalPhaseTransition {
        from: crate::engine::phase::Phase,
        to: crate::engine::phase::Phase,
    },

    #[error("insufficient resources: needed {needed}, available {available}")]
    InsufficientResources { needed: i32, available: i32 },

    #[error("rule violation: {0}")]
    RuleViolation(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("game is not initialized")]
    GameNotInitialized,
}

impl Serialize for GameError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<serde_json::Error> for GameError {
    fn from(value: serde_json::Error) -> Self {
        GameError::Serialization(value.to_string())
    }
}

impl From<std::io::Error> for GameError {
    fn from(value: std::io::Error) -> Self {
        GameError::Serialization(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GameError>;
