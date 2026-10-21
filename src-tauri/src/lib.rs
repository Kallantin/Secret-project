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
            commands::place_doom,
            commands::advance_act,
            commands::play_card,
            commands::save_game,
            commands::load_game,
            commands::open_pdf,
            commands::start_next_scenario,
            commands::add_campaign_log_entry,
            commands::discard_card,
            commands::draw_card,
            commands::add_card_to_play_area,
            commands::gain_resource,
            commands::spend_resources_cmd,
            commands::heal_damage_cmd,
            commands::heal_horror_cmd,
            commands::take_damage_cmd,
            commands::take_horror_cmd,
            commands::next_investigator,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Arkham Horror LCG");
}
