use std::sync::Mutex;

use tauri::State;

use crate::engine::chaos::ChaosToken;
use crate::engine::game_state::GameState;
use crate::engine::investigator::Investigator;
use crate::error::{GameError, Result};

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

#[tauri::command]
pub fn new_game(
    state: State<'_, AppState>,
    campaign: String,
    scenario: String,
    seed: Option<u64>,
) -> Result<GameState> {
    let game = GameState::new(campaign, scenario, seed.unwrap_or(DEFAULT_SEED));
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
