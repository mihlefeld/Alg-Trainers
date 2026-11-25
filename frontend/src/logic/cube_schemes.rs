use crate::logic::theme::default_cube_colors;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const PRESETS: [(&str, &str); 3] = [
    ("default", "Default"),
    ("redgreen", "Red-Green color blind"),
    ("blueyellow", "Blue-Yellow color blind"),
];

pub fn preset_colors(id: &str) -> BTreeMap<&'static str, &'static str> {
    let mut colors = default_cube_colors();
    match id {
        "redgreen" => {
            colors.insert("red", "#E157BD");
            colors.insert("pink", "#FF9AD5");
            colors.insert("purple", "#900067");
            colors.insert("orange", "#9E6405");
        }
        "blueyellow" => {
            colors.insert("lightblue", "#A0E8E5");
            colors.insert("blue", "#4E75A7");
        }
        _ => {}
    }
    colors
}

fn normalized_hex(value: &str) -> String {
    let value = value.to_ascii_lowercase();
    if value.len() == 4
        && value.starts_with('#')
        && value[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        let mut expanded = String::from("#");
        for c in value[1..].chars() {
            expanded.push(c);
            expanded.push(c);
        }
        expanded
    } else {
        value
    }
}

pub fn migrate(settings: &mut Value) -> bool {
    if !settings.is_object() {
        *settings = json!({});
    }
    let mut changed = false;
    if settings.get("cubeColorSchemes").is_none() {
        let mut colors = preset_colors("default")
            .into_iter()
            .map(|(key, value)| (key.to_owned(), Value::String(value.to_owned())))
            .collect::<serde_json::Map<_, _>>();
        if let Some(old) = settings.get("cubecolors").and_then(Value::as_object) {
            for (key, value) in old {
                colors.insert(key.clone(), value.clone());
            }
        }
        let matching = PRESETS.iter().find(|(id, _)| {
            preset_colors(id).iter().all(|(key, value)| {
                colors
                    .get(*key)
                    .and_then(Value::as_str)
                    .is_some_and(|stored| normalized_hex(stored) == normalized_hex(value))
            })
        });
        if let Some((id, _)) = matching {
            settings["cubeColorSchemes"] = json!([]);
            settings["defaultCubeScheme"] = json!(id);
        } else {
            settings["cubeColorSchemes"] = json!([{
                "id": "legacy", "name": "Previous custom colors", "colors": colors
            }]);
            settings["defaultCubeScheme"] = json!("legacy");
        }
        changed = true;
    }
    for (key, default) in [
        ("defaultCubeScheme", json!("default")),
        ("trainerCubeSchemes", json!({})),
        ("puzzleCubeSchemes", json!({})),
    ] {
        if settings.get(key).is_none() {
            settings[key] = default;
            changed = true;
        }
    }
    changed
}

pub fn scheme_colors(settings: &Value, id: &str) -> BTreeMap<String, String> {
    let mut colors = preset_colors(id)
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect::<BTreeMap<_, _>>();
    if let Some(custom) = settings
        .get("cubeColorSchemes")
        .and_then(Value::as_array)
        .and_then(|schemes| {
            schemes
                .iter()
                .find(|scheme| scheme.get("id").and_then(Value::as_str) == Some(id))
        })
        .and_then(|scheme| scheme.get("colors"))
        .and_then(Value::as_object)
    {
        for (key, value) in custom {
            if let Some(value) = value.as_str() {
                colors.insert(key.clone(), value.to_owned());
            }
        }
    }
    colors
}

pub fn active_colors(settings: &Value, trainer: &str, puzzle: &str) -> BTreeMap<String, String> {
    if settings.get("cubeColorSchemes").is_none() {
        let mut migrated = settings.clone();
        migrate(&mut migrated);
        return active_colors(&migrated, trainer, puzzle);
    }
    let exists = |id: &&str| {
        PRESETS.iter().any(|(preset, _)| *preset == *id)
            || settings
                .get("cubeColorSchemes")
                .and_then(Value::as_array)
                .is_some_and(|schemes| {
                    schemes
                        .iter()
                        .any(|scheme| scheme.get("id").and_then(Value::as_str) == Some(*id))
                })
    };
    let id = settings
        .get("trainerCubeSchemes")
        .and_then(|v| v.get(trainer))
        .and_then(Value::as_str)
        .filter(exists)
        .or_else(|| {
            settings
                .get("puzzleCubeSchemes")
                .and_then(|v| v.get(puzzle))
                .and_then(|v| v.get("scheme"))
                .and_then(Value::as_str)
                .filter(exists)
        })
        .or_else(|| {
            settings
                .get("defaultCubeScheme")
                .and_then(Value::as_str)
                .filter(exists)
        })
        .unwrap_or("default");
    scheme_colors(settings, id)
}

pub fn delete_scheme(settings: &mut Value, id: &str) {
    if let Some(schemes) = settings
        .get_mut("cubeColorSchemes")
        .and_then(Value::as_array_mut)
    {
        schemes.retain(|scheme| scheme.get("id").and_then(Value::as_str) != Some(id));
    }
    if settings.get("defaultCubeScheme").and_then(Value::as_str) == Some(id) {
        settings["defaultCubeScheme"] = json!("default");
    }
    if let Some(overrides) = settings
        .get_mut("trainerCubeSchemes")
        .and_then(Value::as_object_mut)
    {
        overrides.retain(|_, value| value.as_str() != Some(id));
    }
    if let Some(overrides) = settings
        .get_mut("puzzleCubeSchemes")
        .and_then(Value::as_object_mut)
    {
        overrides.retain(|_, value| value.get("scheme").and_then(Value::as_str) != Some(id));
    }
}

pub fn scheme_label(settings: &Value, id: &str) -> String {
    if id.is_empty() {
        return "Use global scheme".to_owned();
    }
    PRESETS
        .iter()
        .find(|(preset_id, _)| *preset_id == id)
        .map(|(_, name)| (*name).to_owned())
        .or_else(|| {
            settings
                .get("cubeColorSchemes")
                .and_then(Value::as_array)
                .and_then(|schemes| {
                    schemes
                        .iter()
                        .find(|scheme| scheme.get("id").and_then(Value::as_str) == Some(id))
                })
                .and_then(|scheme| scheme.get("name"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "Unknown scheme".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_preserves_custom_colors_and_is_idempotent() {
        let mut settings = json!({"cubecolors": {"red": "#123456"}, "baseSize": 1.5});
        assert!(migrate(&mut settings));
        assert_eq!(settings["defaultCubeScheme"], "legacy");
        assert_eq!(active_colors(&settings, "t", "p")["red"], "#123456");
        assert_eq!(active_colors(&settings, "t", "p")["white"], "#FAFAFA");
        assert_eq!(settings["baseSize"], 1.5);
        assert!(!migrate(&mut settings));
    }

    #[test]
    fn migration_recognizes_short_hex_preset() {
        let mut settings = json!({"cubecolors": preset_colors("redgreen")});
        settings["cubecolors"]["ignore"] = json!("#777");
        assert!(migrate(&mut settings));
        assert_eq!(settings["defaultCubeScheme"], "redgreen");
        assert_eq!(settings["cubeColorSchemes"], json!([]));
    }

    #[test]
    fn overrides_and_deletion_follow_inheritance() {
        let mut settings = json!({
            "cubeColorSchemes": [{"id":"custom", "name":"Test", "colors":{"red":"#123456"}}],
            "defaultCubeScheme":"redgreen",
            "puzzleCubeSchemes":{"3×3":{"scheme":"blueyellow","trainers":["pll"]}},
            "trainerCubeSchemes":{"pll":"custom"}
        });
        assert_eq!(active_colors(&settings, "pll", "3×3")["red"], "#123456");
        assert_eq!(active_colors(&settings, "oll", "3×3")["blue"], "#4E75A7");
        assert_eq!(active_colors(&settings, "eg", "2×2")["red"], "#E157BD");
        delete_scheme(&mut settings, "custom");
        assert_eq!(active_colors(&settings, "pll", "3×3")["blue"], "#4E75A7");
        assert_eq!(settings["trainerCubeSchemes"], json!({}));
        settings["trainerCubeSchemes"]["pll"] = json!("missing");
        assert_eq!(active_colors(&settings, "pll", "3×3")["blue"], "#4E75A7");
        settings["defaultCubeScheme"] = json!("custom");
        settings["puzzleCubeSchemes"]["3×3"]["scheme"] = json!("custom");
        delete_scheme(&mut settings, "custom");
        assert_eq!(settings["defaultCubeScheme"], "default");
        assert_eq!(settings["puzzleCubeSchemes"], json!({}));
    }
}
