use crate::logic::theme::case_image_data;
use std::collections::BTreeMap;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CaseImageProps {
    pub svg: AttrValue,
    pub colors: Rc<BTreeMap<String, String>>,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: Option<AttrValue>,
}

#[function_component(CaseImage)]
pub fn case_image(props: &CaseImageProps) -> Html {
    let src = case_image_data(&props.svg, &props.colors);
    html! {
        <img class={props.class.clone()} style={props.style.clone()} {src} />
    }
}
