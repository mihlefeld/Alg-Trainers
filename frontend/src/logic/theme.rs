use crate::services::storage::{colorInputStyle, prefersLightTheme};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::collections::BTreeMap;

pub fn set_body_background(value: &str) {
    if let Some(body) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.body())
    {
        let _ = body.style().set_property("background", value);
    }
}

pub fn apply_settings(settings: &serde_json::Value) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };
    let base_size = settings
        .get("baseSize")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(1.0);
    let style = body.style();
    let _ = style.set_property("font-size", &format!("{base_size}em"));
    let colors = settings
        .get("colors")
        .and_then(serde_json::Value::as_object);
    let default_theme = if prefersLightTheme().unwrap_or(false) {
        "light"
    } else {
        "dark"
    };
    for (key, fallback) in default_theme_colors(default_theme) {
        let value = colors
            .and_then(|colors| colors.get(key))
            .and_then(serde_json::Value::as_str)
            .unwrap_or(fallback);
        let _ = style.set_property(key, value);
    }
    let _ = style.set_property(
        "--backgroundDarker",
        "color-mix(in oklch, var(--background), black 20%)",
    );
    let derived_text = if default_theme == "light" {
        ("var(--text)", "var(--background)", "var(--text)")
    } else {
        (
            "var(--background)",
            "var(--background)",
            "var(--background)",
        )
    };
    let _ = style.set_property("--secondaryText", derived_text.0);
    let _ = style.set_property("--primaryText", derived_text.1);
    let _ = style.set_property("--accentText", derived_text.2);
    let primary = setting_color_with_theme(settings, "--primary", default_theme);
    let secondary = setting_color_with_theme(settings, "--secondary", default_theme);
    let accent = setting_color_with_theme(settings, "--accent", default_theme);
    let _ = style.set_property("--primaryBG", &primary);
    let _ = style.set_property(
        "--primaryBGHover",
        &format!("color-mix(in oklch, {primary}, var(--background) 18%)"),
    );
    let _ = style.set_property(
        "--secondaryHover",
        &format!("color-mix(in oklch, {secondary}, var(--background) 18%)"),
    );
    let _ = style.set_property(
        "--accentHover",
        &format!("color-mix(in oklch, {accent}, var(--background) 18%)"),
    );
}

pub fn default_theme_colors(theme: &str) -> BTreeMap<&'static str, &'static str> {
    if theme == "light" {
        BTreeMap::from([
            ("--text", "oklch(25% 0 0)"),
            ("--background", "oklch(96% 0.01 97)"),
            ("--secondary", "oklch(75% 0.09 99)"),
            ("--primary", "oklch(45% 0.08 136)"),
            ("--accent", "oklch(85% 0.16 98)"),
        ])
    } else {
        BTreeMap::from([
            ("--text", "oklch(99% 0 91)"),
            ("--background", "oklch(33% 0 0)"),
            ("--secondary", "oklch(69% 0.09 98)"),
            ("--primary", "oklch(82% 0.07 135)"),
            ("--accent", "oklch(91% 0.16 98)"),
        ])
    }
}

pub fn setting_color(settings: &serde_json::Value, key: &'static str) -> String {
    let theme = if prefersLightTheme().unwrap_or(false) {
        "light"
    } else {
        "dark"
    };
    setting_color_with_theme(settings, key, theme)
}

pub fn setting_color_with_theme(
    settings: &serde_json::Value,
    key: &'static str,
    theme: &str,
) -> String {
    settings
        .get("colors")
        .and_then(|colors| colors.get(key))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| default_theme_colors(theme)[key])
        .to_owned()
}

pub fn default_cube_colors() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([
        ("white", "#FAFAFA"),
        ("ignore", "#777777"),
        ("black", "#222222"),
        ("lightblue", "#A0CBE8"),
        ("blue", "#4E79A7"),
        ("red", "#E15759"),
        ("pink", "#FF9D9A"),
        ("beige", "#F4F3AA"),
        ("yellow", "#F1CE63"),
        ("green", "#59A14F"),
        ("lightgreen", "#8CD17D"),
        ("purple", "#B07AA1"),
        ("orange", "#F28E2B"),
    ])
}

pub fn color_input_style(value: &str, background: &str) -> String {
    match colorInputStyle(value, background) {
        Ok(style) => style,
        Err(error) => {
            web_sys::console::warn_1(&error);
            "background-color:var(--background);color:var(--text)".to_owned()
        }
    }
}

pub fn case_image_data(svg: &str, colors: &BTreeMap<String, String>) -> String {
    let originals = default_cube_colors();
    let mut rendered = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some(index) = rest.find('#') {
        rendered.push_str(&rest[..index]);
        rest = &rest[index..];
        if let Some((name, original)) = originals
            .iter()
            .find(|(_, original)| rest.starts_with(**original))
        {
            rendered.push_str(colors.get(*name).map(String::as_str).unwrap_or(original));
            rest = &rest[original.len()..];
        } else {
            rendered.push('#');
            rest = &rest[1..];
        }
    }
    rendered.push_str(rest);
    format!("data:image/svg+xml;base64,{}", STANDARD.encode(rendered))
}

pub fn adjust_size_setting(
    settings: &serde_json::Value,
    key: &'static str,
    amount: f64,
) -> serde_json::Value {
    let mut next = settings.clone();
    let current = next
        .get(key)
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(match key {
            "timerSize" => 5.0,
            "scrambleSize" => 2.0,
            _ => 1.0,
        });
    next[key] = serde_json::json!((current + amount).max(0.5));
    next
}

pub fn update_scramble_override(
    settings: &serde_json::Value,
    selection_key: &str,
    option: &'static str,
    value: &str,
) -> serde_json::Value {
    let mut next = settings.clone();
    if next
        .get("trainerScrambleOverrides")
        .and_then(serde_json::Value::as_object)
        .is_none()
    {
        next["trainerScrambleOverrides"] = serde_json::json!({});
    }
    if next["trainerScrambleOverrides"]
        .get(selection_key)
        .and_then(serde_json::Value::as_object)
        .is_none()
    {
        next["trainerScrambleOverrides"][selection_key] = serde_json::json!({});
    }
    let options = value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if options.is_empty() {
        if let Some(overrides) = next["trainerScrambleOverrides"][selection_key].as_object_mut() {
            overrides.remove(option);
        }
    } else {
        next["trainerScrambleOverrides"][selection_key][option] =
            serde_json::to_value(options).unwrap();
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recoloring_does_not_replace_new_colors_twice() {
        let colors = BTreeMap::from([
            ("red".to_owned(), "#4E79A7".to_owned()),
            ("blue".to_owned(), "#123456".to_owned()),
        ]);
        let image = case_image_data("<svg fill='#E15759' stroke='#4E79A7'/>", &colors);
        let encoded = image.strip_prefix("data:image/svg+xml;base64,").unwrap();
        let decoded = String::from_utf8(STANDARD.decode(encoded).unwrap()).unwrap();
        assert_eq!(decoded, "<svg fill='#4E79A7' stroke='#123456'/>");
    }

    #[test]
    fn adjust_size_clamps_to_minimum() {
        let settings = serde_json::json!({"baseSize": 0.55});
        let updated = adjust_size_setting(&settings, "baseSize", -0.2);
        assert_eq!(updated["baseSize"], serde_json::json!(0.5));
    }

    #[test]
    fn scramble_override_trims_and_clears_when_empty() {
        let settings = serde_json::json!({});
        let updated = update_scramble_override(&settings, "pllSel", "preMoves", " U \n\n U2 ");
        assert_eq!(
            updated["trainerScrambleOverrides"]["pllSel"]["preMoves"],
            serde_json::json!(["U", "U2"])
        );
        let cleared = update_scramble_override(&updated, "pllSel", "preMoves", "   \n ");
        assert!(cleared["trainerScrambleOverrides"]["pllSel"]
            .get("preMoves")
            .is_none());
    }
}
