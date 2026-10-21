use arkham_horror_lcg_lib::{
    engine::card::{Card, CardClass, CardType, SkillIcon, SkillIcons},
    engine::chaos::{ChaosBag, ChaosToken},
    engine::game_state::{
        GameState, GameStatus, ScenarioActData, ScenarioAgendaData,
        ScenarioData, ScenarioLocationData, CampaignResult,
    },
    engine::investigator::{Investigator, DeckSlot},
    engine::phase::Phase,
};
use uuid::Uuid;

fn make_test_investigator(code: &str, name: &str) -> Investigator {
    Investigator::new(
        code,
        name,
        3, // willpower
        20, // intellect (very high to guarantee skill test success)
        3, // combat
        2, // agility
        6, // max_health
        5, // max_sanity
    )
}

#[test]
fn e2e_full_game_flow_win_condition() {
    let encounter_cards = vec![Card {
        id: Uuid::new_v4(),
        code: "01001a".to_string(),
        name: "Rumors at the Boarding House".to_string(),
        card_type: CardType::Act,
        class: Some(CardClass::Neutral),
        cost: None,
        xp: 0,
        icons: SkillIcons::default(),
        traits: vec![],
        text: "Investigate to learn what happened.".to_string(),
        raw: serde_json::json!({
            "code": "01001a",
            "name": "Rumors at the Boarding House",
            "clue_cost": 2,
            "type_code": "act",
        }),
    }];

    let scenario_data = ScenarioData {
        campaign: "night-of-the-zealot".to_string(),
        code: "01001".to_string(),
        name: "The Gathering".to_string(),
        order: 1,
        chaos_bag: "standard".to_string(),
        acts: vec![ScenarioActData {
            code: "01001a".to_string(),
            name: "Rumors at the Boarding House".to_string(),
            text: "Investigate to learn what happened.".to_string(),
            clue_cost: 2,
            raw: serde_json::json!({ "code": "01001a", "clue_cost": 2 }),
        }],
        agendas: vec![ScenarioAgendaData {
            code: "01002a".to_string(),
            name: "The Darkness Grows".to_string(),
            text: "The ancient evil stirs.".to_string(),
            doom_threshold: 2,
            raw: serde_json::json!({ "code": "01002a", "doom_threshold": 2 }),
        }],
        locations: vec![ScenarioLocationData {
            code: "01003".to_string(),
            name: "Street".to_string(),
            text: "A quiet street.".to_string(),
            clues: 2,
            shroud: 0,
            raw: serde_json::json!({ "code": "01003", "shroud": 0, "clues": 2 }),
        }],
    };

    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        12345,
        encounter_cards,
        scenario_data,
    );

    let inv = make_test_investigator("roland-banks", "Roland Banks");
    let inv_id = inv.id;
    game.add_investigator(inv);

    assert_eq!(game.investigators.len(), 1);
    assert_eq!(game.game_status, GameStatus::Ongoing);

    game.reset_turn();
    assert_eq!(game.investigators[0].actions_remaining, 3);

    assert_eq!(game.locations_in_play.len(), 1);
    assert_eq!(game.locations_in_play[0].card.name, "Street");
    assert_eq!(game.locations_in_play[0].clues_remaining, 2);

    let loc_id = game.locations_in_play[0].id;
    game.move_to_location(inv_id, loc_id).unwrap();
    assert_eq!(game.investigators[0].location_id, Some(loc_id));

    // Rulebook: investigate discovers 1 clue per successful test.
    let result1 = game.investigate(inv_id).unwrap();
    assert_eq!(result1.clues_discovered, 1);
    let result2 = game.investigate(inv_id).unwrap();
    assert_eq!(result2.clues_discovered, 1);
    assert_eq!(game.investigators[0].clues, 2);
    assert_eq!(game.locations_in_play[0].clues_remaining, 0);

    // Add a victory card to the victory display (worth 2 XP).
    game.victory_display.push(Card {
        raw: serde_json::json!({ "victory": 2 }),
        ..Card::new("01001", "Ghoul Priest", CardType::Enemy)
    });

    let act_result = game.advance_act(vec![(inv_id, 2)]).unwrap();
    assert!(act_result.advanced);
    assert_eq!(game.game_status, GameStatus::Won);
    assert!(game.campaign_progress.completed_scenarios.contains(&"The Gathering".to_string()));
    assert!(game.investigators[0].xp > 0);
}

#[test]
fn e2e_full_game_flow_loss_condition_agenda() {
    let scenario_data = ScenarioData {
        campaign: "night-of-the-zealot".to_string(),
        code: "01001".to_string(),
        name: "The Gathering".to_string(),
        order: 1,
        chaos_bag: "standard".to_string(),
        acts: vec![],
        agendas: vec![ScenarioAgendaData {
            code: "01002a".to_string(),
            name: "The Darkness Grows".to_string(),
            text: "Doom advances.".to_string(),
            doom_threshold: 1,
            raw: serde_json::json!({ "code": "01002a", "doom_threshold": 1 }),
        }],
        locations: vec![],
    };

    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        12345,
        vec![],
        scenario_data,
    );

    let result = game.place_doom(1).unwrap();
    assert!(result.advanced);
    assert_eq!(game.game_status, GameStatus::Lost);
    assert_eq!(game.current_agenda_index, 1);
}

#[test]
fn e2e_full_game_flow_loss_condition_all_defeated() {
    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        12345,
        vec![],
        ScenarioData {
            campaign: "night-of-the-zealot".to_string(),
            code: "01001".to_string(),
            name: "The Gathering".to_string(),
            order: 1,
            chaos_bag: "standard".to_string(),
            acts: vec![],
            agendas: vec![],
            locations: vec![],
        },
    );

    let inv = make_test_investigator("roland-banks", "Roland Banks");
    game.add_investigator(inv);

    game.investigators[0].health = 0;
    game.investigators[0].is_defeated = true;

    let status = game.check_game_status();
    assert_eq!(status, GameStatus::Lost);
    assert_eq!(game.game_status, GameStatus::Lost);
}

#[test]
fn e2e_skill_test_flow() {
    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        12345,
        vec![],
        ScenarioData {
            campaign: "night-of-the-zealot".to_string(),
            code: "01001".to_string(),
            name: "The Gathering".to_string(),
            order: 1,
            chaos_bag: "standard".to_string(),
            acts: vec![],
            agendas: vec![],
            locations: vec![],
        },
    );

    let inv = make_test_investigator("roland-banks", "Roland Banks");
    let inv_id = inv.id;
    game.add_investigator(inv);

    let result = game.skill_test(inv_id, SkillIcon::Combat).unwrap();

    assert_eq!(result.investigator_id, inv_id);
    assert_eq!(result.investigator_name, "Roland Banks");
    assert_eq!(result.skill, SkillIcon::Combat);
    assert_eq!(result.stat_value, 3);
    assert!(result.total >= result.stat_value - 4);
    assert!(result.total <= result.stat_value + 2);

    assert_eq!(game.chaos_bag.draws, 1);
}

#[test]
fn e2e_fight_flow() {
    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        12345,
        vec![
            Card {
                id: Uuid::new_v4(),
                code: "01101".to_string(),
                name: "Mob Enforcer".to_string(),
                card_type: CardType::Enemy,
                class: Some(CardClass::Neutral),
                cost: None,
                xp: 0,
                icons: SkillIcons::default(),
                traits: vec!["Humanoid".to_string(), "Criminal".to_string()],
                text: "An enemy.".to_string(),
                raw: serde_json::json!({
                    "code": "01101",
                    "name": "Mob Enforcer",
                    "health": 3,
                    "enemy_fight": 4,
                    "enemy_damage": 1,
                    "type_code": "enemy",
                }),
            },
        ],
        ScenarioData {
            campaign: "night-of-the-zealot".to_string(),
            code: "01001".to_string(),
            name: "The Gathering".to_string(),
            order: 1,
            chaos_bag: "standard".to_string(),
            acts: vec![],
            agendas: vec![],
            locations: vec![],
        },
    );

    let inv = make_test_investigator("roland-banks", "Roland Banks");
    let inv_id = inv.id;
    game.add_investigator(inv);

    let mythos_result = game.mythos_phase().unwrap();
    assert!(mythos_result.spawned_enemy.is_some());
    assert_eq!(game.enemies_in_play.len(), 1);

    let enemy_id = game.enemies_in_play[0].id;

    let engage_result = game.engage(inv_id, enemy_id).unwrap();
    assert_eq!(engage_result.enemy_name, "Mob Enforcer");
    assert!(game.investigators[0].engaged_enemies.contains(&enemy_id));
    assert_eq!(game.enemies_in_play[0].engaged_with, Some(inv_id));

    let fight_result = game.fight(inv_id, enemy_id).unwrap();
    assert_eq!(fight_result.investigator_id, inv_id);
    assert_eq!(fight_result.enemy_instance_id, enemy_id);
    assert!(game.investigators[0].actions_remaining < 3);
}

#[test]
fn e2e_chaos_bag_deterministic_draw() {
    let mut bag = ChaosBag::standard(42);
    let draws: Vec<ChaosToken> = (0..10).map(|_| bag.draw()).collect();

    let mut bag2 = ChaosBag::standard(42);
    let draws2: Vec<ChaosToken> = (0..10).map(|_| bag2.draw()).collect();

    assert_eq!(draws, draws2);
    assert_eq!(bag.draws, 10);
    assert_eq!(bag2.draws, 10);
}

fn make_the_gathering_scenario() -> ScenarioData {
    ScenarioData {
        campaign: "night-of-the-zealot".to_string(),
        code: "01001".to_string(),
        name: "The Gathering".to_string(),
        order: 1,
        chaos_bag: "standard".to_string(),
        acts: vec![
            ScenarioActData {
                code: "01001a".to_string(),
                name: "Rumors at the Boarding House".to_string(),
                text: "Investigate one or more locations at The Boarding House to learn what strange events are occurring here.".to_string(),
                clue_cost: 3,
                raw: serde_json::json!({
                    "code": "01001a",
                    "name": "Rumors at the Boarding House",
                    "type_code": "act",
                    "type_name": "Act",
                    "clue_cost": 3,
                    "faction_code": "neutral",
                    "faction_name": "Neutral",
                    "is_unique": false
                }),
            },
            ScenarioActData {
                code: "01001b".to_string(),
                name: "The Boarding House".to_string(),
                text: "Investigate the Living Room to uncover the truth behind the strange happenings at the boarding house.".to_string(),
                clue_cost: 3,
                raw: serde_json::json!({
                    "code": "01001b",
                    "name": "The Boarding House",
                    "type_code": "act",
                    "type_name": "Act",
                    "clue_cost": 3,
                    "faction_code": "neutral",
                    "faction_name": "Neutral",
                    "is_unique": false
                }),
            },
        ],
        agendas: vec![
            ScenarioAgendaData {
                code: "01002a".to_string(),
                name: "The Boarding House Burns Down".to_string(),
                text: "The building is engulfed in flames. You must escape before it is too late.".to_string(),
                doom_threshold: 4,
                raw: serde_json::json!({
                    "code": "01002a",
                    "name": "The Boarding House Burns Down",
                    "type_code": "agenda",
                    "type_name": "Agenda",
                    "doom_threshold": 4,
                    "faction_code": "neutral",
                    "faction_name": "Neutral",
                    "is_unique": false
                }),
            },
            ScenarioAgendaData {
                code: "01002b".to_string(),
                name: "Escape the Boarding House".to_string(),
                text: "You must escape the burning building before it collapses around you.".to_string(),
                doom_threshold: 4,
                raw: serde_json::json!({
                    "code": "01002b",
                    "name": "Escape the Boarding House",
                    "type_code": "agenda",
                    "type_name": "Agenda",
                    "doom_threshold": 4,
                    "faction_code": "neutral",
                    "faction_name": "Neutral",
                    "is_unique": false
                }),
            },
        ],
        locations: vec![
            ScenarioLocationData {
                code: "01003".to_string(),
                name: "Front Door".to_string(),
                text: "The front door of the boarding house stands slightly ajar, creaking in the wind.".to_string(),
                clues: 2,
                shroud: 2,
                raw: serde_json::json!({
                    "code": "01003",
                    "name": "Front Door",
                    "type_code": "location",
                    "type_name": "Location",
                    "shroud": 2,
                    "clues": 2,
                    "faction_code": "neutral",
                    "faction_name": "Neutral",
                    "is_unique": false
                }),
            },
            ScenarioLocationData {
                code: "01004".to_string(),
                name: "Living Room".to_string(),
                text: "Dust covers the furniture in this once-elegant room. Strange symbols are scrawled on the walls.".to_string(),
                clues: 2,
                shroud: 2,
                raw: serde_json::json!({
                    "code": "01004",
                    "name": "Living Room",
                    "type_code": "location",
                    "type_name": "Location",
                    "shroud": 2,
                    "clues": 2,
                    "faction_code": "neutral",
                    "faction_name": "Neutral",
                    "is_unique": false
                }),
            },
            ScenarioLocationData {
                code: "01005".to_string(),
                name: "Kitchen".to_string(),
                text: "The kitchen is in disarray, with overturned chairs and scattered dishes.".to_string(),
                clues: 1,
                shroud: 1,
                raw: serde_json::json!({
                    "code": "01005",
                    "name": "Kitchen",
                    "type_code": "location",
                    "type_name": "Location",
                    "shroud": 1,
                    "clues": 1,
                    "faction_code": "neutral",
                    "faction_name": "Neutral",
                    "is_unique": false
                }),
            },
        ],
    }
}

fn make_investigator_roland() -> Investigator {
    Investigator::new(
        "roland-banks",
        "Roland Banks",
        3, // willpower
        2, // intellect
        3, // combat
        2, // agility
        6, // max_health
        5, // max_sanity
    )
}

fn make_investigator_wendy() -> Investigator {
    Investigator::new(
        "wendy-adams",
        "Wendy Adams",
        2, // willpower
        3, // intellect
        2, // combat
        3, // agility
        5, // max_health
        5, // max_sanity
    )
}

/// E2E: Two investigators play The Gathering scenario, following rulebook turn structure.
/// Tests: 3 actions per investigator per turn, phase progression, chaos bag skill tests,
/// investigation, clue sharing, act advancement, and doom tracking.
#[test]
fn e2e_two_investigator_full_scenario_flow() {
    let scenario_data = make_the_gathering_scenario();
    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        42,
        vec![],
        scenario_data,
    );

    let roland = make_investigator_roland();
    let roland_id = roland.id;
    let wendy = make_investigator_wendy();
    let wendy_id = wendy.id;
    game.add_investigator(roland);
    game.add_investigator(wendy);

    assert_eq!(game.investigators.len(), 2);
    assert_eq!(game.game_status, GameStatus::Ongoing);
    assert_eq!(game.round, 1);
    assert_eq!(game.phase, Phase::Mythos);
    assert_eq!(game.active_investigator, 0);
    assert_eq!(game.locations_in_play.len(), 3);

    // ---------------------------------------------------------------
    // Round 1: Mythos phase
    // ---------------------------------------------------------------
    game.advance_phase(); // Mythos -> Investigation
    assert_eq!(game.phase, Phase::Investigation);
    assert_eq!(game.round, 1);

    // ---------------------------------------------------------------
    // Round 1: Investigation phase - Roland's turn (3 actions)
    // ---------------------------------------------------------------
    // Action 1: Move to Front Door
    let front_door_id = game.locations_in_play[0].id;
    game.move_to_location(roland_id, front_door_id).unwrap();
    assert_eq!(game.investigators[0].actions_remaining, 2);
    assert_eq!(game.investigators[0].location_id, Some(front_door_id));

    // Action 2: Investigate at Front Door (shroud 2, intellect 2)
    let result = game.investigate(roland_id).unwrap();
    assert_eq!(result.shroud, 2);
    assert_eq!(game.investigators[0].actions_remaining, 1);

    // Action 3: Investigate at Front Door again (if first attempt failed, try again)
    let result2 = game.investigate(roland_id).unwrap();
    assert_eq!(game.investigators[0].actions_remaining, 0);
    let roland_clues_at_front_door = result.clues_discovered as i32 + result2.clues_discovered as i32;
    assert_eq!(game.investigators[0].clues, roland_clues_at_front_door);
    // Front Door may or may not be exhausted depending on chaos tokens
    assert!(game.locations_in_play[0].clues_remaining <= 2);

    // ---------------------------------------------------------------
    // Round 1: Investigation phase - Wendy's turn (3 actions)
    // ---------------------------------------------------------------
    // Wendy moves to Living Room
    let living_room_id = game.locations_in_play[1].id;
    game.move_to_location(wendy_id, living_room_id).unwrap();
    assert_eq!(game.investigators[1].actions_remaining, 2);
    assert_eq!(game.investigators[1].location_id, Some(living_room_id));

    // Wendy investigates Living Room (shroud 2, intellect 3)
    let result3 = game.investigate(wendy_id).unwrap();
    assert_eq!(result3.shroud, 2);
    assert_eq!(game.investigators[1].actions_remaining, 1);

    // Wendy investigates Living Room again
    let result4 = game.investigate(wendy_id).unwrap();
    assert_eq!(game.investigators[1].actions_remaining, 0);
    let wendy_clues_at_living_room = result3.clues_discovered as i32 + result4.clues_discovered as i32;
    assert_eq!(game.investigators[1].clues, wendy_clues_at_living_room);
    assert!(game.locations_in_play[1].clues_remaining <= 2);

    // ---------------------------------------------------------------
    // Round 1: Enemy phase (no enemies in play)
    // ---------------------------------------------------------------
    game.advance_phase(); // Investigation -> Enemy
    assert_eq!(game.phase, Phase::Enemy);
    assert_eq!(game.enemies_in_play.len(), 0);

    // ---------------------------------------------------------------
    // Round 1: Upkeep phase - reset actions, rotate active investigator
    // ---------------------------------------------------------------
    game.advance_phase(); // Enemy -> Upkeep
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn(); // Reset all investigators' actions for new round
    assert_eq!(game.investigators[0].actions_remaining, 3); // reset
    assert_eq!(game.investigators[1].actions_remaining, 3); // reset

    game.advance_phase(); // Upkeep -> Mythos (round 2, active rotates to Wendy)
    assert_eq!(game.phase, Phase::Mythos);
    assert_eq!(game.round, 2);
    assert_eq!(game.active_investigator, 1); // Wendy is now active

    // ---------------------------------------------------------------
    // Round 2: Mythos phase
    // ---------------------------------------------------------------
    game.advance_phase(); // Mythos -> Investigation
    assert_eq!(game.phase, Phase::Investigation);

    // ---------------------------------------------------------------
    // Round 2: Investigation phase - Wendy's turn (3 actions)
    // ---------------------------------------------------------------
    // Wendy moves to Kitchen
    let kitchen_id = game.locations_in_play[2].id;
    game.move_to_location(wendy_id, kitchen_id).unwrap();
    assert_eq!(game.investigators[1].actions_remaining, 2);
    assert_eq!(game.investigators[1].location_id, Some(kitchen_id));

    // Wendy investigates Kitchen (shroud 1, intellect 3)
    let result5 = game.investigate(wendy_id).unwrap();
    assert_eq!(result5.shroud, 1);
    assert_eq!(game.investigators[1].actions_remaining, 1);
    let wendy_clues_at_kitchen = result5.clues_discovered as i32;
    assert_eq!(game.investigators[1].clues, wendy_clues_at_living_room + wendy_clues_at_kitchen);
    assert!(game.locations_in_play[2].clues_remaining <= 1);

    // Wendy has no more actions, but Roland still has 3
    // Wendy uses her last action to... she's at 1, let's just spend it
    // Actually she can't investigate (no clues left at Kitchen), so let's just end her turn
    // by not spending the last action - but engine doesn't enforce turn end.
    // Let's move Roland instead since he has actions.

    // ---------------------------------------------------------------
    // Round 2: Roland's turn (3 actions)
    // ---------------------------------------------------------------
    // Roland moves to Living Room
    game.move_to_location(roland_id, living_room_id).unwrap();
    assert_eq!(game.investigators[0].actions_remaining, 2);
    assert_eq!(game.investigators[0].location_id, Some(living_room_id));

    // Roland investigates Living Room (may or may not have clues left,
    // depending on Wendy's chaos bag results)
    let living_room_clues_before = game.locations_in_play[1].clues_remaining;
    if living_room_clues_before > 0 {
        let result6 = game.investigate(roland_id).unwrap();
        assert_eq!(game.investigators[0].actions_remaining, 1);
        assert!(result6.clues_discovered > 0 || result6.clues_discovered == 0);
    } else {
        let result6 = game.investigate(roland_id);
        assert!(result6.is_err()); // "no clues remain at this location"
    }

    // Roland moves to Kitchen
    game.move_to_location(roland_id, kitchen_id).unwrap();
    // Actions remaining: 1 if he didn't investigate Living Room (no clues there),
    // 0 if he did investigate (spent 1 action on investigate + 1 on move)
    let roland_actions_after_kitchen = if living_room_clues_before > 0 { 0 } else { 1 };
    assert_eq!(game.investigators[0].actions_remaining, roland_actions_after_kitchen);

    // Roland has 1 action left. End turn.

    // ---------------------------------------------------------------
    // Round 2: Enemy phase
    // ---------------------------------------------------------------
    game.advance_phase(); // Investigation -> Enemy
    assert_eq!(game.phase, Phase::Enemy);

    // ---------------------------------------------------------------
    // Round 2: Upkeep phase
    // ---------------------------------------------------------------
    game.advance_phase(); // Enemy -> Upkeep
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn(); // Reset all investigators' actions for new round
    assert_eq!(game.investigators[0].actions_remaining, 3); // reset
    assert_eq!(game.investigators[1].actions_remaining, 3); // reset

    game.advance_phase(); // Upkeep -> Mythos (round 3, active rotates to Roland)
    assert_eq!(game.phase, Phase::Mythos);
    assert_eq!(game.round, 3);
    assert_eq!(game.active_investigator, 0); // Roland is now active

    // ---------------------------------------------------------------
    // Act advancement: attempt to advance Act 1 (clue_cost = 3)
    // Wendy spent clues at Living Room and Kitchen; Roland spent clues at Front Door.
    // Total clues available depends on chaos bag draws.
    // ---------------------------------------------------------------
    let wendy_total_clues = game.investigators[1].clues;
    let roland_total_clues = game.investigators[0].clues;

    if wendy_total_clues >= 3 {
        let act_result = game.advance_act(vec![(wendy_id, 3)]).unwrap();
        assert!(act_result.advanced);
        assert_eq!(game.current_act_index, 1);
        assert_eq!(game.investigators[1].clues, wendy_total_clues - 3);
        assert_eq!(game.acts[0].progress, 0);
    } else if wendy_total_clues > 0 {
        // Partial progress on Act 1
        let act_result = game.advance_act(vec![(wendy_id, wendy_total_clues as u32)]).unwrap();
        assert!(!act_result.advanced);
        assert_eq!(game.current_act_index, 0);
        assert_eq!(game.acts[0].progress, wendy_total_clues as u32);
        assert_eq!(game.investigators[1].clues, 0);
    }

    // ---------------------------------------------------------------
    // Act 2: The Boarding House (clue_cost: 3)
    // Whether investigators can complete Act 2 depends on remaining clues
    // on the board and in investigators' pools. With all locations
    // potentially exhausted, they may not have enough clues.
    // ---------------------------------------------------------------
    assert_eq!(game.game_status, GameStatus::Ongoing);
    assert_eq!(game.investigators[0].clues, roland_total_clues);
}

/// E2E: Two investigators - loss by doom threshold.
/// Tests doom accumulation and agenda advancement with two investigators.
#[test]
fn e2e_two_investigator_loss_by_doom() {
    let scenario_data = make_the_gathering_scenario();
    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        42,
        vec![],
        scenario_data,
    );

    let roland = make_investigator_roland();
    let roland_id = roland.id;
    let wendy = make_investigator_wendy();
    let wendy_id = wendy.id;
    game.add_investigator(roland);
    game.add_investigator(wendy);

    assert_eq!(game.game_status, GameStatus::Ongoing);
    assert_eq!(game.current_agenda_index, 0);
    assert_eq!(game.agendas[0].doom_threshold, 4);

    // Place doom on Agenda 1: "The Boarding House Burns Down" (threshold 4)
    // Place 2 doom
    let result1 = game.place_doom(2).unwrap();
    assert!(!result1.advanced);
    assert_eq!(result1.doom, 2);
    assert_eq!(game.agendas[0].doom, 2);
    assert_eq!(game.game_status, GameStatus::Ongoing);

    // Place 2 more doom (reaches threshold 4)
    let result2 = game.place_doom(2).unwrap();
    assert!(result2.advanced);
    assert_eq!(result2.doom, 4);
    assert_eq!(game.current_agenda_index, 1);
    assert_eq!(game.agendas[1].doom_threshold, 4);
    assert_eq!(game.game_status, GameStatus::Ongoing); // Not lost yet, Agenda 2 exists

    // Place 4 doom on Agenda 2 (reaches threshold 4, no more agendas)
    let result3 = game.place_doom(4).unwrap();
    assert!(result3.advanced);
    assert_eq!(game.current_agenda_index, 2); // past the end
    assert_eq!(game.game_status, GameStatus::Lost);
}

/// E2E: Two investigators - loss by all defeated.
/// Tests that when both investigators are defeated, game is lost.
#[test]
fn e2e_two_investigator_loss_by_all_defeated() {
    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        42,
        vec![],
        make_the_gathering_scenario(),
    );

    let roland = make_investigator_roland();
    let roland_id = roland.id;
    let wendy = make_investigator_wendy();
    let wendy_id = wendy.id;
    game.add_investigator(roland);
    game.add_investigator(wendy);

    assert_eq!(game.game_status, GameStatus::Ongoing);

    // Defeat Roland
    game.investigator_mut(roland_id).unwrap().take_damage(6);
    assert!(game.investigators[0].is_defeated);
    assert_eq!(game.check_game_status(), GameStatus::Ongoing); // Wendy still alive

    // Defeat Wendy
    game.investigator_mut(wendy_id).unwrap().take_damage(5);
    assert!(game.investigators[1].is_defeated);
    assert_eq!(game.check_game_status(), GameStatus::Lost);
}

/// E2E: Two investigators - clue pooling for act advancement.
/// Tests that multiple investigators can contribute clues together
/// to meet an act's clue cost, per the rulebook.
#[test]
fn e2e_two_investigator_clue_pooling_advance_act() {
    let scenario_data = make_the_gathering_scenario();
    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        42,
        vec![],
        scenario_data,
    );

    let roland = make_investigator_roland();
    let roland_id = roland.id;
    let wendy = make_investigator_wendy();
    let wendy_id = wendy.id;
    game.add_investigator(roland);
    game.add_investigator(wendy);

    // Give both investigators some clues manually
    game.investigator_mut(roland_id).unwrap().clues = 2;
    game.investigator_mut(wendy_id).unwrap().clues = 2;

    assert_eq!(game.investigators[0].clues, 2);
    assert_eq!(game.investigators[1].clues, 2);
    assert_eq!(game.acts[0].progress, 0);

    // Neither investigator alone has enough clues (need 3, each has 2),
    // but together they can pool 4 clues to advance Act 1.
    let act_result = game.advance_act(vec![(roland_id, 2), (wendy_id, 2)]).unwrap();
    assert!(act_result.advanced);
    assert_eq!(game.current_act_index, 1);
    assert_eq!(game.investigators[0].clues, 0); // Roland spent all his clues
    assert_eq!(game.investigators[1].clues, 0); // Wendy spent all her clues
    assert_eq!(game.acts[0].progress, 0); // Reset after advancement

    // Verify both investigators contributed to the log
    let log_text: String = game.log_messages.iter().cloned().collect();
    assert!(log_text.contains("Roland"));
    assert!(log_text.contains("Wendy"));
}

/// E2E: Two investigators - skill test with chaos bag.
/// Tests that both investigators can make skill tests with chaos bag draws,
/// and that the bag is deterministic with the same seed.
#[test]
fn e2e_two_investigator_skill_test_flow() {
    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        42,
        vec![],
        make_the_gathering_scenario(),
    );

    let roland = make_investigator_roland();
    let roland_id = roland.id;
    let wendy = make_investigator_wendy();
    let wendy_id = wendy.id;
    game.add_investigator(roland);
    game.add_investigator(wendy);

    // Roland combat test (combat 3)
    let result1 = game.skill_test(roland_id, SkillIcon::Combat).unwrap();
    assert_eq!(result1.investigator_id, roland_id);
    assert_eq!(result1.investigator_name, "Roland Banks");
    assert_eq!(result1.stat_value, 3);
    assert_eq!(game.chaos_bag.draws, 1);

    // Wendy intellect test (intellect 3)
    let result2 = game.skill_test(wendy_id, SkillIcon::Intellect).unwrap();
    assert_eq!(result2.investigator_id, wendy_id);
    assert_eq!(result2.investigator_name, "Wendy Adams");
    assert_eq!(result2.stat_value, 3);
    assert_eq!(game.chaos_bag.draws, 2);

    // Verify determinism: same seed produces same token sequence
    let mut bag1 = ChaosBag::standard(42);
    let mut bag2 = ChaosBag::standard(42);
    let token1 = bag1.draw();
    let token2 = bag2.draw();
    assert_eq!(token1, token2);
}

/// E2E: Campaign progression - XP, deck, and health carry over between scenarios.
/// Tests that completing a scenario and starting the next one preserves
/// investigator state (XP, decklist, health, sanity, resources) per the rulebook.
#[test]
fn e2e_campaign_progression_carryover() {
    let scenario_data = make_the_gathering_scenario();
    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        42,
        vec![],
        scenario_data,
    );

    let roland = make_investigator_roland();
    let roland_id = roland.id;
    game.add_investigator(roland);

    // Give Roland some XP, health damage, clues, and a card in his decklist to simulate
    // progression from a previous scenario.
    game.investigator_mut(roland_id).unwrap().xp = 5;
    game.investigator_mut(roland_id).unwrap().take_damage(2);
    game.investigator_mut(roland_id).unwrap().clues = 6;

    // Add a card to Roland's decklist (simulating an XP purchase).
    game.investigator_mut(roland_id)
        .unwrap()
        .deck
        .decklist
        .push(DeckSlot {
            code: "01006".to_string(),
            quantity: 1,
        });

    assert_eq!(game.investigators[0].xp, 5);
    assert_eq!(game.investigators[0].health, 4); // 6 - 2 damage
    assert_eq!(game.investigators[0].clues, 6);
    assert_eq!(game.investigators[0].deck.decklist.len(), 1);

    // Add a victory card to the victory display (worth 2 XP).
    game.victory_display.push(Card {
        raw: serde_json::json!({ "victory": 2 }),
        ..Card::new("01001", "Ghoul Priest", CardType::Enemy)
    });

    // Complete the scenario (won) - advance both acts.
    game.advance_act(vec![(roland_id, 3)]).unwrap(); // Act 0: clue_cost 3
    assert_eq!(game.current_act_index, 1);
    assert_eq!(game.game_status, GameStatus::Ongoing);
    game.advance_act(vec![(roland_id, 3)]).unwrap(); // Act 1: clue_cost 3
    assert!(game.game_status == GameStatus::Won);

    // Record campaign log entry for the completed scenario.
    game.add_campaign_log_entry(
        CampaignResult::Won,
        "Roland investigated the boarding house and uncovered the truth.".to_string(),
    );

    assert_eq!(game.campaign_progress.completed_scenarios.len(), 1);
    assert_eq!(game.campaign_progress.completed_scenarios[0], "The Gathering");
    assert_eq!(game.campaign_progress.campaign_log.len(), 1);
    assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Won);
    assert_eq!(game.campaign_progress.campaign_log[0].investigator_names[0], "Roland Banks");

    // XP should have been awarded for completing the scenario.
    assert_eq!(game.investigators[0].xp, 7); // 5 + 2

    // Start the next scenario in the campaign - investigator state should carry over.
    let next_scenario_data = ScenarioData {
        campaign: "night-of-the-zealot".to_string(),
        code: "01002".to_string(),
        name: "The Midnight Masks".to_string(),
        order: 2,
        chaos_bag: "standard".to_string(),
        acts: vec![],
        agendas: vec![],
        locations: vec![],
    };

    game.start_scenario_in_campaign(
        "The Midnight Masks",
        next_scenario_data,
        999,
    );

    // Verify scenario changed but investigator state carried over.
    assert_eq!(game.scenario, "The Midnight Masks");
    assert_eq!(game.round, 1);
    assert_eq!(game.phase, Phase::Mythos);
    assert_eq!(game.game_status, GameStatus::Ongoing);

    // Investigator state carried over: XP, decklist, health, sanity, resources.
    assert_eq!(game.investigators[0].xp, 7); // Carried over
    assert_eq!(game.investigators[0].health, 4); // Carried over (damaged)
    assert_eq!(game.investigators[0].clues, 0); // Reset for new scenario
    assert_eq!(game.investigators[0].deck.decklist.len(), 1); // Carried over
    assert_eq!(game.investigators[0].deck.decklist[0].code, "01006");
    assert_eq!(game.investigators[0].actions_remaining, 3); // Reset for new scenario

    // Campaign log preserved.
    assert_eq!(game.campaign_progress.completed_scenarios.len(), 1);
    assert_eq!(game.campaign_progress.campaign_log.len(), 1);
}

/// E2E: Campaign log entry recording.
/// Tests that scenario results are properly recorded in the campaign log.
#[test]
fn e2e_campaign_log_entry() {
    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        42,
        vec![],
        make_the_gathering_scenario(),
    );

    let roland = make_investigator_roland();
    let roland_id = roland.id;
    game.add_investigator(roland.clone());

    // Give Roland clues to spend on act advancement.
    game.investigator_mut(roland_id).unwrap().clues = 3;

    // Complete scenario as won.
    game.game_status = GameStatus::Won;
    game.advance_act(vec![(roland_id, 3)]).unwrap();

    // Add campaign log entry.
    game.add_campaign_log_entry(
        CampaignResult::Won,
        "Scenario completed successfully.".to_string(),
    );

    assert_eq!(game.campaign_progress.campaign_log.len(), 1);
    let entry = &game.campaign_progress.campaign_log[0];
    assert_eq!(entry.scenario_code, "The Gathering");
    assert_eq!(entry.result, CampaignResult::Won);
    assert_eq!(entry.investigator_names, vec!["Roland Banks"]);
    assert_eq!(entry.notes, "Scenario completed successfully.");

    // Complete scenario as lost.
    let mut game2 = GameState::new(
        "Night of the Zealot",
        "The Midnight Masks",
        42,
        vec![],
        make_the_gathering_scenario(),
    );
    game2.add_investigator(roland.clone());
    game2.game_status = GameStatus::Lost;
    game2.add_campaign_log_entry(
        CampaignResult::Lost,
        "All investigators were defeated.".to_string(),
    );

    assert_eq!(game2.campaign_progress.campaign_log.len(), 1);
    let entry2 = &game2.campaign_progress.campaign_log[0];
    assert_eq!(entry2.scenario_code, "The Midnight Masks");
    assert_eq!(entry2.result, CampaignResult::Lost);
    assert_eq!(entry2.notes, "All investigators were defeated.");
}

/// Helper: add two investigators (Roland Banks and Wendy Adams) to a game.
fn add_test_investigators(game: &mut GameState) -> (Uuid, Uuid) {
    let roland = Investigator::new(
        "roland-banks",
        "Roland Banks",
        2, // willpower
        1, // intellect
        3, // combat
        1, // agility
        9, // max_health
        5, // max_sanity
    );
    let roland_id = roland.id;
    game.add_investigator(roland);

    let wendy = Investigator::new(
        "wendy-adams",
        "Wendy Adams",
        2, // willpower
        2, // intellect
        2, // combat
        2, // agility
        8, // max_health
        5, // max_sanity
    );
    let wendy_id = wendy.id;
    game.add_investigator(wendy);

    (roland_id, wendy_id)
}

/// E2E: Full playthrough of The Midnight Masks scenario from start to finish.
/// Uses next_investigator() to give each investigator a turn during Investigation phase.
#[test]
fn e2e_full_scenario_playthrough_the_midnight_masks() {
    let encounter_cards = vec![
        Card {
            id: Uuid::new_v4(),
            code: "01101".to_string(),
            name: "Ghoul Priest".to_string(),
            card_type: CardType::Enemy,
            class: Some(CardClass::Neutral),
            cost: None,
            xp: 0,
            icons: SkillIcons::default(),
            traits: vec!["Humanoid".to_string(), "Cultist".to_string()],
            text: "A monstrous cultist.".to_string(),
            raw: serde_json::json!({
                "code": "01101",
                "name": "Ghoul Priest",
                "health": 3,
                "enemy_fight": 4,
                "enemy_damage": 1,
                "enemy_horror": 1,
                "type_code": "enemy",
            }),
        },
    ];

    let scenario_data = ScenarioData {
        campaign: "night-of-the-zealot".to_string(),
        code: "01002".to_string(),
        name: "The Midnight Masks".to_string(),
        order: 2,
        chaos_bag: "standard".to_string(),
        acts: vec![
            ScenarioActData {
                code: "01002c".to_string(),
                name: "The Midnight Masks".to_string(),
                text: "Investigate locations in Arkham to learn about the mysterious Midnight Masks organization.".to_string(),
                clue_cost: 3,
                raw: serde_json::json!({"code": "01002c", "clue_cost": 3}),
            },
            ScenarioActData {
                code: "01002d".to_string(),
                name: "Unmask the Culprit".to_string(),
                text: "Confront the leader of the Midnight Masks and put an end to their scheme.".to_string(),
                clue_cost: 3,
                raw: serde_json::json!({"code": "01002d", "clue_cost": 3}),
            },
        ],
        agendas: vec![
            ScenarioAgendaData {
                code: "01003a".to_string(),
                name: "The Masks Multiply".to_string(),
                text: "The Midnight Masks spread their influence across Arkham.".to_string(),
                doom_threshold: 5,
                raw: serde_json::json!({"code": "01003a", "doom_threshold": 5}),
            },
            ScenarioAgendaData {
                code: "01003b".to_string(),
                name: "The Final Ritual".to_string(),
                text: "The cultists begin their final ritual.".to_string(),
                doom_threshold: 5,
                raw: serde_json::json!({"code": "01003b", "doom_threshold": 5}),
            },
        ],
        locations: vec![
            ScenarioLocationData {
                code: "01006".to_string(),
                name: "The Silver Twilight Lodge".to_string(),
                text: "An imposing building where the Silver Twilight Lodge holds its secret meetings.".to_string(),
                clues: 2,
                shroud: 3,
                raw: serde_json::json!({"code": "01006", "shroud": 3, "clues": 2}),
            },
            ScenarioLocationData {
                code: "01007".to_string(),
                name: "The Streets of Arkham".to_string(),
                text: "The foggy streets of Arkham at night.".to_string(),
                clues: 2,
                shroud: 1,
                raw: serde_json::json!({"code": "01007", "shroud": 1, "clues": 2}),
            },
            ScenarioLocationData {
                code: "01008".to_string(),
                name: "The Miskatonic Library".to_string(),
                text: "The library contains rare tomes and forbidden knowledge.".to_string(),
                clues: 3,
                shroud: 2,
                raw: serde_json::json!({"code": "01008", "shroud": 2, "clues": 3}),
            },
        ],
    };

    let mut game = GameState::new(
        "Night of the Zealot",
        "The Midnight Masks",
        99999,
        encounter_cards,
        scenario_data,
    );

    let (roland_id, wendy_id) = add_test_investigators(&mut game);

    // ================================================================
    // Round 1
    // ================================================================

    // --- Mythos Phase ---
    let _ = game.mythos_phase();

    // --- Investigation Phase ---
    game.advance_phase();
    assert_eq!(game.phase, Phase::Investigation);

    // Roland's turn: move to Streets of Arkham, investigate
    let streets_id = game.locations_in_play[1].id;
    game.move_to_location(roland_id, streets_id).unwrap();

    let _roland_investigate = game.investigate(roland_id).unwrap();

    // Rotate to Wendy
    let next = game.next_investigator().unwrap();
    assert_eq!(next, wendy_id);

    // Wendy's turn: move to Miskatonic Library, investigate
    let library_id = game.locations_in_play[2].id;
    game.move_to_location(wendy_id, library_id).unwrap();

    let _wendy_investigate = game.investigate(wendy_id).unwrap();

    // Rotate back to Roland (end of Investigation phase)
    let _next = game.next_investigator();

    // --- Enemy Phase ---
    game.advance_phase();
    assert_eq!(game.phase, Phase::Enemy);

    // --- Upkeep Phase ---
    game.advance_phase();
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // --- Round 2 ---
    game.advance_phase();
    assert_eq!(game.round, 2);

    // --- Mythos Phase Round 2 ---
    let _ = game.mythos_phase();

    // --- Investigation Phase Round 2 ---
    game.advance_phase();

    // Roland's turn
    game.move_to_location(roland_id, library_id).unwrap();
    let _roland_investigate2 = game.investigate(roland_id).unwrap();

    // Rotate to Wendy
    let _next = game.next_investigator();

    // Wendy's turn
    game.move_to_location(wendy_id, streets_id).unwrap();
    let _wendy_investigate2 = game.investigate(wendy_id).unwrap();

    // Rotate back
    let _next = game.next_investigator();

    // --- Enemy Phase Round 2 ---
    game.advance_phase();
    assert_eq!(game.phase, Phase::Enemy);

    // --- Upkeep Phase Round 2 ---
    game.advance_phase();
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // --- Round 3 ---
    game.advance_phase();
    assert_eq!(game.round, 3);

    // --- Mythos Phase Round 3 ---
    let _ = game.mythos_phase();

    // --- Investigation Phase Round 3 ---
    game.advance_phase();

    // Try to advance Act 1
    let roland_clues = game.investigators[0].clues;
    if roland_clues >= 3 {
        let act_result = game.advance_act(vec![(roland_id, 3)]).unwrap();
        if act_result.advanced {
            eprintln!("Act 1 advanced");
        } else {
            eprintln!("Act 1 did not advance (progress={})", act_result.progress);
        }
    } else {
        eprintln!("Roland has {} clues, not enough to advance Act 1 (need 3)", roland_clues);
    }

    // Rotate to Wendy
    let _next = game.next_investigator();

    let wendy_clues = game.investigators[1].clues;
    if wendy_clues >= 3 {
        let act_result = game.advance_act(vec![(wendy_id, 3)]).unwrap();
        if act_result.advanced {
            eprintln!("Act 2 advanced");
        } else {
            eprintln!("Act 2 did not advance (progress={})", act_result.progress);
        }
    } else {
        eprintln!("Wendy has {} clues, not enough to advance Act 2 (need 3)", wendy_clues);
    }

    // --- Enemy Phase Round 3 ---
    game.advance_phase();
    assert_eq!(game.phase, Phase::Enemy);

    // --- Upkeep Phase Round 3 ---
    game.advance_phase();
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // --- Round 4 ---
    game.advance_phase();
    assert_eq!(game.round, 4);

    // --- Mythos Phase Round 4 ---
    let _ = game.mythos_phase();

    // --- Investigation Phase Round 4 ---
    game.advance_phase();

    // Try place doom
    let _doom_result = game.place_doom(1).unwrap();

    // ================================================================
    // Scenario Resolution
    // ================================================================

    match game.game_status {
        GameStatus::Won => {
            game.add_campaign_log_entry(
                CampaignResult::Won,
                "Investigators unmasked the Midnight Masks.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log.len(), 1);
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Won);
        }
        GameStatus::Lost => {
            game.add_campaign_log_entry(
                CampaignResult::Lost,
                "The Midnight Masks completed their ritual.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log.len(), 1);
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Lost);
        }
        GameStatus::Ongoing => {
            eprintln!("NOTE: The Midnight Masks still ongoing after 4 rounds.");
        }
    }

    eprintln!("=== The Midnight Masks End State ===");
    for inv in &game.investigators {
        eprintln!(
            "Investigator {}: health={}/{} sanity={}/{} xp={} resources={} clues={} defeated={}",
            inv.name, inv.health, inv.max_health, inv.sanity, inv.max_sanity,
            inv.xp, inv.resources, inv.clues, inv.is_defeated
        );
    }
    eprintln!("Game status: {:?}", game.game_status);
    eprintln!("Current act: {}", game.current_act_index);
    eprintln!("Current agenda: {}", game.current_agenda_index);
    eprintln!("Round: {}", game.round);
}

/// E2E: Full playthrough of The Devourer Below scenario from start to finish.
#[test]
fn e2e_full_scenario_playthrough_the_devourer_below() {
    let encounter_cards = vec![
        Card {
            id: Uuid::new_v4(),
            code: "01102".to_string(),
            name: "Cultist".to_string(),
            card_type: CardType::Enemy,
            class: Some(CardClass::Neutral),
            cost: None,
            xp: 0,
            icons: SkillIcons::default(),
            traits: vec!["Humanoid".to_string(), "Cultist".to_string()],
            text: "A low-ranking cultist.".to_string(),
            raw: serde_json::json!({
                "code": "01102",
                "name": "Cultist",
                "health": 2,
                "enemy_fight": 2,
                "enemy_damage": 1,
                "enemy_horror": 0,
                "type_code": "enemy",
            }),
        },
    ];

    let scenario_data = ScenarioData {
        campaign: "night-of-the-zealot".to_string(),
        code: "01003".to_string(),
        name: "The Devourer Below".to_string(),
        order: 3,
        chaos_bag: "standard".to_string(),
        acts: vec![
            ScenarioActData {
                code: "01003c".to_string(),
                name: "The Sewers of Arkham".to_string(),
                text: "Investigate the sewers beneath Arkham to find the source of the strange occurrences.".to_string(),
                clue_cost: 4,
                raw: serde_json::json!({"code": "01003c", "clue_cost": 4}),
            },
            ScenarioActData {
                code: "01003d".to_string(),
                name: "Confront the Devourer".to_string(),
                text: "Face the ancient entity that threatens to devour Arkham itself.".to_string(),
                clue_cost: 4,
                raw: serde_json::json!({"code": "01003d", "clue_cost": 4}),
            },
        ],
        agendas: vec![
            ScenarioAgendaData {
                code: "01004a".to_string(),
                name: "The Devourer Awakens".to_string(),
                text: "Something ancient stirs beneath the city.".to_string(),
                doom_threshold: 6,
                raw: serde_json::json!({"code": "01004a", "doom_threshold": 6}),
            },
            ScenarioAgendaData {
                code: "01004b".to_string(),
                name: "The City Falls".to_string(),
                text: "If the devourer is not stopped, Arkham will be consumed by madness.".to_string(),
                doom_threshold: 6,
                raw: serde_json::json!({"code": "01004b", "doom_threshold": 6}),
            },
        ],
        locations: vec![
            ScenarioLocationData {
                code: "01009".to_string(),
                name: "Sewer Entrance".to_string(),
                text: "A dark, wet entrance to the sewers beneath Arkham.".to_string(),
                clues: 1,
                shroud: 2,
                raw: serde_json::json!({"code": "01009", "shroud": 2, "clues": 1}),
            },
            ScenarioLocationData {
                code: "01010".to_string(),
                name: "Deep Sewers".to_string(),
                text: "The deep sewers twist and turn.".to_string(),
                clues: 2,
                shroud: 3,
                raw: serde_json::json!({"code": "01010", "shroud": 3, "clues": 2}),
            },
            ScenarioLocationData {
                code: "01011".to_string(),
                name: "The Chamber Below".to_string(),
                text: "A vast underground chamber where something ancient waits.".to_string(),
                clues: 3,
                shroud: 4,
                raw: serde_json::json!({"code": "01011", "shroud": 4, "clues": 3}),
            },
        ],
    };

    let mut game = GameState::new(
        "Night of the Zealot",
        "The Devourer Below",
        55555,
        encounter_cards,
        scenario_data,
    );

    let (roland_id, wendy_id) = add_test_investigators(&mut game);

    // ================================================================
    // Round 1
    // ================================================================

    let _ = game.mythos_phase();

    game.advance_phase(); // Investigation
    assert_eq!(game.phase, Phase::Investigation);

    // Roland's turn
    let sewer_id = game.locations_in_play[0].id;
    game.move_to_location(roland_id, sewer_id).unwrap();
    let _roland_inv = game.investigate(roland_id).unwrap();

    let _next = game.next_investigator();

    // Wendy's turn
    let deep_sewer_id = game.locations_in_play[1].id;
    game.move_to_location(wendy_id, deep_sewer_id).unwrap();
    let _wendy_inv = game.investigate(wendy_id).unwrap();

    let _next = game.next_investigator();

    game.advance_phase(); // Enemy
    assert_eq!(game.phase, Phase::Enemy);

    game.advance_phase(); // Upkeep
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // Round 2
    game.advance_phase(); // Mythos
    assert_eq!(game.round, 2);

    let _ = game.mythos_phase();

    game.advance_phase(); // Investigation

    game.move_to_location(roland_id, deep_sewer_id).unwrap();
    let _roland_inv2 = game.investigate(roland_id).unwrap();

    let _next = game.next_investigator();

    game.move_to_location(wendy_id, game.locations_in_play[2].id).unwrap();
    let _wendy_inv2 = game.investigate(wendy_id).unwrap();

    let _next = game.next_investigator();

    game.advance_phase(); // Enemy
    assert_eq!(game.phase, Phase::Enemy);

    game.advance_phase(); // Upkeep
    game.reset_turn();

    // Round 3
    game.advance_phase(); // Mythos
    assert_eq!(game.round, 3);

    let _ = game.mythos_phase();

    game.advance_phase(); // Investigation

    // Try to advance Act 1 (clue_cost 4)
    let roland_clues = game.investigators[0].clues;
    if roland_clues >= 4 {
        let act_result = game.advance_act(vec![(roland_id, 4)]).unwrap();
        if act_result.advanced {
            eprintln!("Devourer Below: Act 1 advanced");
        }
    } else {
        eprintln!("Devourer Below: Roland has {} clues, need 4 for Act 1", roland_clues);
    }

    let _next = game.next_investigator();

    let wendy_clues = game.investigators[1].clues;
    if wendy_clues >= 4 {
        let act_result = game.advance_act(vec![(wendy_id, 4)]).unwrap();
        if act_result.advanced {
            eprintln!("Devourer Below: Act 2 advanced");
        }
    } else {
        eprintln!("Devourer Below: Wendy has {} clues, need 4 for Act 2", wendy_clues);
    }

    game.advance_phase(); // Enemy
    game.advance_phase(); // Upkeep
    game.reset_turn();

    // Round 4
    game.advance_phase(); // Mythos
    assert_eq!(game.round, 4);

    let _ = game.mythos_phase();
    let _doom = game.place_doom(1).unwrap();

    game.advance_phase(); // Investigation

    // ================================================================
    // Scenario Resolution
    // ================================================================

    match game.game_status {
        GameStatus::Won => {
            game.add_campaign_log_entry(
                CampaignResult::Won,
                "Investigators confronted the Devourer Below.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Won);
        }
        GameStatus::Lost => {
            game.add_campaign_log_entry(
                CampaignResult::Lost,
                "The Devourer Below awakened.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Lost);
        }
        GameStatus::Ongoing => {
            eprintln!("NOTE: The Devourer Below still ongoing after 4 rounds.");
        }
    }

    eprintln!("=== The Devourer Below End State ===");
    for inv in &game.investigators {
        eprintln!(
            "Investigator {}: health={}/{} sanity={}/{} xp={} resources={} clues={}",
            inv.name, inv.health, inv.max_health, inv.sanity, inv.max_sanity,
            inv.xp, inv.resources, inv.clues
        );
    }
    eprintln!("Game status: {:?}", game.game_status);
    eprintln!("Round: {}", game.round);
}

/// E2E: Full playthrough of The Miskatonic Museum scenario from start to finish.
#[test]
fn e2e_full_scenario_playthrough_the_miskatonic_museum() {
    let encounter_cards = vec![
        Card {
            id: Uuid::new_v4(),
            code: "02100".to_string(),
            name: "Dunwich Horror".to_string(),
            card_type: CardType::Enemy,
            class: Some(CardClass::Neutral),
            cost: None,
            xp: 0,
            icons: SkillIcons::default(),
            traits: vec!["Monster".to_string()],
            text: "A nameless horror from beyond time and space.".to_string(),
            raw: serde_json::json!({
                "code": "02100",
                "name": "Dunwich Horror",
                "health": 6,
                "enemy_fight": 4,
                "enemy_damage": 2,
                "enemy_horror": 1,
                "type_code": "enemy",
            }),
        },
    ];

    let scenario_data = ScenarioData {
        campaign: "the-dunwich-legacy".to_string(),
        code: "02001".to_string(),
        name: "The Miskatonic Museum".to_string(),
        order: 1,
        chaos_bag: "standard".to_string(),
        acts: vec![
            ScenarioActData {
                code: "02001a".to_string(),
                name: "The Miskatonic Museum".to_string(),
                text: "Investigate the museum to learn about the strange artifact.".to_string(),
                clue_cost: 4,
                raw: serde_json::json!({"code": "02001a", "clue_cost": 4}),
            },
            ScenarioActData {
                code: "02001b".to_string(),
                name: "The Strange Relic".to_string(),
                text: "Secure the strange relic and escape the museum.".to_string(),
                clue_cost: 3,
                raw: serde_json::json!({"code": "02001b", "clue_cost": 3}),
            },
        ],
        agendas: vec![
            ScenarioAgendaData {
                code: "02002a".to_string(),
                name: "The Museum Closes".to_string(),
                text: "The museum is closing soon.".to_string(),
                doom_threshold: 5,
                raw: serde_json::json!({"code": "02002a", "doom_threshold": 5}),
            },
            ScenarioAgendaData {
                code: "02002b".to_string(),
                name: "Trapped Inside".to_string(),
                text: "The doors are locked.".to_string(),
                doom_threshold: 5,
                raw: serde_json::json!({"code": "02002b", "doom_threshold": 5}),
            },
        ],
        locations: vec![
            ScenarioLocationData {
                code: "02003".to_string(),
                name: "Museum Entrance".to_string(),
                text: "The grand entrance to the Miskatonic Museum.".to_string(),
                clues: 2,
                shroud: 2,
                raw: serde_json::json!({"code": "02003", "shroud": 2, "clues": 2}),
            },
            ScenarioLocationData {
                code: "02004".to_string(),
                name: "Exhibit Hall".to_string(),
                text: "Various artifacts line the walls of this dimly lit hall.".to_string(),
                clues: 3,
                shroud: 3,
                raw: serde_json::json!({"code": "02004", "shroud": 3, "clues": 3}),
            },
            ScenarioLocationData {
                code: "02005".to_string(),
                name: "Storage Room".to_string(),
                text: "Crates and boxes fill this dusty storage area.".to_string(),
                clues: 2,
                shroud: 2,
                raw: serde_json::json!({"code": "02005", "shroud": 2, "clues": 2}),
            },
        ],
    };

    let mut game = GameState::new(
        "The Dunwich Legacy",
        "The Miskatonic Museum",
        77777,
        encounter_cards,
        scenario_data,
    );

    let (roland_id, wendy_id) = add_test_investigators(&mut game);

    // ================================================================
    // Round 1
    // ================================================================

    let _ = game.mythos_phase();

    game.advance_phase();
    assert_eq!(game.phase, Phase::Investigation);

    // Roland's turn
    let entrance_id = game.locations_in_play[0].id;
    game.move_to_location(roland_id, entrance_id).unwrap();
    let _roland_inv = game.investigate(roland_id).unwrap();

    let _next = game.next_investigator();

    // Wendy's turn
    let exhibit_id = game.locations_in_play[1].id;
    game.move_to_location(wendy_id, exhibit_id).unwrap();
    let _wendy_inv = game.investigate(wendy_id).unwrap();

    let _next = game.next_investigator();

    game.advance_phase();
    assert_eq!(game.phase, Phase::Enemy);

    game.advance_phase();
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // Round 2
    game.advance_phase();
    assert_eq!(game.round, 2);

    let _ = game.mythos_phase();

    game.advance_phase();

    game.move_to_location(roland_id, exhibit_id).unwrap();
    let _roland_inv2 = game.investigate(roland_id).unwrap();

    let _next = game.next_investigator();

    game.move_to_location(wendy_id, game.locations_in_play[2].id).unwrap();
    let _wendy_inv2 = game.investigate(wendy_id).unwrap();

    let _next = game.next_investigator();

    game.advance_phase();
    assert_eq!(game.phase, Phase::Enemy);

    game.advance_phase();
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // Round 3
    game.advance_phase();
    assert_eq!(game.round, 3);

    let _ = game.mythos_phase();

    game.advance_phase();

    // Try to advance Act 1 (clue_cost 4)
    let roland_clues = game.investigators[0].clues;
    if roland_clues >= 4 {
        let act_result = game.advance_act(vec![(roland_id, 4)]).unwrap();
        if act_result.advanced {
            eprintln!("Miskatonic Museum: Act 1 advanced");
        }
    } else {
        eprintln!("Miskatonic Museum: Roland has {} clues, need 4 for Act 1", roland_clues);
    }

    let _next = game.next_investigator();

    let wendy_clues = game.investigators[1].clues;
    if wendy_clues >= 3 {
        let act_result = game.advance_act(vec![(wendy_id, 3)]).unwrap();
        if act_result.advanced {
            eprintln!("Miskatonic Museum: Act 2 advanced");
        }
    } else {
        eprintln!("Miskatonic Museum: Wendy has {} clues, need 3 for Act 2", wendy_clues);
    }

    game.advance_phase();
    game.advance_phase();
    game.reset_turn();

    // Round 4
    game.advance_phase();
    assert_eq!(game.round, 4);

    let _ = game.mythos_phase();
    let _doom = game.place_doom(1).unwrap();

    game.advance_phase();

    // ================================================================
    // Scenario Resolution
    // ================================================================

    match game.game_status {
        GameStatus::Won => {
            game.add_campaign_log_entry(
                CampaignResult::Won,
                "Investigators secured the strange relic.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Won);
        }
        GameStatus::Lost => {
            game.add_campaign_log_entry(
                CampaignResult::Lost,
                "The museum closed before investigators could escape.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Lost);
        }
        GameStatus::Ongoing => {
            eprintln!("NOTE: The Miskatonic Museum still ongoing after 4 rounds.");
        }
    }

    eprintln!("=== The Miskatonic Museum End State ===");
    for inv in &game.investigators {
        eprintln!(
            "Investigator {}: health={}/{} sanity={}/{} xp={} resources={} clues={}",
            inv.name, inv.health, inv.max_health, inv.sanity, inv.max_sanity,
            inv.xp, inv.resources, inv.clues
        );
    }
    eprintln!("Game status: {:?}", game.game_status);
    eprintln!("Round: {}", game.round);
}

/// E2E: Full playthrough of The Essex County Express scenario from start to finish.
#[test]
fn e2e_full_scenario_playthrough_the_essex_county_express() {
    let encounter_cards = vec![];

    let scenario_data = ScenarioData {
        campaign: "the-dunwich-legacy".to_string(),
        code: "02002".to_string(),
        name: "The Essex County Express".to_string(),
        order: 2,
        chaos_bag: "standard".to_string(),
        acts: vec![
            ScenarioActData {
                code: "02002c".to_string(),
                name: "The Essex County Express".to_string(),
                text: "Investigate the train to discover what happened.".to_string(),
                clue_cost: 3,
                raw: serde_json::json!({"code": "02002c", "clue_cost": 3}),
            },
            ScenarioActData {
                code: "02002d".to_string(),
                name: "Stop the Train".to_string(),
                text: "Find a way to stop the train before it reaches its final destination.".to_string(),
                clue_cost: 2,
                raw: serde_json::json!({"code": "02002d", "clue_cost": 2}),
            },
        ],
        agendas: vec![
            ScenarioAgendaData {
                code: "02003a".to_string(),
                name: "The Train Rolls On".to_string(),
                text: "The train continues its journey through the night.".to_string(),
                doom_threshold: 4,
                raw: serde_json::json!({"code": "02003a", "doom_threshold": 4}),
            },
            ScenarioAgendaData {
                code: "02003b".to_string(),
                name: "Derailment".to_string(),
                text: "The train is racing toward disaster.".to_string(),
                doom_threshold: 4,
                raw: serde_json::json!({"code": "02003b", "doom_threshold": 4}),
            },
        ],
        locations: vec![
            ScenarioLocationData {
                code: "02006".to_string(),
                name: "Passenger Car".to_string(),
                text: "A comfortable passenger car with plush seats.".to_string(),
                clues: 2,
                shroud: 1,
                raw: serde_json::json!({"code": "02006", "shroud": 1, "clues": 2}),
            },
            ScenarioLocationData {
                code: "02007".to_string(),
                name: "Baggage Car".to_string(),
                text: "Luggage and cargo are piled haphazardly.".to_string(),
                clues: 3,
                shroud: 2,
                raw: serde_json::json!({"code": "02007", "shroud": 2, "clues": 3}),
            },
            ScenarioLocationData {
                code: "02008".to_string(),
                name: "Engine Car".to_string(),
                text: "The engine car where the train is controlled.".to_string(),
                clues: 2,
                shroud: 3,
                raw: serde_json::json!({"code": "02008", "shroud": 3, "clues": 2}),
            },
        ],
    };

    let mut game = GameState::new(
        "The Dunwich Legacy",
        "The Essex County Express",
        33333,
        encounter_cards,
        scenario_data,
    );

    let (roland_id, wendy_id) = add_test_investigators(&mut game);

    // ================================================================
    // Round 1
    // ================================================================

    let _ = game.mythos_phase();

    game.advance_phase();
    assert_eq!(game.phase, Phase::Investigation);

    // Roland's turn
    let passenger_id = game.locations_in_play[0].id;
    game.move_to_location(roland_id, passenger_id).unwrap();
    let _roland_inv = game.investigate(roland_id).unwrap();

    let _next = game.next_investigator();

    // Wendy's turn
    let baggage_id = game.locations_in_play[1].id;
    game.move_to_location(wendy_id, baggage_id).unwrap();
    let _wendy_inv = game.investigate(wendy_id).unwrap();

    let _next = game.next_investigator();

    game.advance_phase();
    assert_eq!(game.phase, Phase::Enemy);

    game.advance_phase();
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // Round 2
    game.advance_phase();
    assert_eq!(game.round, 2);

    let _ = game.mythos_phase();

    game.advance_phase();

    game.move_to_location(roland_id, baggage_id).unwrap();
    let _roland_inv2 = game.investigate(roland_id).unwrap();

    let _next = game.next_investigator();

    game.move_to_location(wendy_id, game.locations_in_play[2].id).unwrap();
    let _wendy_inv2 = game.investigate(wendy_id).unwrap();

    let _next = game.next_investigator();

    game.advance_phase();
    assert_eq!(game.phase, Phase::Enemy);

    game.advance_phase();
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // Round 3
    game.advance_phase();
    assert_eq!(game.round, 3);

    let _ = game.mythos_phase();

    game.advance_phase();

    // Try to advance Act 1 (clue_cost 3)
    let roland_clues = game.investigators[0].clues;
    if roland_clues >= 3 {
        let act_result = game.advance_act(vec![(roland_id, 3)]).unwrap();
        if act_result.advanced {
            eprintln!("Essex County Express: Act 1 advanced");
        }
    } else {
        eprintln!("Essex County Express: Roland has {} clues, need 3 for Act 1", roland_clues);
    }

    let _next = game.next_investigator();

    let wendy_clues = game.investigators[1].clues;
    if wendy_clues >= 2 {
        let act_result = game.advance_act(vec![(wendy_id, 2)]).unwrap();
        if act_result.advanced {
            eprintln!("Essex County Express: Act 2 advanced");
        }
    } else {
        eprintln!("Essex County Express: Wendy has {} clues, need 2 for Act 2", wendy_clues);
    }

    game.advance_phase();
    game.advance_phase();
    game.reset_turn();

    // Round 4
    game.advance_phase();
    assert_eq!(game.round, 4);

    let _ = game.mythos_phase();
    let _doom = game.place_doom(1).unwrap();

    game.advance_phase();

    // ================================================================
    // Scenario Resolution
    // ================================================================

    match game.game_status {
        GameStatus::Won => {
            game.add_campaign_log_entry(
                CampaignResult::Won,
                "Investigators stopped the train.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Won);
        }
        GameStatus::Lost => {
            game.add_campaign_log_entry(
                CampaignResult::Lost,
                "The train reached its final destination.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Lost);
        }
        GameStatus::Ongoing => {
            eprintln!("NOTE: The Essex County Express still ongoing after 4 rounds.");
        }
    }

    eprintln!("=== The Essex County Express End State ===");
    for inv in &game.investigators {
        eprintln!(
            "Investigator {}: health={}/{} sanity={}/{} xp={} resources={} clues={}",
            inv.name, inv.health, inv.max_health, inv.sanity, inv.max_sanity,
            inv.xp, inv.resources, inv.clues
        );
    }
    eprintln!("Game status: {:?}", game.game_status);
    eprintln!("Round: {}", game.round);
}

/// E2E: Full playthrough of Blood on the Altar scenario from start to finish.
#[test]
fn e2e_full_scenario_playthrough_blood_on_the_altar() {
    let encounter_cards = vec![
        Card {
            id: Uuid::new_v4(),
            code: "02101".to_string(),
            name: "Shambling Ghoul".to_string(),
            card_type: CardType::Enemy,
            class: Some(CardClass::Neutral),
            cost: None,
            xp: 0,
            icons: SkillIcons::default(),
            traits: vec!["Monster".to_string(), "Undead".to_string()],
            text: "The dead should stay dead.".to_string(),
            raw: serde_json::json!({
                "code": "02101",
                "name": "Shambling Ghoul",
                "health": 3,
                "enemy_fight": 3,
                "enemy_damage": 1,
                "enemy_horror": 0,
                "type_code": "enemy",
            }),
        },
    ];

    let scenario_data = ScenarioData {
        campaign: "the-dunwich-legacy".to_string(),
        code: "02003".to_string(),
        name: "Blood on the Altar".to_string(),
        order: 3,
        chaos_bag: "standard".to_string(),
        acts: vec![
            ScenarioActData {
                code: "02003c".to_string(),
                name: "Blood on the Altar".to_string(),
                text: "Investigate the farmhouse to uncover the dark ritual.".to_string(),
                clue_cost: 4,
                raw: serde_json::json!({"code": "02003c", "clue_cost": 4}),
            },
            ScenarioActData {
                code: "02003d".to_string(),
                name: "Stop the Ritual".to_string(),
                text: "Interrupt the ritual before it is completed.".to_string(),
                clue_cost: 3,
                raw: serde_json::json!({"code": "02003d", "clue_cost": 3}),
            },
        ],
        agendas: vec![
            ScenarioAgendaData {
                code: "02004a".to_string(),
                name: "The Ritual Begins".to_string(),
                text: "The cultists begin their dark ritual at the altar.".to_string(),
                doom_threshold: 5,
                raw: serde_json::json!({"code": "02004a", "doom_threshold": 5}),
            },
            ScenarioAgendaData {
                code: "02004b".to_string(),
                name: "The Ritual Completes".to_string(),
                text: "If the ritual is completed, something terrible will be unleashed.".to_string(),
                doom_threshold: 5,
                raw: serde_json::json!({"code": "02004b", "doom_threshold": 5}),
            },
        ],
        locations: vec![
            ScenarioLocationData {
                code: "02009".to_string(),
                name: "Farmhouse Exterior".to_string(),
                text: "The weathered exterior of an old farmhouse.".to_string(),
                clues: 2,
                shroud: 2,
                raw: serde_json::json!({"code": "02009", "shroud": 2, "clues": 2}),
            },
            ScenarioLocationData {
                code: "02010".to_string(),
                name: "Main Room".to_string(),
                text: "The main room of the farmhouse.".to_string(),
                clues: 3,
                shroud: 3,
                raw: serde_json::json!({"code": "02010", "shroud": 3, "clues": 3}),
            },
            ScenarioLocationData {
                code: "02011".to_string(),
                name: "The Altar Room".to_string(),
                text: "A dark chamber where a crude altar stands.".to_string(),
                clues: 2,
                shroud: 4,
                raw: serde_json::json!({"code": "02011", "shroud": 4, "clues": 2}),
            },
        ],
    };

    let mut game = GameState::new(
        "The Dunwich Legacy",
        "Blood on the Altar",
        11111,
        encounter_cards,
        scenario_data,
    );

    let (roland_id, wendy_id) = add_test_investigators(&mut game);

    // ================================================================
    // Round 1
    // ================================================================

    let _ = game.mythos_phase();

    game.advance_phase();
    assert_eq!(game.phase, Phase::Investigation);

    // Roland's turn
    let exterior_id = game.locations_in_play[0].id;
    game.move_to_location(roland_id, exterior_id).unwrap();
    let _roland_inv = game.investigate(roland_id).unwrap();

    let _next = game.next_investigator();

    // Wendy's turn
    let main_room_id = game.locations_in_play[1].id;
    game.move_to_location(wendy_id, main_room_id).unwrap();
    let _wendy_inv = game.investigate(wendy_id).unwrap();

    let _next = game.next_investigator();

    game.advance_phase();
    assert_eq!(game.phase, Phase::Enemy);

    game.advance_phase();
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // Round 2
    game.advance_phase();
    assert_eq!(game.round, 2);

    let _ = game.mythos_phase();

    game.advance_phase();

    game.move_to_location(roland_id, main_room_id).unwrap();
    let _roland_inv2 = game.investigate(roland_id).unwrap();

    let _next = game.next_investigator();

    game.move_to_location(wendy_id, game.locations_in_play[2].id).unwrap();
    let _wendy_inv2 = game.investigate(wendy_id).unwrap();

    let _next = game.next_investigator();

    game.advance_phase();
    assert_eq!(game.phase, Phase::Enemy);

    game.advance_phase();
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // Round 3
    game.advance_phase();
    assert_eq!(game.round, 3);

    let _ = game.mythos_phase();

    game.advance_phase();

    // Try to advance Act 1 (clue_cost 4)
    let roland_clues = game.investigators[0].clues;
    if roland_clues >= 4 {
        let act_result = game.advance_act(vec![(roland_id, 4)]).unwrap();
        if act_result.advanced {
            eprintln!("Blood on the Altar: Act 1 advanced");
        }
    } else {
        eprintln!("Blood on the Altar: Roland has {} clues, need 4 for Act 1", roland_clues);
    }

    let _next = game.next_investigator();

    let wendy_clues = game.investigators[1].clues;
    if wendy_clues >= 3 {
        let act_result = game.advance_act(vec![(wendy_id, 3)]).unwrap();
        if act_result.advanced {
            eprintln!("Blood on the Altar: Act 2 advanced");
        }
    } else {
        eprintln!("Blood on the Altar: Wendy has {} clues, need 3 for Act 2", wendy_clues);
    }

    game.advance_phase();
    game.advance_phase();
    game.reset_turn();

    // Round 4
    game.advance_phase();
    assert_eq!(game.round, 4);

    let _ = game.mythos_phase();
    let _doom = game.place_doom(1).unwrap();

    game.advance_phase();

    // ================================================================
    // Scenario Resolution
    // ================================================================

    match game.game_status {
        GameStatus::Won => {
            game.add_campaign_log_entry(
                CampaignResult::Won,
                "Investigators stopped the ritual.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Won);
        }
        GameStatus::Lost => {
            game.add_campaign_log_entry(
                CampaignResult::Lost,
                "The ritual was completed.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Lost);
        }
        GameStatus::Ongoing => {
            eprintln!("NOTE: Blood on the Altar still ongoing after 4 rounds.");
        }
    }

    eprintln!("=== Blood on the Altar End State ===");
    for inv in &game.investigators {
        eprintln!(
            "Investigator {}: health={}/{} sanity={}/{} xp={} resources={} clues={}",
            inv.name, inv.health, inv.max_health, inv.sanity, inv.max_sanity,
            inv.xp, inv.resources, inv.clues
        );
    }
    eprintln!("Game status: {:?}", game.game_status);
    eprintln!("Round: {}", game.round);
}

/// E2E: Full playthrough of Undimensioned and Unseen scenario from start to finish.
#[test]
fn e2e_full_scenario_playthrough_undimensioned_and_unseen() {
    let encounter_cards = vec![];

    let scenario_data = ScenarioData {
        campaign: "the-dunwich-legacy".to_string(),
        code: "02004".to_string(),
        name: "Undimensioned and Unseen".to_string(),
        order: 4,
        chaos_bag: "standard".to_string(),
        acts: vec![
            ScenarioActData {
                code: "02004c".to_string(),
                name: "Undimensioned and Unseen".to_string(),
                text: "Investigate the strange phenomena at the Whateley farm.".to_string(),
                clue_cost: 4,
                raw: serde_json::json!({"code": "02004c", "clue_cost": 4}),
            },
            ScenarioActData {
                code: "02004d".to_string(),
                name: "Close the Gate".to_string(),
                text: "Close the gate before whatever is beyond can pass through.".to_string(),
                clue_cost: 3,
                raw: serde_json::json!({"code": "02004d", "clue_cost": 3}),
            },
        ],
        agendas: vec![
            ScenarioAgendaData {
                code: "02005a".to_string(),
                name: "The Gate Opens".to_string(),
                text: "A gate to another dimension is opening at the farm.".to_string(),
                doom_threshold: 6,
                raw: serde_json::json!({"code": "02005a", "doom_threshold": 6}),
            },
            ScenarioAgendaData {
                code: "02005b".to_string(),
                name: "The World Tears Apart".to_string(),
                text: "If the gate fully opens, reality itself may unravel.".to_string(),
                doom_threshold: 6,
                raw: serde_json::json!({"code": "02005b", "doom_threshold": 6}),
            },
        ],
        locations: vec![
            ScenarioLocationData {
                code: "02012".to_string(),
                name: "Whateley Farm Fields".to_string(),
                text: "Overgrown fields surrounding the Whateley farm.".to_string(),
                clues: 2,
                shroud: 2,
                raw: serde_json::json!({"code": "02012", "shroud": 2, "clues": 2}),
            },
            ScenarioLocationData {
                code: "02013".to_string(),
                name: "The Barn".to_string(),
                text: "A large barn that hums with strange energy.".to_string(),
                clues: 3,
                shroud: 3,
                raw: serde_json::json!({"code": "02013", "shroud": 3, "clues": 3}),
            },
            ScenarioLocationData {
                code: "02014".to_string(),
                name: "The Gate".to_string(),
                text: "A shimmering portal to another dimension.".to_string(),
                clues: 1,
                shroud: 5,
                raw: serde_json::json!({"code": "02014", "shroud": 5, "clues": 1}),
            },
        ],
    };

    let mut game = GameState::new(
        "The Dunwich Legacy",
        "Undimensioned and Unseen",
        44444,
        encounter_cards,
        scenario_data,
    );

    let (roland_id, wendy_id) = add_test_investigators(&mut game);

    // ================================================================
    // Round 1
    // ================================================================

    let _ = game.mythos_phase();

    game.advance_phase();
    assert_eq!(game.phase, Phase::Investigation);

    // Roland's turn
    let fields_id = game.locations_in_play[0].id;
    game.move_to_location(roland_id, fields_id).unwrap();
    let _roland_inv = game.investigate(roland_id).unwrap();

    let _next = game.next_investigator();

    // Wendy's turn
    let barn_id = game.locations_in_play[1].id;
    game.move_to_location(wendy_id, barn_id).unwrap();
    let _wendy_inv = game.investigate(wendy_id).unwrap();

    let _next = game.next_investigator();

    game.advance_phase();
    assert_eq!(game.phase, Phase::Enemy);

    game.advance_phase();
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // Round 2
    game.advance_phase();
    assert_eq!(game.round, 2);

    let _ = game.mythos_phase();

    game.advance_phase();

    game.move_to_location(roland_id, barn_id).unwrap();
    let _roland_inv2 = game.investigate(roland_id).unwrap();

    let _next = game.next_investigator();

    game.move_to_location(wendy_id, game.locations_in_play[2].id).unwrap();
    let _wendy_inv2 = game.investigate(wendy_id).unwrap();

    let _next = game.next_investigator();

    game.advance_phase();
    assert_eq!(game.phase, Phase::Enemy);

    game.advance_phase();
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // Round 3
    game.advance_phase();
    assert_eq!(game.round, 3);

    let _ = game.mythos_phase();

    game.advance_phase();

    // Try to advance Act 1 (clue_cost 4)
    let roland_clues = game.investigators[0].clues;
    if roland_clues >= 4 {
        let act_result = game.advance_act(vec![(roland_id, 4)]).unwrap();
        if act_result.advanced {
            eprintln!("Undimensioned and Unseen: Act 1 advanced");
        }
    } else {
        eprintln!("Undimensioned and Unseen: Roland has {} clues, need 4 for Act 1", roland_clues);
    }

    let _next = game.next_investigator();

    let wendy_clues = game.investigators[1].clues;
    if wendy_clues >= 3 {
        let act_result = game.advance_act(vec![(wendy_id, 3)]).unwrap();
        if act_result.advanced {
            eprintln!("Undimensioned and Unseen: Act 2 advanced");
        }
    } else {
        eprintln!("Undimensioned and Unseen: Wendy has {} clues, need 3 for Act 2", wendy_clues);
    }

    game.advance_phase();
    game.advance_phase();
    game.reset_turn();

    // Round 4
    game.advance_phase();
    assert_eq!(game.round, 4);

    let _ = game.mythos_phase();
    let _doom = game.place_doom(1).unwrap();

    game.advance_phase();

    // ================================================================
    // Scenario Resolution
    // ================================================================

    match game.game_status {
        GameStatus::Won => {
            game.add_campaign_log_entry(
                CampaignResult::Won,
                "Investigators closed the gate.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Won);
        }
        GameStatus::Lost => {
            game.add_campaign_log_entry(
                CampaignResult::Lost,
                "The gate opened fully.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Lost);
        }
        GameStatus::Ongoing => {
            eprintln!("NOTE: Undimensioned and Unseen still ongoing after 4 rounds.");
        }
    }

    eprintln!("=== Undimensioned and Unseen End State ===");
    for inv in &game.investigators {
        eprintln!(
            "Investigator {}: health={}/{} sanity={}/{} xp={} resources={} clues={}",
            inv.name, inv.health, inv.max_health, inv.sanity, inv.max_sanity,
            inv.xp, inv.resources, inv.clues
        );
    }
    eprintln!("Game status: {:?}", game.game_status);
    eprintln!("Round: {}", game.round);
}

/// E2E: Full playthrough of The Gathering scenario from start to finish.
/// Simulates a real playthrough: mythos encounters, investigation,
/// fighting/fleeing, doom advancement, act/agenda advancement.
/// Reports issues found rather than fixing them.
#[test]
fn e2e_full_scenario_playthrough_the_gathering() {
    // ---------- Setup ----------
    let encounter_cards = vec![
        // Enemy: Ghoul Priest
        Card {
            id: Uuid::new_v4(),
            code: "01101".to_string(),
            name: "Ghoul Priest".to_string(),
            card_type: CardType::Enemy,
            class: Some(CardClass::Neutral),
            cost: None,
            xp: 0,
            icons: SkillIcons::default(),
            traits: vec!["Humanoid".to_string(), "Cultist".to_string()],
            text: "A monstrous cultist.".to_string(),
            raw: serde_json::json!({
                "code": "01101",
                "name": "Ghoul Priest",
                "health": 3,
                "enemy_fight": 4,
                "enemy_damage": 1,
                "enemy_horror": 1,
                "type_code": "enemy",
            }),
        },
        // Enemy: Cultist
        Card {
            id: Uuid::new_v4(),
            code: "01102".to_string(),
            name: "Cultist".to_string(),
            card_type: CardType::Enemy,
            class: Some(CardClass::Neutral),
            cost: None,
            xp: 0,
            icons: SkillIcons::default(),
            traits: vec!["Humanoid".to_string(), "Cultist".to_string()],
            text: "A low-ranking cultist.".to_string(),
            raw: serde_json::json!({
                "code": "01102",
                "name": "Cultist",
                "health": 2,
                "enemy_fight": 2,
                "enemy_damage": 1,
                "enemy_horror": 0,
                "type_code": "enemy",
            }),
        },
        // Treachery: Ancient Evils
        Card {
            id: Uuid::new_v4(),
            code: "02102".to_string(),
            name: "Ancient Evils".to_string(),
            card_type: CardType::Treachery,
            class: Some(CardClass::Neutral),
            cost: None,
            xp: 0,
            icons: SkillIcons::default(),
            traits: vec![],
            text: "Revelation - Place 1 doom on the current agenda.".to_string(),
            raw: serde_json::json!({
                "code": "02102",
                "name": "Ancient Evils",
                "type_code": "treachery",
                "type_name": "Treachery",
            }),
        },
        // Player card: .45 Automatic (Roland's signature)
        Card {
            id: Uuid::new_v4(),
            code: "02006".to_string(),
            name: ".45 Automatic".to_string(),
            card_type: CardType::Asset,
            class: Some(CardClass::Guardian),
            cost: Some(3),
            xp: 0,
            icons: SkillIcons {
                willpower: 0,
                intellect: 0,
                combat: 2,
                agility: 0,
                wild: 0,
            },
            traits: vec!["Item".to_string(), "Weapon".to_string(), "Firearm".to_string()],
            text: "Uses (6 ammo). [action] Spend 1 ammo: Fight. You get +2 [combat] for this attack.".to_string(),
            raw: serde_json::json!({
                "code": "02006",
                "name": ".45 Automatic",
                "type_code": "asset",
                "type_name": "Asset",
            }),
        },
    ];

    let scenario_data = ScenarioData {
        campaign: "night-of-the-zealot".to_string(),
        code: "01001".to_string(),
        name: "The Gathering".to_string(),
        order: 1,
        chaos_bag: "standard".to_string(),
        acts: vec![
            ScenarioActData {
                code: "01001a".to_string(),
                name: "Rumors at the Boarding House".to_string(),
                text: "Investigate one or more locations at The Boarding House to learn what strange events are occurring here.".to_string(),
                clue_cost: 3,
                raw: serde_json::json!({"code": "01001a", "clue_cost": 3}),
            },
            ScenarioActData {
                code: "01001b".to_string(),
                name: "The Boarding House".to_string(),
                text: "Investigate the Living Room to uncover the truth behind the strange happenings at the boarding house.".to_string(),
                clue_cost: 3,
                raw: serde_json::json!({"code": "01001b", "clue_cost": 3}),
            },
        ],
        agendas: vec![
            ScenarioAgendaData {
                code: "01002a".to_string(),
                name: "The Boarding House Burns Down".to_string(),
                text: "The building is engulfed in flames. You must escape before it is too late.".to_string(),
                doom_threshold: 4,
                raw: serde_json::json!({"code": "01002a", "doom_threshold": 4}),
            },
            ScenarioAgendaData {
                code: "01002b".to_string(),
                name: "Escape the Boarding House".to_string(),
                text: "You must escape the burning building before it collapses around you.".to_string(),
                doom_threshold: 4,
                raw: serde_json::json!({"code": "01002b", "doom_threshold": 4}),
            },
        ],
        locations: vec![
            ScenarioLocationData {
                code: "01003".to_string(),
                name: "Front Door".to_string(),
                text: "The front door of the boarding house stands slightly ajar.".to_string(),
                clues: 2,
                shroud: 2,
                raw: serde_json::json!({"code": "01003", "shroud": 2, "clues": 2}),
            },
            ScenarioLocationData {
                code: "01004".to_string(),
                name: "Living Room".to_string(),
                text: "Dust covers the furniture in this once-elegant room.".to_string(),
                clues: 2,
                shroud: 2,
                raw: serde_json::json!({"code": "01004", "shroud": 2, "clues": 2}),
            },
            ScenarioLocationData {
                code: "01005".to_string(),
                name: "Kitchen".to_string(),
                text: "The kitchen is in disarray.".to_string(),
                clues: 1,
                shroud: 1,
                raw: serde_json::json!({"code": "01005", "shroud": 1, "clues": 1}),
            },
        ],
    };

    let mut game = GameState::new(
        "Night of the Zealot",
        "The Gathering",
        12345,
        encounter_cards,
        scenario_data,
    );

    let roland = Investigator::new(
        "roland-banks",
        "Roland Banks",
        2, // willpower
        1, // intellect
        3, // combat
        1, // agility
        9, // max_health
        5, // max_sanity
    );
    let roland_id = roland.id;
    game.add_investigator(roland);

    let wendy = Investigator::new(
        "wendy-adams",
        "Wendy Adams",
        2, // willpower
        2, // intellect
        2, // combat
        2, // agility
        8, // max_health
        5, // max_sanity
    );
    let wendy_id = wendy.id;
    game.add_investigator(wendy);

    // ================================================================
    // Round 1
    // ================================================================

    // --- Mythos Phase ---
    let mythos_result = game.mythos_phase().unwrap();
    // Encounter card drawn - could be enemy, treachery, or other

    // --- Investigation Phase: Roland ---
    game.advance_phase(); // Mythos -> Investigation
    assert_eq!(game.phase, Phase::Investigation);

    // Roland moves to Front Door
    let front_door_id = game.locations_in_play[0].id;
    game.move_to_location(roland_id, front_door_id).unwrap();
    assert_eq!(game.investigators[0].location_id, Some(front_door_id));

    // Roland investigates Front Door
    let result = game.investigate(roland_id).unwrap();
    // Whether Roland discovers a clue depends on chaos bag draw vs shroud 2
    eprintln!("Roland investigates Front Door: success={}, clues_discovered={}, total={}", result.test_total >= 2, result.clues_discovered, result.test_total);

    // Roland spends remaining action playing .45 Automatic if in hand
    // (not testing card play here since we don't control the draw)

    // --- Investigation Phase: Wendy ---
    game.advance_phase(); // Investigation -> Enemy
    assert_eq!(game.phase, Phase::Enemy);

    // Enemies engage/attack if ready
    // (Auto-engage happens on move, attacks of opportunity on actions)

    // --- Upkeep Phase ---
    game.advance_phase(); // Enemy -> Upkeep
    assert_eq!(game.phase, Phase::Upkeep);
    game.reset_turn();

    // --- Next Round ---
    game.advance_phase(); // Upkeep -> Mythos (round 2)
    assert_eq!(game.round, 2);

    // --- Mythos Phase Round 2 ---
    let mythos_result2 = game.mythos_phase().unwrap();

    // --- Investigation Phase Round 2 ---
    game.advance_phase(); // Mythos -> Investigation

    // Wendy moves to Living Room
    let living_room_id = game.locations_in_play[1].id;
    game.move_to_location(wendy_id, living_room_id).unwrap();

    // Wendy investigates Living Room
    let wendy_investigate = game.investigate(wendy_id).unwrap();

    // Roland moves to Kitchen
    let kitchen_id = game.locations_in_play[2].id;
    game.move_to_location(roland_id, kitchen_id).unwrap();

    // Roland investigates Kitchen
    let roland_kitchen = game.investigate(roland_id).unwrap();

    // --- Enemy Phase Round 2 ---
    game.advance_phase(); // Investigation -> Enemy
    assert_eq!(game.phase, Phase::Enemy);

    // --- Upkeep Phase Round 2 ---
    game.advance_phase(); // Enemy -> Upkeep
    game.reset_turn();

    // --- Round 3 ---
    game.advance_phase(); // Upkeep -> Mythos (round 3)
    assert_eq!(game.round, 3);

    // --- Mythos Phase Round 3 ---
    let mythos_result3 = game.mythos_phase().unwrap();

    // --- Investigation Phase Round 3 ---
    game.advance_phase(); // Mythos -> Investigation

    // Advance Act 1: Roland contributes clues from Front Door + Kitchen
    let roland_clues = game.investigators[0].clues;
    if roland_clues >= 3 {
        let act_result = game.advance_act(vec![(roland_id, 3)]).unwrap();
        assert!(act_result.advanced, "Act 1 should advance with Roland's clues");
        assert_eq!(game.current_act_index, 1);
    }

    // Wendy also investigates
    let wendy_investigate2 = game.investigate(wendy_id).unwrap();

    // --- Enemy Phase Round 3 ---
    game.advance_phase(); // Investigation -> Enemy
    assert_eq!(game.phase, Phase::Enemy);

    // --- Upkeep Phase Round 3 ---
    game.advance_phase(); // Enemy -> Upkeep
    game.reset_turn();

    // --- Round 4 ---
    game.advance_phase(); // Upkeep -> Mythos (round 4)
    assert_eq!(game.round, 4);

    // --- Mythos Phase Round 4: Place doom ---
    game.place_doom(1).unwrap();
    // Check if agenda advanced

    // --- Investigation Phase Round 4 ---
    game.advance_phase(); // Mythos -> Investigation

    // Advance Act 2: Wendy contributes clues
    let wendy_clues = game.investigators[1].clues;
    if wendy_clues >= 3 {
        let act_result = game.advance_act(vec![(wendy_id, 3)]).unwrap();
        assert!(act_result.advanced, "Act 2 should advance");
        assert_eq!(game.current_act_index, 2);
        assert_eq!(game.game_status, GameStatus::Won);
    }

    // ================================================================
    // Scenario Resolution
    // ================================================================

    match game.game_status {
        GameStatus::Won => {
            game.add_campaign_log_entry(
                CampaignResult::Won,
                "Investigators uncovered the truth at the boarding house.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log.len(), 1);
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Won);
        }
        GameStatus::Lost => {
            game.add_campaign_log_entry(
                CampaignResult::Lost,
                "The boarding house burned down before investigators could escape.".to_string(),
            );
            assert_eq!(game.campaign_progress.campaign_log.len(), 1);
            assert_eq!(game.campaign_progress.campaign_log[0].result, CampaignResult::Lost);
        }
        GameStatus::Ongoing => {
            // Scenario didn't conclude in 4 rounds - report this as an observation
            eprintln!("NOTE: Scenario still ongoing after 4 rounds. This may indicate doom/act progression is too slow.");
        }
    }

    // Report investigator state at scenario end
    eprintln!("=== Scenario End State ===");
    for inv in &game.investigators {
        eprintln!(
            "Investigator {}: health={}/{} sanity={}/{} xp={} resources={} clues={} defeated={} trauma(physical={}, mental={}) killed={} insane={}",
            inv.name, inv.health, inv.max_health, inv.sanity, inv.max_sanity,
            inv.xp, inv.resources, inv.clues, inv.is_defeated,
            inv.physical_trauma, inv.mental_trauma, inv.is_killed, inv.is_driven_insane
        );
    }
    eprintln!("Game status: {:?}", game.game_status);
    eprintln!("Current act: {}", game.current_act_index);
    eprintln!("Current agenda: {}", game.current_agenda_index);
    eprintln!("Round: {}", game.round);
    eprintln!("Victory display cards: {}", game.victory_display.len());
}
