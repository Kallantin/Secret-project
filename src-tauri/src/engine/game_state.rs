use rand::prelude::SliceRandom;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::card::{Card, SkillIcon};
use crate::engine::chaos::{ChaosBag, ChaosToken};
use crate::engine::investigator::Investigator;
use crate::engine::phase::Phase;
use crate::error::{GameError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub id: Uuid,
    pub campaign: String,
    pub scenario: String,
    pub round: u32,
    pub phase: Phase,
    pub active_investigator: usize,
    pub investigators: Vec<Investigator>,
    pub chaos_bag: ChaosBag,
    pub encounter_deck: Vec<Card>,
    pub encounter_discard: Vec<Card>,
    pub enemies_in_play: Vec<Card>,
    pub log: Vec<String>,
}

impl GameState {
    pub fn new(
        campaign: impl Into<String>,
        scenario: impl Into<String>,
        seed: u64,
        encounter_cards: Vec<Card>,
    ) -> Self {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let mut encounter_deck: Vec<Card> = encounter_cards;
        encounter_deck.shuffle(&mut rng);

        let mut state = Self {
            id: Uuid::new_v4(),
            campaign: campaign.into(),
            scenario: scenario.into(),
            round: 1,
            phase: Phase::Mythos,
            active_investigator: 0,
            investigators: Vec::new(),
            chaos_bag: ChaosBag::standard(seed),
            encounter_deck,
            encounter_discard: Vec::new(),
            enemies_in_play: Vec::new(),
            log: Vec::new(),
        };
        let message = format!(
            "Started campaign: {} / {} (seed {:#x})",
            state.campaign, state.scenario, seed
        );
        state.log(message);
        state
    }

    pub fn add_investigator(&mut self, investigator: Investigator) {
        let message = format!("Investigator joined: {}", investigator.name);
        self.investigators.push(investigator);
        self.log(message);
    }

    pub fn active(&self) -> Option<&Investigator> {
        self.investigators.get(self.active_investigator)
    }

    pub fn active_mut(&mut self) -> Option<&mut Investigator> {
        self.investigators.get_mut(self.active_investigator)
    }

    pub fn investigator(&self, id: Uuid) -> Option<&Investigator> {
        self.investigators.iter().find(|investigator| investigator.id == id)
    }

    pub fn investigator_mut(&mut self, id: Uuid) -> Option<&mut Investigator> {
        self.investigators
            .iter_mut()
            .find(|investigator| investigator.id == id)
    }

    pub fn advance_phase(&mut self) {
        let next = self.phase.next();
        self.phase = next;

        if matches!(next, Phase::Mythos) {
            self.round += 1;
            if !self.investigators.is_empty() {
                self.active_investigator =
                    (self.active_investigator + 1) % self.investigators.len();
            }
        }

        let message = format!("Phase: {:?} (round {})", self.phase, self.round);
        self.log(message);
    }

    pub fn draw_chaos_token(&mut self) -> ChaosToken {
        let token = self.chaos_bag.draw();
        let message = format!(
            "Chaos token drawn: {:?} (modifier {})",
            token,
            token.modifier()
        );
        self.log(message);
        token
    }

    /// Resolve a skill test for an investigator: draw a chaos token and combine
    /// the investigator's stat value with the token modifier.
    pub fn skill_test(
        &mut self,
        investigator_id: Uuid,
        skill: SkillIcon,
    ) -> Result<SkillTestResult> {
        let (investigator_name, stat_value) = {
            let investigator = self
                .investigator(investigator_id)
                .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
            (investigator.name.clone(), investigator.skill_value(skill))
        };
        let token = self.draw_chaos_token();
        let modifier = token.modifier();
        let total = stat_value + modifier;
        let message = format!(
            "Skill test: {investigator_name} tests {} (stat {stat_value}) + {modifier} = {total}",
            skill_as_str(skill)
        );
        self.log(message);
        Ok(SkillTestResult {
            investigator_id,
            investigator_name,
            skill,
            stat_value,
            token,
            token_modifier: modifier,
            total,
        })
    }

    /// Spend one action for the active investigator.
    pub fn spend_action(&mut self, investigator_id: Uuid) -> Result<()> {
        let investigator = self
            .investigator_mut(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        investigator.spend_action()
    }

    /// Reset all investigators' actions for a new turn (upkeep).
    pub fn reset_turn(&mut self) {
        for investigator in &mut self.investigators {
            investigator.reset_actions();
        }
        self.log("Turn reset: actions restored for all investigators.");
    }

    /// Draw the top card of the encounter deck (mythos phase). If the deck is
    /// empty, reshuffle the discard pile.
    pub fn draw_encounter_card(&mut self) -> Result<Card> {
        if self.encounter_deck.is_empty() && !self.encounter_discard.is_empty() {
            let mut rng = ChaCha20Rng::seed_from_u64(self.chaos_bag.seed.wrapping_add(self.chaos_bag.draws));
            std::mem::swap(&mut self.encounter_deck, &mut self.encounter_discard);
            self.encounter_deck.shuffle(&mut rng);
            self.log("Encounter deck reshuffled from discard pile.");
        }
        let card = self
            .encounter_deck
            .pop()
            .ok_or_else(|| GameError::RuleViolation("encounter deck is empty".into()))?;
        let message = format!("Mythos: drew encounter card {}", card.name);
        self.log(message);
        Ok(card)
    }

    /// Resolve the mythos phase: draw an encounter card. Enemies spawn into play,
    /// treacheries are resolved immediately and discarded.
    pub fn mythos_phase(&mut self) -> Result<MythosResult> {
        let card = self.draw_encounter_card()?;

        match card.card_type {
            crate::engine::card::CardType::Enemy => {
                self.enemies_in_play.push(card.clone());
                let message = format!("Enemy enters play: {}", card.name);
                self.log(message);
                Ok(MythosResult {
                    card: card.clone(),
                    spawned_enemy: Some(card),
                    discarded: false,
                })
            }
            _ => {
                self.encounter_discard.push(card.clone());
                Ok(MythosResult {
                    card,
                    spawned_enemy: None,
                    discarded: true,
                })
            }
        }
    }

    /// Play a card from an investigator's hand into their play area, deducting
    /// its cost from resources.
    pub fn play_card(
        &mut self,
        investigator_id: Uuid,
        card_id: Uuid,
    ) -> Result<PlayCardResult> {
        let (card, investigator_name, remaining_resources) = {
            let investigator = self
                .investigator_mut(investigator_id)
                .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;

            let card_index = investigator
                .deck
                .hand
                .iter()
                .position(|card| card.id == card_id)
                .ok_or_else(|| GameError::CardNotFound(card_id.to_string()))?;

            let card = investigator.deck.hand.remove(card_index);

            if let Some(cost) = card.cost {
                if cost > investigator.resources {
                    investigator.deck.hand.push(card.clone());
                    return Err(GameError::InsufficientResources {
                        needed: cost,
                        available: investigator.resources,
                    });
                }
                investigator.resources -= cost;
            }

            let remaining = investigator.resources;
            investigator.deck.play_area.push(card.clone());
            (card, investigator.name.clone(), remaining)
        };

        let message = format!("{investigator_name} plays {} from hand.", card.name);
        self.log(message);

        Ok(PlayCardResult {
            investigator_id,
            card,
            remaining_resources,
        })
    }

    pub fn log(&mut self, message: impl Into<String>) {
        self.log.push(message.into());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTestResult {
    pub investigator_id: Uuid,
    pub investigator_name: String,
    pub skill: SkillIcon,
    pub stat_value: i8,
    pub token: ChaosToken,
    pub token_modifier: i8,
    pub total: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MythosResult {
    pub card: Card,
    pub spawned_enemy: Option<Card>,
    pub discarded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayCardResult {
    pub investigator_id: Uuid,
    pub card: Card,
    pub remaining_resources: i32,
}

fn skill_as_str(skill: SkillIcon) -> &'static str {
    match skill {
        SkillIcon::Willpower => "Willpower",
        SkillIcon::Intellect => "Intellect",
        SkillIcon::Combat => "Combat",
        SkillIcon::Agility => "Agility",
        SkillIcon::Wild => "Wild",
    }
}
