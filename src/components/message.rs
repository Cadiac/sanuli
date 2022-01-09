use yew::prelude::*;

const FORMS_LINK_TEMPLATE_ADD: &str = "https://docs.google.com/forms/d/e/1FAIpQLSfH8gs4sq-Ynn8iGOvlc99J_zOG2rJEC4m8V0kCgF_en3RHFQ/viewform?usp=pp_url&entry.461337706=Lis%C3%A4yst%C3%A4&entry.560255602=";
const FORMS_LINK_TEMPLATE_DEL: &str = "https://docs.google.com/forms/d/e/1FAIpQLSfH8gs4sq-Ynn8iGOvlc99J_zOG2rJEC4m8V0kCgF_en3RHFQ/viewform?usp=pp_url&entry.461337706=Poistoa&entry.560255602=";
const DICTIONARY_LINK_TEMPLATE: &str = "https://www.kielitoimistonsanakirja.fi/#/";

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub message: String,
    pub is_unknown: bool,
    pub is_winner: bool,
    pub is_guessing: bool,
    pub word: String,
    pub last_guess: String,
}

#[function_component(Message)]
pub fn message(props: &Props) -> Html {
    html! {
        <div class="message">
            { &props.message }
            <div class="message-small">{{
                if props.is_unknown {
                    let last_guess = props.last_guess.to_lowercase();

                    html! {
                        <a class="link" href={format!("{}{}", FORMS_LINK_TEMPLATE_ADD, last_guess)}
                            target="_blank">{ "Ehdota lisäystä?" }
                        </a>
                    }
                } else if !props.is_winner & !props.is_guessing {
                    let word = props.word.to_lowercase();

                    html! {
                        <>
                            <a class="link" href={format!("{}{}?searchMode=all", DICTIONARY_LINK_TEMPLATE, word)}
                                target="_blank">{ "Sanakirja" }
                            </a>
                            {" | "}
                            <a class="link" href={format!("{}{}", FORMS_LINK_TEMPLATE_DEL, word)}
                                target="_blank">{ "Ehdota poistoa?" }
                            </a>
                        </>
                    }
                } else {
                    html! {}
                }
            }}
            </div>
        </div>
    }
}