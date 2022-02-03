use yew::prelude::*;

use crate::manager::GameMode;
use crate::Msg as GameMsg;

const FORMS_LINK_TEMPLATE_ADD: &str = "https://docs.google.com/forms/d/e/1FAIpQLSfH8gs4sq-Ynn8iGOvlc99J_zOG2rJEC4m8V0kCgF_en3RHFQ/viewform?usp=pp_url&entry.461337706=Lis%C3%A4yst%C3%A4&entry.560255602=";
const FORMS_LINK_TEMPLATE_DEL: &str = "https://docs.google.com/forms/d/e/1FAIpQLSfH8gs4sq-Ynn8iGOvlc99J_zOG2rJEC4m8V0kCgF_en3RHFQ/viewform?usp=pp_url&entry.461337706=Poistoa&entry.560255602=";
const DICTIONARY_LINK_TEMPLATE: &str = "https://www.kielitoimistonsanakirja.fi/#/";

#[derive(Properties, Clone, PartialEq)]
pub struct MessageProps {
    pub message: String,
    pub is_unknown: bool,
    pub is_winner: bool,
    pub is_guessing: bool,
    pub is_hidden: bool,

    pub is_emojis_copied: bool,
    pub is_link_copied: bool,

    pub word: String,
    pub last_guess: String,
    pub game_mode: GameMode,
    pub callback: Callback<GameMsg>,
}

#[function_component(Message)]
pub fn message(props: &MessageProps) -> Html {
    html! {
        <div class="message">
            { &props.message }
            <div class="message-small">{
                if props.is_hidden {
                    let callback = props.callback.clone();
                    let reveal_hidden_tiles = Callback::from(move |e: MouseEvent| {
                        e.prevent_default();
                        callback.emit(GameMsg::RevealHiddenTiles);
                    });
                    let callback = props.callback.clone();
                    let reset_game = Callback::from(move |e: MouseEvent| {
                        e.prevent_default();
                        callback.emit(GameMsg::ResetGame);
                    });

                    html! {
                        <>
                            <a class="link" href={"javascript:void(0)"} onclick={reset_game}>
                                {"Kokeile ratkaista"}
                            </a>
                            {" | "}
                            <a class="link" href={"javascript:void(0)"} onclick={reveal_hidden_tiles}>
                                {"Paljasta"}
                            </a>
                        </>
                    }
                } else if !props.is_guessing {
                    html! {
                        <SubMessage
                            is_winner={props.is_winner}
                            is_emojis_copied={props.is_emojis_copied}
                            is_link_copied={props.is_link_copied}
                            word={props.word.clone()}
                            game_mode={props.game_mode}
                            callback={props.callback.clone()}
                        />
                    }
                } else if props.is_guessing && props.is_unknown {
                    let last_guess = props.last_guess.to_lowercase();
                    html! {
                        <a class="link" href={format!("{}{}", FORMS_LINK_TEMPLATE_ADD, last_guess)}
                            target="_blank">{ "Ehdota lisäystä?" }
                        </a>
                    }
                } else {
                    html! {}
                }
            }
            </div>
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct SubMessageProps {
    pub is_winner: bool,
    pub is_emojis_copied: bool,
    pub is_link_copied: bool,
    pub word: String,
    pub game_mode: GameMode,
    pub callback: Callback<GameMsg>,
}

#[function_component(SubMessage)]
fn sub_message(props: &SubMessageProps) -> Html {
    let word = props.word.to_lowercase();

    let callback = props.callback.clone();
    let share_emojis = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        callback.emit(GameMsg::ShareEmojis);
    });
    let callback = props.callback.clone();
    let share_link = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        callback.emit(GameMsg::ShareLink);
    });

    html! {
        <>
            <a class="link" href={format!("{}{}?searchMode=all", DICTIONARY_LINK_TEMPLATE, word)}
                target="_blank">{ "Sanakirja" }
            </a>
            {" | "}
            <a class="link" href={"javascript:void(0)"} onclick={share_link}>
                {
                    if !props.is_link_copied {
                        {"Linkki"}
                    } else {
                        {"Kopioitu!"}
                    }
                }
            </a>
            {
                if matches!(props.game_mode, GameMode::DailyWord(_)) {
                    html! {
                        <>
                            {" | "}
                            <a class="link" href={"javascript:void(0)"} onclick={share_emojis}>
                                {
                                    if !props.is_emojis_copied {
                                        {"Emojit"}
                                    } else {
                                        {"Kopioitu!"}
                                    }
                                }
                            </a>
                        </>
                    }
                } else if !props.is_winner {
                    html! {
                        <>
                            {" | "}
                            <a class="link" href={format!("{}{}", FORMS_LINK_TEMPLATE_DEL, word)}
                                target="_blank">{ "Ehdota poistoa?" }
                            </a>
                        </>
                    }
                } else {
                    html! {}
                }
            }
        </>
    }
}
