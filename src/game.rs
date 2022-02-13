use std::collections::HashMap;

use gloo_storage::{errors::StorageError};

pub type KnownStates = HashMap<(char, usize), CharacterState>;
pub type KnownCounts = HashMap<char, CharacterCount>;

use crate::manager::{
    CharacterCount, CharacterState, GameMode, Theme, TileState, WordList, KeyState,
};

pub const SUCCESS_EMOJIS: [&str; 8] = ["🥳", "🤩", "🤗", "🎉", "😊", "😺", "😎", "👏"];
pub const DEFAULT_WORD_LENGTH: usize = 5;
pub const DEFAULT_MAX_GUESSES: usize = 6;
pub const DEFAULT_ALLOW_PROFANITIES: bool = false;

pub trait Game {
    fn title(&self) -> String;
    fn next_word(&mut self);
    fn keyboard_tilestate(&self, key: &char) -> KeyState;
    fn submit_guess(&mut self);
    fn push_character(&mut self, character: char);
    fn pop_character(&mut self);
    fn share_emojis(&self, theme: Theme) -> Option<String>;
    fn share_link(&self) -> Option<String>;
    fn reveal_hidden_tiles(&mut self);
    fn reset(&mut self);
    fn refresh(&mut self);
    fn persist(&self) -> Result<(), StorageError>;
    fn set_allow_profanities(&mut self, is_allowed: bool);

    fn prepare_previous_guesses_animation(&mut self, previous_length: usize);

    fn game_mode(&self) -> &GameMode;
    fn word_list(&self) -> &WordList;
    fn word_length(&self) -> usize;
    fn max_guesses(&self) -> usize;
    fn word(&self) -> Vec<char>;

    fn last_guess(&self) -> String;
    fn boards(&self) -> Vec<Board>;
    fn streak(&self) -> usize;

    fn is_guessing(&self) -> bool;
    fn is_reset(&self) -> bool;
    fn is_hidden(&self) -> bool;
    fn is_winner(&self) -> bool;
    fn is_unknown(&self) -> bool;

    fn message(&self) -> String;

    fn previous_guesses(&self) -> Vec<Vec<(char, TileState)>>;
}

impl PartialEq for dyn Game {
    fn eq(&self, other: &Self) -> bool {
        self.title() == other.title()
            && self.game_mode() == other.game_mode()
            && self.word_list() == other.word_list()
            && self.word_length() == other.word_length()
            && self.max_guesses() == other.max_guesses()
            && self.boards() == other.boards()
            && self.streak() == other.streak()
            && self.is_reset() == other.is_reset()
            && self.is_hidden() == other.is_hidden()
            && self.message() == other.message()
            && self.previous_guesses() == other.previous_guesses()
    }
}

#[derive(PartialEq)]
pub struct Board {
    pub guesses: Vec<Vec<(char, TileState)>>,
    pub current_guess: usize,
    pub is_guessing: bool,
}
