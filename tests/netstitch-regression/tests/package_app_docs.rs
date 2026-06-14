use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

#[test]
fn portable_apps_ship_bilingual_txt_readme_variants() {
    let apps_dir = repo_root()
        .join("resources")
        .join("connectors")
        .join("apps");

    for doc_name in ["README_RU.txt", "README_EN.txt"] {
        let path = apps_dir.join(doc_name);
        assert!(path.is_file(), "{} must exist", path.display());

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        assert!(
            !content.trim().is_empty(),
            "{} must not be empty",
            path.display()
        );
    }
}

#[test]
fn portable_apps_ship_app_manifests_without_toml_connector_presets() {
    let apps_dir = repo_root()
        .join("resources")
        .join("connectors")
        .join("apps");
    let entries = fs::read_dir(&apps_dir)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", apps_dir.display()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| {
            panic!("{} entries should be readable: {error}", apps_dir.display())
        });

    let app_count = entries
        .iter()
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "app")
        })
        .count();
    assert!(
        app_count > 0,
        "{} must ship *.app manifests",
        apps_dir.display()
    );

    let toml_files = entries
        .iter()
        .filter_map(|entry| {
            let path = entry.path();
            (path
                .extension()
                .is_some_and(|extension| extension == "toml"))
            .then(|| path.display().to_string())
        })
        .collect::<Vec<_>>();
    assert!(
        toml_files.is_empty(),
        "portable connector presets must use *.app, not *.toml:\n{}",
        toml_files.join("\n")
    );
}

#[test]
fn chrome_linux_connector_matches_real_chrome_process_name() {
    let root = repo_root();
    let chrome_app_path = root
        .join("resources")
        .join("connectors")
        .join("apps")
        .join("chrome.app");
    let chrome_app = fs::read_to_string(&chrome_app_path).unwrap_or_else(|error| {
        panic!("{} should be readable: {error}", chrome_app_path.display())
    });
    assert!(
        chrome_app.contains("[[process_aliases]]\nos = \"linux\"\nname = \"chrome\""),
        "runtime Chrome connector must match Linux Chrome processes named 'chrome'"
    );

    let connector_source_path = root
        .join("src")
        .join("netstitch-connectors")
        .join("src")
        .join("lib.rs");
    let connector_source = fs::read_to_string(&connector_source_path).unwrap_or_else(|error| {
        panic!(
            "{} should be readable: {error}",
            connector_source_path.display()
        )
    });
    for required in [
        "filter_map(canonical_existing_file)",
        "fn canonical_existing_file(path: PathBuf) -> Option<PathBuf>",
        "path.canonicalize().unwrap_or(path)",
    ] {
        assert!(
            connector_source.contains(required),
            "Linux connector discovery must keep canonical executable token {required}"
        );
    }
}

#[test]
fn source_tree_does_not_ship_user_specific_runtime_connector_examples() {
    let root = repo_root();
    let scan_roots = [
        root.join("README.md"),
        root.join("docs"),
        root.join("integrations").join("README.md"),
        root.join("src"),
        root.join("resources"),
        root.join("scripts"),
        root.join("tests").join("README.md"),
        root.join("tools"),
    ];
    let forbidden = ["user-specific connector example", "manual runtime preset"];
    let mut matches = Vec::new();
    for scan_root in scan_roots {
        collect_forbidden_matches(&scan_root, &forbidden, &mut matches);
    }
    assert!(
        matches.is_empty(),
        "tracked source/resources/tooling must not ship user-specific runtime connector examples:\n{}",
        matches.join("\n")
    );
}

#[test]
fn core_tree_keeps_external_modules_out_of_main_package_by_default() {
    let root = repo_root();
    let tracked_integration_files = tracked_paths_under(&root.join("integrations"));
    let unexpected = tracked_integration_files
        .iter()
        .filter(|path| {
            !path.ends_with("integrations/README.md")
                && !path.ends_with("integrations/host/Cargo.toml")
                && !path.ends_with("integrations/host/src/lib.rs")
        })
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        unexpected.is_empty(),
        "core NetStitch tree must not track bundled external module source/runtime files by default:\n{}",
        unexpected.join("\n")
    );
}

#[test]
fn module_sdk_docs_and_examples_are_tracked() {
    let root = repo_root();
    for relative in [
        "docs/module-sdk/README_RU.md",
        "docs/module-sdk/README_EN.md",
        "docs/module-sdk/CREATING_MODULES_RU.md",
        "docs/module-sdk/CREATING_MODULES_EN.md",
        "docs/module-sdk/EXAMPLES_RU.md",
        "docs/module-sdk/EXAMPLES_EN.md",
        "docs/module-sdk/REFERENCE_RU.md",
        "docs/module-sdk/REFERENCE_EN.md",
        "docs/module-sdk/UI_ENTITIES_RU.md",
        "docs/module-sdk/UI_ENTITIES_EN.md",
        "docs/module-sdk/HOST_CONTEXT_RU.md",
        "docs/module-sdk/HOST_CONTEXT_EN.md",
        "docs/module-sdk/COMMANDS_RU.md",
        "docs/module-sdk/COMMANDS_EN.md",
        "docs/module-sdk/assets/ui-entities.svg",
        "docs/module-sdk/examples/ui-entity-showcase-rust/README_RU.md",
        "docs/module-sdk/examples/ui-entity-showcase-rust/README_EN.md",
        "docs/module-sdk/examples/ui-entity-showcase-rust/Cargo.toml",
        "docs/module-sdk/examples/ui-entity-showcase-rust/Cargo.lock",
        "docs/module-sdk/examples/ui-entity-showcase-rust/src/lib.rs",
        "docs/module-sdk/examples/ui-entity-showcase-rust/module.json",
        "docs/module-sdk/examples/ui-entity-showcase-rust/locales/en-en.ini",
        "docs/module-sdk/examples/ui-entity-showcase-rust/locales/ru-ru.ini",
        "docs/module-sdk/examples/ui-entity-showcase-cpp/README_RU.md",
        "docs/module-sdk/examples/ui-entity-showcase-cpp/README_EN.md",
        "docs/module-sdk/examples/ui-entity-showcase-cpp/CMakeLists.txt",
        "docs/module-sdk/examples/ui-entity-showcase-cpp/ui_entity_showcase_module.cpp",
        "docs/module-sdk/examples/ui-entity-showcase-cpp/module.json",
        "docs/module-sdk/examples/ui-entity-showcase-cpp/locales/en-en.ini",
        "docs/module-sdk/examples/ui-entity-showcase-cpp/locales/ru-ru.ini",
        "scripts/package_module_sdk_examples.ps1",
    ] {
        let path = root.join(relative);
        assert!(path.is_file(), "{} must exist", path.display());
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        assert!(
            !content.trim().is_empty(),
            "{} must not be empty",
            path.display()
        );
    }

    for removed in [
        "docs/module-sdk/examples/hello-world-rust",
        "docs/module-sdk/examples/hello-world-cpp",
    ] {
        assert!(
            !root.join(removed).exists(),
            "{removed} should not remain after replacing SDK examples with UI Entity Showcase"
        );
    }
}

#[test]
fn module_sdk_markdown_docs_are_bilingual() {
    let sdk_root = repo_root().join("docs").join("module-sdk");
    let mut missing = Vec::new();
    collect_missing_module_sdk_english_docs(&sdk_root, &mut missing);
    assert!(
        missing.is_empty(),
        "every Module SDK *_RU.md document must have a matching *_EN.md document:\n{}",
        missing.join("\n")
    );
}

#[test]
fn module_sdk_example_archives_are_ready_to_install() {
    let root = repo_root();
    let packages_dir = root.join("docs").join("module-sdk").join("packages");
    let archives = [
        (
            "ui-entity-showcase-rust-windows-x86_64.zip",
            "ui-entity-showcase-rust/",
            "ui-entity-showcase-rust/bin/ui_entity_showcase_rust.dll",
            "ui-entity-showcase-rust/src/lib.rs",
        ),
        (
            "ui-entity-showcase-rust-linux-x86_64.zip",
            "ui-entity-showcase-rust/",
            "ui-entity-showcase-rust/bin/libui_entity_showcase_rust.so",
            "ui-entity-showcase-rust/src/lib.rs",
        ),
        (
            "ui-entity-showcase-cpp-windows-x86_64.zip",
            "ui-entity-showcase-cpp/",
            "ui-entity-showcase-cpp/bin/ui_entity_showcase_cpp.dll",
            "ui-entity-showcase-cpp/ui_entity_showcase_module.cpp",
        ),
        (
            "ui-entity-showcase-cpp-linux-x86_64.zip",
            "ui-entity-showcase-cpp/",
            "ui-entity-showcase-cpp/bin/libui_entity_showcase_cpp.so",
            "ui-entity-showcase-cpp/ui_entity_showcase_module.cpp",
        ),
    ];

    for (archive_name, module_root, binary_entry, source_entry) in archives {
        let archive_path = packages_dir.join(archive_name);
        assert!(
            archive_path.is_file(),
            "{} must exist",
            archive_path.display()
        );
        let file = File::open(&archive_path).unwrap_or_else(|error| {
            panic!("{} should be readable: {error}", archive_path.display())
        });
        let mut archive = zip::ZipArchive::new(file).unwrap_or_else(|error| {
            panic!("{} should be a valid zip: {error}", archive_path.display())
        });
        let mut names = Vec::new();
        for index in 0..archive.len() {
            let entry = archive.by_index(index).unwrap_or_else(|error| {
                panic!(
                    "{} zip entry {index} should be readable: {error}",
                    archive_path.display()
                )
            });
            names.push(entry.name().replace('\\', "/"));
        }

        assert!(
            names.iter().all(|name| name.starts_with(module_root)),
            "{} must keep {module_root} as the archive root",
            archive_path.display()
        );

        let manifest_entry = format!("{module_root}module.json");
        let readme_en_entry = format!("{module_root}README_EN.md");
        let en_locale_entry = format!("{module_root}locales/en-en.ini");
        let ru_locale_entry = format!("{module_root}locales/ru-ru.ini");
        for required in [
            manifest_entry.as_str(),
            readme_en_entry.as_str(),
            binary_entry,
            source_entry,
            en_locale_entry.as_str(),
            ru_locale_entry.as_str(),
        ] {
            assert!(
                names.iter().any(|name| name == required),
                "{} must contain {required}",
                archive_path.display()
            );
        }

        let forbidden = names
            .iter()
            .filter(|name| {
                name.contains("/target/")
                    || name.contains("/build/")
                    || name.contains("/data/")
                    || name.contains("/.local/")
                    || name.contains("/temp/")
                    || name.ends_with(".pdb")
            })
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            forbidden.is_empty(),
            "{} must not include build/runtime/local artifacts:\n{}",
            archive_path.display(),
            forbidden.join("\n")
        );
    }
}

#[test]
fn module_sdk_contract_docs_use_current_host_commands() {
    let root = repo_root();
    let readme = fs::read_to_string(root.join("docs/module-sdk/README_RU.md"))
        .expect("module SDK readme should be readable");
    let readme_en = fs::read_to_string(root.join("docs/module-sdk/README_EN.md"))
        .expect("module SDK EN readme should be readable");
    let creating = fs::read_to_string(root.join("docs/module-sdk/CREATING_MODULES_RU.md"))
        .expect("module SDK creating doc should be readable");
    let creating_en = fs::read_to_string(root.join("docs/module-sdk/CREATING_MODULES_EN.md"))
        .expect("module SDK EN creating doc should be readable");
    let examples = fs::read_to_string(root.join("docs/module-sdk/EXAMPLES_RU.md"))
        .expect("module SDK examples doc should be readable");
    let commands = fs::read_to_string(root.join("docs/module-sdk/COMMANDS_RU.md"))
        .expect("module SDK commands doc should be readable");
    let reference = fs::read_to_string(root.join("docs/module-sdk/REFERENCE_RU.md"))
        .expect("module SDK reference doc should be readable");
    let host_context = fs::read_to_string(root.join("docs/module-sdk/HOST_CONTEXT_RU.md"))
        .expect("module SDK host context doc should be readable");
    let ui_entities = fs::read_to_string(root.join("docs/module-sdk/UI_ENTITIES_RU.md"))
        .expect("module SDK UI entities doc should be readable");
    let commands_en = fs::read_to_string(root.join("docs/module-sdk/COMMANDS_EN.md"))
        .expect("module SDK EN commands doc should be readable");
    let ui_entities_en = fs::read_to_string(root.join("docs/module-sdk/UI_ENTITIES_EN.md"))
        .expect("module SDK EN UI entities doc should be readable");
    let rust_showcase_manifest = fs::read_to_string(
        root.join("docs/module-sdk/examples/ui-entity-showcase-rust/module.json"),
    )
    .expect("Rust UI entity showcase manifest should be readable");
    let cpp_showcase_manifest = fs::read_to_string(
        root.join("docs/module-sdk/examples/ui-entity-showcase-cpp/module.json"),
    )
    .expect("C++ UI entity showcase manifest should be readable");
    let joined = format!(
        "{readme}\n{readme_en}\n{creating}\n{creating_en}\n{examples}\n{commands}\n{reference}\n{host_context}\n{ui_entities}\n{commands_en}\n{ui_entities_en}\n{rust_showcase_manifest}\n{cpp_showcase_manifest}"
    );

    for required in [
        "\"command_type\": \"start_background\"",
        "\"subscriptions\"",
        "\"command_type\": \"stop_background\"",
        "\"command_type\": \"set_module_page\"",
        "\"command_type\": \"set_ui_values\"",
        "\"command_type\": \"browse_window\"",
        "\"status_target\": \"export_status\"",
        "\"mode\": \"file_save\"",
        "\"default_extension\": \"csv\"",
        "\"selected_status\": \"Save target selected; file was not written\"",
        "\"overwrite_policy\": \"prompt\"",
        "\"can_create_directories\": true",
        "\"extensions\": [\"csv\"]",
        "\"command_type\": \"log_event\"",
        "\"message\"",
        "\"severity\"",
        "\"command_type\": \"show_dialog\"",
        "\"buttons\": \"ok_cancel\"",
        "background_event",
        "ui.dialog_result",
        "core",
        "system",
        "{context.tables.monitoring.selected_total}",
        "{context.monitoring.latest_rows}",
        "module_background_active",
        "payload.ui_values",
        "\"entity_type\": \"text_input\"",
        "\"entity_type\": \"textarea\"",
        "\"clear_button\": true",
        "\"entity_type\": \"select\"",
        "\"entity_type\": \"switch\"",
        "\"entity_type\": \"grid\"",
        "\"entity_type\": \"tabs\"",
        "\"value\": \"browse_windows\"",
        "\"id\": \"browse_folder_window\"",
        "\"id\": \"browse_save_window\"",
        "\"entity_type\": \"separator\"",
        "\"entity_type\": \"progress\"",
        "\"progress_stages\"",
        "\"color\": \"rust\"",
        "\"percent\": 70",
        "\"name\": \"Processing\"",
        "\"compact\": true",
        "\"hide_label\": false",
        "\"hide_host_back_button\": false",
        "hide_host_back_button: true",
        "host-owned `Назад` всегда отображается справа",
        "host-owned `Back` button is shown on the right by default",
        "standard module overlay footer",
        "\"title\": \"Mini progress bar\"",
        "\"icon_path\": \"assets/brand-rust-svgrepo-com.svg\"",
        "\"columns\": \"repeat(2, minmax(0, 1fr))\"",
        "repeat(auto-fit, minmax(180px, 1fr))",
        "\"gap\": \"8px\"",
        "\"grid_column\": \"1 / -1\"",
        "\"table_columns\"",
        "\"text_field\": true",
        "\"scroll\": \"both\"",
        "\"size\": \"stretch\"",
        "\"height\": \"180px\"",
        "\"align\": \"left\"",
        "\"align\": \"center\"",
        "\"align\": \"right\"",
        "\"margin\": \"0\"",
        "\"padding\": \"0\"",
        "\"style\": \"primary\"",
        "\"pulse\": true",
        "\"pulse_when_background_active\": true",
        "Предсказуемые дефолты layout",
        "любая сущность, включая неизвестный будущий тип",
        "opacity = \"100%\"",
        "height = \"auto\"",
        "растёт по высоте только по содержимому",
        "If a row-like entity has no `title`",
        "убирает пустую label-колонку",
        "`separator`, `help_text`, `table`, `progress` и `footer`",
        "Строковые сущности (`row`, `value`, `status`, `path_field`, `input`, `textarea`, `select`, `switch`)",
        "Predictable Layout Defaults",
        "UI entities have centralized defaults",
        "Supported Types",
        "Canonical JSON values and aliases",
        "grid.children",
        "pending `ui_action`",
        "module runner process",
        "in-process DLL",
        "locales/en-en.ini",
        "locales/ru-ru.ini",
        "\"display_name_key\"",
        "\"value_key\"",
        "\"placeholder_key\"",
        "ui_entity_showcase_rust.module.display_name",
        "ui_entity_showcase_cpp.module.display_name",
        "scripts\\package_module_sdk_examples.ps1",
        "ui-entity-showcase-rust-windows-x86_64.zip",
        "ui-entity-showcase-cpp-linux-x86_64.zip",
    ] {
        assert!(
            joined.contains(required),
            "module SDK docs must keep current command token {required}"
        );
    }

    for entity_type in [
        "panel",
        "subpanel",
        "grid",
        "layout_grid",
        "tabs",
        "tab_view",
        "row",
        "value_label",
        "value",
        "status_label",
        "status",
        "path_field",
        "input",
        "text_input",
        "text_field",
        "textarea",
        "text_area",
        "select",
        "dropdown",
        "combo_box",
        "switch",
        "toggle",
        "help_text",
        "separator",
        "button",
        "action_button",
        "progress",
        "table",
        "footer",
    ] {
        let token = format!("\"entity_type\": \"{entity_type}\"");
        assert!(
            joined.contains(&token),
            "module SDK docs must document UI entity type token {token}"
        );
    }

    for forbidden in [
        "variant:",
        "\"variant\"",
        "event_subscriptions",
        "\"action\": \"host_event\"",
        "hello-world",
        "Hello World",
    ] {
        assert!(
            !joined.contains(forbidden),
            "module SDK docs must not reintroduce obsolete/forbidden token {forbidden}"
        );
    }

    assert!(
        joined.contains("cloud upload/download") && joined.contains("CSV import/export"),
        "module SDK docs should explicitly keep cloud and CSV automation manual-only"
    );

    assert!(
        joined.contains("первого уровня") || joined.contains("first-level"),
        "module SDK docs should state that first-level module open is an explicit user action"
    );

    assert!(
        joined.contains("desktop shell")
            && joined.contains("browser shell")
            && joined.contains("ui_schema"),
        "module SDK docs should state that one ui_schema is rendered by both desktop and browser shells"
    );
}

#[test]
fn cloud_worker_observation_download_uses_limit_offset_pagination() {
    let worker_path = repo_root()
        .join("src")
        .join("netstitch-cloud-worker")
        .join("src")
        .join("worker.js");
    let source = fs::read_to_string(&worker_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", worker_path.display()));
    let start = source
        .find("async function getObservations(")
        .expect("cloud Worker getObservations endpoint");
    let end = source[start..]
        .find("async function appendObservations(")
        .map(|offset| start + offset)
        .expect("cloud Worker appendObservations endpoint");
    let get_observations = &source[start..end];

    for token in [
        "const limit = boundedLimit(url.searchParams.get(\"limit\"), 500, 1000);",
        "const offset = boundedOffset(url.searchParams.get(\"offset\"));",
        "SELECT COUNT(*) AS total",
        "LOWER(r.visibility) AS visibility",
        "GROUP BY r.app_id, LOWER(r.visibility), r.ip, r.port, LOWER(r.protocol)",
        "GROUP BY r.app_id,\n                LOWER(r.visibility),",
        "LIMIT ? OFFSET ?",
        ".bind(...params, limit, offset)",
        "total, limit, offset, rows",
    ] {
        assert!(
            get_observations.contains(token),
            "cloud observation download must keep paged query token {token}"
        );
    }
    assert!(
        !get_observations.contains("LIMIT 500"),
        "cloud observation download must not hardcode the first page size in SQL"
    );
}

#[test]
fn cloud_worker_visibility_contract_is_lowercase_and_enforced() {
    let root = repo_root();
    let worker_path = root
        .join("src")
        .join("netstitch-cloud-worker")
        .join("src")
        .join("worker.js");
    let source = fs::read_to_string(&worker_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", worker_path.display()));

    let list_user_apps_start = source
        .find("async function listUserApps")
        .expect("listUserApps function");
    let get_quota_start = source[list_user_apps_start..]
        .find("async function getQuota")
        .map(|offset| list_user_apps_start + offset)
        .expect("getQuota function after listUserApps");
    let list_user_apps = &source[list_user_apps_start..get_quota_start];
    let get_observations_start = source
        .find("async function getObservations")
        .expect("getObservations function");
    let append_observations_start = source[get_observations_start..]
        .find("async function appendObservations")
        .map(|offset| get_observations_start + offset)
        .expect("appendObservations function after getObservations");
    let get_observations = &source[get_observations_start..append_observations_start];
    let api_visibility_source = format!("{list_user_apps}\n{get_observations}");

    for forbidden in ["'Private'", "\"Private\"", "'Public'", "\"Public\""] {
        assert!(
            !api_visibility_source.contains(forbidden),
            "cloud Worker API must not emit mixed-case visibility token {forbidden}"
        );
    }
    for required in [
        "WHEN 'private' THEN 'private'",
        "ELSE 'public'",
        "LOWER(r.visibility) = 'public'",
        "LOWER(r.visibility) = 'private'",
        "LOWER(r.visibility) = ?",
    ] {
        assert!(
            source.contains(required),
            "cloud Worker visibility contract must keep token {required}"
        );
    }

    let migration_path = root
        .join("src")
        .join("netstitch-cloud-worker")
        .join("migrations")
        .join("0014_normalize_observation_visibility.sql");
    let migration = fs::read_to_string(&migration_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", migration_path.display()));
    for required in [
        "UPDATE observations",
        "SET visibility = LOWER(visibility)",
        "UPDATE observation_author_rows",
        "observations_visibility_insert_check",
        "observation_author_rows_visibility_insert_check",
        "NEW.visibility NOT IN ('public', 'private')",
    ] {
        assert!(
            migration.contains(required),
            "visibility migration must keep token {required}"
        );
    }
}

#[test]
fn cloud_worker_google_identity_hashes_require_server_side_pepper() {
    let root = repo_root();
    let worker_path = root
        .join("src")
        .join("netstitch-cloud-worker")
        .join("src")
        .join("worker.js");
    let source = fs::read_to_string(&worker_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", worker_path.display()));

    for required in [
        "async function identityHmacBase64Url(env, value)",
        "requireEnv(env, \"NETSTITCH_IDENTITY_PEPPER\")",
        "crypto.subtle.importKey(\n    \"raw\",",
        "{ name: \"HMAC\", hash: \"SHA-256\" }",
        "const subjectHash = await identityHmacBase64Url(env, `${provider}:${googleInfo.sub}`);",
        "const emailHash = email ? await identityHmacBase64Url(env, `email:${email}`) : null;",
    ] {
        assert!(
            source.contains(required),
            "cloud Worker Google identity hashing must keep token {required}"
        );
    }

    for forbidden in [
        "const subjectHash = await sha256Base64Url(`${provider}:${googleInfo.sub}`);",
        "const emailHash = email ? await sha256Base64Url(`email:${email}`) : null;",
    ] {
        assert!(
            !source.contains(forbidden),
            "cloud Worker must not use unsalted SHA-256 identity hash token {forbidden}"
        );
    }

    let worker_readme = fs::read_to_string(root.join("src/netstitch-cloud-worker/README.md"))
        .expect("cloud Worker README should be readable");
    let env_example =
        fs::read_to_string(root.join(".env.example")).expect(".env.example should be readable");
    for content in [worker_readme, env_example] {
        assert!(
            content.contains("NETSTITCH_IDENTITY_PEPPER"),
            "cloud secret documentation must mention NETSTITCH_IDENTITY_PEPPER"
        );
    }
}

#[test]
fn cloud_worker_observation_response_uses_rust_enum_casing() {
    let worker_path = repo_root()
        .join("src")
        .join("netstitch-cloud-worker")
        .join("src")
        .join("worker.js");
    let source = fs::read_to_string(&worker_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", worker_path.display()));
    let get_observations_start = source
        .find("async function getObservations")
        .expect("getObservations function");
    let append_observations_start = source[get_observations_start..]
        .find("async function appendObservations")
        .map(|offset| get_observations_start + offset)
        .expect("appendObservations function after getObservations");
    let get_observations = &source[get_observations_start..append_observations_start];

    for forbidden in [
        "THEN 'established'",
        "THEN 'failed'",
        "r.connection_state,",
        "r.domain_status,",
        "r.trust_level,",
        "r.source_kind,",
    ] {
        assert!(
            !get_observations.contains(forbidden),
            "cloud observation response must not expose non-Rust enum casing token {forbidden}"
        );
    }
    for required in [
        "THEN 'Established'",
        "THEN 'Failed'",
        "WHEN 'attempting' THEN 'Attempting'",
        "WHEN 'established' THEN 'Established'",
        "WHEN 'closing' THEN 'Closing'",
        "WHEN 'failed' THEN 'Failed'",
        "WHEN 'mismatch' THEN 'Mismatch'",
        "WHEN 'unresolved' THEN 'Unresolved'",
        "WHEN 'invalid' THEN 'Invalid'",
        "WHEN SUM(CASE WHEN LOWER(r.trust_level) = 'communityverified' THEN 1 ELSE 0 END) > 0 THEN 'CommunityVerified'",
        "WHEN SUM(CASE WHEN LOWER(r.trust_level) = 'verifieduploadcandidate' THEN 1 ELSE 0 END) > 0 THEN 'VerifiedUploadCandidate'",
        "WHEN SUM(CASE WHEN LOWER(r.trust_level) = 'cloudimportuntrusted' THEN 1 ELSE 0 END) > 0 THEN 'CloudImportUntrusted'",
        "ELSE 'BlockedOrSuspect'",
        "WHEN SUM(CASE WHEN LOWER(r.source_kind) = 'verifiedupload' THEN 1 ELSE 0 END) > 0 THEN 'VerifiedUpload'",
        "WHEN SUM(CASE WHEN LOWER(r.source_kind) = 'cloudimport' THEN 1 ELSE 0 END) > 0 THEN 'CloudImport'",
        "ELSE 'CloudImportUntrusted'",
        "LOWER(r.protocol) = ?",
    ] {
        assert!(
            get_observations.contains(required),
            "cloud observation response must keep Rust enum casing token {required}"
        );
    }
}

#[test]
fn existing_windows_release_zip_ships_clean_portable_apps_folder() {
    let release_dir = repo_root().join("dist").join("release-assets");
    let Some(zip_path) = latest_release_asset(&release_dir, "NetStitch-win64-portable-", ".zip")
    else {
        return;
    };

    let file = File::open(&zip_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", zip_path.display()));
    let mut archive = zip::ZipArchive::new(file)
        .unwrap_or_else(|error| panic!("{} should be a valid zip: {error}", zip_path.display()));
    let mut names = Vec::new();
    for index in 0..archive.len() {
        let file = archive.by_index(index).unwrap_or_else(|error| {
            panic!(
                "{} zip entry {index} should be readable: {error}",
                zip_path.display()
            )
        });
        names.push(file.name().to_string());
    }

    assert!(
        names
            .iter()
            .all(|name| name.starts_with("NetStitch-win64-portable/")),
        "{} must keep NetStitch-win64-portable/ as the archive root",
        zip_path.display()
    );
    assert!(
        names.iter().any(|name| {
            name.starts_with("NetStitch-win64-portable/apps/") && name.ends_with(".app")
        }),
        "{} must ship portable apps/*.app manifests",
        zip_path.display()
    );

    let forbidden = names
        .iter()
        .filter(|name| {
            (name.starts_with("NetStitch-win64-portable/apps/")
                && (name.ends_with(".toml") || name.contains("/manual_")))
                || (name.starts_with("NetStitch-win64-portable/storage/")
                    && (name.ends_with(".sqlite3") || name.ends_with(".db")))
                || (name.starts_with("NetStitch-win64-portable/integrations/")
                    && name.contains("/data/"))
        })
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        forbidden.is_empty(),
        "{} must not ship legacy presets, user-specific examples, runtime manual icon cache, or runtime databases:\n{}",
        zip_path.display(),
        forbidden.join("\n")
    );
}

#[test]
fn existing_windows_release_zip_ships_network_runtime_files() {
    let release_dir = repo_root().join("dist").join("release-assets");
    let Some(zip_path) = latest_release_asset(&release_dir, "NetStitch-win64-portable-", ".zip")
    else {
        return;
    };

    let file = File::open(&zip_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", zip_path.display()));
    let mut archive = zip::ZipArchive::new(file)
        .unwrap_or_else(|error| panic!("{} should be a valid zip: {error}", zip_path.display()));
    let mut names = Vec::new();
    for index in 0..archive.len() {
        let file = archive.by_index(index).unwrap_or_else(|error| {
            panic!(
                "{} zip entry {index} should be readable: {error}",
                zip_path.display()
            )
        });
        names.push(file.name().to_string());
    }

    for required in [
        "NetStitch-win64-portable/WinDivert.dll",
        "NetStitch-win64-portable/WinDivert64.sys",
        "NetStitch-win64-portable/libs/netstitch-tool/bin/windows-x86_64/netstitch_tool.dll",
        "NetStitch-win64-portable/libs/netstitch-watcher/bin/windows-x86_64/netstitch_watcher.dll",
    ] {
        assert!(
            names.iter().any(|name| name == required),
            "{} must ship required network runtime file {required}",
            zip_path.display()
        );
    }
}

#[test]
fn windows_portable_packaging_preserves_user_app_manifests_and_icons() {
    let script_path = repo_root().join("scripts").join("package_portable.ps1");
    let source = fs::read_to_string(&script_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", script_path.display()));

    assert!(
        source.contains("Where-Object { $_.Extension -eq \".toml\" }"),
        "portable packaging may remove legacy *.toml connector presets"
    );
    assert!(
        !source.contains("Where-Object { $_.Extension -in @(\".app\", \".toml\") }"),
        "portable packaging must not delete user-created *.app connector manifests"
    );
    assert!(
        source.contains("Clear-PortableIconCache -IconDir $IconDir"),
        "portable packaging should sanitize apps/icons without deleting user svg/png icons"
    );
    assert!(
        source.contains("Set-SystemEventCleanupMarker -StorageDir $portableStorageDir")
            && source.contains("clear-system-events-on-next-start"),
        "portable packaging should mark system_events for cleanup without deleting user storage"
    );
    assert!(
        source.contains("resources\") \"runtime\") \"windivert\\windows-x86_64\""),
        "portable packaging should use tracked WinDivert runtime fallback for clean CI builds"
    );
    for token in [
        "function Sync-PortableModuleLocales",
        "Join-Path $SourceModuleDir \"locales\"",
        "Sync-PortableModuleLocales `",
    ] {
        assert!(
            source.contains(token),
            "portable packaging must keep module-owned locales with token {token}"
        );
    }
    for obsolete_module in ["hello-world-rust", "hello-world-cpp", "ui-entity-showcase"] {
        assert!(
            source.contains(obsolete_module),
            "portable packaging should remove obsolete SDK/test integration folder {obsolete_module}"
        );
    }
    assert!(
        !source.contains("Remove-Item -LiteralPath $IconDir -Recurse -Force"),
        "portable packaging must not delete the whole apps/icons folder"
    );
}

#[test]
fn portable_packaging_keeps_external_modules_opt_in() {
    let root = repo_root();
    let windows_script_path = root.join("scripts").join("package_portable.ps1");
    let windows_source = fs::read_to_string(&windows_script_path).unwrap_or_else(|error| {
        panic!(
            "{} should be readable: {error}",
            windows_script_path.display()
        )
    });
    for token in [
        "[string[]]$PackageModules",
        "NETSTITCH__PACKAGE_MODULES",
        "Test-PackageModuleIncluded",
        "return $false",
        "-not (Test-PackageModuleIncluded -ModuleName $moduleName",
        "-IncludedModules $includedPackageModules",
    ] {
        assert!(
            windows_source.contains(token),
            "Windows portable packaging must keep external modules opt-in with token {token}"
        );
    }

    let linux_script_path = root.join("scripts").join("package_portable_linux.sh");
    let linux_source = fs::read_to_string(&linux_script_path).unwrap_or_else(|error| {
        panic!(
            "{} should be readable: {error}",
            linux_script_path.display()
        )
    });
    for token in [
        "package_modules=\"${NETSTITCH__PACKAGE_MODULES:-}\"",
        "--package-modules",
        "module_is_included()",
        "return 1",
        "if ! module_is_included",
        "rm -rf \"$integrations_dir/$module_name\"",
        "rm -rf \"$module_dir/locales\"",
        "cp -a \"$module_root/locales\" \"$module_dir/locales\"",
        "hello-world-rust hello-world-cpp ui-entity-showcase",
        "clear-system-events-on-next-start",
        "clear system_events on next NetStitch startup",
    ] {
        assert!(
            linux_source.contains(token),
            "Linux portable packaging must keep external modules opt-in with token {token}"
        );
    }
}

#[test]
fn release_asset_packaging_sanitizes_staging_copy_not_user_portable() {
    let root = repo_root();
    let windows_script_path = root.join("scripts").join("package_release_assets.ps1");
    let windows_source = fs::read_to_string(&windows_script_path).unwrap_or_else(|error| {
        panic!(
            "{} should be readable: {error}",
            windows_script_path.display()
        )
    });
    for token in [
        "function New-ReleaseStagingPortable",
        "Copy-Item -LiteralPath $PortablePath -Destination $stagingRoot -Recurse -Force",
        "Sync-ReleaseConnectorApps -PortablePath $stagedPortablePath",
        "Remove-Item -LiteralPath $stagedStoragePath -Recurse -Force",
        "New-Item -ItemType Directory -Force -Path (Join-Path $stagedStoragePath \"exports\")",
        "function Remove-ReleaseModuleRuntimeData",
        "Remove-ReleaseModuleRuntimeData -PortablePath $stagedPortablePath",
        "Assert-WindowsReleaseRuntimeFiles -PortablePath $stagedPortablePath",
        "WinDivert.dll",
        "WinDivert64.sys",
        "Compress-Archive -Path $stagedPortablePath",
    ] {
        assert!(
            windows_source.contains(token),
            "Windows release packaging must sanitize a staging copy using token {token}"
        );
    }
    assert!(
        !windows_source.contains("Sync-ReleaseConnectorApps -PortablePath $portablePath"),
        "Windows release packaging must not sanitize the user's portable apps folder directly"
    );

    let linux_script_path = root.join("scripts").join("package_release_assets_linux.sh");
    let linux_source = fs::read_to_string(&linux_script_path).unwrap_or_else(|error| {
        panic!(
            "{} should be readable: {error}",
            linux_script_path.display()
        )
    });
    for token in [
        "staging_root=\"$release_path/.release-staging\"",
        "cp -a \"$portable_path\" \"$staging_root/\"",
        "find \"$staged_portable_path/apps\" -maxdepth 1 \\( -name '*.app' -o -name '*.toml' \\) -type f -delete",
        "rm -rf \"$staged_portable_path/storage\"",
        "mkdir -p \"$staged_portable_path/storage/exports\"",
        "find \"$staged_portable_path/integrations\" -mindepth 2 -maxdepth 2 -type d -name data -exec rm -rf {} +",
        "libs/netstitch-tool/bin/linux-x86_64/libnetstitch_tool.so",
        "libs/netstitch-watcher/bin/linux-x86_64/libnetstitch_watcher.so",
        "tar -C \"$staging_root\"",
    ] {
        assert!(
            linux_source.contains(token),
            "Linux release packaging must sanitize a staging copy using token {token}"
        );
    }
}

#[test]
fn linux_deb_packaging_installs_runtime_dependencies_and_launcher() {
    let root = repo_root();
    let script_path = root.join("scripts").join("package_linux_deb.sh");
    let source = fs::read_to_string(&script_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", script_path.display()));

    for required in [
        "bash \"$script_dir/package_portable_linux.sh\"",
        "dist/release-assets",
        "netstitch_${file_version}_${architecture}.deb",
        "$package_root/opt/netstitch",
        "$package_root/usr/bin",
        "$package_root/usr/share/applications",
        "$package_root/usr/share/pixmaps",
        "find \"$package_root/opt/netstitch/integrations\" -mindepth 2 -maxdepth 2 -type d -name data -exec rm -rf {} +",
        "opt/netstitch/libs/netstitch-tool/bin/linux-x86_64/libnetstitch_tool.so",
        "opt/netstitch/libs/netstitch-watcher/bin/linux-x86_64/libnetstitch_watcher.so",
        "opt/netstitch/resources/shin0by.png",
        "data_root=\"${XDG_DATA_HOME:-$HOME/.local/share}/netstitch\"",
        "cp -an \"$app_root/apps/.\" \"$data_root/apps/\"",
        "find \"$app_root/integrations\" -mindepth 1 -maxdepth 1 -type d",
        "find \"$module_dir\" -mindepth 1 -maxdepth 1 ! -name data -exec cp -a {} \"$target_module_dir/\" \\;",
        "export NETSTITCH__DATA_DIR=\"$data_root\"",
        "export NETSTITCH__CONNECTORS_DIR=\"$data_root/apps\"",
        "export NETSTITCH__INTEGRATIONS_DIR=\"$data_root/integrations\"",
        "exec /opt/netstitch/NetStitch \"$@\"",
        "Exec=/usr/bin/netstitch",
        "Icon=netstitch",
        "libwebkit2gtk-4.1-0",
        "libgtk-3-0 | libgtk-3-0t64",
        "libayatana-appindicator3-1",
        "libxdo3",
        "libssl3 | libssl3t64",
        "libsqlite3-0",
        "ca-certificates",
        "xdg-utils",
        "find \"$package_root\" -type d -exec chmod 0755 {} +",
        "find \"$package_root\" -type f -exec chmod 0644 {} +",
        "dpkg-deb --root-owner-group --build",
    ] {
        assert!(
            source.contains(required),
            "Linux deb packaging must keep token {required}"
        );
    }

    assert!(
        !source.contains("apt-get install"),
        "deb packaging must declare dependencies instead of installing host packages from the build script"
    );
    assert!(
        !source.contains("/mnt/c/"),
        "deb packaging must not hardcode the local WSL mount path"
    );
}

#[test]
fn linux_portable_launcher_installer_supports_cross_distro_fallback() {
    let root = repo_root();
    let script_path = root
        .join("resources")
        .join("linux")
        .join("install-desktop-launcher.sh");
    let source = fs::read_to_string(&script_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", script_path.display()));

    for required in [
        "--copy-to",
        "cp -a \"$portable_dir/.\" \"$target_dir/\"",
        "installs launchers that point to this portable folder",
        "installs launchers that point to the copied app",
        "report_missing_runtime_libraries",
        "ldd \"$app_path\"",
        "not found",
        "Debian/Ubuntu/Mint",
        "Debian 13",
        "Fedora",
        "Arch",
        "openSUSE",
        "libwebkit2gtk-4.1-0",
        "libgtk-3-0t64",
        "webkit2gtk4.1",
        "webkit2gtk-4.1",
        "Exec=$(escape_exec_path \"$app_path\")",
        "Icon=netstitch",
        "metadata::trusted",
    ] {
        assert!(
            source.contains(required),
            "Linux portable launcher installer must keep token {required}"
        );
    }
}

#[test]
fn release_branch_push_workflow_builds_archives_and_publishes_release() {
    let workflow_path = repo_root()
        .join(".github")
        .join("workflows")
        .join("release.yml");
    let release_source = fs::read_to_string(&workflow_path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", workflow_path.display()));

    for required in [
        "branches:",
        "- release",
        "base_version=\"$(awk",
        "config/release-version.json",
        "REQUESTED_VERSION: ${{ github.event.inputs.version || '' }}",
        "tested_version=\"$(python3 -c 'import json,sys;",
        "configured release version $tested_version must use Cargo base version $base_version",
        "tag=\"v${version}\"",
        "if [[ \"$GITHUB_EVENT_NAME\" == \"push\" && \"$GITHUB_REF\" == \"refs/heads/release\" ]]; then",
        "if [[ \"$GITHUB_EVENT_NAME\" == \"workflow_dispatch\" && \"$REQUESTED_PUBLISH\" == \"true\" ]]; then",
        "NETSTITCH__PACKAGE_REVISION: ${{ needs.version.outputs.revision }}",
        "scripts\\package_release_assets.ps1 -Version \"${{ needs.version.outputs.version }}\"",
        "NetStitch-win64-portable/WinDivert.dll",
        "NetStitch-win64-portable/WinDivert64.sys",
        "NetStitch-win64-portable/libs/netstitch-tool/bin/windows-x86_64/netstitch_tool.dll",
        "NetStitch-win64-portable/libs/netstitch-watcher/bin/windows-x86_64/netstitch_watcher.dll",
        "bash scripts/package_release_assets_linux.sh --version \"${{ needs.version.outputs.version }}\"",
        "NetStitch-linux64-portable/libs/netstitch-tool/bin/linux-x86_64/libnetstitch_tool.so",
        "NetStitch-linux64-portable/libs/netstitch-watcher/bin/linux-x86_64/libnetstitch_watcher.so",
        "bash scripts/package_linux_deb.sh --version \"${{ needs.version.outputs.version }}\" --skip-build",
        "netstitch_${{ needs.version.outputs.version }}_amd64.deb",
        "./usr/bin/netstitch",
        "./usr/share/applications/netstitch.desktop",
        "^./opt/netstitch/integrations/.+/data/",
        "dist/release-assets/*.deb",
        "if: needs.version.outputs.publish_release == 'true'",
        "git tag \"${{ needs.version.outputs.tag }}\" \"$GITHUB_SHA\"",
        "git push origin \"${{ needs.version.outputs.tag }}\"",
        "softprops/action-gh-release@v2",
        "tag_name: ${{ needs.version.outputs.tag }}",
    ] {
        assert!(
            release_source.contains(required),
            "release branch workflow must keep token {required}"
        );
    }

    assert!(
        !release_source.contains("push:\n    tags:"),
        "release deploy workflow should not rely on tag push recursion"
    );
    assert!(
        !release_source.contains("if [[ \"$GITHUB_REF\" == \"refs/heads/release\" || \"$REQUESTED_PUBLISH\" == \"true\" ]]; then"),
        "release branch publish gating should remain explicit by event kind"
    );
}

fn collect_forbidden_matches(path: &Path, forbidden: &[&str], matches: &mut Vec<String>) {
    collect_forbidden_matches_skipping(path, forbidden, matches, &[]);
}

fn collect_forbidden_matches_skipping(
    path: &Path,
    forbidden: &[&str],
    matches: &mut Vec<String>,
    skip_paths: &[&Path],
) {
    if skip_paths.iter().any(|skip_path| path.ends_with(skip_path)) {
        return;
    }
    if path.is_dir() {
        let entries = fs::read_dir(path)
            .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
        for entry in entries {
            let entry = entry.unwrap_or_else(|error| {
                panic!("{} entry should be readable: {error}", path.display())
            });
            collect_forbidden_matches_skipping(&entry.path(), forbidden, matches, skip_paths);
        }
        return;
    }
    if !path.is_file() {
        return;
    }
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return;
    };
    if !matches!(
        extension,
        "app"
            | "css"
            | "html"
            | "ini"
            | "js"
            | "json"
            | "md"
            | "ps1"
            | "rs"
            | "sh"
            | "svg"
            | "toml"
            | "txt"
    ) {
        return;
    }
    let content = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    for needle in forbidden {
        if content.contains(needle) {
            matches.push(format!("{} contains {needle}", path.display()));
        }
    }
}

fn latest_release_asset(dir: &Path, prefix: &str, suffix: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    entries
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.starts_with(prefix) && name.ends_with(suffix)
        })
        .filter_map(|entry| {
            let modified = entry.metadata().ok()?.modified().ok()?;
            Some((modified, entry.path()))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn tracked_paths_under(path: &Path) -> Vec<String> {
    let root = repo_root();
    let relative = path
        .strip_prefix(&root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/");
    let output = std::process::Command::new("git")
        .arg("-c")
        .arg(format!(
            "safe.directory={}",
            root.display().to_string().replace('\\', "/")
        ))
        .arg("ls-files")
        .arg(relative)
        .current_dir(root)
        .output()
        .expect("git ls-files should run");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(ToOwned::to_owned)
        .collect()
}

fn collect_missing_module_sdk_english_docs(dir: &Path, missing: &mut Vec<String>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", dir.display()));
    for entry in entries {
        let entry =
            entry.unwrap_or_else(|error| panic!("directory entry should be readable: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_missing_module_sdk_english_docs(&path, missing);
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some(prefix) = name.strip_suffix("_RU.md") else {
            continue;
        };
        let expected = path.with_file_name(format!("{prefix}_EN.md"));
        if !expected.is_file() {
            missing.push(format!(
                "{} -> missing {}",
                path.display(),
                expected.display()
            ));
        }
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("regression crate should live under tests/netstitch-regression")
        .to_path_buf()
}
