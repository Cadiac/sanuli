use rand::seq::SliceRandom;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::mem;
use std::rc::Rc;
use std::str::FromStr;

use chrono::{Local, NaiveDate};
use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use serde::{Deserialize, Serialize};

use crate::migration;

const FULL_WORDS: &str = include_str!("../full-words.txt");
const COMMON_WORDS: &str = include_str!("../common-words.txt");
const DAILY_WORDS: &str = include_str!("../daily-words.txt");
const PROFANITIES: &str = include_str!("../profanities.txt");
const SUCCESS_EMOJIS: [&str; 8] = ["🥳", "🤩", "🤗", "🎉", "😊", "😺", "😎", "👏"];
pub const EMPTY: char = '\u{00a0}'; // &nbsp;
pub const DEFAULT_WORD_LENGTH: usize = 5;
pub const DEFAULT_MAX_GUESSES: usize = 6;
pub const DAILY_WORD_LEN: usize = 5;

type WordLists = HashMap<(WordList, usize), HashSet<Vec<char>>>;

fn parse_all_words() -> Rc<WordLists> {
    let mut word_lists: HashMap<(WordList, usize), HashSet<Vec<char>>> = HashMap::with_capacity(3);
    for word in FULL_WORDS.lines() {
        let chars = word.chars();
        let word_length = chars.clone().count();
        word_lists
            .entry((WordList::Full, word_length))
            .or_insert(HashSet::new())
            .insert(chars.collect());
    }

    for word in COMMON_WORDS.lines() {
        let chars = word.chars();
        let word_length = chars.clone().count();
        word_lists
            .entry((WordList::Common, word_length))
            .or_insert(HashSet::new())
            .insert(chars.collect());
    }

    for word in PROFANITIES.lines() {
        let chars = word.chars();
        let word_length = chars.clone().count();
        word_lists
            .entry((WordList::Profanities, word_length))
            .or_insert(HashSet::new())
            .insert(chars.collect());
    }

    Rc::new(word_lists)
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum WordList {
    Full,
    Common,
    Profanities,
    Daily,
}

impl Default for WordList {
    fn default() -> Self {
        WordList::Common
    }
}

impl FromStr for WordList {
    type Err = ();

    fn from_str(input: &str) -> Result<WordList, Self::Err> {
        match input {
            "full" => Ok(WordList::Full),
            "common" => Ok(WordList::Common),
            "profanities" => Ok(WordList::Profanities),
            "daily" => Ok(WordList::Daily),
            _ => Err(()),
        }
    }
}

impl fmt::Display for WordList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WordList::Full => write!(f, "full"),
            WordList::Common => write!(f, "common"),
            WordList::Profanities => write!(f, "profanities"),
            WordList::Daily => write!(f, "daily"),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum GameMode {
    Classic,
    Relay,
    DailyWord(NaiveDate),
}

impl Default for GameMode {
    fn default() -> Self {
        GameMode::Classic
    }
}

impl FromStr for GameMode {
    type Err = ();

    fn from_str(input: &str) -> Result<GameMode, Self::Err> {
        match input {
            "classic" => Ok(GameMode::Classic),
            "relay" => Ok(GameMode::Relay),
            "daily_word" => {
                let today = Local::now().naive_local().date();
                Ok(GameMode::DailyWord(today))
            }
            _ => Err(()),
        }
    }
}

impl fmt::Display for GameMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameMode::Classic => write!(f, "classic"),
            GameMode::Relay => write!(f, "relay"),
            GameMode::DailyWord(_) => write!(f, "daily_word"),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Colorblind,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

impl FromStr for Theme {
    type Err = ();

    fn from_str(input: &str) -> Result<Theme, Self::Err> {
        match input {
            "dark" => Ok(Theme::Dark),
            "colorblind" => Ok(Theme::Colorblind),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Theme::Dark => write!(f, "dark"),
            Theme::Colorblind => write!(f, "colorblind"),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum CharacterState {
    Correct,
    Absent,
    Unknown,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum TileState {
    Correct,
    Absent,
    Present,
    Unknown,
}

impl fmt::Display for TileState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TileState::Correct => write!(f, "correct"),
            TileState::Absent => write!(f, "absent"),
            TileState::Present => write!(f, "present"),
            TileState::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct DailyWordHistory {
    date: NaiveDate,
    word: String,
    guesses: Vec<Vec<char>>,
    current_guess: usize,
    is_guessing: bool,
    is_winner: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum CharacterCount {
    AtLeast(usize),
    Exactly(usize),
}

#[derive(Clone, PartialEq)]
pub struct State {
    pub game_manager: Rc<RefCell<GameManager>>,
    pub game: Game,
    pub background_games: HashMap<(GameMode, WordList, usize), Game>,
}

impl State {
    pub fn new() -> Self {
        // Attempt to rehydrate old game manager from localStorage
        if let Ok(rehydrated) = GameManager::rehydrate() {
            let game_manager = Rc::new(RefCell::new(rehydrated));

            let game = Game::new_or_rehydrate(
                game_manager.borrow().current_game_mode,
                game_manager.borrow().current_word_list,
                game_manager.borrow().current_word_length,
                game_manager.clone(),
            );

            return Self {
                game_manager,
                game,
                background_games: HashMap::new(),
            };
        } else {
            // Otherwise either create everything from scratch or recover some data from legacy storage state
            let game_manager = Rc::new(RefCell::new(GameManager::new()));
            let background_games = HashMap::new();
            let game = Game::new(
                GameMode::Classic,
                WordList::Common,
                DEFAULT_WORD_LENGTH,
                game_manager.clone(),
            );

            let mut state = Self {
                game_manager,
                game,
                background_games,
            };

            // Try to migrate old settings and stats from localStorage to current format
            // TODO: Doesn't do anything if the old state isn't present, but get rid of this at some point
            let _res = migration::migrate_state(&mut state);
            state.switch_active_game();

            // Try to migrate old game streak from localStorage to current format, if the game mode is not daily
            // TODO: Doesn't do anything if the old state isn't present, but get rid of this at some point
            let _res = migration::migrate_game(&mut state.game);

            let _res = state.game_manager.borrow().persist();
            let _res = state.game.persist();

            return state;
        };
    }

    pub fn change_word_length(&mut self, new_length: usize) {
        if self.game_manager.borrow().current_word_length == new_length {
            return;
        }

        self.game_manager
            .borrow_mut()
            .change_word_length(new_length);
        self.switch_active_game();
        let _res = self.game_manager.borrow_mut().persist();
        let _res = self.game.persist();
    }

    pub fn change_game_mode(&mut self, new_mode: GameMode) {
        if self.game_manager.borrow().current_game_mode == new_mode {
            return;
        }

        if matches!(
            self.game_manager.borrow().current_game_mode,
            GameMode::DailyWord(_)
        ) {
            let previous_game = self.game_manager.borrow().previous_game.clone();
            self.game_manager
                .borrow_mut()
                .change_word_list(previous_game.1);
            self.game_manager
                .borrow_mut()
                .change_word_length(previous_game.2);
        }

        if matches!(new_mode, GameMode::DailyWord(_)) {
            self.game_manager
                .borrow_mut()
                .change_word_list(WordList::Daily);
            self.game_manager
                .borrow_mut()
                .change_word_length(DAILY_WORD_LEN);
        }

        self.game_manager.borrow_mut().change_game_mode(new_mode);
        self.switch_active_game();
        let _res = self.game_manager.borrow_mut().persist();
        let _res = self.game.persist();
    }

    pub fn change_word_list(&mut self, new_list: WordList) {
        if self.game_manager.borrow().current_word_list == new_list {
            return;
        }

        self.game_manager.borrow_mut().change_word_list(new_list);
        self.switch_active_game();
        let _res = self.game_manager.borrow_mut().persist();
        let _res = self.game.persist();
    }

    pub fn change_previous_game_mode(&mut self) {
        let (game_mode, word_list, word_length) = self.game_manager.borrow().previous_game;

        self.game_manager.borrow_mut().change_game_mode(game_mode);
        self.game_manager.borrow_mut().change_word_list(word_list);
        self.game_manager
            .borrow_mut()
            .change_word_length(word_length);
        self.switch_active_game();

        let _res = self.game_manager.borrow_mut().persist();
        let _res = self.game.persist();
    }

    pub fn switch_active_game(&mut self) -> bool {
        let next_game = (
            self.game_manager.borrow().current_game_mode,
            self.game_manager.borrow().current_word_list,
            self.game_manager.borrow().current_word_length,
        );

        let previous_game = (
            self.game.game_mode,
            self.game.word_list,
            self.game.word_length,
        );

        if next_game.0 == previous_game.0
            && next_game.1 == previous_game.1
            && next_game.2 == previous_game.2
        {
            return false;
        }

        self.game_manager.borrow_mut().previous_game = previous_game;

        // Restore a suspended game or create a new one
        let mut game = self
            .background_games
            .remove(&next_game)
            .unwrap_or(Game::new_or_rehydrate(
                next_game.0,
                next_game.1,
                next_game.2,
                self.game_manager.clone(),
            ));

        // For playing the animation populate previous_guesses
        if previous_game.2 <= next_game.2 {
            game.previous_guesses = self.game.guesses.clone();
        } else {
            game.previous_guesses = self
                .game
                .guesses
                .iter()
                .cloned()
                .map(|guess| guess.into_iter().take(game.word_length).collect())
                .collect();
        }

        if self.game.current_guess < game.max_guesses - 1 {
            game.previous_guesses.truncate(self.game.current_guess);
        }
        game.is_reset = true;

        self.background_games
            .insert(previous_game, mem::replace(&mut self.game, game));

        true
    }
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GameManager {
    #[serde(skip)]
    pub word_lists: Rc<WordLists>,

    pub allow_profanities: bool,
    pub current_game_mode: GameMode,
    pub current_word_list: WordList,
    pub current_word_length: usize,

    pub previous_game: (GameMode, WordList, usize),

    pub current_max_guesses: usize,

    pub theme: Theme,

    pub max_streak: usize,
    pub total_played: usize,
    pub total_solved: usize,
}

impl GameManager {
    pub fn new() -> Self {
        let word_lists = parse_all_words();
        let current_word_list = WordList::Common;
        let current_word_length = DEFAULT_WORD_LENGTH;
        let current_max_guesses = DEFAULT_MAX_GUESSES;
        let allow_profanities = false;

        Self {
            word_lists,
            current_word_list,
            allow_profanities,

            current_max_guesses,

            current_game_mode: GameMode::Classic,
            previous_game: (GameMode::Classic, WordList::Common, DEFAULT_WORD_LENGTH),
            current_word_length,

            theme: Theme::Dark,

            max_streak: 0,
            total_played: 0,
            total_solved: 0,
        }
    }

    pub fn get_random_word(&self, word_list: WordList, word_length: usize) -> Vec<char> {
        let mut words = self
            .word_lists
            .get(&(word_list, word_length))
            .unwrap()
            .iter()
            .collect::<Vec<_>>();

        if !self.allow_profanities {
            if let Some(profanities) = self
                .word_lists
                .get(&(WordList::Profanities, self.current_word_length))
            {
                words.retain(|word| !profanities.contains(*word));
            }
        }

        let chosen = words.choose(&mut rand::thread_rng()).unwrap();
        (*chosen).clone()
    }

    pub fn get_daily_word_index(&self, date: NaiveDate) -> usize {
        let epoch = NaiveDate::from_ymd(2022, 1, 07); // Epoch of the daily word mode, index 0
        date.signed_duration_since(epoch).num_days() as usize
    }

    pub fn get_daily_word(&self, date: NaiveDate) -> Vec<char> {
        DAILY_WORDS
            .lines()
            .nth(self.get_daily_word_index(date))
            .unwrap()
            .chars()
            .collect()
    }

    fn update_game_statistics(&mut self, is_winner: bool, streak: usize) {
        self.total_played += 1;

        if is_winner {
            self.total_solved += 1;

            if streak > self.max_streak {
                self.max_streak = streak;
            }
        }
    }

    fn change_word_length(&mut self, new_length: usize) {
        self.current_word_length = new_length;
    }

    fn change_game_mode(&mut self, new_mode: GameMode) {
        self.current_game_mode = new_mode;
    }

    fn change_word_list(&mut self, new_list: WordList) {
        self.current_word_list = new_list;
    }

    pub fn change_allow_profanities(&mut self, is_allowed: bool) {
        self.allow_profanities = is_allowed;
        let _result = self.persist();
    }

    pub fn change_theme(&mut self, theme: Theme) -> bool {
        self.theme = theme;
        let _result = self.persist();
        true
    }

    fn persist(&self) -> Result<(), StorageError> {
        LocalStorage::set("settings", self)
    }

    fn rehydrate() -> Result<GameManager, StorageError> {
        let mut game_manager: GameManager = LocalStorage::get("settings")?;
        game_manager.word_lists = parse_all_words();
        Ok(game_manager)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Game {
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
    pub message: String,

    #[serde(skip)]
    pub game_manager: Rc<RefCell<GameManager>>,
    #[serde(skip)]
    pub known_states: Vec<HashMap<(char, usize), CharacterState>>,
    #[serde(skip)]
    pub discovered_counts: Vec<HashMap<char, CharacterCount>>,
    #[serde(skip)]
    pub previous_guesses: Vec<Vec<(char, TileState)>>,
}

impl Game {
    pub fn new(
        game_mode: GameMode,
        word_list: WordList,
        word_length: usize,
        game_manager: Rc<RefCell<GameManager>>,
    ) -> Self {
        let max_guesses = DEFAULT_MAX_GUESSES;

        let guesses = std::iter::repeat(Vec::with_capacity(word_length))
            .take(max_guesses)
            .collect::<Vec<_>>();

        let known_states = std::iter::repeat(HashMap::new())
            .take(max_guesses)
            .collect::<Vec<_>>();

        let discovered_counts = std::iter::repeat(HashMap::new())
            .take(max_guesses)
            .collect::<Vec<_>>();

        let word = if let GameMode::DailyWord(date) = game_mode {
            game_manager.borrow().get_daily_word(date)
        } else {
            game_manager
                .borrow()
                .get_random_word(word_list, word_length)
        };

        Self {
            game_mode,
            word_list,
            word_length,
            max_guesses,
            word,
            is_guessing: true,
            is_winner: false,
            is_unknown: false,
            is_reset: false,
            message: EMPTY.to_string(),
            known_states,
            discovered_counts,
            guesses,
            previous_guesses: Vec::new(),
            current_guess: 0,

            game_manager,

            streak: 0,
        }
    }

    pub fn new_or_rehydrate(
        game_mode: GameMode,
        word_list: WordList,
        word_length: usize,
        game_manager: Rc<RefCell<GameManager>>,
    ) -> Self {
        if let Ok(game) = Game::rehydrate(game_mode, word_list, word_length, game_manager.clone()) {
            game
        } else {
            Game::new(game_mode, word_list, word_length, game_manager.clone())
        }
    }

    pub fn next_word(&mut self) -> bool {
        let next_word =
            if let GameMode::DailyWord(date) = self.game_manager.borrow().current_game_mode {
                self.game_manager.borrow().get_daily_word(date)
            } else {
                self.game_manager.borrow().get_random_word(
                    self.game_manager.borrow().current_word_list,
                    self.game_manager.borrow().current_word_length,
                )
            };

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
        self.discovered_counts = std::iter::repeat(HashMap::new())
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
            self.calculate_current_guess();
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

        true
    }

    pub fn keyboard_tilestate(&self, key: &char) -> TileState {
        let is_correct = self.known_states[self.current_guess]
            .iter()
            .any(|((c, _index), state)| c == key && state == &CharacterState::Correct);
        if is_correct {
            return TileState::Correct;
        }

        match self.discovered_counts[self.current_guess].get(key) {
            Some(CharacterCount::AtLeast(count)) => {
                if *count == 0 {
                    return TileState::Unknown;
                }
                TileState::Present
            }
            Some(CharacterCount::Exactly(count)) => {
                if *count == 0 {
                    return TileState::Absent;
                }
                TileState::Present
            }
            None => TileState::Unknown,
        }
    }

    fn current_guess_state(&mut self, character: char, index: usize) -> TileState {
        match self.known_states[self.current_guess].get(&(character, index)) {
            Some(CharacterState::Correct) => TileState::Correct,
            Some(CharacterState::Absent) => TileState::Absent,
            _ => {
                match self.discovered_counts[self.current_guess].get(&character) {
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

    fn reveal_row_tiles(&mut self, row: usize) {
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

            for (index, (character, tile_state)) in guess.into_iter().enumerate() {
                match self.known_states[row].get(&(*character, index)) {
                    Some(CharacterState::Correct) => {
                        *tile_state = TileState::Correct;
                    }
                    Some(CharacterState::Absent) => {
                        let revealed = revealed_count_on_row
                            .entry(*character)
                            .and_modify(|count| *count += 1)
                            .or_insert(1);

                        let discovered_count = self.discovered_counts[row]
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

    pub fn calculate_current_guess(&mut self) {
        for (index, (character, _)) in self.guesses[self.current_guess].iter().enumerate() {
            let known = self.known_states[self.current_guess]
                .entry((*character, index))
                .or_insert(CharacterState::Unknown);

            if self.word[index] == *character {
                *known = CharacterState::Correct;
            } else {
                *known = CharacterState::Absent;

                let discovered_count = self.discovered_counts[self.current_guess]
                    .entry(*character)
                    .or_insert(CharacterCount::AtLeast(0));

                // At most the same amount of characters are highlighted as there are in the word
                let count_in_word = self.word.iter().filter(|c| *c == character).count();
                if count_in_word == 0 {
                    *discovered_count = CharacterCount::Exactly(0);
                    continue;
                }

                let count_in_guess = self.guesses[self.current_guess]
                    .iter()
                    .filter(|(c, _)| c == character)
                    .count();

                match discovered_count {
                    CharacterCount::AtLeast(count) => {
                        if count_in_guess > count_in_word {
                            if count_in_word >= *count {
                                // The guess had more copies of the character than the word,
                                // the exact count is revealed
                                *discovered_count = CharacterCount::Exactly(count_in_word);
                            }
                        } else if count_in_guess == count_in_word {
                            // The count had the exact count but that isn't revealed yet
                            *discovered_count = CharacterCount::AtLeast(count_in_guess);
                        } else if count_in_guess > *count {
                            // Found more, but the exact count is still unknown
                            *discovered_count = CharacterCount::AtLeast(count_in_guess);
                        }
                    }
                    // Exact count should never change
                    CharacterCount::Exactly(_) => {}
                }
            }
        }

        // Copy the previous knowledge to the next round
        if self.current_guess < self.max_guesses - 1 {
            let next = self.current_guess + 1;
            self.known_states[next] = self.known_states[self.current_guess].clone();
            self.discovered_counts[next] = self.discovered_counts[self.current_guess].clone();
        }

        self.reveal_row_tiles(self.current_guess);
    }

    pub fn push_character(&mut self, character: char) -> bool {
        if !self.is_guessing || self.guesses[self.current_guess].len() >= self.word_length {
            return false;
        }

        self.clear_message();

        let tile_state =
            self.current_guess_state(character, self.guesses[self.current_guess].len());
        self.guesses[self.current_guess].push((character, tile_state));
        true
    }

    pub fn pop_character(&mut self) -> bool {
        if !self.is_guessing || self.guesses[self.current_guess].is_empty() {
            return false;
        }

        self.clear_message();
        self.guesses[self.current_guess].pop();

        true
    }

    fn is_guess_allowed(&self) -> bool {
        self.is_guessing && self.guesses[self.current_guess].len() == self.word_length
    }

    fn is_guess_real_word(&self) -> bool {
        match self
            .game_manager
            .borrow()
            .word_lists
            .get(&(WordList::Full, self.word_length))
        {
            Some(list) => {
                let word: &Vec<char> = &self.guesses[self.current_guess]
                    .iter()
                    .map(|(c, _)| *c)
                    .collect();

                return list.contains(word);
            }
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
        self.message = EMPTY.to_string();
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

    pub fn submit_guess(&mut self) -> bool {
        if !self.is_guess_allowed() {
            self.message = "Liian vähän kirjaimia!".to_owned();
            return true;
        }
        if !self.is_guess_real_word() {
            self.is_unknown = true;
            self.message = "Ei sanulistalla.".to_owned();
            return true;
        }

        self.is_reset = false;
        self.clear_message();

        self.is_winner = self.is_correct_word();
        self.calculate_current_guess();
        if self.is_game_ended() {
            self.is_guessing = false;

            if let GameMode::DailyWord(_) = self.game_mode {
                // Do nothing?
            } else {
                if self.is_winner {
                    self.streak += 1;
                } else {
                    self.streak = 0;
                }

                self.game_manager
                    .borrow_mut()
                    .update_game_statistics(self.is_winner, self.streak);
            }

            self.set_game_end_message();

            let _result = self.game_manager.borrow().persist();
        } else {
            self.current_guess += 1;
        }

        let _result = self.persist();

        true
    }

    pub fn persist(&self) -> Result<(), StorageError> {
        let game_key = &format!(
            "game|{}|{}|{}",
            serde_json::to_string(&self.game_mode).unwrap(),
            serde_json::to_string(&self.word_list).unwrap(),
            self.word_length
        );

        LocalStorage::set(game_key, self)
    }

    fn rehydrate(
        game_mode: GameMode,
        word_list: WordList,
        word_length: usize,
        game_manager: Rc<RefCell<GameManager>>,
    ) -> Result<Game, StorageError> {
        let game_key = &format!(
            "game|{}|{}|{}",
            serde_json::to_string(&game_mode).unwrap(),
            serde_json::to_string(&word_list).unwrap(),
            word_length
        );

        let mut game: Game = LocalStorage::get(game_key)?;
        game.game_manager = game_manager;

        game.known_states = std::iter::repeat(HashMap::new())
            .take(game.max_guesses)
            .collect::<Vec<_>>();

        game.discovered_counts = std::iter::repeat(HashMap::new())
            .take(game.max_guesses)
            .collect::<Vec<_>>();

        let current_guess = game.current_guess;
        // Rerrun the game to repuplate known_states and discovered_counts
        for guess_index in 0..game.current_guess {
            game.current_guess = guess_index;
            game.calculate_current_guess();
        }

        // Restore the current guess
        game.current_guess = current_guess;

        // If the game is ended also recalculate the current guess
        if !game.is_guessing {
            game.calculate_current_guess();
        }

        Ok(game)
    }
}
