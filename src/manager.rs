use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::mem;
use std::rc::Rc;
use std::str::FromStr;

use chrono::{Local, NaiveDate};
use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use web_sys::{window, Window};
use wasm_bindgen::{JsValue};

use crate::migration;
use crate::game::{Game};

const FULL_WORDS: &str = include_str!("../full-words.txt");
const COMMON_WORDS: &str = include_str!("../common-words.txt");
const PROFANITIES: &str = include_str!("../profanities.txt");

pub const EMPTY: char = '\u{00a0}'; // &nbsp;
pub const DEFAULT_WORD_LENGTH: usize = 5;
pub const DEFAULT_MAX_GUESSES: usize = 6;
pub const DEFAULT_ALLOW_PROFANITIES: bool = false;
pub const DAILY_WORD_LEN: usize = 5;

pub type WordLists = HashMap<(WordList, usize), HashSet<Vec<char>>>;

fn parse_all_words() -> Rc<WordLists> {
    let mut word_lists: HashMap<(WordList, usize), HashSet<Vec<char>>> = HashMap::with_capacity(3);
    for word in FULL_WORDS.lines() {
        let chars = word.chars();
        let word_length = chars.clone().count();
        word_lists
            .entry((WordList::Full, word_length))
            .or_insert_with(HashSet::new)
            .insert(chars.collect());
    }

    for word in COMMON_WORDS.lines() {
        let chars = word.chars();
        let word_length = chars.clone().count();
        word_lists
            .entry((WordList::Common, word_length))
            .or_insert_with(HashSet::new)
            .insert(chars.collect());
    }

    for word in PROFANITIES.lines() {
        let chars = word.chars();
        let word_length = chars.clone().count();
        word_lists
            .entry((WordList::Profanities, word_length))
            .or_insert_with(HashSet::new)
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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum GameMode {
    Classic,
    Relay,
    DailyWord(NaiveDate),
    Shared,
}

impl Default for GameMode {
    fn default() -> Self {
        GameMode::Classic
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
pub enum CharacterCount {
    AtLeast(usize),
    Exactly(usize),
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Manager {
    pub allow_profanities: bool,
    pub current_game_mode: GameMode,
    pub current_word_list: WordList,
    pub current_word_length: usize,
    pub current_max_guesses: usize,

    pub previous_game: (GameMode, WordList, usize),

    pub theme: Theme,

    pub max_streak: usize,
    pub total_played: usize,
    pub total_solved: usize,

    #[serde(skip)]
    pub game: Game,
    #[serde(skip)]
    pub background_games: HashMap<(GameMode, WordList, usize), Game>,
    #[serde(skip)]
    pub word_lists: Rc<WordLists>,
}

impl Default for Manager {
    fn default() -> Self {
        Self {
            allow_profanities: DEFAULT_ALLOW_PROFANITIES,
            current_game_mode: GameMode::default(),
            current_word_list: WordList::default(),
            current_word_length: DEFAULT_WORD_LENGTH,
            current_max_guesses: DEFAULT_MAX_GUESSES,

            previous_game: (
                GameMode::default(),
                WordList::default(),
                DEFAULT_WORD_LENGTH,
            ),

            theme: Theme::default(),

            max_streak: 0,
            total_played: 0,
            total_solved: 0,

            game: Game::default(),
            background_games: HashMap::new(),
            word_lists: Rc::new(HashMap::new()),
        }
    }
}

impl Manager {
    pub fn new() -> Self {
        let word_lists = parse_all_words();

        // Attempt to rehydrate manager from localStorage
        let mut initial_manager = if let Ok(mut manager) = Manager::rehydrate() {
            if let GameMode::DailyWord(date) = manager.current_game_mode {
                let today = Local::today().naive_local();

                if date < today {
                    // Page was refreshed after the day changed - rehydrate the daily word of today
                    manager.current_game_mode = GameMode::DailyWord(today);
                }
            }

            let game = Game::new_or_rehydrate(
                manager.current_game_mode,
                manager.current_word_list,
                manager.current_word_length,
                manager.allow_profanities,
                word_lists.clone(),
            );

            manager.game = game;
            manager.word_lists = word_lists;

            manager
        } else {
            // Otherwise either create everything from scratch or recover some data from legacy storage manager
            let game = Game::new(
                GameMode::Classic,
                WordList::Common,
                DEFAULT_WORD_LENGTH,
                DEFAULT_ALLOW_PROFANITIES,
                word_lists.clone(),
            );

            let mut manager = Self {
                game,
                word_lists,
                ..Self::default()
            };

            // Try to migrate old settings and stats from localStorage to current format
            // TODO: Get rid of this at some point, noop if already migrated or there's no data
            let _res = migration::migrate_settings_and_stats(&mut manager);
            manager.switch_active_game();

            // Try to migrate old game streak from localStorage to current format, if the game mode is not daily
            // TODO: Get rid of this at some point, noop if already migrated or there's no data
            let _res = migration::migrate_game(&mut manager.game);

            let _res = manager.persist();
            let _res = manager.game.persist();

            manager
        };

        // IF this is a shared game switch to it immediately. Set the game we were going to display in the background
        if let Some(game) = initial_manager.rehydrate_shared_game() {
            initial_manager.current_game_mode = game.game_mode;
            initial_manager.current_word_length = game.word_length;
            initial_manager.current_word_list = game.word_list;

            initial_manager.background_games.insert((game.game_mode, game.word_list, game.word_length), game);

            initial_manager.switch_active_game();
        }

        initial_manager
    }

    fn rehydrate_shared_game(&self) -> Option<Game> {
        let window: Window = window().expect("window not available");
        let qs = window.location().search().ok()?;
        if qs.is_empty() {
            return None;
        }

        // Skip the leading "?"
        for param in qs.chars().skip(1).collect::<String>().split("&") {
            let mut parts = param.split("=");

            let key = parts.next()?;
            let value = parts.next()?;

            if key == "game" && !value.is_empty() {
                let game_str = window
                    .atob(value)
                    .ok()?;

                let game = Game::from_shared_link(&game_str, self.word_lists.clone());

                // Remove the query string
                window.history().ok()?.replace_state_with_url(&JsValue::null(), "", Some("/")).ok()?;

                return game;
            }
        }

        return None;
    }

    pub fn submit_guess(&mut self) -> bool {
        self.game.submit_guess();
        if !self.game.is_guessing {
            self.update_game_statistics(self.game.is_winner, self.game.streak);
        }

        true
    }

    pub fn change_word_length(&mut self, new_length: usize) {
        if self.current_word_length == new_length {
            return;
        }

        self.current_word_length = new_length;
        self.switch_active_game();

        let _res = self.persist();
        let _res = self.game.persist();
    }

    pub fn change_game_mode(&mut self, new_mode: GameMode) {
        if self.current_game_mode == new_mode {
            return;
        }

        if matches!(self.current_game_mode, GameMode::DailyWord(_)) {
            self.current_word_list = self.previous_game.1;
            self.current_word_length = self.previous_game.2;
        }

        if matches!(new_mode, GameMode::DailyWord(_)) {
            self.current_word_list = WordList::Daily;
            self.current_word_length = DAILY_WORD_LEN;
        } else if self.current_word_list == WordList::Daily {
            // Prevent getting stuck in non-daily word gamemode with
            // daily list somehow, for instance by having a daily game as
            // the previous game in manager state
            self.current_word_list = WordList::default();
        }

        self.current_game_mode = new_mode;
        self.switch_active_game();
        let _res = self.persist();
        let _res = self.game.persist();
    }

    pub fn change_word_list(&mut self, new_list: WordList) {
        if self.current_word_list == new_list {
            return;
        }

        self.current_word_list = new_list;
        self.switch_active_game();

        let _res = self.persist();
        let _res = self.game.persist();
    }

    pub fn change_previous_game_mode(&mut self) {
        let (game_mode, word_list, word_length) = self.previous_game;

        if matches!(game_mode, GameMode::DailyWord(_))
            && matches!(self.current_game_mode, GameMode::DailyWord(_))
        {
            // Force the user to reset to the base game
            self.current_game_mode = GameMode::default();
            self.current_word_list = WordList::default();
            self.current_word_length = DEFAULT_WORD_LENGTH;
        } else {
            self.current_game_mode = game_mode;
            self.current_word_list = word_list;
            self.current_word_length = word_length;
        }

        self.switch_active_game();

        let _res = self.persist();
        let _res = self.game.persist();
    }

    pub fn change_allow_profanities(&mut self, is_allowed: bool) {
        self.allow_profanities = is_allowed;
        self.game.allow_profanities = self.allow_profanities;
        self.background_games.values_mut().for_each(|game| {
            game.allow_profanities = self.allow_profanities;
        });
        let _result = self.persist();
    }

    pub fn change_theme(&mut self, theme: Theme) -> bool {
        self.theme = theme;
        let _result = self.persist();
        true
    }

    fn switch_active_game(&mut self) -> bool {
        let next_game = (
            self.current_game_mode,
            self.current_word_list,
            self.current_word_length,
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

        self.previous_game = previous_game;

        // Restore a suspended game or create a new one
        let mut game = self.background_games.remove(&next_game).unwrap_or_else(|| {
            Game::new_or_rehydrate(
                next_game.0,
                next_game.1,
                next_game.2,
                self.allow_profanities,
                self.word_lists.clone(),
            )
        });

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

    fn update_game_statistics(&mut self, is_winner: bool, streak: usize) {
        self.total_played += 1;

        if is_winner {
            self.total_solved += 1;

            if streak > self.max_streak {
                self.max_streak = streak;
            }
        }
    }

    #[cfg(web_sys_unstable_apis)]
    pub fn share_emojis(&self) -> String {
        self.game.share_emojis(self.theme)
    }

    #[cfg(web_sys_unstable_apis)]
    pub fn share_link(&self) -> Option<String> {
        self.game.share_link()
    }

    pub fn reveal_hidden_tiles(&mut self) -> bool {
        self.game.reveal_hidden_tiles()
    }

    pub fn reset_game(&mut self) -> bool {
        self.game.reset()
    }

    fn persist(&self) -> Result<(), StorageError> {
        if self.current_game_mode == GameMode::Shared {
            return Ok(());
        }

        LocalStorage::set("settings", self)
    }

    fn rehydrate() -> Result<Self, StorageError> {
        let mut manager: Self = LocalStorage::get("settings")?;
        manager.word_lists = parse_all_words();
        Ok(manager)
    }
}

