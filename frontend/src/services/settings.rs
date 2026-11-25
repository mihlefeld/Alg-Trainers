use crate::logic::{cube_schemes, theme};
use crate::services::storage::{load_settings, save_settings_async};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct SettingsContext {
    pub settings: Rc<serde_json::Value>,
    pub ready: bool,
    pub update: Callback<serde_json::Value>,
}

#[derive(Properties, PartialEq)]
pub struct SettingsProviderProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(SettingsProvider)]
pub fn settings_provider(props: &SettingsProviderProps) -> Html {
    let settings = use_state(|| {
        let mut initial = serde_json::json!({});
        cube_schemes::migrate(&mut initial);
        Rc::new(initial)
    });
    let ready = use_state(|| false);

    {
        let settings = settings.clone();
        let ready = ready.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let loaded = load_settings(true).await;
                theme::apply_settings(&loaded);
                settings.set(Rc::new(loaded));
                ready.set(true);
            });
            || ()
        });
    }

    let update = {
        let settings = settings.clone();
        Callback::from(move |next: serde_json::Value| {
            theme::apply_settings(&next);
            settings.set(Rc::new(next.clone()));
            save_settings_async(next);
        })
    };

    let context = SettingsContext {
        settings: (*settings).clone(),
        ready: *ready,
        update,
    };

    html! {
        <ContextProvider<SettingsContext> {context}>
            if *ready {
                { for props.children.iter() }
            }
        </ContextProvider<SettingsContext>>
    }
}

#[hook]
pub fn use_settings() -> SettingsContext {
    use_context::<SettingsContext>().expect("SettingsProvider must wrap the application")
}
