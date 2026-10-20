pub mod commands;
pub mod engine;
pub mod error;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::new_game,
            commands::add_investigator,
            commands::get_game_state,
            commands::advance_phase,
            commands::draw_chaos_token,
            commands::skill_test,
            commands::spend_action,
            commands::reset_turn,
            commands::mythos_phase,
            commands::engage,
            commands::fight,
            commands::move_to_location,
            commands::investigate,
            commands::play_card,
            commands::save_game,
            commands::load_game,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Arkham Horror LCG");
}
