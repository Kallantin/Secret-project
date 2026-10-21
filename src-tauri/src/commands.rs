use rand::Rng;
use serde::Deserialize;
use std::process::Command;
use std::sync::Mutex;

use tauri::State;

use crate::engine::card::{Card, SkillIcon};
use crate::engine::chaos::ChaosToken;
use crate::engine::game_state::{
    GameState, AdvanceActResult, CampaignProgress, CampaignResult, EngageResult, FightResult,
    InvestigateResult, MoveResult, MythosResult, PlaceDoomResult, PlayCardResult, ScenarioData,
    SkillTestResult,
};
use crate::engine::investigator::Investigator;
use crate::error::{GameError, Result};
use uuid::Uuid;

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
    #[serde(default)]
    pub scenario_data: ScenarioData,
    #[serde(default)]
    pub campaign_progress: Option<CampaignProgress>,
}

#[derive(Deserialize)]
pub struct CampaignProgressInput {
    #[serde(default)]
    pub campaign_progress: Option<CampaignProgress>,
}

#[derive(Deserialize)]
pub struct StartNextScenarioInput {
    pub scenario: String,
    pub seed: Option<u64>,
    #[serde(default)]
    pub scenario_data: ScenarioData,
}

#[derive(Deserialize)]
pub struct AddCampaignLogEntryInput {
    pub result: String,
    pub notes: String,
}

#[tauri::command]
pub fn new_game(
    state: State<'_, AppState>,
    input: NewGameInput,
) -> Result<GameState> {
    // Rulebook: each playthrough uses a unique seed; default to random when none provided.
    let seed = input.seed.unwrap_or_else(|| rand::thread_rng().gen());
    let mut game = GameState::new(
        input.campaign,
        input.scenario,
        seed,
        input.encounter_cards,
        input.scenario_data,
    );
    // If campaign progress is provided, carry it over (continuing a campaign).
    if let Some(cp) = input.campaign_progress {
        game.campaign_progress = cp;
    }
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
pub fn engage(
    state: State<'_, AppState>,
    investigator_id: String,
    enemy_instance_id: String,
) -> Result<EngageResult> {
    let investigator_id = Uuid::parse_str(&investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {investigator_id}")))?;
    let enemy_instance_id = Uuid::parse_str(&enemy_instance_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid enemy instance id: {enemy_instance_id}")))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.engage(investigator_id, enemy_instance_id)
}

#[tauri::command]
pub fn fight(
    state: State<'_, AppState>,
    investigator_id: String,
    enemy_instance_id: String,
) -> Result<FightResult> {
    let investigator_id = Uuid::parse_str(&investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {investigator_id}")))?;
    let enemy_instance_id = Uuid::parse_str(&enemy_instance_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid enemy instance id: {enemy_instance_id}")))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.fight(investigator_id, enemy_instance_id)
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
pub fn move_to_location(
    state: State<'_, AppState>,
    investigator_id: String,
    location_instance_id: String,
) -> Result<MoveResult> {
    let investigator_id = Uuid::parse_str(&investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {investigator_id}")))?;
    let location_instance_id = Uuid::parse_str(&location_instance_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid location instance id: {location_instance_id}")))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.move_to_location(investigator_id, location_instance_id)
}

#[tauri::command]
pub fn investigate(
    state: State<'_, AppState>,
    investigator_id: String,
) -> Result<InvestigateResult> {
    let investigator_id = Uuid::parse_str(&investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {investigator_id}")))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.investigate(investigator_id)
}

#[tauri::command]
pub fn place_doom(
    state: State<'_, AppState>,
    amount: u32,
) -> Result<PlaceDoomResult> {
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.place_doom(amount)
}

#[derive(Deserialize)]
pub struct ActContribution {
    pub investigator_id: String,
    pub clues_spent: u32,
}

#[derive(Deserialize)]
pub struct AdvanceActInput {
    pub contributions: Vec<ActContribution>,
}

#[tauri::command]
pub fn advance_act(
    state: State<'_, AppState>,
    input: AdvanceActInput,
) -> Result<AdvanceActResult> {
    let contributions: Result<Vec<(Uuid, u32)>> = input
        .contributions
        .iter()
        .map(|c| {
            Ok((
                Uuid::parse_str(&c.investigator_id)
                    .map_err(|_| {
                        GameError::RuleViolation(format!(
                            "invalid investigator id: {}",
                            c.investigator_id
                        ))
                    })?,
                c.clues_spent,
            ))
        })
        .collect();
    let contributions = contributions?;

    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.advance_act(contributions)
}

#[tauri::command]
pub fn start_next_scenario(
    state: State<'_, AppState>,
    input: StartNextScenarioInput,
) -> Result<GameState> {
    let seed = input.seed.unwrap_or_else(|| rand::thread_rng().gen());
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.start_scenario_in_campaign(input.scenario, input.scenario_data, seed);
    let game_state = game.clone();
    Ok(game_state)
}

#[tauri::command]
pub fn add_campaign_log_entry(
    state: State<'_, AppState>,
    input: AddCampaignLogEntryInput,
) -> Result<GameState> {
    let result = match input.result.as_str() {
        "won" => CampaignResult::Won,
        "lost" => CampaignResult::Lost,
        _ => return Err(GameError::RuleViolation(format!(
            "invalid campaign result: {}",
            input.result
        ))),
    };
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.add_campaign_log_entry(result, input.notes);
    let game_state = game.clone();
    Ok(game_state)
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

#[tauri::command]
pub fn open_pdf(filename: String) -> Result<()> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| GameError::RuleViolation(format!("cannot locate exe: {e}")))?
        .parent()
        .ok_or(GameError::RuleViolation("exe has no parent directory".to_string()))?
        .to_path_buf();

    let candidates: Vec<std::path::PathBuf> = vec![
        exe_dir.join("rules").join(&filename),
        exe_dir.join("src").join("rules").join(&filename),
        exe_dir.join("..").join("src-tauri").join("src").join("rules").join(&filename),
        exe_dir.join("..").join("..").join("src-tauri").join("src").join("rules").join(&filename),
    ];

    let path = candidates
        .into_iter()
        .find(|p| p.exists())
        .ok_or_else(|| GameError::RuleViolation(format!("PDF not found: {filename}")))?;

    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", "start", "", path.to_str().unwrap()])
            .spawn()
            .map_err(|e| GameError::RuleViolation(format!("failed to open PDF: {e}")))?;
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(path.to_str().unwrap())
            .spawn()
            .map_err(|e| GameError::RuleViolation(format!("failed to open PDF: {e}")))?;
    } else {
        Command::new("xdg-open")
            .arg(path.to_str().unwrap())
            .spawn()
            .map_err(|e| GameError::RuleViolation(format!("failed to open PDF: {e}")))?;
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct DiscardCardInput {
    pub investigator_id: String,
    pub card_id: String,
}

#[tauri::command]
pub fn discard_card(
    state: State<'_, AppState>,
    input: DiscardCardInput,
) -> Result<Card> {
    let investigator_id = Uuid::parse_str(&input.investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {}", input.investigator_id)))?;
    let card_id = Uuid::parse_str(&input.card_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid card id: {}", input.card_id)))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.discard_card(investigator_id, card_id)
}

#[derive(Deserialize)]
pub struct DrawCardInput {
    pub investigator_id: String,
}

#[tauri::command]
pub fn draw_card(
    state: State<'_, AppState>,
    input: DrawCardInput,
) -> Result<Card> {
    let investigator_id = Uuid::parse_str(&input.investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {}", input.investigator_id)))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.draw_card(investigator_id)
}

#[derive(Deserialize)]
pub struct AddCardToPlayAreaInput {
    pub investigator_id: String,
    pub card: Card,
}

#[tauri::command]
pub fn add_card_to_play_area(
    state: State<'_, AppState>,
    input: AddCardToPlayAreaInput,
) -> Result<()> {
    let investigator_id = Uuid::parse_str(&input.investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {}", input.investigator_id)))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.add_card_to_play_area(investigator_id, input.card)
}

#[derive(Deserialize)]
pub struct GainResourceInput {
    pub investigator_id: String,
    pub amount: i32,
}

#[tauri::command]
pub fn gain_resource(
    state: State<'_, AppState>,
    input: GainResourceInput,
) -> Result<i32> {
    let investigator_id = Uuid::parse_str(&input.investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {}", input.investigator_id)))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.gain_resource(investigator_id, input.amount)
}

#[derive(Deserialize)]
pub struct SpendResourcesInput {
    pub investigator_id: String,
    pub amount: i32,
}

#[tauri::command]
pub fn spend_resources_cmd(
    state: State<'_, AppState>,
    input: SpendResourcesInput,
) -> Result<()> {
    let investigator_id = Uuid::parse_str(&input.investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {}", input.investigator_id)))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.spend_resources(investigator_id, input.amount)
}

#[derive(Deserialize)]
pub struct HealDamageInput {
    pub investigator_id: String,
    pub amount: i32,
}

#[tauri::command]
pub fn heal_damage_cmd(
    state: State<'_, AppState>,
    input: HealDamageInput,
) -> Result<i32> {
    let investigator_id = Uuid::parse_str(&input.investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {}", input.investigator_id)))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.heal_damage(investigator_id, input.amount)
}

#[derive(Deserialize)]
pub struct HealHorrorInput {
    pub investigator_id: String,
    pub amount: i32,
}

#[tauri::command]
pub fn heal_horror_cmd(
    state: State<'_, AppState>,
    input: HealHorrorInput,
) -> Result<i32> {
    let investigator_id = Uuid::parse_str(&input.investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {}", input.investigator_id)))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.heal_horror(investigator_id, input.amount)
}

#[derive(Deserialize)]
pub struct TakeDamageInput {
    pub investigator_id: String,
    pub amount: i32,
}

#[tauri::command]
pub fn take_damage_cmd(
    state: State<'_, AppState>,
    input: TakeDamageInput,
) -> Result<i32> {
    let investigator_id = Uuid::parse_str(&input.investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {}", input.investigator_id)))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.take_damage(investigator_id, input.amount)
}

#[derive(Deserialize)]
pub struct TakeHorrorInput {
    pub investigator_id: String,
    pub amount: i32,
}

#[tauri::command]
pub fn take_horror_cmd(
    state: State<'_, AppState>,
    input: TakeHorrorInput,
) -> Result<i32> {
    let investigator_id = Uuid::parse_str(&input.investigator_id)
        .map_err(|_| GameError::RuleViolation(format!("invalid investigator id: {}", input.investigator_id)))?;
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.take_horror(investigator_id, input.amount)
}

#[tauri::command]
pub fn next_investigator(
    state: State<'_, AppState>,
) -> Result<String> {
    let mut guard = state.lock_game()?;
    let game = guard.as_mut().ok_or(GameError::GameNotInitialized)?;
    game.next_investigator()
        .map(|id| id.to_string())
        .ok_or_else(|| GameError::RuleViolation("no investigators in game".into()))
}
