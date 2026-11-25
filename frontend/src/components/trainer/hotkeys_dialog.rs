use crate::components::modal::ModalDialog;
use yew::prelude::*;

const HOTKEYS: [(&str, &str); 9] = [
    ("Delete", "Delete last time"),
    ("Shift + Delete", "Clear session"),
    ("R", "Remove last case from selection"),
    ("X", "Return to case selection"),
    ("B", "Bookmark case"),
    ("H", "Show hint for current case"),
    ("Shift + H", "Show hint for last case"),
    ("Space", "Start / stop timer"),
    ("Escape", "Close dialog"),
];

#[derive(Properties, PartialEq)]
pub struct HotkeysDialogProps {
    pub on_close: Callback<()>,
}

#[function_component(HotkeysDialog)]
pub fn hotkeys_dialog(props: &HotkeysDialogProps) -> Html {
    html! {
        <ModalDialog id="hotkeysDialog" on_close={props.on_close.clone()}>
            <h2>{"Hotkeys"}</h2>
            {for HOTKEYS.into_iter().map(|(key, action)| html! {
                <div class="borderedContainer pad rowFlex hotkeyRow">
                    <span class="hotKeyText">{action}</span>
                    <kbd class="hotkey">{key}</kbd>
                </div>
            })}
        </ModalDialog>
    }
}
