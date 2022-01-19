use std::collections::HashMap;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, Window};
use yew::prelude::*;
use chrono::{Local, NaiveDate};

mod components;
mod state;

use components::{
    board::Board,
    header::Header,
    keyboard::Keyboard,
    modal::{HelpModal, MenuModal},
};
use state::{GameMode, State, TileState, WordList, Theme};

const ALLOWED_KEYS: [char; 28] = [
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K',
    'L', 'Ö', 'Ä', 'Z', 'X', 'C', 'V', 'B', 'N', 'M',
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
    ChangeTheme(Theme)
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
        let mut initial_state = Self {
            state: State::new(),
            is_help_visible: false,
            is_menu_visible: false,
            keyboard_listener: None,
        };

        // if initial_state.state.rehydrate().is_err() {
        //     // Reinitialize and just continue with defaults
        //     initial_state.state =
        //         State::new();
        // }

        initial_state
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
            Msg::Guess => self.state.game.submit_guess(),
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
                self.state.game_manager.borrow_mut().change_word_length(new_length);
                self.is_menu_visible = false;
                self.is_help_visible = false;
                self.state.switch_active_game()
            }
            Msg::ChangeGameMode(new_mode) => {
                self.state.game_manager.borrow_mut().change_game_mode(new_mode);
                self.is_menu_visible = false;
                self.is_help_visible = false;
                self.state.switch_active_game()
            }
            Msg::ChangePreviousGameMode => {
                self.state.game_manager.borrow_mut().change_game_mode(self.state.game_manager.borrow().previous_game.0);
                self.state.game_manager.borrow_mut().change_word_list(self.state.game_manager.borrow().previous_game.1);
                self.state.game_manager.borrow_mut().change_word_length(self.state.game_manager.borrow().previous_game.2);
                self.state.switch_active_game()
            }
            Msg::ChangeWordList(list) => {
                self.state.game_manager.borrow_mut().change_word_list(list);
                self.is_menu_visible = false;
                self.is_help_visible = false;
                self.state.switch_active_game()
            }
            Msg::ChangeAllowProfanities(is_allowed) => {
                self.state.game_manager.borrow_mut().change_allow_profanities(is_allowed);
                self.is_menu_visible = false;
                self.is_help_visible = false;
                true
            },
            Msg::ChangeTheme(theme) => {
                self.state.game_manager.borrow_mut().change_theme(theme);
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

        let game_manager = self.state.game_manager.borrow();
        let today = Local::now().naive_local().date();

        html! {
            <div class={classes!("game", game_manager.theme.to_string())}>
                <Header
                    on_toggle_help_cb={link.callback(|_| Msg::ToggleHelp)}
                    on_toggle_menu_cb={link.callback(|_| Msg::ToggleMenu)}
                    streak={self.state.game.streak}
                    game_mode={self.state.game.game_mode}
                    daily_word_number={game_manager.get_daily_word_index(today) + 1}
                />

                <Board
                    is_guessing={self.state.game.is_guessing}
                    is_reset={self.state.game.is_reset}
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
                                game_mode={game_manager.current_game_mode}
                                word_length={game_manager.current_word_length}
                                current_word_list={game_manager.current_word_list}
                                allow_profanities={game_manager.allow_profanities}
                                theme={game_manager.theme}
                                max_streak={game_manager.max_streak}
                                total_played={game_manager.total_played}
                                total_solved={game_manager.total_solved}
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
    yew::start_app::<App>();
}
