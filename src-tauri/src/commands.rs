use serde::Deserialize;
use std::sync::Mutex;

use tauri::State;

use crate::engine::card::{Card, SkillIcon};
use crate::engine::chaos::ChaosToken;
use crate::engine::game_state::{GameState, MythosResult, PlayCardResult, SkillTestResult};
use crate::engine::investigator::Investigator;
use crate::error::{GameError, Result};
use uuid::Uuid;

const DEFAULT_SEED: u64 = 0xA2C4_1F0D;

#[derive(Default)]
pub struct AppState {
    pub game: Mutex<Option<GameState>>,
}

impl AppState {
    fn lock_game(&self) -> Result<std::sync::MutexGuard<'_, Option<GameState>>> {
        self.game
            .lock()
            .map_err(|_| GameError::RuleViolation("game state lock poisoned".into()))
    }
}

#[derive(Deserialize)]
pub struct NewGameInput {
    pub campaign: String,
    pub scenario: String,
    pub seed: Option<u64>,
    #[serde(default)]
    pub encounter_cards: Vec<Card>,
}

#[tauri::command]
pub fn new_game(
    state: State<'_, AppState>,
    input: NewGameInput,
) -> Result<GameState> {
    let game = GameState::new(
        input.campaign,
        input.scenario,
        input.seed.unwrap_or(DEFAULT_SEED),
        input.encounter_cards,
    );
    let mut guard = state.lock_game()?;
    *guard = Some(game.clone());
    Ok(game)
}

#[tauri::command]
pub fn add_investigator(
    state: State<'_, AppState>,
    code: String,
    name: String,
    willpower: i8,
    intellect: i8,
    combat: i8,
    agility: i8,
    max_health: i32,
    max_sanity: i32,
) -> Result<GameState> {
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    let investigator = Investigator::new(
        code,
        name,
        willpower,
        intellect,
        combat,
        agility,
        max_health,
        max_sanity,
    );
    game.add_investigator(investigator);
    Ok(game.clone())
}

#[tauri::command]
pub fn get_game_state(state: State<'_, AppState>) -> Result<GameState> {
    let guard = state.lock_game()?;
    guard
        .as_ref()
        .cloned()
        .ok_or(GameError::GameNotInitialized)
}

#[tauri::command]
pub fn advance_phase(state: State<'_, AppState>) -> Result<GameState> {
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.advance_phase();
    Ok(game.clone())
}

#[tauri::command]
pub fn draw_chaos_token(state: State<'_, AppState>) -> Result<ChaosToken> {
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    Ok(game.draw_chaos_token())
}

#[tauri::command]
pub fn skill_test(
    state: State<'_, AppState>,
    investigator_id: String,
    skill: String,
) -> Result<SkillTestResult> {
    let id = Uuid::parse_str(&investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {investigator_id}")))?;
    let skill_enum = match skill.to_lowercase().as_str() {
        "willpower" => SkillIcon::Willpower,
        "intellect" => SkillIcon::Intellect,
        "combat" => SkillIcon::Combat,
        "agility" => SkillIcon::Agility,
        _ => return Err(GameError::RuleViolation(format!("unknown skill: {skill}"))),
    };
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.skill_test(id, skill_enum)
}

#[tauri::command]
pub fn spend_action(state: State<'_, AppState>, investigator_id: String) -> Result<GameState> {
    let id = Uuid::parse_str(&investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {investigator_id}")))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.spend_action(id)?;
    Ok(game.clone())
}

#[tauri::command]
pub fn reset_turn(state: State<'_, AppState>) -> Result<GameState> {
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.reset_turn();
    Ok(game.clone())
}

#[tauri::command]
pub fn mythos_phase(state: State<'_, AppState>) -> Result<MythosResult> {
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.mythos_phase()
}

#[tauri::command]
pub fn play_card(
    state: State<'_, AppState>,
    investigator_id: String,
    card_id: String,
) -> Result<PlayCardResult> {
    let investigator_id = Uuid::parse_str(&investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {investigator_id}")))?;
    let card_id = Uuid::parse_str(&card_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid card id: {card_id}")))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.play_card(investigator_id, card_id)
}

#[tauri::command]
pub fn save_game(state: State<'_, AppState>, path: String) -> Result<()> {
    let guard = state.lock_game()?;
    let game = guard.as_ref().ok_or(GameError::GameNotInitialized)?;
    let json = serde_json::to_string_pretty(game).map_err(GameError::from)?;
    std::fs::write(&path, json).map_err(GameError::from)?;
    Ok(())
}

#[tauri::command]
pub fn load_game(state: State<'_, AppState>, path: String) -> Result<GameState> {
    let data = std::fs::read_to_string(&path).map_err(GameError::from)?;
    let game: GameState = serde_json::from_str(&data).map_err(GameError::from)?;
    let mut guard = state.lock_game()?;
    *guard = Some(game.clone());
    Ok(game)
}
