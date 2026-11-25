use crate::logic::results::format_time;
use gloo_timers::callback::{Interval, Timeout};
use js_sys::Date;
use web_sys::PointerEvent;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TrainerTimer {
    pub running: bool,
    pub ready: bool,
    pub stopped: bool,
    pub elapsed_ms: i64,
    pub start_hold: Callback<bool>,
    pub release: Callback<bool>,
    pub finish: Callback<()>,
    pub abort: Callback<()>,
    pub cancel_hold: Callback<()>,
    pub on_pointer_down: Callback<PointerEvent>,
    pub on_pointer_up: Callback<PointerEvent>,
    pub on_pointer_cancel: Callback<PointerEvent>,
}

impl TrainerTimer {
    pub fn color(&self) -> &'static str {
        if self.ready {
            "#008500"
        } else if self.running {
            "inherit"
        } else if self.stopped {
            "#850000"
        } else {
            "inherit"
        }
    }

    pub fn display_text(&self) -> String {
        if self.running || self.elapsed_ms > 0 {
            format_time(self.elapsed_ms)
        } else {
            "ready".to_owned()
        }
    }
}

#[hook]
pub fn use_trainer_timer(
    settings: UseStateHandle<serde_json::Value>,
    on_finish: Callback<i64>,
) -> TrainerTimer {
    let running = use_state(|| false);
    let ready = use_state(|| false);
    let stopped = use_state(|| false);
    let elapsed_ms = use_state(|| 0_i64);
    let started_at = use_mut_ref(|| 0.0_f64);
    let hold_timeout = use_mut_ref(|| None::<Timeout>);
    let interval = use_mut_ref(|| None::<Interval>);
    let stopped_timeout = use_mut_ref(|| None::<Timeout>);

    let finish = {
        let running = running.clone();
        let ready = ready.clone();
        let stopped = stopped.clone();
        let elapsed_ms = elapsed_ms.clone();
        let started_at = started_at.clone();
        let interval = interval.clone();
        let stopped_timeout = stopped_timeout.clone();
        Callback::from(move |_| {
            if !*running {
                return;
            }
            let ms = (Date::now() - *started_at.borrow()).round() as i64;
            elapsed_ms.set(ms);
            running.set(false);
            ready.set(false);
            stopped.set(true);
            if let Some(timeout) = stopped_timeout.borrow_mut().take() {
                timeout.cancel();
            }
            let stopped = stopped.clone();
            *stopped_timeout.borrow_mut() = Some(Timeout::new(100, move || stopped.set(false)));
            if let Some(timer) = interval.borrow_mut().take() {
                timer.cancel();
            }
            on_finish.emit(ms);
        })
    };

    let abort = {
        let running = running.clone();
        let ready = ready.clone();
        let stopped = stopped.clone();
        let elapsed_ms = elapsed_ms.clone();
        let interval = interval.clone();
        let hold_timeout = hold_timeout.clone();
        let stopped_timeout = stopped_timeout.clone();
        Callback::from(move |_| {
            running.set(false);
            ready.set(false);
            stopped.set(false);
            elapsed_ms.set(0);
            if let Some(timeout) = stopped_timeout.borrow_mut().take() {
                timeout.cancel();
            }
            if let Some(timer) = interval.borrow_mut().take() {
                timer.cancel();
            }
            if let Some(timeout) = hold_timeout.borrow_mut().take() {
                timeout.cancel();
            }
        })
    };

    let start_hold = {
        let ready = ready.clone();
        let stopped = stopped.clone();
        let hold_timeout = hold_timeout.clone();
        let stopped_timeout = stopped_timeout.clone();
        let settings = settings.clone();
        Callback::from(move |clear_stopped: bool| {
            if hold_timeout.borrow().is_none() && !*ready {
                if clear_stopped {
                    stopped.set(false);
                    if let Some(timeout) = stopped_timeout.borrow_mut().take() {
                        timeout.cancel();
                    }
                }
                let delay = if settings
                    .get("enableHoldDelay")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true)
                {
                    200
                } else {
                    0
                };
                let ready = ready.clone();
                *hold_timeout.borrow_mut() = Some(Timeout::new(delay, move || ready.set(true)));
            }
        })
    };

    let release = {
        let running = running.clone();
        let ready = ready.clone();
        let stopped = stopped.clone();
        let elapsed_ms = elapsed_ms.clone();
        let started_at = started_at.clone();
        let hold_timeout = hold_timeout.clone();
        let interval = interval.clone();
        let stopped_timeout = stopped_timeout.clone();
        Callback::from(move |reset_elapsed_on_early_release: bool| {
            if *running {
                return;
            }
            if let Some(timeout) = hold_timeout.borrow_mut().take() {
                timeout.cancel();
            }
            if !*ready {
                if reset_elapsed_on_early_release {
                    elapsed_ms.set(0);
                }
                return;
            }
            ready.set(false);
            stopped.set(false);
            if let Some(timeout) = stopped_timeout.borrow_mut().take() {
                timeout.cancel();
            }
            let start = Date::now();
            *started_at.borrow_mut() = start;
            running.set(true);
            elapsed_ms.set(0);
            let elapsed_ms = elapsed_ms.clone();
            let started_at = started_at.clone();
            *interval.borrow_mut() = Some(Interval::new(20, move || {
                elapsed_ms.set((Date::now() - *started_at.borrow()).round() as i64);
            }));
        })
    };

    let cancel_hold = {
        let ready = ready.clone();
        let hold_timeout = hold_timeout.clone();
        Callback::from(move |_| {
            if let Some(timeout) = hold_timeout.borrow_mut().take() {
                timeout.cancel();
            }
            ready.set(false);
        })
    };

    let on_pointer_down = {
        let is_running = *running;
        let finish = finish.clone();
        let start_hold = start_hold.clone();
        Callback::from(move |event: PointerEvent| {
            if event.pointer_type() != "touch" && event.pointer_type() != "pen" {
                return;
            }
            event.prevent_default();
            if is_running {
                finish.emit(());
            } else {
                start_hold.emit(true);
            }
        })
    };

    let on_pointer_up = {
        let release = release.clone();
        Callback::from(move |event: PointerEvent| {
            if event.pointer_type() != "touch" && event.pointer_type() != "pen" {
                return;
            }
            event.prevent_default();
            release.emit(true);
        })
    };

    let on_pointer_cancel = {
        let cancel_hold = cancel_hold.clone();
        Callback::from(move |event: PointerEvent| {
            event.prevent_default();
            cancel_hold.emit(());
        })
    };

    TrainerTimer {
        running: *running,
        ready: *ready,
        stopped: *stopped,
        elapsed_ms: *elapsed_ms,
        start_hold,
        release,
        finish,
        abort,
        cancel_hold,
        on_pointer_down,
        on_pointer_up,
        on_pointer_cancel,
    }
}
