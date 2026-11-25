use crate::logic::theme;
use crate::models::{MeResponse, TrainerSummary};
use crate::services::api::{fetch_account_status, fetch_trainers, logout};
use crate::services::pwa::isNavigatorOnline;
use crate::Route;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Home)]
pub fn home() -> Html {
    let trainers = use_state(Vec::<TrainerSummary>::new);
    let me = use_state(MeResponse::default);
    let offline = use_state(|| !isNavigatorOnline());
    let error = use_state(|| None::<String>);

    {
        let trainers = trainers.clone();
        let me = me.clone();
        let offline = offline.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            theme::set_body_background("var(--backgroundDarker)");
            spawn_local(async move {
                match fetch_trainers().await {
                    Ok(value) => trainers.set(value),
                    Err(message) => error.set(Some(message)),
                }
            });
            spawn_local(async move {
                let (account, is_offline) = fetch_account_status().await;
                me.set(account);
                offline.set(is_offline);
            });
            || ()
        });
    }

    let mut puzzles = Vec::<(String, Vec<TrainerSummary>)>::new();
    for trainer in trainers.iter() {
        match puzzles
            .iter_mut()
            .find(|(puzzle, _)| puzzle == &trainer.puzzle)
        {
            Some((_, entries)) => entries.push(trainer.clone()),
            None => puzzles.push((trainer.puzzle.clone(), vec![trainer.clone()])),
        }
    }

    let on_logout = Callback::from(|_| {
        spawn_local(async move {
            let _ = logout().await;
            if let Some(window) = web_sys::window() {
                let _ = window.location().reload();
            }
        });
    });

    html! {
        <main class="colFlex" style="background-color: var(--backgroundDarker); align-items:center; width:100%; min-height:100%;">
            <div class="colFlex pad"><h3>{"Welcome to ALLG-Trainer"}</h3></div>
            <div class="rowFlex pad" style="justify-content: center;">
                if me.authenticated {
                    <span class="borderedContainer pad">
                        {if *offline {
                            format!("Signed in as {} (offline)", me.email.as_deref().unwrap_or("WCA user"))
                        } else {
                            format!("Signed in as {}", me.email.as_deref().unwrap_or("WCA user"))
                        }}
                    </span>
                    if !*offline {
                        <button class="abutton" onclick={on_logout}>{"Sign out"}</button>
                    }
                } else if *offline {
                    <span class="borderedContainer pad">{"Offline mode"}</span>
                } else {
                    <a class="abutton" href="/api/auth/wca">{"Sign in with the WCA"}</a>
                }
            </div>
            <div id="algtrainer-selections" class="rowFlex pad">
                { for puzzles.into_iter().map(|(puzzle, entries)| {
                    let icon = format!("/catalog/{}/icon.png", entries[0].trainer_id);
                    html! {
                        <div class="borderedContainer pad colFlex">
                            <div class="puzzle-header borderedContainer pad" style="background-color: var(--backgroundDarker);">
                                <span>{puzzle}</span>
                                <img style="height: 1.4em;" src={icon} />
                            </div>
                            <div class="rowFlex">
                                { for entries.into_iter().map(|entry| html! {
                                    <Link<Route> classes="abutton grow" to={Route::Trainer { id: entry.trainer_id }}>
                                        <b>{entry.name}</b>
                                    </Link<Route>>
                                }) }
                            </div>
                        </div>
                    }
                }) }
                if let Some(message) = error.as_ref() {
                    <div class="borderedContainer pad">{message}</div>
                }
            </div>
            <span class="version" style="text-align: center;">{"v2.0.0"}</span>
        </main>
    }
}
