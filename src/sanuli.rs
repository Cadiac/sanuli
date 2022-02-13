use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

use chrono::NaiveDate;
use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use web_sys::{window, Window};

pub type KnownStates = HashMap<(char, usize), CharacterState>;
pub type KnownCounts = HashMap<char, CharacterCount>;

use crate::game::{
    Board, Game, DEFAULT_ALLOW_PROFANITIES, DEFAULT_MAX_GUESSES, DEFAULT_WORD_LENGTH,
    SUCCESS_EMOJIS,
};
use crate::logic;
use crate::manager::{
    CharacterCount, CharacterState, GameMode, Theme, TileState, WordList, WordLists, KeyState,
};

const DAILY_WORDS: &str = include_str!("../daily-words.txt");

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Sanuli {
    pub game_mode: GameMode,
    pub word_list: WordList,
    pub word_length: usize,
    pub max_guesses: usize,

    pub word: Vec<char>,
    pub guesses: Vec<Vec<(char, TileState)>>,
    pub current_guess: usize,
    pub streak: usize,

    pub is_guessing: bool,
    pub is_winner: bool,
    pub is_unknown: bool,
    pub is_reset: bool,
    #[serde(skip)]
    pub is_hidden: bool,

    pub message: String,

    #[serde(skip)]
    pub previous_guesses: Vec<Vec<(char, TileState)>>,

    #[serde(skip)]
    allow_profanities: bool,
    #[serde(skip)]
    word_lists: Rc<WordLists>,
    #[serde(skip)]
    known_states: Vec<KnownStates>,
    #[serde(skip)]
    known_counts: Vec<KnownCounts>,
}

impl Default for Sanuli {
    fn default() -> Self {
        Sanuli::new(
            GameMode::default(),
            WordList::default(),
            DEFAULT_WORD_LENGTH,
            DEFAULT_MAX_GUESSES,
            DEFAULT_ALLOW_PROFANITIES,
            Rc::new(HashMap::new()),
        )
    }
}

impl Sanuli {
    pub fn new(
        game_mode: GameMode,
        word_list: WordList,
        word_length: usize,
        max_guesses: usize,
        allow_profanities: bool,
        word_lists: Rc<WordLists>,
    ) -> Self {
        let guesses = std::iter::repeat(Vec::with_capacity(word_length))
            .take(max_guesses)
            .collect::<Vec<_>>();

        let known_states = std::iter::repeat(HashMap::new())
            .take(max_guesses)
            .collect::<Vec<_>>();

        let known_counts = std::iter::repeat(HashMap::new())
            .take(max_guesses)
            .collect::<Vec<_>>();

        let word = if word_lists.is_empty() {
            // Default initialization runs into this
            vec!['X'; word_length]
        } else {
            Self::get_word(
                game_mode,
                word_list,
                word_length,
                allow_profanities,
                &word_lists,
            )
        };

        Self {
            game_mode,
            word_list,
            word_lists,
            word_length,
            max_guesses,
            word,
            allow_profanities,
            is_guessing: true,
            is_winner: false,
            is_unknown: false,
            is_reset: false,
            is_hidden: false,
            message: String::new(),
            known_states,
            known_counts,
            guesses,
            previous_guesses: Vec::new(),
            current_guess: 0,
            streak: 0,
        }
    }

    pub fn from_shared_link(game_str: &str, word_lists: Rc<WordLists>) -> Option<Self> {
        let max_guesses = DEFAULT_MAX_GUESSES;

        let mut parts = game_str.split("|");
        let word = parts.next()?.chars().collect::<Vec<_>>();
        let word_length = word.len();

        let guesses_str = parts.next()?;

        let mut guesses = guesses_str
            .chars()
            .map(|c| (c, TileState::Unknown))
            .collect::<Vec<_>>()
            .chunks(word_length)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<_>>();

        let current_guess = guesses.len() - 1;

        guesses.resize(max_guesses, Vec::with_capacity(word_length));

        let known_states = std::iter::repeat(HashMap::new())
            .take(max_guesses)
            .collect::<Vec<_>>();

        let known_counts = std::iter::repeat(HashMap::new())
            .take(max_guesses)
            .collect::<Vec<_>>();

        let mut game = Self {
            game_mode: GameMode::Shared,
            word_list: WordList::Full,
            word_lists,
            word_length,
            max_guesses,
            word,
            allow_profanities: true,
            is_guessing: false,
            is_winner: false,
            is_unknown: false,
            is_reset: false,
            is_hidden: true,
            message: String::new(),
            known_states,
            known_counts,
            guesses,
            previous_guesses: Vec::new(),
            current_guess,
            streak: 0,
        };

        game.refresh();

        return Some(game);
    }

    pub fn new_or_rehydrate(
        game_mode: GameMode,
        word_list: WordList,
        word_length: usize,
        allow_profanities: bool,
        word_lists: Rc<WordLists>,
    ) -> Self {
        if let Ok(game) = Self::rehydrate(
            game_mode,
            word_list,
            word_length,
            allow_profanities,
            word_lists.clone(),
        ) {
            game
        } else {
            Self::new(
                game_mode,
                word_list,
                word_length,
                DEFAULT_MAX_GUESSES,
                allow_profanities,
                word_lists,
            )
        }
    }

    fn get_word(
        game_mode: GameMode,
        word_list: WordList,
        word_length: usize,
        allow_profanities: bool,
        word_lists: &Rc<WordLists>,
    ) -> Vec<char> {
        if let GameMode::DailyWord(date) = game_mode {
            Self::get_daily_word(date)
        } else {
            Self::get_random_word(word_list, word_length, allow_profanities, word_lists)
        }
    }

    fn get_random_word(
        word_list: WordList,
        word_length: usize,
        allow_profanities: bool,
        word_lists: &Rc<WordLists>,
    ) -> Vec<char> {
        let mut words = word_lists
            .get(&(word_list, word_length))
            .unwrap()
            .iter()
            .collect::<Vec<_>>();

        if !allow_profanities {
            if let Some(profanities) = word_lists.get(&(WordList::Profanities, word_length)) {
                words.retain(|word| !profanities.contains(*word));
            }
        }

        let chosen = words.choose(&mut rand::thread_rng()).unwrap();
        (*chosen).clone()
    }

    fn get_daily_word_index(date: NaiveDate) -> usize {
        let epoch = NaiveDate::from_ymd(2022, 1, 7); // Epoch of the daily word mode, index 0
        date.signed_duration_since(epoch).num_days() as usize
    }

    fn get_daily_word(date: NaiveDate) -> Vec<char> {
        DAILY_WORDS
            .lines()
            .nth(Self::get_daily_word_index(date))
            .unwrap()
            .chars()
            .collect()
    }

    pub fn is_guess_correct_length(&self) -> bool {
       self.guesses[self.current_guess].len() == self.word_length
    }

    pub fn is_guess_accepted_word(&self) -> bool {
        // Always allow correct words, even if they aren't on the list
        if self.is_correct_word() {
            return true;
        }

        let word: &Vec<char> = &self.guesses[self.current_guess]
            .iter()
            .map(|(c, _)| *c)
            .collect();

        match self.word_lists.get(&(WordList::Full, self.word_length)) {
            Some(list) => list.contains(word),
            None => false,
        }
    }

    fn is_correct_word(&self) -> bool {
        self.guesses[self.current_guess]
            .iter()
            .map(|(c, _)| *c)
            .collect::<Vec<char>>()
            == self.word
    }

    pub fn is_game_ended(&self) -> bool {
        self.is_winner || self.current_guess == self.max_guesses - 1
    }

    fn clear_message(&mut self) {
        self.is_unknown = false;
        self.message = String::new();
    }

    fn set_game_end_message(&mut self) {
        if self.is_winner {
            if let GameMode::DailyWord(_) = self.game_mode {
                self.message = format!(
                    "Löysit päivän sanulin! {}",
                    SUCCESS_EMOJIS.choose(&mut rand::thread_rng()).unwrap()
                );
            } else {
                self.message = format!(
                    "Löysit sanan! {}",
                    SUCCESS_EMOJIS.choose(&mut rand::thread_rng()).unwrap()
                );
            }
        } else {
            self.message = format!("Sana oli \"{}\"", self.word.iter().collect::<String>());
        }
    }

    fn rehydrate(
        game_mode: GameMode,
        word_list: WordList,
        word_length: usize,
        allow_profanities: bool,
        word_lists: Rc<WordLists>,
    ) -> Result<Self, StorageError> {
        let game_key = &format!(
            "game|{}|{}|{}",
            serde_json::to_string(&game_mode).unwrap(),
            serde_json::to_string(&word_list).unwrap(),
            word_length
        );

        let mut game: Self = LocalStorage::get(game_key)?;
        game.allow_profanities = allow_profanities;
        game.word_lists = word_lists;

        game.refresh();

        Ok(game)
    }
}

impl Game for Sanuli {
    fn game_mode(&self) -> &GameMode {
        &self.game_mode
    }
    fn word_list(&self) -> &WordList {
        &self.word_list
    }
    fn word_length(&self) -> usize {
        self.word_length
    }
    fn max_guesses(&self) -> usize {
        self.max_guesses
    }
    fn boards(&self) -> Vec<Board> {
        let board = Board {
            guesses: self.guesses.clone(),
            current_guess: self.current_guess,
            is_guessing: self.is_guessing,
        };

        vec![board]
    }
    fn word(&self) -> Vec<char> {
        self.word.clone()
    }

    fn streak(&self) -> usize {
        self.streak
    }

    fn is_guessing(&self) -> bool {
        self.is_guessing
    }
    fn is_winner(&self) -> bool {
        self.is_winner
    }
    fn is_reset(&self) -> bool {
        self.is_reset
    }
    fn is_hidden(&self) -> bool {
        self.is_hidden
    }
    fn is_unknown(&self) -> bool {
        self.is_unknown
    }
    fn message(&self) -> String {
        self.message.clone()
    }
    fn previous_guesses(&self) -> Vec<Vec<(char, TileState)>> {
        self.previous_guesses.clone()
    }

    fn set_allow_profanities(&mut self, is_allowed: bool) {
        self.allow_profanities = is_allowed;
    }

    fn title(&self) -> String {
        if let GameMode::DailyWord(date) = self.game_mode {
            format!("Päivän sanuli #{}", Self::get_daily_word_index(date) + 1)
        } else if self.game_mode == GameMode::Shared {
            "Jaettu sanuli".to_owned()
        } else if self.streak > 0 {
            format!("Sanuli — Putki: {}", self.streak)
        } else {
            "Sanuli".to_owned()
        }
    }

    fn next_word(&mut self) {
        let next_word = Self::get_word(
            self.game_mode,
            self.word_list,
            self.word_length,
            self.allow_profanities,
            &self.word_lists,
        );

        let previous_word = mem::replace(&mut self.word, next_word);

        if previous_word.len() <= self.word_length {
            self.previous_guesses = mem::take(&mut self.guesses);
            if self.game_mode == GameMode::Relay && self.is_winner {
                self.previous_guesses.truncate(self.current_guess);
            } else {
                self.previous_guesses.truncate(self.current_guess + 1);
            }
        } else {
            let previous_guesses = mem::take(&mut self.guesses);
            self.previous_guesses = previous_guesses
                .into_iter()
                .map(|guess| guess.into_iter().take(self.word_length).collect())
                .collect();
            self.previous_guesses.truncate(self.current_guess);
        }

        self.guesses = Vec::with_capacity(self.max_guesses);

        self.known_states = std::iter::repeat(HashMap::new())
            .take(self.max_guesses)
            .collect::<Vec<_>>();
        self.known_counts = std::iter::repeat(HashMap::new())
            .take(self.max_guesses)
            .collect::<Vec<_>>();

        if previous_word.len() == self.word_length
            && self.is_winner
            && self.game_mode == GameMode::Relay
        {
            let empty_guesses = std::iter::repeat(Vec::with_capacity(self.word_length))
                .take(self.max_guesses - 1)
                .collect::<Vec<_>>();

            self.guesses.push(
                previous_word
                    .iter()
                    .map(|c| (*c, TileState::Unknown))
                    .collect(),
            );
            self.guesses.extend(empty_guesses);

            self.current_guess = 0;
            // Update the known states of the word from previous round
            logic::update_known_information(
                &mut self.known_states,
                &mut self.known_counts,
                &mut self.guesses[self.current_guess],
                self.current_guess,
                &self.word,
                self.max_guesses,
            );
            self.current_guess = 1;
        } else {
            self.guesses = std::iter::repeat(Vec::with_capacity(self.word_length))
                .take(self.max_guesses)
                .collect::<Vec<_>>();
            self.current_guess = 0;
        }

        self.is_guessing = true;
        self.is_winner = false;
        self.is_reset = true;
        self.clear_message();

        let _result = self.persist();
    }

    fn prepare_previous_guesses_animation(&mut self, previous_length: usize) {
        // For playing the animation populate previous_guesses.
        // This renders the previous game that slides and fades out.
        if previous_length <= self.word_length {
            self.previous_guesses = self.guesses.clone();
        } else {
            self.previous_guesses = self
                .guesses
                .iter()
                .cloned()
                .map(|guess| guess.into_iter().take(self.word_length).collect())
                .collect();
        }

        if self.current_guess < self.max_guesses - 1 {
            self.previous_guesses.truncate(self.current_guess);
        }
        self.is_reset = true;
    }

    fn keyboard_tilestate(&self, key: &char) -> KeyState {
        KeyState::Single(logic::keyboard_tile_state(
            key,
            self.current_guess,
            &self.known_states,
            &self.known_counts,
        ))
    }

    fn submit_guess(&mut self) {
        if !self.is_guess_correct_length() {
            self.message = "Liian vähän kirjaimia!".to_owned();
            return;
        }
        if !self.is_guess_accepted_word() {
            self.is_unknown = true;
            self.message = "Ei sanulistalla.".to_owned();
            return;
        }

        self.is_reset = false;
        self.clear_message();

        self.is_winner = self.is_correct_word();
        logic::update_known_information(
            &mut self.known_states,
            &mut self.known_counts,
            &mut self.guesses[self.current_guess],
            self.current_guess,
            &self.word,
            self.max_guesses,
        );
        if self.is_game_ended() {
            self.is_guessing = false;

            if matches!(self.game_mode, GameMode::DailyWord(_))
                || matches!(self.game_mode, GameMode::Shared)
            {
                // Do nothing, don't update streaks
            } else if self.is_winner {
                self.streak += 1;
            } else {
                self.streak = 0;
            }

            self.set_game_end_message();
        } else {
            self.current_guess += 1;
        }

        let _result = self.persist();
    }

    fn push_character(&mut self, character: char) {
        if !self.is_guessing || self.guesses[self.current_guess].len() >= self.word_length {
            return;
        }

        self.clear_message();

        // Display a hint of the tile state based on already known information
        let tile_state = logic::hint_tile_state(
            character,
            self.guesses[self.current_guess].len(),
            self.current_guess,
            &self.known_states,
            &self.known_counts,
        );
        self.guesses[self.current_guess].push((character, tile_state));
    }

    fn pop_character(&mut self) {
        if !self.is_guessing || self.guesses[self.current_guess].is_empty() {
            return;
        }

        self.clear_message();
        self.guesses[self.current_guess].pop();
    }

    fn share_emojis(&self, theme: Theme) -> Option<String> {
        let mut message = String::new();

        if let GameMode::DailyWord(date) = self.game_mode {
            let index = Self::get_daily_word_index(date) + 1;
            let guess_count = if self.is_winner {
                format!("{}", self.current_guess + 1)
            } else {
                "X".to_owned()
            };

            message += &format!("Sanuli #{} {}/{}", index, guess_count, self.max_guesses);
            message += "\n\n";

            for guess in self.guesses.iter() {
                if guess.is_empty() {
                    continue;
                }
                let guess_string = guess
                    .iter()
                    .map(|(_, state)| match state {
                        TileState::Correct => match theme {
                            Theme::Colorblind => "🟧",
                            _ => "🟩",
                        },
                        TileState::Present => match theme {
                            Theme::Colorblind => "🟦",
                            _ => "🟨",
                        },
                        TileState::Absent => "⬛",
                        TileState::Unknown => "⬜",
                    })
                    .collect::<String>();

                message += &guess_string;
                message += "\n";
            }
        }

        Some(message)
    }

    fn share_link(&self) -> Option<String> {
        let game_str = format!(
            "{}|{}",
            self.word.iter().collect::<String>(),
            self.guesses
                .iter()
                .flat_map(|guess| guess.iter().map(|(c, _)| c))
                .collect::<String>(),
        );
        let window: Window = window().expect("window not available");
        let share_str = window.btoa(&game_str).ok()?;

        let base_url = window.location().origin().ok()?;

        // Replace +/= at the base64 with URL safe characters
        let safe_str = share_str
            .replace("+", "-")
            .replace("/", ".")
            .replace("=", "_");

        return Some(format!("{}/?peli={}", base_url, safe_str));
    }

    fn reveal_hidden_tiles(&mut self) {
        self.is_hidden = false;
        self.message = format!("Sana oli \"{}\"", self.word.iter().collect::<String>());
    }

    fn reset(&mut self) {
        self.guesses = std::iter::repeat(Vec::with_capacity(self.word_length))
            .take(self.max_guesses)
            .collect::<Vec<_>>();

        self.current_guess = 0;

        self.is_guessing = true;
        self.is_winner = false;
        self.is_unknown = false;
        self.is_reset = false;
        self.is_hidden = false;
        self.message = "Peli nollattu, arvaa sanuli!".to_owned();

        self.known_states = std::iter::repeat(HashMap::new())
            .take(self.max_guesses)
            .collect::<Vec<_>>();

        self.known_counts = std::iter::repeat(HashMap::new())
            .take(self.max_guesses)
            .collect::<Vec<_>>();

        self.previous_guesses = Vec::new();
    }

    fn refresh(&mut self) {
        self.known_states = std::iter::repeat(HashMap::new())
            .take(self.max_guesses)
            .collect::<Vec<_>>();

        self.known_counts = std::iter::repeat(HashMap::new())
            .take(self.max_guesses)
            .collect::<Vec<_>>();

        // Rerun the game to refresh known_states and known_counts
        for guess_index in 0..self.current_guess {
            logic::update_known_information(
                &mut self.known_states,
                &mut self.known_counts,
                &mut self.guesses[guess_index],
                guess_index,
                &self.word,
                self.max_guesses,
            );
        }

        // If the game is ended also update the current guess
        if !self.is_guessing {
            logic::update_known_information(
                &mut self.known_states,
                &mut self.known_counts,
                &mut self.guesses[self.current_guess],
                self.current_guess,
                &self.word,
                self.max_guesses,
            );
        }
    }

    fn persist(&self) -> Result<(), StorageError> {
        if matches!(self.game_mode, GameMode::Shared | GameMode::Quadruple) {
            // Never persist shared or quadruple games
            return Ok(());
        }

        let game_key = &format!(
            "game|{}|{}|{}",
            serde_json::to_string(&self.game_mode).unwrap(),
            serde_json::to_string(&self.word_list).unwrap(),
            self.word_length
        );

        LocalStorage::set(game_key, self)
    }
}
