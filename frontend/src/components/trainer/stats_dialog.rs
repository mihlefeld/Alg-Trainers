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

    html! {
        <ModalDialog id="caseTimeDetails" class="colFlex" on_close={props.on_close.clone()}>
            <div class="colFlex borderedContainer pad" style="width:100%;box-sizing:border-box;">
                <h2 id="caseTimeDetailsCaseHeader">{title}</h2>
                <div class="rowFlex" style="justify-content:space-around;flex-wrap:wrap;">
                    <b>{format!("Times: {}", summary.count)}</b>
                    <b>{format!("Mean: {:.2}s", summary.mean)}</b>
                    <b>{format!("Range: {:.2}-{:.2}s", summary.min, summary.max)}</b>
                </div>
                <div id="timeHistogram" class="rowFlex" style="height:8em;align-items:end;padding:0.5em;">
                    { for summary.histogram_counts.iter().map(|count| html! {
                        <div
                            style={format!(
                                "flex:1;height:{}%;background:var(--primaryBG);min-width:0.5em;",
                                *count as f64 * 100.0 / summary.max_histogram_count as f64
                            )}
                            title={count.to_string()}
                        ></div>
                    }) }
                </div>
                <svg id="timePlot" viewBox="0 0 280 130" style="width:100%;height:10em;background:var(--background);">
                    <polyline points={summary.trend_points} fill="none" stroke="var(--secondary)" stroke-width="2" />
                    <polyline points={summary.plot_points} fill="none" stroke="var(--primary)" stroke-width="3" />
                </svg>
                <div id="caseTimeDetailsTimes" style="max-height:12em;overflow-y:auto;">
                    { for case_stats.iter().rev().map(|result| html! {
                        <div>{&result.time}{format!(" - {}", &result.datetime)}</div>
                    }) }
                </div>
            </div>
        </ModalDialog>
    }
}
