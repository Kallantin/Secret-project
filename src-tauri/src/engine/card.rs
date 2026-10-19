use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stable identifier for a card instance within a game.
pub type CardId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardType {
    Asset,
    Event,
    Skill,
    Treachery,
    Enemy,
    Location,
    Act,
    Agenda,
    Investigator,
    Story,
    Weakness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardClass {
    Guardian,
    Seeker,
    Rogue,
    Mystic,
    Survivor,
    Neutral,
    Mythos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillIcon {
    Willpower,
    Intellect,
    Combat,
    Agility,
    Wild,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillIcons {
    pub willpower: u8,
    pub intellect: u8,
    pub combat: u8,
    pub agility: u8,
    pub wild: u8,
}

impl SkillIcons {
    /// Total number of skill icons, counting `wild` towards every test type.
    pub fn total(&self) -> u8 {
        self.willpower + self.intellect + self.combat + self.agility + self.wild
    }

    /// Value contributed to a specific skill test, including wild icons.
    pub fn value(&self, skill: SkillIcon) -> u8 {
        let base = match skill {
            SkillIcon::Willpower => self.willpower,
            SkillIcon::Intellect => self.intellect,
            SkillIcon::Combat => self.combat,
            SkillIcon::Agility => self.agility,
            SkillIcon::Wild => return self.wild,
        };
        base + self.wild
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: CardId,
    /// ArkhamDB card code, e.g. `01001`.
    pub code: String,
    pub name: String,
    pub card_type: CardType,
    #[serde(default)]
    pub class: Option<CardClass>,
    #[serde(default)]
    pub cost: Option<i32>,
    #[serde(default)]
    pub xp: u8,
    #[serde(default)]
    pub icons: SkillIcons,
    #[serde(default)]
    pub traits: Vec<String>,
    #[serde(default)]
    pub text: String,
    /// Raw ArkhamDB-compatible JSON payload for fields not yet modelled.
    #[serde(default)]
    pub raw: serde_json::Value,
}

impl Card {
    pub fn new(code: impl Into<String>, name: impl Into<String>, card_type: CardType) -> Self {
        Self {
            id: Uuid::new_v4(),
            code: code.into(),
            name: name.into(),
            card_type,
            class: None,
            cost: None,
            xp: 0,
            icons: SkillIcons::default(),
            traits: Vec::new(),
            text: String::new(),
            raw: serde_json::Value::Null,
        }
    }
}
