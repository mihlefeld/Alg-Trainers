use std::{env, fs, path::Path};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let frontend_manifest = Path::new(&manifest_dir).join("../frontend/Cargo.toml");
    println!("cargo:rerun-if-changed={}", frontend_manifest.display());

    let content = fs::read_to_string(&frontend_manifest).unwrap_or_else(|error| {
        panic!(
            "Failed to read frontend Cargo.toml at {}: {error}",
            frontend_manifest.display()
        )
    });

    let version = parse_package_version(&content).unwrap_or_else(|| {
        panic!(
            "Failed to find [package] version in {}",
            frontend_manifest.display()
        )
    });

    println!("cargo:rustc-env=FRONTEND_CARGO_VERSION={version}");
}

fn parse_package_version(toml: &str) -> Option<String> {
    let mut in_package = false;
    for raw_line in toml.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line == "[package]";
            continue;
        }
        if in_package {
            if let Some(rest) = line.strip_prefix("version") {
                let rest = rest.trim_start();
                if let Some(rest) = rest.strip_prefix('=') {
                    let value = rest.trim();
                    if let Some(unquoted) = value
                        .strip_prefix('"')
                        .and_then(|s| s.strip_suffix('"'))
                        .or_else(|| value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
                    {
                        if !unquoted.is_empty() {
                            return Some(unquoted.to_owned());
                        }
                    }
                }
            }
        }
    }
    None
}
