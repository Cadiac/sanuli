use yew::prelude::*;

use crate::state::{GameMode, WordList};
use crate::Msg;

const FORMS_LINK_TEMPLATE_ADD: &str = "https://docs.google.com/forms/d/e/1FAIpQLSfH8gs4sq-Ynn8iGOvlc99J_zOG2rJEC4m8V0kCgF_en3RHFQ/viewform?usp=pp_url&entry.461337706=Lis%C3%A4yst%C3%A4&entry.560255602=";
const CHANGELOG_URL: &str = "https://github.com/Cadiac/sanuli/blob/master/CHANGELOG.md";
const VERSION: &str = "v1.2";

macro_rules! onmousedown {
    ( $cb:ident, $msg:expr ) => {
        {
            let $cb = $cb.clone();
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                $cb.emit($msg);
            })
        }
    };
}

#[derive(Properties, Clone, PartialEq)]
pub struct HelpModalProps {
    pub callback: Callback<Msg>
}

#[function_component(HelpModal)]
pub fn help_modal(props: &HelpModalProps) -> Html {
    let callback = props.callback.clone();
    let toggle_help = onmousedown!(callback, Msg::ToggleHelp);

    html! {
        <div class="modal">
            <span onmousedown={toggle_help} class="modal-close">{"✖"}</span>
            <p>{"Arvaa kätketty "}<i>{"sanuli"}</i>{" kuudella yrityksellä."}</p>
            <p>{"Jokaisen yrityksen jälkeen arvatut kirjaimet vaihtavat väriään."}</p>
    
            <div class="row-5 example">
                <div class={classes!("tile", "correct")}>{"K"}</div>
                <div class={classes!("tile", "absent")}>{"O"}</div>
                <div class={classes!("tile", "present")}>{"I"}</div>
                <div class={classes!("tile", "absent")}>{"R"}</div>
                <div class={classes!("tile", "absent")}>{"A"}</div>
            </div>
    
            <p><span class="present">{"Keltainen"}</span>{": kirjain löytyy kätketystä sanasta, mutta on arvauksessa väärällä paikalla."}</p>
            <p><span class="correct">{"Vihreä"}</span>{": kirjain on arvauksessa oikealla paikalla."}</p>
            <p><span class="absent">{"Harmaa"}</span>{": kirjain ei löydy sanasta."}</p>
    
            <p>
                {"Käytetyn sanalistan pohjana on Kotimaisten kielten keskuksen (Kotus) julkaisema "}
                <a class="link" href="https://creativecommons.org/licenses/by/3.0/deed.fi" target="_blank">{"\"CC Nimeä 3.0 Muokkaamaton\""}</a>
                {" lisensoitu nykysuomen sanalista, josta on poimittu ne sanat, jotka sisältävät vain kirjaimia A-Ö. Sanat ovat enimmäkseen perusmuodossa. "}
                {"Sanalistaa muokataan jatkuvasti käyttäjien ehdotusten perusteella, ja voit jättää omat ehdotuksesi sanuihin "}
                <a class="link" href={FORMS_LINK_TEMPLATE_ADD}>{"täällä"}</a>
                {"."}
            </p>
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct MenuModalProps {
    pub callback: Callback<Msg>,
    pub word_length: usize,
    pub game_mode: GameMode,
    pub word_list: WordList,
}

#[function_component(MenuModal)]
pub fn menu_modal(props: &MenuModalProps) -> Html {
    let callback = props.callback.clone();

    let toggle_menu = onmousedown!(callback, Msg::ToggleMenu);
    let change_word_length_5 = onmousedown!(callback, Msg::ChangeWordLength(5));
    let change_word_length_6 = onmousedown!(callback, Msg::ChangeWordLength(6));
    let change_game_mode_classic = onmousedown!(callback, Msg::ChangeGameMode(GameMode::Classic));
    let change_game_mode_relay = onmousedown!(callback, Msg::ChangeGameMode(GameMode::Relay));
    let change_game_mode_daily = onmousedown!(callback, Msg::ChangeGameMode(GameMode::DailyWord));
    let change_word_list_full = onmousedown!(callback, Msg::ChangeWordList(WordList::Full));
    let change_word_list_common = onmousedown!(callback, Msg::ChangeWordList(WordList::Common));

    html! {
        <div class="modal">
            <span onmousedown={toggle_menu} class="modal-close">{"✖"}</span>
            <div>
                <label class="label">{"Sanulien pituus:"}</label>
                <div class="select-container">
                    <button class={classes!("select", (props.word_length == 5).then(|| Some("select-active")))}
                        onmousedown={change_word_length_5}>
                        {"5 merkkiä"}
                    </button>
                    <button class={classes!("select", (props.word_length == 6).then(|| Some("select-active")))}
                        onmousedown={change_word_length_6}>
                        {"6 merkkiä"}
                    </button>
                </div>
            </div>
            <div>
                <label class="label">{"Sanulista:"}</label>
                <div class="select-container">
                    <button class={classes!("select", (props.word_list == WordList::Full).then(|| Some("select-active")))}
                        onmousedown={change_word_list_full}>
                        {"Kaikki"}
                    </button>
                    <button class={classes!("select", (props.word_list == WordList::Common).then(|| Some("select-active")))}
                        onmousedown={change_word_list_common}>
                        {"Suppea"}
                    </button>
                </div>
            </div>
            <div>
                <label class="label">{"Pelimuoto:"}</label>
                <div class="select-container">
                    <button class={classes!("select", (props.game_mode == GameMode::Classic).then(|| Some("select-active")))}
                        onmousedown={change_game_mode_classic}>
                        {"Peruspeli"}
                    </button>
                    <button class={classes!("select", (props.game_mode == GameMode::Relay).then(|| Some("select-active")))}
                        onmousedown={change_game_mode_relay}>
                        {"Sanuliketju"}
                    </button>
                    <button class={classes!("select", (props.game_mode == GameMode::DailyWord).then(|| Some("select-active")))}
                        onclick={change_game_mode_daily}>
                        {"Päivän sanuli"}
                    </button>
                </div>
            </div>
            <div class="version">
                <a class="version" href={CHANGELOG_URL} target="_blank">{ VERSION }</a>
            </div>
        </div>
    }
}