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
    /// Scenario codes that have been completed in this campaign.
    pub completed_scenarios: Vec<String>,
    pub log: Vec<String>,
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
            completed_scenarios: Vec::new(),
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
                let health = enemy_health(&card);
                let instance = EnemyInstance {
                    id: Uuid::new_v4(),
                    card: card.clone(),
                    health,
                    engaged_with: None,
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

        // Spend one action to fight.
        self.spend_action(investigator_id)?;

        // Resolve combat skill test.
        let test = self.skill_test(investigator_id, SkillIcon::Combat)?;
        let success = test.total >= enemy_fight;

        let enemy = &mut self.enemies_in_play[enemy_index];
        let (damage_dealt, damage_taken) = if success {
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
                (damage, 0)
            } else {
                let message = format!(
                    "{investigator_name} hits {enemy_name} for {damage} damage"
                );
                self.log(message);
                (damage, 0)
            }
        } else {
            let investigator = self
                .investigator_mut(investigator_id)
                .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
            investigator.take_damage(enemy_damage);
            let message = format!(
                "{investigator_name} misses {enemy_name}; {enemy_name} hits {investigator_name} for {enemy_damage} damage"
            );
            self.log(message);
            (0, enemy_damage)
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

        let message = format!("{investigator_name} moves to {location_name}");
        self.log(message);

        Ok(MoveResult {
            investigator_id,
            location_instance_id,
            location_name,
        })
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

        // Resolve intellect skill test.
        let test = self.skill_test(investigator_id, SkillIcon::Intellect)?;
        let successes = test.total.saturating_sub(shroud).max(0);
        let clues_discovered = (successes as u32).min(clues_remaining);

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
                "{} discovers {} clue{} at {}",
                investigator_name,
                clues_discovered,
                if clues_discovered == 1 { "" } else { "s" },
                location_name
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
            successes,
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

    /// Advance the current act by spending clues. Each investigator may
    /// contribute clues equal to the act's clue cost divided among them.
    pub fn advance_act(&mut self, investigator_id: Uuid, clues_spent: u32) -> Result<AdvanceActResult> {
        let act_index = self.current_act_index;

        // Step 1: Modify progress on the current act; borrow ends here.
        let (_clue_cost, act_name, advanced, progress_after) = {
            let act = self
                .acts
                .get_mut(act_index)
                .ok_or_else(|| GameError::RuleViolation("no current act".into()))?;

            let cost = act.card.raw.get("clue_cost")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32)
                .unwrap_or(1);

            let name = act.card.name.clone();

            if clues_spent == 0 {
                return Err(GameError::RuleViolation("must spend at least 1 clue".into()));
            }

            act.progress += clues_spent;
            let advanced = act.progress >= cost;
            let progress = act.progress;

            (cost, name, advanced, progress)
        };

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

        // Step 3: Spend clues from the investigator.
        let investigator = self
            .investigator(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        if investigator.clues < clues_spent as i32 {
            return Err(GameError::RuleViolation(
                "investigator does not have enough clues".into(),
            ));
        }

        let investigator_name = investigator.name.clone();
        let investigator = self
            .investigator_mut(investigator_id)
            .ok_or_else(|| GameError::InvestigatorNotFound(investigator_id.to_string()))?;
        investigator.clues -= clues_spent as i32;

        let message = format!(
            "{investigator_name} spends {clues_spent} clue{} on act {act_name}",
            if clues_spent == 1 { "" } else { "s" }
        );
        self.log(message);

        // Step 4: If scenario completed, award XP and mark scenario complete.
            if scenario_completed {
                if !self.completed_scenarios.contains(&self.scenario) {
                    self.completed_scenarios.push(self.scenario.clone());
                }
                let mut messages: Vec<String> = Vec::new();
                for inv in &mut self.investigators {
                    inv.xp += 2; // Award 2 XP per completed scenario
                    messages.push(format!("{} gains 2 XP for completing scenario", inv.name));
                }
                for msg in messages {
                    self.log(msg);
                }
            }

        Ok(AdvanceActResult {
            act_index,
            progress: progress_after,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyInstance {
    pub id: Uuid,
    pub card: Card,
    pub health: i32,
    pub engaged_with: Option<Uuid>,
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
