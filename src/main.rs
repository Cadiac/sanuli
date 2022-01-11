use std::collections::HashMap;

use yew::prelude::*;

use gloo_timers::callback::Timeout;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, Window};

mod components;
mod state;

use components::{
    board::Board,
    header::Header,
    keyboard::Keyboard,
    modal::{HelpModal, MenuModal},
    timer::Timer,
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
    NewGame,
    LoseGame,
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
    timeout: Option<Timeout>
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
            timeout: None,
        };

        if initial_state.state.rehydrate().is_err() {
            // Reinitialize and just continue with defaults
            initial_state.state =
                State::new();
        }

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
            Msg::KeyPress(c) => self.state.push_character(c),
            Msg::Backspace => self.state.pop_character(),
            Msg::Enter => {
                let link = ctx.link();

                if !self.state.is_guessing {
                    if self.state.game_mode == GameMode::DailyWord {
                        link.send_message(Msg::ChangePreviousGameMode);
                    } else {
                        link.send_message(Msg::NewGame);
                    }
                } else {
                    link.send_message(Msg::Guess);
                }

                true
            }
            Msg::Guess => self.state.submit_guess(),
            Msg::NewGame => {
                if self.state.game_mode == GameMode::TimeAttack {
                    if let Some(timeout) = self.timeout.take() {
                        timeout.cancel();
                    }
    
                    if self.state.is_winner && self.state.time_attack.is_some() {
                        self.state.time_attack.as_mut().unwrap().next_round();
                    } else {
                        self.state.time_attack = Some(state::TimeAttack::new())
                    }
    
                    self.state.create_new_game();
    
                    let link = ctx.link().clone();
                    let duration = self.state.time_attack.as_ref().unwrap().duration;
    
                    self.timeout = Some(
                        Timeout::new(1_000 * duration, move || {
                            link.send_message(Msg::LoseGame)
                        }),
                    );    
                } else {
                    self.state.create_new_game();
                }

                true
            }
            Msg::LoseGame => {
                if let Some(timeout) = self.timeout.take() {
                    timeout.cancel();
                }

                self.state.lose_game_timed()
            }
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
                self.state.create_new_game()
            }
            Msg::ChangeGameMode(new_mode) => {
                self.state.change_game_mode(new_mode);
                self.is_menu_visible = false;
                self.is_help_visible = false;
                self.state.create_new_game()
            }
            Msg::ChangePreviousGameMode => {
                self.state.change_game_mode(self.state.previous_game_mode.clone());
                self.state.create_new_game()
            }
            Msg::ChangeWordList(list) => {
                self.state.change_word_list(list);
                self.is_menu_visible = false;
                self.is_help_visible = false;
                self.state.create_new_game()
            }
            Msg::ChangeAllowProfanities(is_allowed) => {
                self.state.change_allow_profanities(is_allowed);
                self.is_menu_visible = false;
                self.is_help_visible = false;
                true
            },
            Msg::ChangeTheme(theme) => {
                self.state.change_theme(theme);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let keyboard_state = ALLOWED_KEYS
            .iter()
            .map(|key| (*key, self.state.keyboard_tilestate(key)))
            .collect::<HashMap<char, TileState>>();

        let word = self.state.word.iter().collect::<String>();

        let last_guess = self.state.guesses[self.state.current_guess]
            .iter()
            .map(|(c, _)| c)
            .collect::<String>();

        html! {
            <div class={classes!("game", self.state.theme.to_string())}>
                <Header
                    on_toggle_help_cb={link.callback(|_| Msg::ToggleHelp)}
                    on_toggle_menu_cb={link.callback(|_| Msg::ToggleMenu)}
                    streak={self.state.streak}
                    game_mode={self.state.game_mode}
                    daily_word_number={self.state.get_daily_word_index() + 1}
                />

                <Board
                    is_guessing={self.state.is_guessing}
                    is_reset={self.state.is_reset}
                    guesses={self.state.guesses.clone()}
                    previous_guesses={self.state.previous_guesses.clone()}
                    current_guess={self.state.current_guess}
                    max_guesses={self.state.max_guesses}
                    word_length={self.state.word_length}
                />

                {
                    if self.state.game_mode == GameMode::TimeAttack {
                        if let Some(time_attack) = self.state.time_attack.as_ref() {
                            html! {
                                <Timer duration={time_attack.duration} is_paused={!self.state.is_guessing} />
                            }
                        } else {
                            html! {}
                        }
                    } else {
                        html! {}
                    }
                }

                <Keyboard
                    callback={link.callback(move |msg| msg)}
                    is_unknown={self.state.is_unknown}
                    is_winner={self.state.is_winner}
                    is_guessing={self.state.is_guessing}
                    game_mode={self.state.game_mode}
                    message={self.state.message.clone()}
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
                                game_mode={self.state.game_mode}
                                word_length={self.state.word_length}
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
    yew::start_app::<App>();
}
