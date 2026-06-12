use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_LANGUAGE: &str = "en-en";
const LANGUAGE_DIR_ENV: &str = "NETSTITCH__LANGUAGE_DIR";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageOption {
    pub code: String,
    pub label: String,
}

#[derive(Clone, Debug)]
struct LanguageBundle {
    code: String,
    native_name: String,
    display_name: Option<String>,
    order: i32,
    strings: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct I18nCatalog {
    fallback_code: String,
    languages: Vec<LanguageBundle>,
}

impl I18nCatalog {
    pub fn load() -> Self {
        let dirs = candidate_language_dirs();
        Self::load_from_dirs(&dirs)
    }

    pub fn load_from_dirs(dirs: &[PathBuf]) -> Self {
        let mut languages = Vec::new();
        for dir in dirs {
            if !dir.is_dir() {
                continue;
            }

            let Ok(entries) = fs::read_dir(dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) != Some("ini") {
                    continue;
                }
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Some(language) = parse_language_ini(&path, &content) {
                        upsert_language(&mut languages, language);
                    }
                }
            }
        }

        if languages.is_empty() {
            languages.push(fallback_language());
        }

        languages.sort_by(|left, right| {
            left.order
                .cmp(&right.order)
                .then_with(|| left.label().cmp(&right.label()))
        });

        Self {
            fallback_code: DEFAULT_LANGUAGE.to_string(),
            languages,
        }
    }

    pub fn default_language_code(&self) -> String {
        if self.has_language(DEFAULT_LANGUAGE) {
            return DEFAULT_LANGUAGE.to_string();
        }
        self.languages
            .first()
            .map(|language| language.code.clone())
            .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string())
    }

    pub fn preferred_language_code(&self, candidate: Option<&str>) -> String {
        if let Some(candidate) = candidate {
            if let Some(language) = self.find_language(candidate) {
                return language.code.clone();
            }
        }
        self.default_language_code()
    }

    pub fn language_options(&self) -> Vec<LanguageOption> {
        self.languages
            .iter()
            .map(|language| LanguageOption {
                code: language.code.clone(),
                label: language.label(),
            })
            .collect()
    }

    pub fn text(&self, language_code: &str, key: &str) -> String {
        self.find_language(language_code)
            .and_then(|language| language.strings.get(key))
            .or_else(|| {
                self.find_language(&self.fallback_code)
                    .and_then(|language| language.strings.get(key))
            })
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    fn has_language(&self, code: &str) -> bool {
        self.languages
            .iter()
            .any(|language| language.code.eq_ignore_ascii_case(code))
    }

    fn find_language(&self, code: &str) -> Option<&LanguageBundle> {
        self.languages
            .iter()
            .find(|language| language.code.eq_ignore_ascii_case(code))
    }
}

impl LanguageBundle {
    fn label(&self) -> String {
        if let Some(display_name) = self.display_name.as_ref() {
            if !display_name.trim().is_empty() {
                return display_name.clone();
            }
        }

        format!("{} ({})", self.native_name, display_code(&self.code))
    }
}

fn display_code(code: &str) -> String {
    let normalized = code.trim().to_ascii_uppercase();
    let mut parts = normalized.split('-');
    let Some(language) = parts.next() else {
        return normalized;
    };
    let Some(region) = parts.next() else {
        return normalized;
    };
    if language == region && parts.next().is_none() {
        return language.to_string();
    }
    normalized
}

fn candidate_language_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(explicit) = env::var(LANGUAGE_DIR_ENV) {
        let explicit = PathBuf::from(explicit);
        if !explicit.as_os_str().is_empty() {
            dirs.push(explicit);
        }
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            dirs.push(parent.join("language"));
        }
    }

    if let Ok(current_dir) = env::current_dir() {
        dirs.push(current_dir.join("language"));
        dirs.push(current_dir.join("resources").join("language"));
    }

    dirs
}

fn parse_language_ini(path: &Path, content: &str) -> Option<LanguageBundle> {
    let mut section = String::new();
    let mut code = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(DEFAULT_LANGUAGE)
        .trim()
        .to_ascii_lowercase();
    let mut native_name = String::new();
    let mut display_name = None::<String>;
    let mut order = 100;
    let mut strings = HashMap::new();

    for raw_line in content.lines() {
        let line = raw_line.trim().trim_start_matches('\u{feff}').trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_ascii_lowercase();
            continue;
        }

        let Some((raw_key, raw_value)) = line.split_once('=') else {
            continue;
        };
        let key = raw_key.trim();
        let value = strip_quotes(raw_value.trim()).replace("\\n", "\n");

        match section.as_str() {
            "language" => match key {
                "code" => code = value.to_ascii_lowercase(),
                "name" | "native_name" => native_name = value,
                "display_name" | "label" => display_name = Some(value),
                "order" => {
                    if let Ok(parsed) = value.parse::<i32>() {
                        order = parsed;
                    }
                }
                _ => {}
            },
            "strings" => {
                strings.insert(key.to_string(), value);
            }
            _ => {}
        }
    }

    if code.is_empty() {
        return None;
    }
    if native_name.is_empty() {
        native_name = code.to_ascii_uppercase();
    }

    Some(LanguageBundle {
        code,
        native_name,
        display_name,
        order,
        strings,
    })
}

fn strip_quotes(value: &str) -> String {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

fn upsert_language(languages: &mut Vec<LanguageBundle>, language: LanguageBundle) {
    if let Some(existing) = languages
        .iter_mut()
        .find(|existing| existing.code.eq_ignore_ascii_case(&language.code))
    {
        *existing = language;
    } else {
        languages.push(language);
    }
}

fn fallback_language() -> LanguageBundle {
    let mut strings = HashMap::new();
    strings.insert("app.title".to_string(), "NetStitch MVP".to_string());
    strings.insert(
        "integration.loaded".to_string(),
        "Loaded: {count}".to_string(),
    );
    strings.insert("footer.language.label".to_string(), "Language".to_string());
    LanguageBundle {
        code: DEFAULT_LANGUAGE.to_string(),
        native_name: "English".to_string(),
        display_name: None,
        order: 10,
        strings,
    }
}

#[cfg(test)]
mod tests {
    use super::{I18nCatalog, parse_language_ini};
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn language_ini_label_uses_native_name_and_locale_code() {
        let language = parse_language_ini(
            "ru-ru.ini".as_ref(),
            r#"
            [language]
            code = ru-ru
            name = Русский
            order = 20

            [strings]
            app.title = NetStitch MVP
            "#,
        )
        .expect("language should parse");

        assert_eq!(language.label(), "Русский (RU)");
        assert_eq!(language.strings["app.title"], "NetStitch MVP");
    }

    #[test]
    fn external_language_files_can_add_user_languages() {
        let root =
            std::env::temp_dir().join(format!("NetStitch-language-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temp language dir should be created");
        fs::write(
            root.join("pirate-pirate.ini"),
            r#"
            [language]
            code = pirate-pirate
            name = Pirate
            order = 5

            [strings]
            app.title = Ahoy NetStitch
            "#,
        )
        .expect("temp language should be written");

        let catalog = I18nCatalog::load_from_dirs(std::slice::from_ref(&root));
        let options = catalog.language_options();

        assert_eq!(options[0].label, "Pirate (PIRATE)");
        assert_eq!(catalog.text("pirate-pirate", "app.title"), "Ahoy NetStitch");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_language_keys_fall_back_to_english() {
        let root = std::env::temp_dir().join(format!(
            "NetStitch-language-fallback-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temp language dir should be created");
        fs::write(
            root.join("en-en.ini"),
            r#"
            [language]
            code = en-en
            name = English
            order = 10

            [strings]
            app.title = English title
            "#,
        )
        .expect("english fallback should be written");
        fs::write(
            root.join("ru-ru.ini"),
            r#"
            [language]
            code = ru-ru
            name = Русский
            order = 20

            [strings]
            footer.language.label = Язык
            "#,
        )
        .expect("partial language should be written");

        let catalog = I18nCatalog::load_from_dirs(std::slice::from_ref(&root));

        assert_eq!(catalog.text("ru-ru", "app.title"), "English title");
        assert_eq!(catalog.text("ru-ru", "footer.language.label"), "Язык");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn preferred_language_uses_saved_code_when_available() {
        let root = std::env::temp_dir().join(format!(
            "NetStitch-language-preferred-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temp language dir should be created");
        fs::write(
            root.join("en-en.ini"),
            r#"
            [language]
            code = en-en
            name = English
            order = 10
            "#,
        )
        .expect("english language should be written");
        fs::write(
            root.join("ru-ru.ini"),
            r#"
            [language]
            code = ru-ru
            name = Русский
            order = 20
            "#,
        )
        .expect("russian language should be written");

        let catalog = I18nCatalog::load_from_dirs(std::slice::from_ref(&root));

        assert_eq!(catalog.preferred_language_code(Some("RU-RU")), "ru-ru");
        assert_eq!(catalog.preferred_language_code(Some("missing")), "en-en");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn shipped_language_files_have_matching_non_empty_values() {
        let language_dir = shipped_language_dir();
        let english = load_test_language(&language_dir, "en-en.ini");
        let russian = load_test_language(&language_dir, "ru-ru.ini");

        let english_keys = english.strings.keys().cloned().collect::<BTreeSet<_>>();
        let russian_keys = russian.strings.keys().cloned().collect::<BTreeSet<_>>();

        assert_eq!(english.code, "en-en");
        assert_eq!(russian.code, "ru-ru");
        assert_eq!(english.label(), "English (EN)");
        assert_eq!(russian.label(), "Русский (RU)");
        assert_eq!(
            english_keys, russian_keys,
            "shipped language files must expose the same string keys"
        );

        for language in [&english, &russian] {
            for (key, value) in &language.strings {
                assert!(
                    !value.trim().is_empty(),
                    "{} has an empty value for {}",
                    language.code,
                    key
                );
            }
        }

        let catalog = I18nCatalog::load_from_dirs(&[language_dir]);
        for key in english_keys {
            assert_ne!(
                catalog.text("en-en", &key),
                key,
                "English fallback must resolve {key}"
            );
            assert_ne!(
                catalog.text("ru-ru", &key),
                key,
                "Russian locale must resolve or fall back for {key}"
            );
        }
    }

    #[test]
    fn app_translation_calls_are_covered_by_shipped_languages() {
        let language_dir = shipped_language_dir();
        let english = load_test_language(&language_dir, "en-en.ini");
        let russian = load_test_language(&language_dir, "ru-ru.ini");
        let app_source = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("app.rs"),
        )
        .expect("app.rs should be readable");

        let used_keys = extract_translation_keys(&app_source);

        assert!(
            !used_keys.is_empty(),
            "app.rs should contain translation lookups"
        );
        for key in used_keys {
            assert!(
                english.strings.contains_key(&key),
                "en-en.ini is missing UI key {key}"
            );
            assert!(
                russian.strings.contains_key(&key),
                "ru-ru.ini is missing UI key {key}"
            );
        }
    }

    fn shipped_language_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("resources")
            .join("language")
    }

    fn load_test_language(dir: &std::path::Path, file_name: &str) -> super::LanguageBundle {
        let path = dir.join(file_name);
        let content = fs::read_to_string(&path).expect("shipped language file should be readable");
        parse_language_ini(&path, &content).expect("shipped language file should parse")
    }

    fn extract_translation_keys(source: &str) -> BTreeSet<String> {
        let mut keys = BTreeSet::new();
        let bytes = source.as_bytes();
        let mut offset = 0;
        while let Some(relative) = source[offset..].find("t(\"") {
            let index = offset + relative;
            let previous = index
                .checked_sub(1)
                .and_then(|position| bytes.get(position).copied());
            if previous.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
                offset = index + 3;
                continue;
            }

            let key_start = index + 3;
            let Some(key_end_relative) = source[key_start..].find('"') else {
                break;
            };
            keys.insert(source[key_start..key_start + key_end_relative].to_string());
            offset = key_start + key_end_relative + 1;
        }
        keys
    }
}
