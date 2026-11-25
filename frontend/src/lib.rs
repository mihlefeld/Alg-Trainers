mod components;
mod logic;
mod models;
mod pages;
mod services;

use crate::components::update_prompt::UpdatePrompt;
use crate::pages::alg_sheet::AlgSheetPage;
use crate::pages::home::Home;
use crate::pages::trainer::TrainerPage;
use crate::services::settings::SettingsProvider;
use crate::services::sync::start_sync_engine;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;

pub const APP_VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AlgSheetQuery {
    pub algset: String,
}

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/trainer/:id")]
    Trainer { id: String },
    #[at("/algsheet")]
    AlgSheet,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(App)]
fn app() -> Html {
    start_sync_engine();
    html! {
        <SettingsProvider>
            <UpdatePrompt />
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </SettingsProvider>
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <Home /> },
        Route::Trainer { id } => html! { <TrainerPage id={id} /> },
        Route::AlgSheet => html! { <AlgSheetPage /> },
        Route::NotFound => html! {
            <main class="colFlex pad"><h1>{"Page not found"}</h1></main>
        },
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<App>::new().render();
}
