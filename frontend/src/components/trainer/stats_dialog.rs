use crate::components::modal::ModalDialog;
use crate::logic::results::compute_case_stats;
use crate::models::{StoredResult, TrainerCase};
use std::collections::BTreeMap;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatsDialogProps {
    pub case_id: String,
    pub results: Vec<StoredResult>,
    pub cases_by_id: Rc<BTreeMap<String, TrainerCase>>,
    pub on_close: Callback<()>,
    pub on_delete_result: Callback<String>,
}

#[function_component(StatsDialog)]
pub fn stats_dialog(props: &StatsDialogProps) -> Html {
    let case_stats = if props.case_id.is_empty() {
        props.results.iter().collect::<Vec<_>>()
    } else {
        props
            .results
            .iter()
            .filter(|result| result.case == props.case_id)
            .collect::<Vec<_>>()
    };
    let title = if props.case_id.is_empty() {
        "All cases".to_owned()
    } else {
        props
            .cases_by_id
            .get(&props.case_id)
            .map(|case| case.name.clone())
            .unwrap_or_else(|| props.case_id.clone())
    };
    let summary = compute_case_stats(&case_stats);
    let histogram_bin_count = summary.histogram_counts.len();
    let histogram_bin_width = 240.0 / histogram_bin_count.max(1) as f64;
    let histogram_range = summary.max - summary.min;
    let histogram_scale = summary.max_histogram_count as f64;

    html! {
        <ModalDialog id="caseTimeDetails" class="colFlex" on_close={props.on_close.clone()}>
            <div class="statsDialogContent borderedContainer pad">
                <div class="statsDialogHeader">
                    <h2 id="caseTimeDetailsCaseHeader">{title}</h2>
                    <div class="statsSummary">
                        <b>{format!("Times: {}", summary.count)}</b>
                        <b>{format!("Mean: {:.2}s", summary.mean)}</b>
                        <b>{format!("Range: {:.2}-{:.2}s", summary.min, summary.max)}</b>
                    </div>
                </div>
                <div class="statsVisualizations">
                    <svg id="timeHistogram" viewBox="0 0 280 130">
                        { for (0..6).map(|index| {
                            let y = 10.0 + index as f64 * 20.0;
                            html! {
                                <line x1="30" y1={y.to_string()} x2="270" y2={y.to_string()} stroke="var(--text)" stroke-opacity="0.18" stroke-width="1" />
                            }
                        }) }
                        { for (1..=5).map(|index| {
                            let x = 30.0 + index as f64 * 48.0;
                            html! {
                                <line x1={x.to_string()} y1="10" x2={x.to_string()} y2="110" stroke="var(--text)" stroke-opacity="0.12" stroke-width="1" />
                            }
                        }) }
                        { for (0..=5).map(|index| {
                            let y = 110.0 - index as f64 * 20.0;
                            let value = histogram_scale * index as f64 / 5.0;
                            html! {
                                <>
                                    <line x1="26" y1={y.to_string()} x2="30" y2={y.to_string()} stroke="var(--text)" stroke-width="1" />
                                    <text x="24" y={(y + 3.0).to_string()} fill="var(--text)" font-size="7" text-anchor="end">{format!("{value:.0}")}</text>
                                </>
                            }
                        }) }
                        { for summary.histogram_counts.iter().enumerate().map(|(index, count)| {
                            let x = 30.0 + index as f64 * histogram_bin_width + 2.0;
                            let height = *count as f64 * 100.0 / histogram_scale;
                            html! {
                                <>
                                    <rect x={x.to_string()} y={(110.0 - height).to_string()} width={(histogram_bin_width - 4.0).to_string()} height={height.to_string()} fill="var(--primaryBG)" title={count.to_string()} />
                                    <line x1={(x + (histogram_bin_width - 4.0) / 2.0).to_string()} y1="110" x2={(x + (histogram_bin_width - 4.0) / 2.0).to_string()} y2="114" stroke="var(--text)" stroke-width="1" />
                                </>
                            }
                        }) }
                        { for (0..=2).map(|index| {
                            let bin = if index == 2 { histogram_bin_count.saturating_sub(1) } else if index == 1 { histogram_bin_count / 2 } else { 0 };
                            let x = 30.0 + (bin as f64 + 0.5) * histogram_bin_width;
                            let value = if histogram_bin_count > 0 {
                                summary.min + histogram_range * bin as f64 / histogram_bin_count as f64
                            } else {
                                summary.min
                            };
                            html! {
                                <text x={x.to_string()} y="127" fill="var(--text)" font-size="7" text-anchor="middle">{format!("{value:.1}s")}</text>
                            }
                        }) }
                    </svg>
                    <svg id="timePlot" viewBox="0 0 280 130">
                        { for (0..6).map(|index| {
                            let y = 20.0 + index as f64 * 20.0;
                            html! {
                                <line x1="30" y1={y.to_string()} x2="270" y2={y.to_string()} stroke="var(--text)" stroke-opacity="0.18" stroke-width="1" />
                            }
                        }) }
                        { for (0..6).map(|index| {
                            let x = 30.0 + index as f64 * 48.0;
                            html! {
                                <line x1={x.to_string()} y1="20" x2={x.to_string()} y2="120" stroke="var(--text)" stroke-opacity="0.12" stroke-width="1" />
                            }
                        }) }
                        { for (0..=5).map(|index| {
                            let y = 120.0 - index as f64 * 20.0;
                            let value = summary.max * index as f64 / 5.0;
                            html! {
                                <>
                                    <line x1="26" y1={y.to_string()} x2="30" y2={y.to_string()} stroke="var(--text)" stroke-width="1" />
                                    <text x="24" y={(y + 3.0).to_string()} fill="var(--text)" font-size="7" text-anchor="end">{format!("{value:.1}")}</text>
                                </>
                            }
                        }) }
                        { for (0..=5).map(|index| {
                            let x = 30.0 + index as f64 * 48.0;
                            html! {
                                <>
                                    <line x1={x.to_string()} y1="120" x2={x.to_string()} y2="124" stroke="var(--text)" stroke-width="1" />
                                    <text x={x.to_string()} y="129" fill="var(--text)" font-size="7" text-anchor="middle">{index}</text>
                                </>
                            }
                        }) }
                        <polyline points={summary.trend_points} fill="none" stroke="var(--secondary)" stroke-width="2" />
                        <polyline points={summary.plot_points} fill="none" stroke="var(--primary)" stroke-width="3" />
                    </svg>
                </div>
                <div id="caseTimeDetailsTimes" class="statsTimes">
                    { for case_stats.iter().rev().map(|result| html! {
                        <button
                            type="button"
                            class="statsTimeEntry"
                            title="Delete this time"
                            onclick={{
                                let result_id = result.id.clone();
                                let result_time = result.time.clone();
                                let on_delete_result = props.on_delete_result.clone();
                                Callback::from(move |_| {
                                    let message = format!("Delete this time ({result_time})?");
                                    if web_sys::window()
                                        .is_some_and(|window| window.confirm_with_message(&message).unwrap_or(false))
                                    {
                                        on_delete_result.emit(result_id.clone());
                                    }
                                })
                            }}
                        >
                            {&result.time}{format!(" - {}", &result.datetime)}
                        </button>
                    }) }
                </div>
            </div>
        </ModalDialog>
    }
}
