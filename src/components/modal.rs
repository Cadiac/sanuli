use chrono::Local;
use yew::prelude::*;

use crate::manager::{GameMode, Theme, WordList};
use crate::Msg;

const FORMS_LINK_TEMPLATE_ADD: &str = "https://docs.google.com/forms/d/e/1FAIpQLSfH8gs4sq-Ynn8iGOvlc99J_zOG2rJEC4m8V0kCgF_en3RHFQ/viewform?usp=pp_url&entry.461337706=Lis%C3%A4yst%C3%A4&entry.560255602=";
const CHANGELOG_URL: &str = "https://github.com/Cadiac/sanuli/blob/master/CHANGELOG.md";
const VERSION: &str = "v1.14";

macro_rules! onmousedown {
    ( $cb:ident, $msg:expr ) => {{
        let $cb = $cb.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            $cb.emit($msg);
        })
    }};
}

#[derive(Properties, Clone, PartialEq)]
pub struct HelpModalProps {
    pub theme: Theme,
    pub callback: Callback<Msg>,
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

            <p>
                {
                    html! {
                        if props.theme == Theme::Colorblind {
                            <span class="present">{"Sininen"}</span>
                        } else {
                            <span class="present">{"Keltainen"}</span>
                        }
                    }
                }
                {": kirjain löytyy kätketystä sanasta, mutta on arvauksessa väärällä paikalla."}
            </p>
            <p>
                {
                    html! {
                        if props.theme == Theme::Colorblind {
                            <span class="correct">{"Oranssi"}</span>
                        } else {
                            <span class="correct">{"Vihreä"}</span>
                        }
                    }
                }
                {": kirjain on arvauksessa oikealla paikalla."}
            </p>
            <p><span class="absent">{"Harmaa"}</span>{": kirjain ei löydy sanasta."}</p>

            <p>
                {"Arvattaviin sanoihin käytetyn sanulistan vaikeusasteen voi valita asetuksista. Sanulistojen pohjana oli
                Kotimaisten kielten keskuksen (Kotus) julkaiseman "}
                <a class="link" href="https://creativecommons.org/licenses/by/3.0/deed.fi" target="_blank">{"\"CC Nimeä 3.0 Muokkaamaton\""}</a>
                {" lisensoidun nykysuomen sanulistan sanat."}
            </p>

            <p><b>{"Tavallinen"}</b>{" lista sisältää täydestä listasta poimitut yleisimmät sanat ilman harvinaisempia laina- ja murressanoja tai muita erikoisuuksia."}</p>
            <p><b>{"Helppo"}</b>{" lista on tavallisesta vielä hieman helpotettu versio, jossa jäljellä ovat vain yleiset arkikielen sanat ilman vanhahtavia sanoja,
                puhekieltä tai rumia sanuleja. Näin lista sopii kaikenikäisille. \"Helppo\" kuusikirjaimisten sanulien lista on kuitenkin vielä kesken."}</p>
            <p><b>{"Vaikea"}</b>{" lista on täysi lista pelin hyväksymiä sanoja. Tälle listalle on myös lisätty jonkin verran käyttäjien uusia ehdotuksia,
                puhekielisyyksiä, murresanoja sekä muita erikoisuuksia, eikä poistoja ole tehty kuin vain jos sanulit eivät selvästi ole oikeita sanoja."}</p>
            <p>
                {"Sanulit ovat yleensä perusmuodossa, mutta eivät välttämättä täysin pelkkää kirjakieltä. Yhdyssanojakin on seassa."}
            </p>
            <p>
                {"Päivän sanulit tulevat omalta listaltaan, joka on jotain tavallisen ja vaikean listan väliltä. Sanulin on aina sama kaikille pelaajille tiettynä päivänä."}
            </p>
            <p>
                {"Sanuliketjussa jos arvaat sanulin, on se suoraan ensimmäinen arvaus seuraavaan peliin. Näin joudut sopeutumaan vaihtuviin alkuarvauksiin, ja peli on hieman vaikeampi."}
            </p>
            <p>
                {"Nelulissa ratkaiset samalla kertaa neljää eri sanulia samoilla arvauksilla. Tavoite on saada kaikki neljä sanulia ratkaistua yhdeksällä arvauksella."}
            </p>
            <p>
                {"Sanulistoja muokkailen aina välillä käyttäjien ehdotusten perusteella, ja voit jättää omat ehdotuksesi sanuleihin "}
                <a class="link" href={FORMS_LINK_TEMPLATE_ADD}>{"täällä"}</a>
                {". Kiitos kaikille ehdotuksia jättäneille ja sanulistojen kasaamisessa auttaneille henkilöille!"}
            </p>
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct MenuModalProps {
    pub callback: Callback<Msg>,
    pub word_length: usize,
    pub game_mode: GameMode,
    pub current_word_list: WordList,
    pub allow_profanities: bool,
    pub theme: Theme,

    pub max_streak: usize,
    pub total_played: usize,
    pub total_solved: usize,
}

#[function_component(MenuModal)]
pub fn menu_modal(props: &MenuModalProps) -> Html {
    let callback = props.callback.clone();
    let today = Local::now().naive_local().date();
    let toggle_menu = onmousedown!(callback, Msg::ToggleMenu);

    let change_word_length_5 = onmousedown!(callback, Msg::ChangeWordLength(5));
    let change_word_length_6 = onmousedown!(callback, Msg::ChangeWordLength(6));

    let change_game_mode_classic = onmousedown!(callback, Msg::ChangeGameMode(GameMode::Classic));
    let change_game_mode_relay = onmousedown!(callback, Msg::ChangeGameMode(GameMode::Relay));
    let change_game_mode_daily =
        onmousedown!(callback, Msg::ChangeGameMode(GameMode::DailyWord(today)));
    let change_game_mode_quadruple =
        onmousedown!(callback, Msg::ChangeGameMode(GameMode::Quadruple));

    let change_word_list_easy = onmousedown!(callback, Msg::ChangeWordList(WordList::Easy));
    let change_word_list_common = onmousedown!(callback, Msg::ChangeWordList(WordList::Common));
    let change_word_list_full = onmousedown!(callback, Msg::ChangeWordList(WordList::Full));

    let change_allow_profanities_yes = onmousedown!(callback, Msg::ChangeAllowProfanities(true));
    let change_allow_profanities_no = onmousedown!(callback, Msg::ChangeAllowProfanities(false));

    let change_theme_dark = onmousedown!(callback, Msg::ChangeTheme(Theme::Dark));
    let change_theme_colorblind = onmousedown!(callback, Msg::ChangeTheme(Theme::Colorblind));

    let is_hide_settings = matches!(props.game_mode, GameMode::DailyWord(_) | GameMode::Shared);

    html! {
        <div class="modal">
            <span onmousedown={toggle_menu} class="modal-close">{"✖"}</span>
            {if !is_hide_settings {
                html! {
                    <>
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
                                <button class={classes!("select", (props.current_word_list == WordList::Easy).then(|| Some("select-active")))}
                                    onmousedown={change_word_list_easy}>
                                    {"Helppo"}
                                </button>
                                <button class={classes!("select", (props.current_word_list == WordList::Common).then(|| Some("select-active")))}
                                    onmousedown={change_word_list_common}>
                                    {"Tavallinen"}
                                </button>
                                <button class={classes!("select", (props.current_word_list == WordList::Full).then(|| Some("select-active")))}
                                    onmousedown={change_word_list_full}>
                                    {"Vaikea"}
                                </button>
                            </div>
                        </div>
                        <div>
                            <label class="label">{"Rumat sanulit:"}</label>
                            <div class="select-container">
                                <button class={classes!("select", (!props.allow_profanities).then(|| Some("select-active")))}
                                    onmousedown={change_allow_profanities_no}>
                                    {"Ei"}
                                </button>
                                <button class={classes!("select", (props.allow_profanities).then(|| Some("select-active")))}
                                    onmousedown={change_allow_profanities_yes}>
                                    {"Kyllä"}
                                </button>
                            </div>
                        </div>
                    </>
                }
            } else {
                html! {}
            }}
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
                    <button class={classes!("select", (props.game_mode == GameMode::Quadruple).then(|| Some("select-active")))}
                        onmousedown={change_game_mode_quadruple}>
                        {"Neluli"}
                    </button>
                    <button class={classes!("select", matches!(props.game_mode, GameMode::DailyWord(_)).then(|| Some("select-active")))}
                        onclick={change_game_mode_daily}>
                        {"Päivän sanuli"}
                    </button>
                </div>
            </div>
            <div>
                <label class="label">{"Omat tilastosi:"}</label>
                <ul>
                    <li class="statistics">{format!("Pisin putki: {}", props.max_streak)}</li>
                    <li class="statistics">{format!("Pelatut sanulit: {}", props.total_played)}</li>
                    <li class="statistics">{format!("Ratkaistut sanulit: {}", props.total_solved)}</li>
                </ul>
            </div>
            <div>
                <label class="label">{"Teema:"}</label>
                <div class="select-container">
                    <button class={classes!("select", (props.theme == Theme::Dark).then(|| Some("select-active")))}
                        onmousedown={change_theme_dark}>
                        {"Oletus"}
                    </button>
                    <button class={classes!("select", (props.theme == Theme::Colorblind).then(|| Some("select-active")))}
                        onmousedown={change_theme_colorblind}>
                        {"Värisokeille"}
                    </button>
                </div>
            </div>
            <div class="version">
                <a class="version" href={CHANGELOG_URL} target="_blank">{ VERSION }</a>
            </div>
        </div>
    }
}
