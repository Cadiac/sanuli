use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::rc::Rc;

use gloo_storage::errors::StorageError;
use serde::{Deserialize, Serialize};

use crate::game::{Board, Game, DEFAULT_ALLOW_PROFANITIES, DEFAULT_WORD_LENGTH, SUCCESS_EMOJIS};
use crate::manager::{GameMode, KeyState, Theme, TileState, WordList, WordLists};
use crate::sanuli::Sanuli;

const MAX_GUESSES: usize = 9;
const WORD_LENGTH: usize = 5; // TODO: Just handle this as a default and get it from manager at new

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Neluli {
    pub word_list: WordList,
    pub boards: Vec<Sanuli>,
    pub streak: usize,
    pub message: String,

    #[serde(skip)]
    allow_profanities: bool,
    #[serde(skip)]
    word_lists: Rc<WordLists>,
}

impl Default for Neluli {
    fn default() -> Self {
        Neluli::new(
            WordList::default(),
            DEFAULT_WORD_LENGTH,
            DEFAULT_ALLOW_PROFANITIES,
            Rc::new(HashMap::new()),
        )
    }
}

impl Neluli {
    pub fn new(
        word_list: WordList,
        word_length: usize,
        allow_profanities: bool,
        word_lists: Rc<WordLists>,
    ) -> Self {
        let boards = vec![
            Sanuli::new(
                GameMode::Quadruple,
                word_list,
                word_length,
                MAX_GUESSES,
                allow_profanities,
                word_lists.clone(),
            ),
            Sanuli::new(
                GameMode::Quadruple,
                word_list,
                word_length,
                MAX_GUESSES,
                allow_profanities,
                word_lists.clone(),
            ),
            Sanuli::new(
                GameMode::Quadruple,
                word_list,
                word_length,
                MAX_GUESSES,
                allow_profanities,
                word_lists.clone(),
            ),
            Sanuli::new(
                GameMode::Quadruple,
                word_list,
                word_length,
                MAX_GUESSES,
                allow_profanities,
                word_lists.clone(),
            ),
        ];

        Self {
            word_list,

            boards,
            streak: 0,

            message: String::new(),

            allow_profanities: DEFAULT_ALLOW_PROFANITIES,
            word_lists,
        }
    }

    fn is_game_ended(&self) -> bool {
        self.boards.iter().all(|board| !board.is_guessing())
    }

    fn clear_message(&mut self) {
        self.message = String::new();
    }

    fn set_game_end_message(&mut self) {
        if self.is_winner() {
            self.message = format!(
                "Löysit sanulit! {}",
                SUCCESS_EMOJIS.choose(&mut rand::thread_rng()).unwrap()
            );
        } else {
            let words: Vec<_> = self
                .boards
                .iter()
                .filter(|game| !game.is_winner())
                .map(|game| game.word().iter().collect::<String>())
                .collect();
            self.message = format!("Löytämättä jäi: \"{}\"", words.join("\", \""));
        }
    }
}

impl Game for Neluli {
    fn game_mode(&self) -> &GameMode {
        &GameMode::Quadruple
    }
    fn word_list(&self) -> &WordList {
        &self.word_list
    }
    fn word_length(&self) -> usize {
        WORD_LENGTH
    }
    fn max_guesses(&self) -> usize {
        MAX_GUESSES
    }
    fn boards(&self) -> Vec<Board> {
        self.boards.iter().flat_map(|game| game.boards()).collect()
    }
    fn word(&self) -> Vec<char> {
        Vec::new()
    }

    fn streak(&self) -> usize {
        self.streak
    }
    fn last_guess(&self) -> String {
        String::new()
    }

    fn is_guessing(&self) -> bool {
        self.boards.iter().any(|board| board.is_guessing())
    }
    fn is_winner(&self) -> bool {
        self.boards.iter().all(|board| board.is_winner())
    }
    fn is_reset(&self) -> bool {
        false
    }
    fn is_hidden(&self) -> bool {
        false
    }
    fn is_unknown(&self) -> bool {
        false
    }
    fn message(&self) -> String {
        self.message.clone()
    }
    fn previous_guesses(&self) -> Vec<Vec<(char, TileState)>> {
        Vec::new()
    }

    fn set_allow_profanities(&mut self, is_allowed: bool) {
        self.allow_profanities = is_allowed;
    }

    fn title(&self) -> String {
        String::from("Neluli")
    }

    fn next_word(&mut self) {
        for board in self.boards.iter_mut() {
            board.next_word();
        }
        self.clear_message();
    }

    fn prepare_previous_guesses_animation(&mut self, _previous_length: usize) {}

    fn keyboard_tilestate(&self, key: &char) -> KeyState {
        KeyState::Quadruple([
            if let KeyState::Single(state) = self.boards[0].keyboard_tilestate(key) { state } else { TileState::Unknown },
            if let KeyState::Single(state) = self.boards[1].keyboard_tilestate(key) { state } else { TileState::Unknown },
            if let KeyState::Single(state) = self.boards[2].keyboard_tilestate(key) { state } else { TileState::Unknown },
            if let KeyState::Single(state) = self.boards[3].keyboard_tilestate(key) { state } else { TileState::Unknown },
        ])
    }

    fn submit_guess(&mut self) {
        for board in self.boards.iter_mut() {
            if board.is_guessing() {
                if !board.is_guess_correct_length() {
                    self.message = "Liian vähän kirjaimia!".to_owned();
                    return;
                }

                if !board.is_guess_accepted_word() {
                    self.message = "Ei sanulistalla.".to_owned();
                    return;
                }

                board.submit_guess();
            }
        }

        if self.is_game_ended() {
            self.set_game_end_message();
        } else {
            self.clear_message();
        }
    }

    fn push_character(&mut self, character: char) {
        if !self.is_guessing() {
            return;
        }

        self.clear_message();

        for board in self.boards.iter_mut() {
            board.push_character(character);
        }
    }

    fn pop_character(&mut self) {
        if !self.is_guessing() {
            return;
        }

        self.clear_message();

        for board in self.boards.iter_mut() {
            board.pop_character();
        }
    }

    fn share_emojis(&self, _theme: Theme) -> Option<String> {
        unimplemented!()
    }

    fn share_link(&self) -> Option<String> {
        unimplemented!()
    }

    fn reveal_hidden_tiles(&mut self) {
        unimplemented!()
    }

    fn reset(&mut self) {
        unimplemented!()
    }

    fn refresh(&mut self) {
        for board in self.boards.iter_mut() {
            board.refresh();
        }
    }

    fn persist(&self) -> Result<(), StorageError> {
        Ok(())
    }
}
