use chrono::Local;
use std::collections::HashMap;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, Window};
use yew::prelude::*;

mod components;
mod migration;
mod state;

use components::{
    board::Board,
    header::Header,
    keyboard::Keyboard,
    modal::{HelpModal, MenuModal},
};
use state::{Game, GameMode, State, Theme, TileState, WordList};

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
}

pub struct App {
    state: State,
    is_help_visible: bool,
    is_menu_visible: bool,
    keyboard_listener: Option<Closure<dyn Fn(KeyboardEvent)>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            state: State::new(),
            is_help_visible: false,
            is_menu_visible: false,
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
            Msg::KeyPress(c) => self.state.game.push_character(c),
            Msg::Backspace => self.state.game.pop_character(),
            Msg::Enter => {
                let link = ctx.link();

                if !self.state.game.is_guessing {
                    if let GameMode::DailyWord(_) = self.state.game.game_mode {
                        link.send_message(Msg::ChangePreviousGameMode);
                    } else {
                        link.send_message(Msg::NextWord);
                    }
                } else {
                    link.send_message(Msg::Guess);
                }

                true
            }
            Msg::Guess => self.state.submit_guess(),
            Msg::NextWord => self.state.game.next_word(),
            Msg::ToggleHelp => {
                self.is_help_visible = !self.is_help_visible;
                self.is_menu_visible = false;
                true
            }
            Msg::ToggleMenu => {
                self.is_menu_visible = !self.is_menu_visible;
                self.is_help_visible = false;
                true
            }
            Msg::ChangeWordLength(new_length) => {
                self.state.change_word_length(new_length);
                self.is_menu_visible = false;
                self.is_help_visible = false;
                true
            }
            Msg::ChangeGameMode(new_mode) => {
                self.state.change_game_mode(new_mode);
                self.is_menu_visible = false;
                self.is_help_visible = false;
                true
            }
            Msg::ChangeWordList(new_list) => {
                self.state.change_word_list(new_list);
                self.is_menu_visible = false;
                self.is_help_visible = false;
                true
            }
            Msg::ChangePreviousGameMode => {
                self.state.change_previous_game_mode();
                true
            }
            Msg::ChangeAllowProfanities(is_allowed) => {
                self.state.change_allow_profanities(is_allowed);
                self.is_menu_visible = false;
                self.is_help_visible = false;
                true
            }
            Msg::ChangeTheme(theme) => {
                self.state.change_theme(theme);
                true
            }
            Msg::ShareEmojis => {
                #[cfg(web_sys_unstable_apis)]
                {
                    use web_sys::Navigator;

                    let emojis = self.state.share_emojis();
                    let window: Window = window().expect("window not available");
                    let navigator: Navigator = window.navigator();
                    if let Some(clipboard) = navigator.clipboard() {
                        let _promise = clipboard.write_text(emojis.as_str());
                    }
                }
                true
            }
            Msg::ShareLink => {
                #[cfg(web_sys_unstable_apis)]
                {
                    use web_sys::Navigator;

                    if let Some(link) = self.state.share_link() {
                        let window: Window = window().expect("window not available");
                        let navigator: Navigator = window.navigator();
                        if let Some(clipboard) = navigator.clipboard() {
                            let _promise = clipboard.write_text(link.as_str());
                        }
                        log::info!("{}", link);
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let keyboard_state = ALLOWED_KEYS
            .iter()
            .map(|key| (*key, self.state.game.keyboard_tilestate(key)))
            .collect::<HashMap<char, TileState>>();

        let word = self.state.game.word.iter().collect::<String>();

        let last_guess = self.state.game.guesses[self.state.game.current_guess]
            .iter()
            .map(|(c, _)| c)
            .collect::<String>();

        let today = Local::now().naive_local().date();

        html! {
            <div class={classes!("game", self.state.theme.to_string())}>
                <Header
                    on_toggle_help_cb={link.callback(|_| Msg::ToggleHelp)}
                    on_toggle_menu_cb={link.callback(|_| Msg::ToggleMenu)}
                    streak={self.state.game.streak}
                    game_mode={self.state.game.game_mode}
                    daily_word_number={Game::get_daily_word_index(today) + 1}
                />

                <Board
                    is_guessing={self.state.game.is_guessing}
                    is_reset={self.state.game.is_reset}
                    is_hidden={self.state.game.is_hidden}
                    guesses={self.state.game.guesses.clone()}
                    previous_guesses={self.state.game.previous_guesses.clone()}
                    current_guess={self.state.game.current_guess}
                    max_guesses={self.state.game.max_guesses}
                    word_length={self.state.game.word_length}
                />

                <Keyboard
                    callback={link.callback(move |msg| msg)}
                    is_unknown={self.state.game.is_unknown}
                    is_winner={self.state.game.is_winner}
                    is_guessing={self.state.game.is_guessing}
                    is_hidden={self.state.game.is_hidden}
                    game_mode={self.state.game.game_mode}
                    message={self.state.game.message.clone()}
                    word={word}
                    last_guess={last_guess}
                    keyboard={keyboard_state}
                />

                {
                    if self.is_help_visible {
                        html! { <HelpModal callback={link.callback(move |msg| msg)} /> }
                    } else {
                        html! {}
                    }
                }

                {
                    if self.is_menu_visible {
                        html! {
                            <MenuModal
                                callback={link.callback(move |msg| msg)}
                                game_mode={self.state.current_game_mode}
                                word_length={self.state.current_word_length}
                                current_word_list={self.state.current_word_list}
                                allow_profanities={self.state.allow_profanities}
                                theme={self.state.theme}
                                max_streak={self.state.max_streak}
                                total_played={self.state.total_played}
                                total_solved={self.state.total_solved}
                            />
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
