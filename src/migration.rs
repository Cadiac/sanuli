use chrono::NaiveDate;
use wasm_bindgen::JsValue;
use web_sys::{window, Window};

use crate::state::{Game, GameMode, State, Theme, TileState, WordList, DAILY_WORD_LEN, EMPTY};

// Migrate the old game data to the new format, removing old data from localStorage.
// TODO: Get rid of this at some point, even if that means data loss to some players
pub fn migrate_state(state: &mut State) -> Result<(), JsValue> {
    let window: Window = window().expect("window not available");
    if let Some(local_storage) = window.local_storage().expect("local storage not available") {
        // Daily words
        if let Some(daily_word_history_str) = local_storage.get_item("daily_word_history")? {
            local_storage.remove_item("daily_word_history")?;

            if daily_word_history_str.len() != 0 {
                daily_word_history_str.split(',').for_each(|date_str| {
                    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
                    let daily_item_key = &format!("daily_word_history[{}]", date_str);
                    let daily_item = local_storage.get_item(daily_item_key).unwrap();
                    if let Some(daily_str) = daily_item {
                        let _res = local_storage.remove_item(daily_item_key);

                        let parts = daily_str.split('|').collect::<Vec<&str>>();

                        // AIVAN|2022-01-07|KOIRA,AVAIN,AIVAN,,,|2|true|true
                        // let word = parts[0];
                        let previous_guesses = parts[2]
                            .split(',')
                            .map(|guess| guess.chars().map(|c| (c, TileState::Unknown)).collect());
                        let current_guess = parts[3].parse::<usize>().unwrap();
                        let is_guessing = parts[4].parse::<bool>().unwrap();
                        let is_winner = parts[5].parse::<bool>().unwrap();

                        // If we haven't got a game in background with this date, create one
                        let game_id = (GameMode::DailyWord(date), WordList::Daily, DAILY_WORD_LEN);

                        if !state.background_games.contains_key(&game_id) {
                            let mut new_daily_game = Game::new(
                                game_id.0,
                                game_id.1,
                                game_id.2,
                                state.game_manager.clone(),
                            );

                            for (guess_index, guess) in previous_guesses.enumerate() {
                                new_daily_game.guesses[guess_index] = guess;
                                new_daily_game.current_guess = guess_index;
                                new_daily_game.calculate_current_guess();
                            }

                            new_daily_game.current_guess = current_guess;
                            new_daily_game.is_guessing = is_guessing;
                            new_daily_game.is_winner = is_winner;

                            if !new_daily_game.is_guessing {
                                new_daily_game.message = "Uusi sanuli huomenna!".to_owned();
                            } else {
                                new_daily_game.message = EMPTY.to_string()
                            }

                            // Persist the game
                            let _res = new_daily_game.persist();

                            state.background_games.insert(game_id, new_daily_game);
                        } else {
                            // We... Already had the game? Don't do anything?
                        }
                    }
                });
            }
        }

        // Current game
        if let Some(game_mode_str) = local_storage.get_item("game_mode")? {
            if let Ok(game_mode) = game_mode_str.parse::<GameMode>() {
                state.game_manager.borrow_mut().current_game_mode = game_mode;
            }
            local_storage.remove_item("game_mode")?;
        }

        if let Some(word_list_str) = local_storage.get_item("word_list")? {
            if let Ok(word_list) = word_list_str.parse::<WordList>() {
                if matches!(
                    state.game_manager.borrow().current_game_mode,
                    GameMode::DailyWord(_)
                ) {
                    // Force the word list as daily word
                    state.game_manager.borrow_mut().current_word_list = WordList::Daily;
                } else {
                    state.game_manager.borrow_mut().current_word_list = word_list;
                }
            }
            local_storage.remove_item("word_list")?;
        }

        if let Some(word_length_str) = local_storage.get_item("word_length")? {
            if let Ok(word_length) = word_length_str.parse::<usize>() {
                if matches!(
                    state.game_manager.borrow().current_game_mode,
                    GameMode::DailyWord(_)
                ) {
                    // Force the word length for daily word
                    state.game_manager.borrow_mut().current_word_length = DAILY_WORD_LEN;
                } else {
                    state.game_manager.borrow_mut().current_word_length = word_length;
                }
            }
            local_storage.remove_item("word_length")?;
        }

        if let Some(allow_profanities_str) = local_storage.get_item("allow_profanities")? {
            if let Ok(allow_profanities) = allow_profanities_str.parse::<bool>() {
                state.game_manager.borrow_mut().allow_profanities = allow_profanities;
            }
            local_storage.remove_item("allow_profanities")?;
        }

        if let Some(theme_str) = local_storage.get_item("theme")? {
            if let Ok(theme) = theme_str.parse::<Theme>() {
                state.game_manager.borrow_mut().theme = theme;
            }
            local_storage.remove_item("theme")?;
        }

        if let Some(message_str) = local_storage.get_item("message")? {
            state.game.message = message_str;
            local_storage.remove_item("message")?;
        }

        if let Some(max_streak_str) = local_storage.get_item("max_streak")? {
            if let Ok(max_streak) = max_streak_str.parse::<usize>() {
                state.game_manager.borrow_mut().max_streak = max_streak;
            }
            local_storage.remove_item("max_streak")?;
        }

        if let Some(total_played_str) = local_storage.get_item("total_played")? {
            if let Ok(total_played) = total_played_str.parse::<usize>() {
                state.game_manager.borrow_mut().total_played = total_played;
            }
            local_storage.remove_item("total_played")?;
        }

        if let Some(total_solved_str) = local_storage.get_item("total_solved")? {
            if let Ok(total_solved) = total_solved_str.parse::<usize>() {
                state.game_manager.borrow_mut().total_solved = total_solved;
            }
            local_storage.remove_item("total_solved")?;
        }
    }

    Ok(())
}

// Migrate the old game data (well, only the streak) to the new format, removing old data from localStorage.
// TODO: Get rid of this at some point, even if that means data loss to some players
pub fn migrate_game(game: &mut Game) -> Result<(), JsValue> {
    let window: Window = window().expect("window not available");
    if let Some(local_storage) = window.local_storage()? {
        match game.game_manager.borrow().current_game_mode {
            GameMode::Classic | GameMode::Relay => {
                if let Some(streak_str) = local_storage.get_item("streak")? {
                    if let Ok(streak) = streak_str.parse::<usize>() {
                        game.streak = streak;
                    }
                }
            }
            _ => {}
        }

        local_storage.remove_item("streak")?;
        local_storage.remove_item("word")?;
        local_storage.remove_item("is_guessing")?;
        local_storage.remove_item("is_winner")?;
        local_storage.remove_item("guesses")?;
        local_storage.remove_item("current_guess")?;
    }

    Ok(())
}
