use serde::{Deserialize, Serialize};

use super::card::Card;
use super::SkillIcon;
use crate::error::{GameError, Result};
use uuid::Uuid;

/// The default number of actions an investigator receives each turn.
pub const DEFAULT_ACTIONS: u8 = 3;
/// The default starting resource pool for an investigator.
pub const DEFAULT_RESOURCES: i32 = 5;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Deck {
    pub draw_pile: Vec<Card>,
    pub hand: Vec<Card>,
    pub discard_pile: Vec<Card>,
    pub play_area: Vec<Card>,
    pub removed_from_game: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_draw_pile(cards: Vec<Card>) -> Self {
        Self {
            draw_pile: cards,
            ..Self::default()
        }
    }

    /// Draw the top card of the draw pile into hand.
    pub fn draw(&mut self) -> Result<Card> {
        let card = self
            .draw_pile
            .pop()
            .ok_or_else(|| GameError::RuleViolation("draw pile is empty".into()))?;
        self.hand.push(card.clone());
        Ok(card)
    }

    pub fn discard_from_hand(&mut self, card_id: Uuid) -> Result<Card> {
        let index = self
            .hand
            .iter()
            .position(|card| card.id == card_id)
            .ok_or_else(|| GameError::CardNotFound(card_id.to_string()))?;
        let card = self.hand.remove(index);
        self.discard_pile.push(card.clone());
        Ok(card)
    }

    pub fn draw_count(&self) -> usize {
        self.draw_pile.len()
    }

    pub fn hand_count(&self) -> usize {
        self.hand.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Investigator {
    pub id: Uuid,
    /// ArkhamDB investigator code, e.g. `01001`.
    pub code: String,
    pub name: String,
    pub willpower: i8,
    pub intellect: i8,
    pub combat: i8,
    pub agility: i8,
    pub health: i32,
    pub max_health: i32,
    pub sanity: i32,
    pub max_sanity: i32,
    pub resources: i32,
    pub actions_remaining: u8,
    pub clues: i32,
    pub is_defeated: bool,
    /// Experience points earned across campaign play.
    pub xp: i32,
    /// Enemy instance IDs currently engaged with this investigator.
    pub engaged_enemies: Vec<Uuid>,
    /// Location instance ID the investigator is currently at, if any.
    pub location_id: Option<Uuid>,
    pub deck: Deck,
}

impl Investigator {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        code: impl Into<String>,
        name: impl Into<String>,
        willpower: i8,
        intellect: i8,
        combat: i8,
        agility: i8,
        max_health: i32,
        max_sanity: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            code: code.into(),
            name: name.into(),
            willpower,
            intellect,
            combat,
            agility,
            health: max_health,
            max_health,
            sanity: max_sanity,
            max_sanity,
            resources: DEFAULT_RESOURCES,
            actions_remaining: DEFAULT_ACTIONS,
            clues: 0,
            is_defeated: false,
            xp: 0,
            engaged_enemies: Vec::new(),
            location_id: None,
            deck: Deck::new(),
        }
    }

    /// Value for a single skill test, before chaos token modifiers.
    pub fn skill_value(&self, skill: SkillIcon) -> i8 {
        match skill {
            SkillIcon::Willpower => self.willpower,
            SkillIcon::Intellect => self.intellect,
            SkillIcon::Combat => self.combat,
            SkillIcon::Agility => self.agility,
            SkillIcon::Wild => 0,
        }
    }

    pub fn reset_actions(&mut self) {
        self.actions_remaining = DEFAULT_ACTIONS;
    }

    pub fn spend_action(&mut self) -> Result<()> {
        if self.actions_remaining == 0 {
            return Err(GameError::RuleViolation(
                "no actions remaining this turn".into(),
            ));
        }
        self.actions_remaining -= 1;
        Ok(())
    }

    pub fn gain_resource(&mut self, amount: i32) {
        self.resources = (self.resources + amount).max(0);
    }

    pub fn spend_resources(&mut self, amount: i32) -> Result<()> {
        if amount > self.resources {
            return Err(GameError::InsufficientResources {
                needed: amount,
                available: self.resources,
            });
        }
        self.resources -= amount;
        Ok(())
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.health = (self.health - amount).max(0);
        if self.health == 0 {
            self.is_defeated = true;
        }
    }

    pub fn take_horror(&mut self, amount: i32) {
        self.sanity = (self.sanity - amount).max(0);
        if self.sanity == 0 {
            self.is_defeated = true;
        }
    }

    pub fn heal_damage(&mut self, amount: i32) {
        self.health = (self.health + amount).min(self.max_health);
        if self.health > 0 {
            self.is_defeated = false;
        }
    }

    pub fn heal_horror(&mut self, amount: i32) {
        self.sanity = (self.sanity + amount).min(self.max_sanity);
        if self.sanity > 0 {
            self.is_defeated = false;
        }
    }
}
