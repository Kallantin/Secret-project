use rand::prelude::SliceRandom;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::card::{Card, SkillIcon};
use crate::engine::chaos::{ChaosBag, ChaosToken};
use crate::engine::investigator::{Investigator, DEFAULT_RESOURCES};
use crate::engine::phase::Phase;
use crate::error::{GameError, Result};
use serde_json::Value;

/// Scenario act card data, deserialized from scenario JSON.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScenarioActData {
    pub code: String,
    pub name: String,
    pub text: String,
    pub clue_cost: u32,
    pub raw: Value,
}

/// Scenario agenda card data, deserialized from scenario JSON.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScenarioAgendaData {
    pub code: String,
    pub name: String,
    pub text: String,
    pub doom_threshold: u32,
    pub raw: Value,
}

/// Scenario location card data, deserialized from scenario JSON.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScenarioLocationData {
    pub code: String,
    pub name: String,
    pub text: String,
    pub clues: u32,
    pub shroud: u32,
    pub raw: Value,
}

/// Complete scenario configuration, deserialized from scenario JSON files.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScenarioData {
    pub campaign: String,
    pub code: String,
    pub name: String,
    pub order: u32,
    pub chaos_bag: String,
    pub acts: Vec<ScenarioActData>,
    pub agendas: Vec<ScenarioAgendaData>,
    pub locations: Vec<ScenarioLocationData>,
}

/// The result of a completed scenario in a campaign.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CampaignResult {
    Won,
    Lost,
}

/// A single entry in the campaign log recording scenario completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignLogEntry {
    pub scenario_code: String,
    pub scenario_name: String,
    pub result: CampaignResult,
    pub investigator_names: Vec<String>,
    pub notes: String,
}

/// A boon or blemish carried between scenarios in a campaign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoonOrBlemish {
    pub name: String,
    pub description: String,
    pub is_boon: bool,
}

/// Campaign-wide progress that carries between scenarios.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CampaignProgress {
    /// Scenario codes completed in this campaign, in order.
    pub completed_scenarios: Vec<String>,
    /// Log of scenario results and notable events.
    pub campaign_log: Vec<CampaignLogEntry>,
    /// Boons and blemishes carried between scenarios.
    pub boons_and_blemishes: Vec<BoonOrBlemish>,
}

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
    pub enemies_in_play: Vec<EnemyInstance>,
    pub locations_in_play: Vec<LocationInstance>,
    pub current_act_index: usize,
    pub current_agenda_index: usize,
    pub acts: Vec<ActInstance>,
    pub agendas: Vec<AgendaInstance>,
    pub game_status: GameStatus,
    /// Cards in the victory display, worth experience at scenario end.
    pub victory_display: Vec<Card>,
    /// Campaign-wide progress carried between scenarios.
    pub campaign_progress: CampaignProgress,
    /// In-game log messages for the current scenario.
    pub log_messages: Vec<String>,
}

impl GameState {
    pub fn new(
        campaign: impl Into<String>,
        scenario: impl Into<String>,
        seed: u64,
        encounter_cards: Vec<Card>,
        scenario_data: ScenarioData,
    ) -> Self {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let mut encounter_deck: Vec<Card> = encounter_cards;
        encounter_deck.shuffle(&mut rng);

        let acts: Vec<ActInstance> = scenario_data
            .acts
            .iter()
            .map(|act_data| ActInstance {
                id: Uuid::new_v4(),
                card: Card {
                    id: Uuid::new_v4(),
                    code: act_data.code.clone(),
                    name: act_data.name.clone(),
                    card_type: crate::engine::card::CardType::Act,
                    class: Some(crate::engine::card::CardClass::Neutral),
                    cost: None,
                    xp: 0,
                    icons: Default::default(),
                    traits: Vec::new(),
                    text: act_data.text.clone(),
                    raw: act_data.raw.clone(),
                },
                progress: 0,
            })
            .collect();

        let agendas: Vec<AgendaInstance> = scenario_data
            .agendas
            .iter()
            .map(|agenda_data| AgendaInstance {
                id: Uuid::new_v4(),
                card: Card {
                    id: Uuid::new_v4(),
                    code: agenda_data.code.clone(),
                    name: agenda_data.name.clone(),
                    card_type: crate::engine::card::CardType::Agenda,
                    class: Some(crate::engine::card::CardClass::Neutral),
                    cost: None,
                    xp: 0,
                    icons: Default::default(),
                    traits: Vec::new(),
                    text: agenda_data.text.clone(),
                    raw: agenda_data.raw.clone(),
                },
                doom: 0,
                doom_threshold: agenda_data.doom_threshold,
            })
            .collect();

        let locations: Vec<LocationInstance> = scenario_data
            .locations
            .iter()
            .map(|location_data| LocationInstance {
                id: Uuid::new_v4(),
                card: Card {
                    id: Uuid::new_v4(),
                    code: location_data.code.clone(),
                    name: location_data.name.clone(),
                    card_type: crate::engine::card::CardType::Location,
                    class: Some(crate::engine::card::CardClass::Neutral),
                    cost: None,
                    xp: 0,
                    icons: Default::default(),
                    traits: Vec::new(),
                    text: location_data.text.clone(),
                    raw: location_data.raw.clone(),
                },
                clues_remaining: location_data.clues,
            })
            .collect();

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
            locations_in_play: locations,
            current_act_index: 0,
            current_agenda_index: 0,
            acts,
            agendas,
            game_status: GameStatus::Ongoing,
            victory_display: Vec::new(),
            campaign_progress: CampaignProgress::default(),
            log_messages: Vec::new(),
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

        // Rulebook: during Upkeep, each investigator resets their action count
        // to their maximum automatically.
        if matches!(next, Phase::Upkeep) {
            self.reset_turn();
        }

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

    /// Rotate to the next investigator during Investigation phase.
    /// Each investigator takes their full turn before rotating to the next.
    /// Returns the ID of the new active investigator.
    pub fn next_investigator(&mut self) -> Option<Uuid> {
        if self.investigators.is_empty() {
            return None;
        }
        self.active_investigator = (self.active_investigator + 1) % self.investigators.len();
        let new_active = self.investigators[self.active_investigator].id;
        self.log(format!(
            "Active investigator: {}",
            self.investigators[self.active_investigator].name
        ));
        Some(new_active)
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
                let health = enemy_health(&card);
                let horror = enemy_horror_value(&card);
                let instance = EnemyInstance {
                    id: Uuid::new_v4(),
                    card: card.clone(),
                    health,
                    horror,
                    engaged_with: None,
                    location_id: None,
                    exhausted: false,
                };
                self.enemies_in_play.push(instance.clone());
                let message = format!("Enemy enters play: {}", card.name);
                self.log(message);
                Ok(MythosResult {
                    card,
                    spawned_enemy: Some(instance.card.clone()),
                    discarded: false,
                })
            }
            crate::engine::card::CardType::Location => {
                let clues = location_clues(&card);
                let instance = LocationInstance {
                    id: Uuid::new_v4(),
                    card: card.clone(),
                    clues_remaining: clues,
                };
                self.locations_in_play.push(instance.clone());
                let message = format!("Location enters play: {} ({} clues)", card.name, clues);
                self.log(message);
                Ok(MythosResult {
                    card,
                    spawned_enemy: None,
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

    /// Engage an enemy with the active investigator.
    pub fn engage(
        &mut self,
        investigator_id: Uuid,
        enemy_instance_id: Uuid,
    ) -> Result<EngageResult> {
        let investigator = self
            .investigator(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        if investigator.engaged_enemies.contains(&enemy_instance_id) {
            return Err(GameError::RuleViolation(
                "already engaged with this enemy".into(),
            ));
        }

        let enemy = self
            .enemies_in_play
            .iter()
            .find(|e| e.id == enemy_instance_id)
            .ok_or_else(|| GameError::CardNotFound(enemy_instance_id.to_string()))?;
        if enemy.engaged_with.is_some() {
            return Err(GameError::RuleViolation(
                "enemy is already engaged with another investigator".into(),
            ));
        }

        let investigator_name = investigator.name.clone();
        let enemy_name = enemy.card.name.clone();

        let investigator = self
            .investigator_mut(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        investigator.engaged_enemies.push(enemy_instance_id);

        let enemy = self
            .enemies_in_play
            .iter_mut()
            .find(|e| e.id == enemy_instance_id)
            .unwrap();
        enemy.engaged_with = Some(investigator_id);

        let message = format!("{investigator_name} engages {enemy_name}");
        self.log(message);

        Ok(EngageResult {
            investigator_id,
            enemy_instance_id,
            enemy_name,
        })
    }

    /// Resolve a fight action: the investigator makes a combat test against the
    /// enemy's fight value. On success, deal damage to the enemy. On failure,
    /// the enemy deals its damage to the investigator.
    pub fn fight(
        &mut self,
        investigator_id: Uuid,
        enemy_instance_id: Uuid,
    ) -> Result<FightResult> {
        let investigator = self
            .investigator(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        if !investigator.engaged_enemies.contains(&enemy_instance_id) {
            return Err(GameError::RuleViolation("not engaged with this enemy".into()));
        }
        let investigator_name = investigator.name.clone();

        let enemy_index = self
            .enemies_in_play
            .iter()
            .position(|e| e.id == enemy_instance_id)
            .ok_or_else(|| GameError::CardNotFound(enemy_instance_id.to_string()))?;
        let enemy_fight = enemy_fight_value(&self.enemies_in_play[enemy_index].card);
        let enemy_name = self.enemies_in_play[enemy_index].card.name.clone();
        let enemy_damage = enemy_damage_value(&self.enemies_in_play[enemy_index].card);
        let enemy_horror = enemy_horror_value(&self.enemies_in_play[enemy_index].card);

        // Spend one action to fight.
        self.spend_action(investigator_id)?;

        // Resolve combat skill test.
        let test = self.skill_test(investigator_id, SkillIcon::Combat)?;
        let success = test.total >= enemy_fight;

        let enemy = &mut self.enemies_in_play[enemy_index];
        let (damage_dealt, damage_taken, horror_taken) = if success {
            let damage = 1;
            enemy.health = (enemy.health - damage).max(0);
            if enemy.health == 0 {
                let defeated_enemy_name = enemy.card.name.clone();
                self.enemies_in_play.remove(enemy_index);
                for inv in &mut self.investigators {
                    inv.engaged_enemies.retain(|id| *id != enemy_instance_id);
                }
                let message = format!("{investigator_name} defeated {defeated_enemy_name}");
                self.log(message);
                (damage, 0, 0)
            } else {
                let message = format!(
                    "{investigator_name} hits {enemy_name} for {damage} damage"
                );
                self.log(message);
                (damage, 0, 0)
            }
        } else {
            let investigator = self
                .investigator_mut(investigator_id)
                .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
            investigator.take_damage(enemy_damage);
            investigator.take_horror(enemy_horror);
            let message = format!(
                "{investigator_name} misses {enemy_name}; {enemy_name} hits {investigator_name} for {enemy_damage} damage and {enemy_horror} horror"
            );
            self.log(message);
            (0, enemy_damage, enemy_horror)
        };

        let enemy_health = self
            .enemies_in_play
            .iter()
            .find(|e| e.id == enemy_instance_id)
            .map(|e| e.health)
            .unwrap_or(0);

        Ok(FightResult {
            investigator_id,
            enemy_instance_id,
            success,
            enemy_health,
            damage_dealt: damage_dealt,
            damage_taken,
            horror_taken,
            skill_test: test,
        })
    }

    /// Play a card from an investigator's hand into their play area, deducting
    /// its cost from resources.
    pub fn play_card(
        &mut self,
        investigator_id: Uuid,
        card_id: Uuid,
    ) -> Result<PlayCardResult> {
        // Spend one action to play a card.
        self.spend_action(investigator_id)?;

        // Attacks of opportunity: each ready engaged enemy attacks the
        // investigator for playing a card (RRG "Attack of Opportunity", page 5).
        self.attacks_of_opportunity(investigator_id)?;

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

    /// Discard a card from an investigator's hand to their discard pile.
    /// Used when a card effect says "discard a card" or "discard X cards."
    pub fn discard_card(
        &mut self,
        investigator_id: Uuid,
        card_id: Uuid,
    ) -> Result<Card> {
        let investigator_name = self
            .investigator(investigator_id)
            .map(|inv| inv.name.clone())
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;

        let card = self
            .investigator_mut(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?
            .deck
            .discard_from_hand(card_id)
            .map_err(|_| GameError::CardNotFound(card_id.to_string()))?;

        let card_name = card.name.clone();
        self.log(format!("{investigator_name} discards {card_name} from hand."));
        Ok(card)
    }

    /// Draw the top card of an investigator's draw pile into their hand.
    /// Used when a card effect says "draw a card" or "add a card to hand."
    pub fn draw_card(&mut self, investigator_id: Uuid) -> Result<Card> {
        let investigator_name = self
            .investigator(investigator_id)
            .map(|inv| inv.name.clone())
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;

        let card = self
            .investigator_mut(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?
            .deck
            .draw()
            .map_err(|_| GameError::RuleViolation("draw pile is empty".into()))?;

        let card_name = card.name.clone();
        self.log(format!("{investigator_name} draws {card_name}."));
        Ok(card)
    }

    /// Add a card directly to an investigator's play area without spending
    /// resources or an action. Used when a card effect says "put into play."
    pub fn add_card_to_play_area(&mut self, investigator_id: Uuid, card: Card) -> Result<()> {
        let investigator_name = self
            .investigator(investigator_id)
            .map(|inv| inv.name.clone())
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;

        let card_name = card.name.clone();
        self
            .investigator_mut(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?
            .deck
            .play_area
            .push(card);

        self.log(format!("{investigator_name} puts {card_name} into play."));
        Ok(())
    }

    /// Gain resources for an investigator. Used by card effects that grant resources.
    pub fn gain_resource(&mut self, investigator_id: Uuid, amount: i32) -> Result<i32> {
        let investigator_name = self
            .investigator(investigator_id)
            .map(|inv| inv.name.clone())
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        let new_total = self
            .investigator_mut(investigator_id)
            .map(|inv| {
                inv.gain_resource(amount);
                inv.resources
            })
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        self.log(format!("{investigator_name} gains {} resource(s).", amount));
        Ok(new_total)
    }

    /// Spend resources for an investigator (outside of playing a card).
    /// Used by card effects that require spending resources.
    pub fn spend_resources(&mut self, investigator_id: Uuid, amount: i32) -> Result<()> {
        let investigator_name = self
            .investigator(investigator_id)
            .map(|inv| inv.name.clone())
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        self.investigator_mut(investigator_id)
            .and_then(|inv| inv.spend_resources(amount).ok())
            .ok_or_else(|| GameError::RuleViolation("insufficient resources".into()))?;
        self.log(format!("{investigator_name} spends {} resource(s).", amount));
        Ok(())
    }

    /// Heal damage on an investigator. Used by card effects that heal damage.
    pub fn heal_damage(&mut self, investigator_id: Uuid, amount: i32) -> Result<i32> {
        let investigator_name = self
            .investigator(investigator_id)
            .map(|inv| inv.name.clone())
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        let new_health = self
            .investigator_mut(investigator_id)
            .map(|inv| {
                inv.heal_damage(amount);
                inv.health
            })
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        self.log(format!(
            "{} heals {} damage. Health: {}/{}",
            investigator_name, amount, new_health,
            self.investigator(investigator_id)
                .map(|inv| inv.max_health)
                .unwrap_or(0)
        ));
        Ok(new_health)
    }

    /// Heal horror on an investigator. Used by card effects that heal horror.
    pub fn heal_horror(&mut self, investigator_id: Uuid, amount: i32) -> Result<i32> {
        let investigator_name = self
            .investigator(investigator_id)
            .map(|inv| inv.name.clone())
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        let new_sanity = self
            .investigator_mut(investigator_id)
            .map(|inv| {
                inv.heal_horror(amount);
                inv.sanity
            })
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        self.log(format!(
            "{} heals {} horror. Sanity: {}/{}",
            investigator_name, amount, new_sanity,
            self.investigator(investigator_id)
                .map(|inv| inv.max_sanity)
                .unwrap_or(0)
        ));
        Ok(new_sanity)
    }

    /// Deal damage to an investigator. Used by card effects that deal damage.
    pub fn take_damage(&mut self, investigator_id: Uuid, amount: i32) -> Result<i32> {
        let investigator_name = self
            .investigator(investigator_id)
            .map(|inv| inv.name.clone())
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        let new_health = self
            .investigator_mut(investigator_id)
            .map(|inv| {
                inv.take_damage(amount);
                inv.health
            })
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        self.log(format!(
            "{} takes {} damage. Health: {}/{}",
            investigator_name, amount, new_health,
            self.investigator(investigator_id)
                .map(|inv| inv.max_health)
                .unwrap_or(0)
        ));
        Ok(new_health)
    }

    /// Deal horror to an investigator. Used by card effects that deal horror.
    pub fn take_horror(&mut self, investigator_id: Uuid, amount: i32) -> Result<i32> {
        let investigator_name = self
            .investigator(investigator_id)
            .map(|inv| inv.name.clone())
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        let new_sanity = self
            .investigator_mut(investigator_id)
            .map(|inv| {
                inv.take_horror(amount);
                inv.sanity
            })
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        self.log(format!(
            "{} takes {} horror. Sanity: {}/{}",
            investigator_name, amount, new_sanity,
            self.investigator(investigator_id)
                .map(|inv| inv.max_sanity)
                .unwrap_or(0)
        ));
        Ok(new_sanity)
    }

    /// Move the active investigator to a location in play.
    pub fn move_to_location(
        &mut self,
        investigator_id: Uuid,
        location_instance_id: Uuid,
    ) -> Result<MoveResult> {
        let investigator = self
            .investigator(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;

        let location = self
            .locations_in_play
            .iter()
            .find(|l| l.id == location_instance_id)
            .ok_or_else(|| GameError::CardNotFound(location_instance_id.to_string()))?;

        let investigator_name = investigator.name.clone();
        let location_name = location.card.name.clone();

        // Spend one action to move.
        self.spend_action(investigator_id)?;

        let investigator = self
            .investigator_mut(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        investigator.location_id = Some(location_instance_id);

        // Auto-engage: any ready, unengaged enemy at this location engages the
        // investigator (RRG "Enemy Engagement", page 10).
        let mut engaged_names: Vec<String> = Vec::new();
        for enemy in &mut self.enemies_in_play {
            if enemy.engaged_with.is_none()
                && enemy.location_id == Some(location_instance_id)
                && !enemy.exhausted
            {
                enemy.engaged_with = Some(investigator_id);
                enemy.location_id = None;
                engaged_names.push(enemy.card.name.clone());
            }
        }
        if !engaged_names.is_empty() {
            let message = format!(
                "{} engages {} at {}",
                investigator_name,
                engaged_names.join(", "),
                location_name
            );
            self.log(message);
        }

        let message = format!("{investigator_name} moves to {location_name}");
        self.log(message);

        // Attacks of opportunity: each ready engaged enemy attacks the
        // investigator for moving (RRG "Attack of Opportunity", page 5).
        self.attacks_of_opportunity(investigator_id)?;

        Ok(MoveResult {
            investigator_id,
            location_instance_id,
            location_name,
        })
    }

    /// Each ready enemy engaged with the investigator attacks them once,
    /// dealing damage and horror (RRG "Attack of Opportunity", page 5).
    fn attacks_of_opportunity(&mut self, investigator_id: Uuid) -> Result<()> {
        let engaged_enemy_ids: Vec<Uuid> = self
            .enemies_in_play
            .iter()
            .filter(|e| {
                e.engaged_with == Some(investigator_id) && !e.exhausted
            })
            .map(|e| e.id)
            .collect();

        for enemy_id in engaged_enemy_ids {
            let enemy = self
                .enemies_in_play
                .iter()
                .find(|e| e.id == enemy_id)
                .cloned()
                .ok_or_else(|| GameError::CardNotFound(enemy_id.to_string()))?;
            let enemy_damage = enemy_damage_value(&enemy.card);
            let enemy_horror = enemy_horror_value(&enemy.card);

            let investigator = self
                .investigator_mut(investigator_id)
                .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
            investigator.take_damage(enemy_damage);
            investigator.take_horror(enemy_horror);

            let message = format!(
                "{} attacks {} for {} damage and {} horror",
                enemy.card.name, investigator.name, enemy_damage, enemy_horror
            );
            self.log(message);
        }

        Ok(())
    }

    /// Investigate at the active investigator's current location: spend an action,
    /// make an intellect test, and discover clues equal to successes above the
    /// location's shroud value.
    pub fn investigate(
        &mut self,
        investigator_id: Uuid,
    ) -> Result<InvestigateResult> {
        let investigator = self
            .investigator(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;

        let location_id = investigator
            .location_id
            .ok_or_else(|| GameError::RuleViolation("investigator is not at a location".into()))?;

        let location_index = self
            .locations_in_play
            .iter()
            .position(|l| l.id == location_id)
            .ok_or_else(|| GameError::CardNotFound(location_id.to_string()))?;

        let shroud = location_shroud_value(&self.locations_in_play[location_index].card);
        let clues_remaining = self.locations_in_play[location_index].clues_remaining;

        if clues_remaining == 0 {
            return Err(GameError::RuleViolation(
                "no clues remain at this location".into(),
            ));
        }

        let investigator_name = investigator.name.clone();

        // Spend one action to investigate.
        self.spend_action(investigator_id)?;

        // Attacks of opportunity: each ready engaged enemy attacks the
        // investigator for investigating (RRG "Attack of Opportunity", page 5).
        self.attacks_of_opportunity(investigator_id)?;

        // Resolve intellect skill test.
        let test = self.skill_test(investigator_id, SkillIcon::Intellect)?;
        let success = test.total >= shroud;
        let clues_discovered = if success { 1 } else { 0 };

        let location = &mut self.locations_in_play[location_index];
        let location_name = location.card.name.clone();
        location.clues_remaining = location.clues_remaining.saturating_sub(clues_discovered);
        let clues_remaining_after = location.clues_remaining;

        let investigator = self
            .investigator_mut(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        investigator.clues += clues_discovered as i32;

        let message = if clues_discovered > 0 {
            format!(
                "{} discovers 1 clue at {}",
                investigator_name, location_name
            )
        } else {
            format!(
                "{} fails to discover clues at {}",
                investigator_name,
                location_name
            )
        };
        self.log(message);

        Ok(InvestigateResult {
            investigator_id,
            location_instance_id: location_id,
            shroud,
            test_total: test.total,
            successes: if success { 1 } else { 0 },
            clues_discovered,
            clues_remaining: clues_remaining_after,
        })
    }

    /// Place doom on the current agenda. If doom reaches the agenda's doom
    /// threshold, advance to the next agenda. Returns whether the agenda
    /// advanced and whether the game is lost.
    pub fn place_doom(&mut self, amount: u32) -> Result<PlaceDoomResult> {
        let agenda_index = self.current_agenda_index;

        // Step 1: Modify doom on the current agenda; borrow ends here.
        let (agenda_name, doom_threshold, doom_after, advanced) = {
            let agenda = self
                .agendas
                .get_mut(agenda_index)
                .ok_or_else(|| GameError::RuleViolation("no current agenda".into()))?;

            let name = agenda.card.name.clone();
            let threshold = agenda.doom_threshold;

            agenda.doom += amount;
            let advanced = agenda.doom >= threshold;
            let doom = agenda.doom;

            (name, threshold, doom, advanced)
        };

        // Step 2: If the threshold was reached, reset doom and advance.
        let mut next_agenda = None;
        if advanced {
            self.agendas[agenda_index].doom = 0;
            self.current_agenda_index += 1;
            if self.current_agenda_index >= self.agendas.len() {
                self.game_status = GameStatus::Lost;
            } else {
                next_agenda = Some(
                    self.agendas[self.current_agenda_index].card.name.clone(),
                );
            }
        }

        let message = format!("Doom placed on {agenda_name} ({doom_after}/{doom_threshold})");
        self.log(message);

        Ok(PlaceDoomResult {
            agenda_index,
            doom: doom_after,
            advanced,
            next_agenda,
            game_status: self.game_status,
        })
    }

    /// Advance the current act by spending clues. Multiple investigators may
    /// contribute clues together to meet the act's clue cost, per the rulebook.
    pub fn advance_act(&mut self, contributions: Vec<(Uuid, u32)>) -> Result<AdvanceActResult> {
        let act_index = self.current_act_index;

        if contributions.is_empty() {
            return Err(GameError::RuleViolation("must spend at least 1 clue".into()));
        }

        let total_clues: u32 = contributions.iter().map(|(_, c)| *c).sum();
        if total_clues == 0 {
            return Err(GameError::RuleViolation("must spend at least 1 clue".into()));
        }

        // Validate all contributors have enough clues before spending anything.
        for (inv_id, clues) in &contributions {
            let investigator = self
                .investigator(*inv_id)
                .ok_or_else(|| GameError::InvestigatorNotFound(inv_id.to_string()))?;
            if investigator.clues < *clues as i32 {
                return Err(GameError::RuleViolation(format!(
                    "{} does not have enough clues to spend {}",
                    investigator.name, clues
                )));
            }
        }

        // Step 1: Modify progress on the current act.
        let act = self
            .acts
            .get_mut(act_index)
            .ok_or_else(|| GameError::RuleViolation("no current act".into()))?;

        let cost = act.card.raw.get("clue_cost")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(1);

        let name = act.card.name.clone();
        act.progress += total_clues;
        let advanced = act.progress >= cost;
        let progress = act.progress;

        // Step 2: If the clue cost was met, reset progress and advance.
        let mut next_act = None;
        let mut scenario_completed = false;
        if advanced {
            self.acts[act_index].progress = 0;
            self.current_act_index += 1;
            if self.current_act_index >= self.acts.len() {
                self.game_status = GameStatus::Won;
                scenario_completed = true;
            } else {
                next_act = Some(self.acts[self.current_act_index].card.name.clone());
            }
        }

        // Collect contributor names before spending (which moves contributions).
        let contributor_names: Vec<String> = contributions
            .iter()
            .map(|(id, _)| {
                self.investigator(*id)
                    .map(|inv| inv.name.clone())
                    .unwrap_or_default()
            })
            .collect();

        // Step 3: Spend clues from each contributing investigator.
        for (inv_id, clues) in contributions {
            let investigator = self
                .investigator_mut(inv_id)
                .ok_or_else(|| GameError::InvestigatorNotFound(inv_id.to_string()))?;
            investigator.clues -= clues as i32;
        }

        let message = format!(
            "{} spend {} clue{} on act {}",
            contributor_names.join(", "),
            total_clues,
            if total_clues == 1 { "" } else { "s" },
            name
        );
        self.log(message);

        // Step 4: If scenario completed, award XP from victory display and mark scenario complete.
        if scenario_completed {
            if !self.campaign_progress.completed_scenarios.contains(&self.scenario) {
                self.campaign_progress.completed_scenarios.push(self.scenario.clone());
            }
    // XP = total victory value of all cards in the victory display
    // plus any campaign guide bonuses/penalties (not yet modelled).
    let victory_xp: i32 = self
        .victory_display
        .iter()
        .map(|card| victory_value(card) as i32)
        .sum();
    let mut messages: Vec<String> = Vec::new();
    for inv in &mut self.investigators {
                inv.xp += victory_xp;
                messages.push(format!(
                    "{} gains {} XP for completing scenario",
                    inv.name, victory_xp
                ));
            }
            for msg in messages {
                self.log(msg);
            }
        }

        Ok(AdvanceActResult {
            act_index,
            progress,
            advanced,
            next_act,
            game_status: self.game_status,
        })
    }

    /// Check whether any investigator is defeated (health or sanity at 0).
    /// If all investigators are defeated, the game is lost.
    pub fn check_game_status(&mut self) -> GameStatus {
        if self.investigators.iter().all(|inv| inv.is_defeated) {
            self.game_status = GameStatus::Lost;
        }
        self.game_status
    }

    pub fn log(&mut self, message: impl Into<String>) {
        self.log_messages.push(message.into());
    }

    /// Record the current scenario's result into the campaign log and prepare
    /// investigator state to carry over to the next scenario.
    pub fn add_campaign_log_entry(&mut self, result: CampaignResult, notes: String) {
        let investigator_names: Vec<String> = self
            .investigators
            .iter()
            .map(|inv| inv.name.clone())
            .collect();
        self.campaign_progress.campaign_log.push(CampaignLogEntry {
            scenario_code: self.scenario.clone(),
            scenario_name: self.scenario.clone(),
            result: result.clone(),
            investigator_names,
            notes,
        });
        // completed_scenarios is managed by advance_act when scenario_completed;
        // do not duplicate here.
    }

    /// Start a new scenario within the same campaign, carrying investigator state
    /// forward per rulebook campaign rules: XP, decklist, and trauma carry over;
    /// killed/insane investigators are removed; resources reset to 5; damage/horror
    /// from trauma applied at setup.
    pub fn start_scenario_in_campaign(
        &mut self,
        scenario: impl Into<String>,
        scenario_data: ScenarioData,
        seed: u64,
    ) {
        // Carry investigator state forward per campaign rules.
        // Killed or driven insane investigators cannot be used for the remainder
        // of the campaign (RRG "Killed/Insane Investigators", page 13).
        let mut carried_investigators: Vec<Investigator> = Vec::new();
        let mut removed_messages: Vec<String> = Vec::new();
        for inv in &self.investigators {
            if inv.is_killed || inv.is_driven_insane {
                removed_messages.push(format!(
                    "{} cannot continue the campaign (killed/insane)",
                    inv.name
                ));
                continue;
            }
            let mut carried = inv.clone();
            // Reset per-scenario state but keep campaign-progression state.
            carried.actions_remaining = 3;
            carried.location_id = None;
            carried.engaged_enemies.clear();
            carried.deck.hand.clear();
            carried.deck.discard_pile.clear();
            carried.deck.play_area.clear();
            // Resources reset to 5 at scenario setup (RRG page 13).
            carried.resources = DEFAULT_RESOURCES;
            // Apply trauma damage/horror at setup: physical trauma = damage,
            // mental trauma = horror (RRG "Trauma", page 5).
            if carried.physical_trauma > 0 {
                carried.health = (carried.health - carried.physical_trauma).max(0);
            }
            if carried.mental_trauma > 0 {
                carried.sanity = (carried.sanity - carried.mental_trauma).max(0);
            }
            carried.is_defeated = carried.health == 0 || carried.sanity == 0;
            carried_investigators.push(carried);
        }
        for msg in removed_messages {
            self.log(msg);
        }

        // Reset scenario-specific state.
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let mut encounter_deck: Vec<Card> = Vec::new();
        encounter_deck.shuffle(&mut rng);

        let acts: Vec<ActInstance> = scenario_data
            .acts
            .iter()
            .map(|act_data| ActInstance {
                id: Uuid::new_v4(),
                card: Card {
                    id: Uuid::new_v4(),
                    code: act_data.code.clone(),
                    name: act_data.name.clone(),
                    card_type: crate::engine::card::CardType::Act,
                    class: Some(crate::engine::card::CardClass::Neutral),
                    cost: None,
                    xp: 0,
                    icons: Default::default(),
                    traits: Vec::new(),
                    text: act_data.text.clone(),
                    raw: act_data.raw.clone(),
                },
                progress: 0,
            })
            .collect();

        let agendas: Vec<AgendaInstance> = scenario_data
            .agendas
            .iter()
            .map(|agenda_data| AgendaInstance {
                id: Uuid::new_v4(),
                card: Card {
                    id: Uuid::new_v4(),
                    code: agenda_data.code.clone(),
                    name: agenda_data.name.clone(),
                    card_type: crate::engine::card::CardType::Agenda,
                    class: Some(crate::engine::card::CardClass::Neutral),
                    cost: None,
                    xp: 0,
                    icons: Default::default(),
                    traits: Vec::new(),
                    text: agenda_data.text.clone(),
                    raw: agenda_data.raw.clone(),
                },
                doom: 0,
                doom_threshold: agenda_data.doom_threshold,
            })
            .collect();

        let locations: Vec<LocationInstance> = scenario_data
            .locations
            .iter()
            .map(|location_data| LocationInstance {
                id: Uuid::new_v4(),
                card: Card {
                    id: Uuid::new_v4(),
                    code: location_data.code.clone(),
                    name: location_data.name.clone(),
                    card_type: crate::engine::card::CardType::Location,
                    class: Some(crate::engine::card::CardClass::Neutral),
                    cost: None,
                    xp: 0,
                    icons: Default::default(),
                    traits: Vec::new(),
                    text: location_data.text.clone(),
                    raw: location_data.raw.clone(),
                },
                clues_remaining: location_data.clues,
            })
            .collect();

        self.scenario = scenario.into();
        self.round = 1;
        self.phase = Phase::Mythos;
        self.active_investigator = 0;
        self.investigators = carried_investigators;
        self.chaos_bag = ChaosBag::standard(seed);
        self.encounter_deck = encounter_deck;
        self.encounter_discard = Vec::new();
        self.enemies_in_play = Vec::new();
        self.locations_in_play = locations;
        self.current_act_index = 0;
        self.current_agenda_index = 0;
        self.acts = acts;
        self.agendas = agendas;
        self.game_status = GameStatus::Ongoing;

        let message = format!(
            "Started scenario: {} in campaign {} (seed {:#x})",
            self.scenario, self.campaign, seed
        );
        self.log(message);
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyInstance {
    pub id: Uuid,
    pub card: Card,
    pub health: i32,
    pub horror: i32,
    pub engaged_with: Option<Uuid>,
    /// Location instance ID where an unengaged enemy sits. `None` when engaged.
    pub location_id: Option<Uuid>,
    /// Whether the enemy is exhausted (cannot engage, attack, or hunt).
    pub exhausted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngageResult {
    pub investigator_id: Uuid,
    pub enemy_instance_id: Uuid,
    pub enemy_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FightResult {
    pub investigator_id: Uuid,
    pub enemy_instance_id: Uuid,
    pub success: bool,
    pub enemy_health: i32,
    pub damage_dealt: i32,
    pub damage_taken: i32,
    pub horror_taken: i32,
    pub skill_test: SkillTestResult,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInstance {
    pub id: Uuid,
    pub card: Card,
    pub clues_remaining: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveResult {
    pub investigator_id: Uuid,
    pub location_instance_id: Uuid,
    pub location_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigateResult {
    pub investigator_id: Uuid,
    pub location_instance_id: Uuid,
    pub shroud: i8,
    pub test_total: i8,
    pub successes: i8,
    pub clues_discovered: u32,
    pub clues_remaining: u32,
}

/// Extract a location's shroud value from its raw ArkhamDB data. Defaults to 1.
fn location_shroud_value(card: &Card) -> i8 {
    card.raw
        .get("shroud")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i8
}

/// Extract a location's clue count from its raw ArkhamDB data. Defaults to 0.
fn location_clues(card: &Card) -> u32 {
    card.raw
        .get("clues")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(0)
}

/// Extract an enemy's health from its raw ArkhamDB data. Defaults to 1.
fn enemy_health(card: &Card) -> i32 {
    card.raw
        .get("health")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32
}

/// Extract an enemy's horror value from its raw ArkhamDB data. Defaults to 1.
fn enemy_horror_value(card: &Card) -> i32 {
    card.raw
        .get("horror")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32
}

/// Extract an enemy's fight value from its raw ArkhamDB data. Defaults to 1.
fn enemy_fight_value(card: &Card) -> i8 {
    card.raw
        .get("enemy_fight")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i8
}

/// Extract an enemy's damage value from its raw ArkhamDB data. Defaults to 1.
fn enemy_damage_value(card: &Card) -> i32 {
    card.raw
        .get("enemy_damage")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32
}

/// Extract a card's victory value from its raw ArkhamDB data. Defaults to 0.
fn victory_value(card: &Card) -> u32 {
    card.raw
        .get("victory")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStatus {
    Ongoing,
    Won,
    Lost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActInstance {
    pub id: Uuid,
    pub card: Card,
    pub progress: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaInstance {
    pub id: Uuid,
    pub card: Card,
    pub doom: u32,
    pub doom_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceDoomResult {
    pub agenda_index: usize,
    pub doom: u32,
    pub advanced: bool,
    pub next_agenda: Option<String>,
    pub game_status: GameStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvanceActResult {
    pub act_index: usize,
    pub progress: u32,
    pub advanced: bool,
    pub next_act: Option<String>,
    pub game_status: GameStatus,
}

/// Default act cards used when no act cards exist in the encounter deck.
fn default_acts() -> Vec<ActInstance> {
    let make_act = |code: &str, name: &str, text: &str, clue_cost: u32| {
        ActInstance {
            id: Uuid::new_v4(),
            card: Card {
                id: Uuid::new_v4(),
                code: code.to_string(),
                name: name.to_string(),
                card_type: crate::engine::card::CardType::Act,
                class: Some(crate::engine::card::CardClass::Neutral),
                cost: None,
                xp: 0,
                icons: Default::default(),
                traits: Vec::new(),
                text: text.to_string(),
                raw: serde_json::json!({
                    "code": code,
                    "name": name,
                    "clue_cost": clue_cost,
                    "type_code": "act",
                }),
            },
            progress: 0,
        }
    };

    vec![
        make_act("act-001", "Investigate the Scene", "Gather clues at the locations to learn what happened.", 3),
        make_act("act-002", "Confront the Threat", "Face the source of the corruption head-on.", 4),
    ]
}

/// Default agenda cards used when no agenda cards exist in the encounter deck.
fn default_agendas() -> Vec<AgendaInstance> {
    let make_agenda = |code: &str, name: &str, text: &str, doom_threshold: u32| {
        AgendaInstance {
            id: Uuid::new_v4(),
            card: Card {
                id: Uuid::new_v4(),
                code: code.to_string(),
                name: name.to_string(),
                card_type: crate::engine::card::CardType::Agenda,
                class: Some(crate::engine::card::CardClass::Neutral),
                cost: None,
                xp: 0,
                icons: Default::default(),
                traits: Vec::new(),
                text: text.to_string(),
                raw: serde_json::json!({
                    "code": code,
                    "name": name,
                    "doom_threshold": doom_threshold,
                    "type_code": "agenda",
                }),
            },
            doom: 0,
            doom_threshold,
        }
    };

    vec![
        make_agenda("agenda-001", "The Darkness Grows", "The ancient evil stirs in the shadows.", 8),
        make_agenda("agenda-002", "The Hour of Reckoning", "Doom consumes Arkham if not stopped.", 8),
    ]
}
