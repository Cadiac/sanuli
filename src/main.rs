extern crate wee_alloc;

use std::collections::HashMap;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, Window};
use yew::prelude::*;

mod components;
mod game;
mod manager;
mod neluli;
mod sanuli;

use components::{
    board::Board,
    header::Header,
    keyboard::Keyboard,
    modal::{HelpModal, MenuModal},
};
use manager::{GameMode, KeyState, Manager, Theme, WordList};

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const ALLOWED_KEYS: [char; 28] = [
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L',
    'Ö', 'Ä', 'Z', 'X', 'C', 'V', 'B', 'N', 'M',
];

pub enum Msg {
    KeyPress(char),
    Backspace,
    Enter,
    Guess,
    NextWord,
    ToggleHelp,
    ToggleMenu,
    ChangeGameMode(GameMode),
    ChangePreviousGameMode,
    ChangeWordLength(usize),
    ChangeWordList(WordList),
    ChangeAllowProfanities(bool),
    ChangeTheme(Theme),
    ShareEmojis,
    ShareLink,
    RevealHiddenTiles,
    ResetGame,
}

pub struct App {
    manager: Manager,
    is_help_visible: bool,
    is_menu_visible: bool,
    is_emojis_copied: bool,
    is_link_copied: bool,
    keyboard_listener: Option<Closure<dyn Fn(KeyboardEvent)>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            manager: Manager::new(),
            is_help_visible: false,
            is_menu_visible: false,
            is_emojis_copied: false,
            is_link_copied: false,
            keyboard_listener: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }

        let window: Window = window().expect("window not available");

        let cb = ctx.link().batch_callback(|e: KeyboardEvent| {
            if e.key().chars().count() == 1 {
                let key = e.key().to_uppercase().chars().next().unwrap();
                if ALLOWED_KEYS.contains(&key) && !e.ctrl_key() && !e.alt_key() && !e.meta_key() {
                    e.prevent_default();
                    Some(Msg::KeyPress(key))
                } else {
                    None
                }
            } else if e.key() == "Backspace" {
                e.prevent_default();
                Some(Msg::Backspace)
            } else if e.key() == "Enter" {
                e.prevent_default();
                Some(Msg::Enter)
            } else {
                None
            }
        });

        let listener =
            Closure::<dyn Fn(KeyboardEvent)>::wrap(Box::new(move |e: KeyboardEvent| cb.emit(e)));

        window
            .add_event_listener_with_callback("keydown", listener.as_ref().unchecked_ref())
            .unwrap();
        self.keyboard_listener = Some(listener);
    }

    fn destroy(&mut self, _: &Context<Self>) {
        // Remove the keyboard listener
        if let Some(listener) = self.keyboard_listener.take() {
            let window: Window = window().expect("window not available");
            window
                .remove_event_listener_with_callback("keydown", listener.as_ref().unchecked_ref())
                .unwrap();
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::KeyPress(c) => self.manager.push_character(c),
            Msg::Backspace => self.manager.pop_character(),
            Msg::Enter => {
                let link = ctx.link();

                if let Some(game) = &self.manager.game {
                    if game.is_guessing() {
                        link.send_message(Msg::Guess);
                    } else {
                        if matches!(game.game_mode(), GameMode::DailyWord(_) | GameMode::Shared) {
                            link.send_message(Msg::ChangePreviousGameMode);
                        } else {
                            link.send_message(Msg::NextWord);
                        }
                    }
                }
            }
            Msg::Guess => self.manager.submit_guess(),
            Msg::NextWord => {
                self.manager.next_word();
                self.is_emojis_copied = false;
                self.is_link_copied = false;
            }
            Msg::ToggleHelp => {
                self.is_help_visible = !self.is_help_visible;
                self.is_menu_visible = false;
            }
            Msg::ToggleMenu => {
                self.is_menu_visible = !self.is_menu_visible;
                self.is_help_visible = false;
            }
            Msg::ChangeWordLength(new_length) => {
                self.manager.change_word_length(new_length);
                self.is_menu_visible = false;
                self.is_help_visible = false;
            }
            Msg::ChangeGameMode(new_mode) => {
                self.manager.change_game_mode(new_mode);
                self.is_menu_visible = false;
                self.is_help_visible = false;
            }
            Msg::ChangeWordList(new_list) => {
                self.manager.change_word_list(new_list);
                self.is_menu_visible = false;
                self.is_help_visible = false;
            }
            Msg::ChangePreviousGameMode => {
                self.manager.change_previous_game_mode();
                self.is_emojis_copied = false;
                self.is_link_copied = false;
            }
            Msg::ChangeAllowProfanities(is_allowed) => {
                self.manager.change_allow_profanities(is_allowed);
                self.is_menu_visible = false;
                self.is_help_visible = false;
            }
            Msg::ChangeTheme(theme) => self.manager.change_theme(theme),
            Msg::ShareEmojis => {
                #[cfg(web_sys_unstable_apis)]
                {
                    use web_sys::Navigator;

                    if let Some(emojis) = self.manager.share_emojis() {
                        let window: Window = window().expect("window not available");
                        let navigator: Navigator = window.navigator();
                        if let Some(clipboard) = navigator.clipboard() {
                            let _promise = clipboard.write_text(emojis.as_str());
                        }
                    }
                }
                self.is_emojis_copied = true;
                self.is_link_copied = false;
            }
            Msg::ShareLink => {
                #[cfg(web_sys_unstable_apis)]
                {
                    use web_sys::Navigator;

                    if let Some(link) = self.manager.share_link() {
                        let window: Window = window().expect("window not available");
                        let navigator: Navigator = window.navigator();
                        if let Some(clipboard) = navigator.clipboard() {
                            let _promise = clipboard.write_text(link.as_str());
                        }
                    }
                }
                self.is_link_copied = true;
                self.is_emojis_copied = false;
            }
            Msg::RevealHiddenTiles => self.manager.reveal_hidden_tiles(),
            Msg::ResetGame => self.manager.reset_game(),
        };

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        if let Some(game) = &self.manager.game {
            let keyboard_state = ALLOWED_KEYS
                .iter()
                .map(|key| (*key, game.keyboard_tilestate(key)))
                .collect::<HashMap<char, KeyState>>();

            let last_guess = game.last_guess();

            let boards = game.boards();

            html! {
                <div class={classes!("game", self.manager.theme.to_string())}>
                    <Header
                        on_toggle_help_cb={link.callback(|_| Msg::ToggleHelp)}
                        on_toggle_menu_cb={link.callback(|_| Msg::ToggleMenu)}
                        title={game.title()}
                    />

                    {
                        match boards.len() {
                            1 => html! {
                                <div class="board-container">
                                    <Board
                                        guesses={boards[0].guesses.clone()}
                                        is_guessing={boards[0].is_guessing}
                                        current_guess={boards[0].current_guess}
                                        is_reset={game.is_reset()}
                                        is_hidden={game.is_hidden()}
                                        previous_guesses={game.previous_guesses().clone()}
                                        max_guesses={game.max_guesses()}
                                        word_length={game.word_length()}
                                    />
                                </div>
                            },
                            4 => html! {
                                <div class="quadruple-container">
                                    <div class="quadruple-grid">
                                        {game.boards().iter().map(|board| {
                                            html! {
                                                <Board
                                                    guesses={board.guesses.clone()}
                                                    is_guessing={board.is_guessing}
                                                    current_guess={board.current_guess}
                                                    is_reset={game.is_reset()}
                                                    is_hidden={game.is_hidden()}
                                                    previous_guesses={game.previous_guesses().clone()}
                                                    max_guesses={game.max_guesses()}
                                                    word_length={game.word_length()}
                                                />
                                            }
                                        }).collect::<Html>()}
                                    </div>
                                </div>
                            },
                            _ => html! {}
                        }
                    }

                    <Keyboard
                        callback={link.callback(move |msg| msg)}
                        is_unknown={game.is_unknown()}
                        is_winner={game.is_winner()}
                        is_guessing={game.is_guessing()}
                        is_hidden={game.is_hidden()}
                        is_emojis_copied={self.is_emojis_copied}
                        is_link_copied={self.is_link_copied}
                        game_mode={game.game_mode().clone()}
                        message={game.message()}
                        word={game.word().iter().collect::<String>()}
                        last_guess={last_guess}
                        keyboard={keyboard_state}
                    />

                    {
                        if self.is_help_visible {
                            html! { <HelpModal theme={self.manager.theme} callback={link.callback(move |msg| msg)} /> }
                        } else {
                            html! {}
                        }
                    }

                    {
                        if self.is_menu_visible {
                            html! {
                                <MenuModal
                                    callback={link.callback(move |msg| msg)}
                                    game_mode={self.manager.current_game_mode}
                                    word_length={self.manager.current_word_length}
                                    current_word_list={self.manager.current_word_list}
                                    allow_profanities={self.manager.allow_profanities}
                                    theme={self.manager.theme}
                                    max_streak={self.manager.max_streak}
                                    total_played={self.manager.total_played}
                                    total_solved={self.manager.total_solved}
                                />
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            }
        } else {
            html! {
                <MenuModal
                    callback={link.callback(move |msg| msg)}
                    game_mode={self.manager.current_game_mode}
                    word_length={self.manager.current_word_length}
                    current_word_list={self.manager.current_word_list}
                    allow_profanities={self.manager.allow_profanities}
                    theme={self.manager.theme}
                    max_streak={self.manager.max_streak}
                    total_played={self.manager.total_played}
                    total_solved={self.manager.total_solved}
                />
            }
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
