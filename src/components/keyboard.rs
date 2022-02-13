use std::collections::HashMap;
use yew::prelude::*;

use crate::manager::{GameMode, KeyState, TileState};
use crate::Msg;

use crate::components::message::Message;

const KEYBOARD_0: [char; 10] = ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'];
const KEYBOARD_1: [char; 11] = ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'Ö', 'Ä'];
const KEYBOARD_2: [char; 7] = ['Z', 'X', 'C', 'V', 'B', 'N', 'M'];

#[derive(Properties, PartialEq)]
pub struct Props {
    pub callback: Callback<Msg>,

    pub is_unknown: bool,
    pub is_winner: bool,
    pub is_guessing: bool,
    pub is_hidden: bool,

    pub is_emojis_copied: bool,
    pub is_link_copied: bool,

    pub game_mode: GameMode,

    pub message: String,
    pub word: String,
    pub last_guess: String,

    pub keyboard: HashMap<char, KeyState>,
}

#[function_component(Keyboard)]
pub fn keyboard(props: &Props) -> Html {
    let callback = props.callback.clone();
    let onbackspace = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        callback.emit(Msg::Backspace);
    });

    html! {
        <div class="keyboard">
            {
                if props.message.is_empty() {
                    html! {}
                } else {
                    html! {
                        <Message
                            message={props.message.clone()}
                            is_unknown={props.is_unknown}
                            is_winner={props.is_winner}
                            is_guessing={props.is_guessing}
                            is_hidden={props.is_hidden}
                            is_emojis_copied={props.is_emojis_copied}
                            is_link_copied={props.is_link_copied}
                            last_guess={props.last_guess.clone()}
                            word={props.word.clone()}
                            game_mode={props.game_mode}
                            callback={props.callback.clone()}
                        />
                    }
                }
            }

            <div class="keyboard-row">
                {
                    KEYBOARD_0.iter().map(|key| {
                        let callback = props.callback.clone();
                        let onkeypress = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(Msg::KeyPress(*key));
                        });

                        let key_state = props.keyboard.get(key).unwrap_or(&KeyState::Single(TileState::Unknown));

                        html! {
                            <KeyboardButton character={*key} is_hidden={props.is_hidden} onkeypress={onkeypress} key_state={*key_state}/>
                        }
                    }).collect::<Html>()
                }
                <button data-nosnippet="" class={classes!("keyboard-button", "keyboard-button-backspace")} onmousedown={onbackspace}>
                    { "⌫" }
                </button>
            </div>
            <div class="keyboard-row">
                <div class="spacer" />
                {
                    KEYBOARD_1.iter().map(|key| {
                        let callback = props.callback.clone();
                        let onkeypress = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(Msg::KeyPress(*key));
                        });

                        let key_state = props.keyboard.get(key).unwrap_or(&KeyState::Single(TileState::Unknown));

                        html! {
                            <KeyboardButton character={*key} is_hidden={props.is_hidden} onkeypress={onkeypress} key_state={*key_state}/>
                        }
                    }).collect::<Html>()
                }
            </div>
            <div class="keyboard-row">
                <div class="spacer" />
                <div class="spacer" />
                <div class="spacer" />
                {
                    KEYBOARD_2.iter().map(|key| {
                        let callback = props.callback.clone();
                        let onkeypress = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(Msg::KeyPress(*key));
                        });

                        let key_state = props.keyboard.get(key).unwrap_or(&KeyState::Single(TileState::Unknown));

                        html! {
                            <KeyboardButton character={*key} is_hidden={props.is_hidden} onkeypress={onkeypress} key_state={*key_state}/>
                        }
                    }).collect::<Html>()
                }
                {
                    if props.is_guessing {
                        let callback = props.callback.clone();
                        let onmousedown = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(Msg::Guess);
                        });

                        html! {
                            <button data-nosnippet="" class={classes!("keyboard-button", "keyboard-button-submit")}
                                onmousedown={onmousedown}>
                                { "ARVAA" }
                            </button>
                        }
                    } else if matches!(props.game_mode, GameMode::DailyWord(_) | GameMode::Shared) {
                        let callback = props.callback.clone();
                        let onmousedown = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(Msg::ChangePreviousGameMode);
                        });

                        html! {
                            <button data-nosnippet="" class={classes!("keyboard-button", "keyboard-button-submit", "correct")}
                                onmousedown={onmousedown}>
                                { "TAKAISIN" }
                            </button>
                        }
                    } else {
                        let callback = props.callback.clone();
                        let onmousedown = Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            callback.emit(Msg::NextWord);
                        });

                        html! {
                            <button data-nosnippet="" class={classes!("keyboard-button", "keyboard-button-submit", "correct")}
                                onmousedown={onmousedown}>
                                { "UUSI?" }
                            </button>
                        }
                    }
                }
                <div class="spacer" />
                <div class="spacer" />
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct KeyboardButtonProps {
    pub onkeypress: Callback<MouseEvent>,
    pub character: char,
    pub is_hidden: bool,
    pub key_state: KeyState,
}

#[function_component(KeyboardButton)]
pub fn keyboard_button(props: &KeyboardButtonProps) -> Html {
    if !props.is_hidden {
        match props.key_state {
            KeyState::Single(state) => {
                html! {
                    <button data-nosnippet="" class={classes!("keyboard-button", state.to_string())} onmousedown={props.onkeypress.clone()}>
                        { props.character }
                    </button>
                }
            }
            KeyState::Quadruple(states) => {
                let background = format!(
                    "background: conic-gradient(var(--{top_right}) 0deg, var(--{top_right}) 90deg, var(--{bottom_right}) 90deg, var(--{bottom_right}) 180deg, var(--{bottom_left}) 180deg, var(--{bottom_left}) 270deg, var(--{top_left}) 270deg, var(--{top_left}) 360deg);",
                    top_left=states[0],
                    top_right=states[1],
                    bottom_left=states[2],
                    bottom_right=states[3],
                );

                html! {
                    <button data-nosnippet="" class={"keyboard-button"} style={background.clone()}
                        onmousedown={props.onkeypress.clone()}>
                        { props.character }
                    </button>
                }
            }
        }
    } else {
        html! {
            <button data-nosnippet="" class={classes!("keyboard-button", "unknown")}>
                { props.character }
            </button>
        }
    }
}
