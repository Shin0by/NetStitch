use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn shipped_language_files_match_english_keys() {
    let language_dir = repo_root().join("resources").join("language");
    let english = parse_language_strings(&language_dir.join("en-en.ini"));
    let english_keys = english.keys().cloned().collect::<BTreeSet<_>>();

    for entry in fs::read_dir(&language_dir).expect("language dir should be readable") {
        let path = entry.expect("language dir entry").path();
        if path.extension().and_then(|value| value.to_str()) != Some("ini") {
            continue;
        }

        let strings = parse_language_strings(&path);
        let keys = strings.keys().cloned().collect::<BTreeSet<_>>();
        assert_eq!(
            keys,
            english_keys,
            "{} must expose the same [strings] keys as en-en.ini",
            path.display()
        );

        for (key, value) in strings {
            assert!(
                !value.trim().is_empty(),
                "{} has an empty value for {key}",
                path.display()
            );
        }
    }
}

#[test]
fn app_language_files_do_not_own_module_example_strings() {
    let language_dir = repo_root().join("resources").join("language");
    for entry in fs::read_dir(&language_dir).expect("language dir should be readable") {
        let path = entry.expect("language dir entry").path();
        if path.extension().and_then(|value| value.to_str()) != Some("ini") {
            continue;
        }

        let strings = parse_language_strings(&path);
        for (key, value) in strings {
            assert!(
                !key.starts_with("ui_entity_showcase_")
                    && !key.starts_with("sample_module.")
                    && !key.starts_with("localized_module."),
                "{} must not contain module-owned locale key {key}",
                path.display()
            );
            assert!(
                !value.contains("UI Entity Showcase")
                    && !value.contains("Витрина UI")
                    && !value.contains("Localized sample")
                    && !value.contains("Локализованный пример"),
                "{} must not contain module example locale text in key {key}",
                path.display()
            );
        }
    }
}

#[test]
fn missing_localized_value_falls_back_to_english_contract() {
    let language_dir = repo_root().join("resources").join("language");
    let english = parse_language_strings(&language_dir.join("en-en.ini"));
    let partial = BTreeMap::from([("app.title".to_string(), "Локальный заголовок".to_string())]);

    assert_eq!(
        localized_text(&partial, &english, "app.title"),
        "Локальный заголовок"
    );
    assert_eq!(
        localized_text(&partial, &english, "tracked_apps.title"),
        english["tracked_apps.title"]
    );
    assert_eq!(
        localized_text(&partial, &english, "missing.key"),
        "missing.key"
    );
}

fn parse_language_strings(path: &Path) -> BTreeMap<String, String> {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let mut section = String::new();
    let mut strings = BTreeMap::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_ascii_lowercase();
            continue;
        }
        if section != "strings" {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        strings.insert(key.trim().to_string(), value.trim().to_string());
    }

    assert!(
        !strings.is_empty(),
        "{} must contain a non-empty [strings] section",
        path.display()
    );
    strings
}

fn localized_text(
    selected: &BTreeMap<String, String>,
    english: &BTreeMap<String, String>,
    key: &str,
) -> String {
    selected
        .get(key)
        .or_else(|| english.get(key))
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("regression crate should live under tests/netstitch-regression")
        .to_path_buf()
}
