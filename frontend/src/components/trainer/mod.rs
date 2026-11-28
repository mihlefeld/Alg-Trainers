mod credits_dialog;
mod hint_dialog;
mod hotkeys_dialog;
mod practice_view;
mod presets_panel;
mod search_panel;
mod selection_view;
mod settings_dialog;
mod state;
mod stats_dialog;
mod timer;

use crate::components::trainer::credits_dialog::CreditsDialog;
use crate::components::trainer::hint_dialog::HintDialog;
use crate::components::trainer::hotkeys_dialog::HotkeysDialog;
use crate::components::trainer::practice_view::PracticeView;
use crate::components::trainer::selection_view::SelectionView;
use crate::components::trainer::state::use_trainer_controller;
use crate::components::trainer::stats_dialog::StatsDialog;
use crate::models::{Dialog, Mode, TrainerBundle};
use std::rc::Rc;
use yew::prelude::*;

pub use settings_dialog::SettingsDialog;

#[derive(Properties, PartialEq)]
pub struct LoadedTrainerProps {
    pub bundle: TrainerBundle,
}

#[function_component(LoadedTrainer)]
pub fn loaded_trainer(props: &LoadedTrainerProps) -> Html {
    let ctrl = use_trainer_controller(Rc::new(props.bundle.clone()));

    html! {
        <main
            id="trainer-main"
            tabindex="0"
            style="display:flex; flex-direction:column; align-items:center; width:100%;"
            onkeydown={ctrl.on_key_down}
            onkeyup={ctrl.on_key_up}
        >
            if ctrl.mode == Mode::Select {
                <SelectionView
                    bundle={ctrl.bundle.clone()}
                    cases_by_id={ctrl.cases_by_id.clone()}
                    selected={ctrl.selected.clone()}
                    selected_algsets={ctrl.selected_algsets.clone()}
                    presets={ctrl.presets.clone()}
                    settings={ctrl.settings.clone()}
                    probabilities={ctrl.probabilities}
                    cube_colors={ctrl.cube_colors.clone()}
                    search_open={ctrl.search_open}
                    search_query={ctrl.search_query}
                    search_cells={ctrl.search_cells}
                    search_color={ctrl.search_color}
                    search_all_angles={ctrl.search_all_angles}
                    on_open_settings={ctrl.on_open_settings}
                    on_start_training={ctrl.on_start_training}
                    on_open_dialog={ctrl.on_open_dialog}
                    on_toggle_case={ctrl.on_toggle_case}
                    on_toggle_case_group={ctrl.on_toggle_case_group}
                    on_toggle_algset={ctrl.on_toggle_algset}
                    on_use_preset={ctrl.on_use_preset}
                    on_add_preset_to_selection={ctrl.on_add_preset_to_selection}
                    on_remove_preset_from_selection={ctrl.on_remove_preset_from_selection}
                    on_create_preset={ctrl.on_save_preset.clone()}
                    on_update_preset={ctrl.on_save_preset}
                    on_delete_preset={ctrl.on_delete_preset}
                    on_open_hint={ctrl.on_open_hint_for_case}
                    on_toggle_search_open={ctrl.on_toggle_search_open}
                    on_search_query_change={ctrl.on_search_query_change}
                    on_search_cells_change={ctrl.on_search_cells_change}
                    on_search_color_change={ctrl.on_search_color_change}
                    on_search_all_angles_change={ctrl.on_search_all_angles_change}
                />
            } else {
                <PracticeView
                    selected_count={ctrl.selected.len()}
                    recap_active={ctrl.recap_active}
                    recap_remaining_count={ctrl.recap_remaining_count}
                    current_scramble={ctrl.current_scramble}
                    last_scramble={ctrl.last_scramble}
                    last_case={ctrl.last_case}
                    presets={ctrl.presets}
                    results={ctrl.results.clone()}
                    cases_by_id={ctrl.cases_by_id.clone()}
                    settings={ctrl.settings.clone()}
                    timer={ctrl.timer}
                    on_back_to_selection={ctrl.on_back_to_selection}
                    on_start_recap={ctrl.on_start_training}
                    on_switch_to_train={ctrl.on_switch_to_train}
                    on_open_settings={ctrl.on_open_settings}
                    on_open_active_hint={ctrl.on_open_active_hint}
                    on_open_last_hint={ctrl.on_open_last_hint}
                    on_open_case_hint={ctrl.on_open_hint_for_case}
                    on_open_stats={ctrl.on_open_stats}
                    on_clear_results={ctrl.on_clear_results}
                    on_bookmark_last_case={ctrl.on_bookmark_last_case}
                    on_remove_last_case={ctrl.on_remove_last_case}
                    on_remove_last_time={ctrl.on_remove_last_time}
                />
            }

            if ctrl.hint_open {
                <HintDialog
                    case={ctrl.hint_case}
                    custom_algs={ctrl.custom_algs}
                    cube_colors={ctrl.cube_colors}
                    on_close={ctrl.on_close_hint}
                    on_prev={{
                        let step = ctrl.on_step_hint.clone();
                        Callback::from(move |_| step.emit(-1))
                    }}
                    on_next={{
                        let step = ctrl.on_step_hint;
                        Callback::from(move |_| step.emit(1))
                    }}
                    on_save_alg={ctrl.on_save_alg}
                    on_hide_alg={ctrl.on_hide_alg}
                    on_delete_alg={ctrl.on_delete_alg}
                    on_reorder_alg={ctrl.on_reorder_alg}
                />
            }

            if ctrl.dialog == Dialog::Stats {
                <StatsDialog
                    case_id={ctrl.stats_case_id}
                    results={ctrl.results}
                    cases_by_id={ctrl.cases_by_id}
                    on_close={ctrl.on_close_dialog.clone()}
                    on_delete_result={ctrl.on_delete_result}
                />
            }

            if ctrl.dialog == Dialog::Settings {
                <SettingsDialog
                    settings={ctrl.settings}
                    settings_ready={ctrl.settings_ready}
                    selection_key={ctrl.bundle.selection_key.clone()}
                    on_update_settings={ctrl.on_update_settings}
                    on_close={ctrl.on_close_dialog.clone()}
                />
            }

            if ctrl.dialog == Dialog::Hotkeys {
                <HotkeysDialog on_close={ctrl.on_close_dialog.clone()} />
            }

            if ctrl.dialog == Dialog::Credits {
                <CreditsDialog on_close={ctrl.on_close_dialog} />
            }
        </main>
    }
}
