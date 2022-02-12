use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

use chrono::NaiveDate;
use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use web_sys::{window, Window};

pub type KnownStates = Vec<HashMap<(char, usize), CharacterState>>;
pub type KnownCounts = Vec<HashMap<char, CharacterCount>>;

use crate::logic;
use crate::manager::{
    CharacterCount, CharacterState, GameMode, Theme, TileState, WordList, WordLists,
};

const DAILY_WORDS: &str = include_str!("../daily-words.txt");
const SUCCESS_EMOJIS: [&str; 8] = ["🥳", "🤩", "🤗", "🎉", "😊", "😺", "😎", "👏"];
pub const DEFAULT_WORD_LENGTH: usize = 5;
pub const DEFAULT_MAX_GUESSES: usize = 6;
pub const DEFAULT_ALLOW_PROFANITIES: bool = false;

pub trait Game {
    fn title(&self) -> String;
    fn next_word(&mut self);
    fn keyboard_tilestate(&self, key: &char) -> TileState;
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

    fn boards(&self) -> Vec<Board>;
    fn streak(&self) -> usize;

    fn is_guessing(&self) -> bool;
    fn is_reset(&self) -> bool;
    fn is_hidden(&self) -> bool;
    fn is_winner(&self) -> bool;
    fn is_unknown(&self) -> bool;

    fn message(&self) -> String;

    fn previous_guesses(&self) -> &Vec<Vec<(char, TileState)>>;
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
    pub word: Vec<char>,
    pub guesses: Vec<Vec<(char, TileState)>>,
    pub current_guess: usize,
    pub is_guessing: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseGame {
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
    known_states: KnownStates,
    #[serde(skip)]
    known_counts: KnownCounts,
}

impl Default for BaseGame {
    fn default() -> Self {
        BaseGame::new(
            GameMode::default(),
            WordList::default(),
            DEFAULT_WORD_LENGTH,
            DEFAULT_ALLOW_PROFANITIES,
            Rc::new(HashMap::new()),
        )
    }
}

impl BaseGame {
    pub fn new(
        game_mode: GameMode,
        word_list: WordList,
        word_length: usize,
        allow_profanities: bool,
        word_lists: Rc<WordLists>,
    ) -> Self {
        let max_guesses = DEFAULT_MAX_GUESSES;

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

    fn tile_state(&mut self, character: char, guess_index: usize) -> TileState {
        match self.known_states[self.current_guess].get(&(character, guess_index)) {
            Some(CharacterState::Correct) => TileState::Correct,
            Some(CharacterState::Absent) => TileState::Absent,
            _ => {
                match self.known_counts[self.current_guess].get(&character) {
                    Some(CharacterCount::Exactly(count)) => {
                        // We may know the exact count, but not the exact index of any characters..
                        if *count == 0 {
                            return TileState::Absent;
                        }

                        let is_every_correct_found = self.known_states[self.current_guess]
                            .iter()
                            .filter(|((c, _i), state)| {
                                c == &character && *state == &CharacterState::Correct
                            })
                            .count()
                            == *count;

                        if !is_every_correct_found {
                            return TileState::Present;
                        }

                        TileState::Absent
                    }
                    Some(CharacterCount::AtLeast(_)) => TileState::Present,
                    None => TileState::Unknown,
                }
            }
        }
    }

    fn update_known_states(&mut self, row: usize) {
        if let Some(guess) = self.guesses.get_mut(row) {
            let mut revealed_count_on_row: HashMap<char, usize> =
                HashMap::with_capacity(self.word_length);

            for (index, (character, _)) in guess.iter().enumerate() {
                if let Some(CharacterState::Correct) =
                    self.known_states[row].get(&(*character, index))
                {
                    revealed_count_on_row
                        .entry(*character)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
            }

            for (index, (character, tile_state)) in guess.iter_mut().enumerate() {
                match self.known_states[row].get(&(*character, index)) {
                    Some(CharacterState::Correct) => {
                        *tile_state = TileState::Correct;
                    }
                    Some(CharacterState::Absent) => {
                        let revealed = revealed_count_on_row
                            .entry(*character)
                            .and_modify(|count| *count += 1)
                            .or_insert(1);

                        let discovered_count = self.known_counts[row]
                            .get(character)
                            .unwrap_or(&CharacterCount::AtLeast(0));

                        match discovered_count {
                            CharacterCount::AtLeast(count) | CharacterCount::Exactly(count) => {
                                if *revealed <= *count {
                                    *tile_state = TileState::Present;
                                } else {
                                    *tile_state = TileState::Absent;
                                }
                            }
                        }
                    }
                    _ => {
                        *tile_state = TileState::Unknown;
                    }
                }
            }
        }
    }

    fn update_states_and_counts(&mut self) {
        for (index, (character, _)) in self.guesses[self.current_guess].iter().enumerate() {
            let known = self.known_states[self.current_guess]
                .entry((*character, index))
                .or_insert(CharacterState::Unknown);

            if self.word[index] == *character {
                *known = CharacterState::Correct;
            } else {
                *known = CharacterState::Absent;

                if let Some(updated_count) = logic::updated_known_count(
                    character,
                    self.current_guess,
                    &self.guesses[self.current_guess],
                    &self.known_counts,
                    &self.word,
                ) {
                    self.known_counts[self.current_guess].insert(*character, updated_count);
                }
            }
        }

        // Copy the previous knowledge to the next round
        if self.current_guess < self.max_guesses - 1 {
            let next = self.current_guess + 1;
            self.known_states[next] = self.known_states[self.current_guess].clone();
            self.known_counts[next] = self.known_counts[self.current_guess].clone();
        }

        self.update_known_states(self.current_guess);
    }

    fn is_guess_allowed(&self) -> bool {
        self.is_guessing && self.guesses[self.current_guess].len() == self.word_length
    }

    fn is_guess_real_word(&self) -> bool {
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

    fn is_game_ended(&self) -> bool {
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

impl Game for BaseGame {
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
            word: self.word.clone(),
            guesses: self.guesses.clone(),
            current_guess: self.current_guess,
            is_guessing: self.is_guessing,
        };

        vec![board]
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
    fn previous_guesses(&self) -> &Vec<Vec<(char, TileState)>> {
        &self.previous_guesses
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
            "Sanuli — Putki: {}".to_owned()
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
            .take(DEFAULT_MAX_GUESSES)
            .collect::<Vec<_>>();
        self.known_counts = std::iter::repeat(HashMap::new())
            .take(DEFAULT_MAX_GUESSES)
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
            self.update_states_and_counts();
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

    fn keyboard_tilestate(&self, key: &char) -> TileState {
        logic::keyboard_tilestate(
            key,
            self.current_guess,
            &self.known_states,
            &self.known_counts,
        )
    }

    fn submit_guess(&mut self) {
        if !self.is_guess_allowed() {
            self.message = "Liian vähän kirjaimia!".to_owned();
            return;
        }
        if !self.is_guess_real_word() {
            self.is_unknown = true;
            self.message = "Ei sanulistalla.".to_owned();
            return;
        }

        self.is_reset = false;
        self.clear_message();

        self.is_winner = self.is_correct_word();
        self.update_states_and_counts();
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

        let tile_state = self.tile_state(character, self.guesses[self.current_guess].len());
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

        let current_guess = self.current_guess;
        // Rerun the game to repuplate known_states and known_counts
        for guess_index in 0..self.current_guess {
            self.current_guess = guess_index;
            self.update_states_and_counts();
        }

        // Restore the current guess
        self.current_guess = current_guess;

        // If the game is ended also recalculate the current guess
        if !self.is_guessing {
            self.update_states_and_counts();
        }
    }

    fn persist(&self) -> Result<(), StorageError> {
        if self.game_mode == GameMode::Shared {
            // Never persist shared games
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
