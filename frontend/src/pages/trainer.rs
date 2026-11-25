use crate::components::trainer::LoadedTrainer;
use crate::models::TrainerBundle;
use crate::services::api::fetch_trainer_bundle;
use crate::Route;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TrainerPageProps {
    pub id: String,
}

#[function_component(TrainerPage)]
pub fn trainer_page(props: &TrainerPageProps) -> Html {
    let bundle = use_state(|| None::<TrainerBundle>);
    let error = use_state(|| None::<String>);

    {
        let bundle = bundle.clone();
        let error = error.clone();
        let id = props.id.clone();
        use_effect_with(id.clone(), move |_| {
            bundle.set(None);
            error.set(None);
            spawn_local(async move {
                match fetch_trainer_bundle(&id).await {
                    Ok(value) => bundle.set(Some(value)),
                    Err(cause) => error.set(Some(cause)),
                }
            });
            || ()
        });
    }

    match bundle.as_ref() {
        Some(bundle) => {
            html! { <LoadedTrainer key={bundle.trainer_id.clone()} bundle={bundle.clone()} /> }
        }
        None => html! {
            <main class="colFlex pad">
                <h1>{error.as_deref().unwrap_or("Loading trainer...")}</h1>
                <Link<Route> to={Route::Home}>{"Back"}</Link<Route>>
            </main>
        },
    }
}
