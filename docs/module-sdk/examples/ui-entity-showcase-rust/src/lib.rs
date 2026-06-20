use std::ffi::c_void;
use std::slice;
use std::thread;
use std::time::Duration;

// NetStitch uses one stable native ABI for all module languages. The host gives
// the module UTF-8 JSON bytes and expects the module to return owned UTF-8 JSON
// bytes through this buffer. `ptr` must point to memory allocated by the module,
// because the host will later return the same pointer to netstitch_integration_free.
#[repr(C)]
pub struct NetStitchAbiBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

type NetStitchEventCallback = unsafe extern "C" fn(*const u8, usize, *mut c_void);

// This is the only required entrypoint. New Module API features are carried in
// the JSON payload, so the C ABI does not need to change when the host learns a
// new command or UI entity.
//
// Request shape used by this sample:
// {
//   "action": "ui_action" | "background_event" | "...",
//   "payload": { ... host context and UI values ... }
// }
// `action_id` is nested in the payload in real host requests. The tiny demo
// extractor below searches the full JSON string, which is enough for the fixed
// showcase ids but should be replaced with serde_json in production modules.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn netstitch_integration_call(
    request_ptr: *const u8,
    request_len: usize,
    event_callback: Option<NetStitchEventCallback>,
    event_user_data: *mut c_void,
    out_response: *mut NetStitchAbiBuffer,
) -> i32 {
    if request_ptr.is_null() || out_response.is_null() {
        return 1;
    }

    // This sample intentionally avoids external crates. Production modules may
    // use serde_json; the tiny string helpers below keep the example copyable as
    // a single file. The host always sends UTF-8 JSON, so lossy conversion is
    // only a last-resort guard for a broken caller.
    let request =
        String::from_utf8_lossy(unsafe { slice::from_raw_parts(request_ptr, request_len) });
    let action = extract_json_string(&request, "action").unwrap_or_default();
    let action_id = extract_json_string(&request, "action_id").unwrap_or_default();
    let russian = request_language_is_russian(&request);
    let response = match action {
        "ui_action" => ui_action_response(
            action_id,
            &request,
            russian,
            event_callback,
            event_user_data,
        ),
        // Background events arrive only after the user starts background work
        // with start_background. The module can log or update its own state here;
        // it should not treat discovery/loading as implicit startup.
        "background_event" => ok_response(
            tr(russian, "background_event_received"),
            "info",
            false,
            &format!(
                r#"[{{"command_type":"log_event","payload":{{"severity":"info","message":"{}"}}}}]"#,
                json_escape(tr(russian, "background_event_log"))
            ),
        ),
        _ => ok_response(tr(russian, "unsupported_action"), "warning", false, "[]"),
    };
    write_response(response, out_response);
    0
}

fn ui_action_response(
    action_id: &str,
    request: &str,
    russian: bool,
    event_callback: Option<NetStitchEventCallback>,
    event_user_data: *mut c_void,
) -> String {
    match action_id {
        // Action from the "Set default" button. The module does not mutate host
        // UI directly; it returns a host command that asks NetStitch to patch
        // host-owned control values.
        //
        // The keys in `values` are entity ids from module.json. This is the usual
        // pattern for module controls: declare the UI in the manifest, then ask
        // the host to update values by stable ids.
        "inspect_ui_values" => {
            let commands = format!(
                r#"[{{"command_type":"set_ui_values","payload":{{"values":{{"showcase-tabs":"ui_entities","showcase-input":"{}","showcase-textarea":"{}","showcase-select":"two","showcase-switch":true,"showcase-progress":68,"showcase-progress-compact":42,"showcase-folder-status":"{}","showcase-folder-path":"{}","showcase-save-status":"{}","showcase-save-path":"{}"}}}}}},{{"command_type":"log_event","payload":{{"severity":"success","message":"{}"}}}}]"#,
                json_escape(tr(russian, "text_input_default")),
                json_escape(tr(russian, "textarea_default")),
                json_escape(tr(russian, "folder_status_default")),
                json_escape(tr(russian, "path_empty")),
                json_escape(tr(russian, "save_status_default")),
                json_escape(tr(russian, "path_empty")),
                json_escape(tr(russian, "reset_log"))
            );
            ok_response(tr(russian, "values_reset"), "success", true, &commands)
        }
        "simulate_download" => {
            simulate_download_progress(event_callback, event_user_data);
            let commands = format!(
                r#"[{{"command_type":"set_ui_values","payload":{{"values":{{"showcase-progress":100}}}}}},{{"command_type":"log_event","payload":{{"severity":"success","message":"{}"}}}}]"#,
                json_escape(tr(russian, "download_simulated_log"))
            );
            ok_response(
                tr(russian, "download_simulated"),
                "success",
                false,
                &commands,
            )
        }
        "browse_folder_window" => {
            let commands = format!(
                r#"[{{"command_type":"browse_window","payload":{{"target":"showcase-folder-path","status_target":"showcase-folder-status","selected_status":"{}","mode":"folder","title":"{}","confirm_label":"{}","start_dir":""}}}}]"#,
                json_escape(tr(russian, "folder_selected_status")),
                json_escape(tr(russian, "folder_window_title")),
                json_escape(tr(russian, "folder_confirm_label"))
            );
            ok_response(
                tr(russian, "folder_window_requested"),
                "info",
                false,
                &commands,
            )
        }
        "browse_save_window" => {
            let commands = format!(
                r#"[{{"command_type":"browse_window","payload":{{"target":"showcase-save-path","status_target":"showcase-save-status","selected_status":"{}","mode":"file_save","title":"{}","confirm_label":"{}","default_name":"netstitch-showcase","default_extension":"txt","overwrite_policy":"prompt","can_create_directories":true,"filters":[{{"name":"{}","patterns":["*.txt","*.md"]}},{{"name":"{}","patterns":["*.conf","*.json"]}},{{"name":"{}","patterns":["*.bat","*.cmd"]}},{{"name":"{}","patterns":["config","config.*","config*"]}},{{"name":"{}","patterns":["profile-??.conf","profile-*.json"]}}]}}}}]"#,
                json_escape(tr(russian, "save_selected_status")),
                json_escape(tr(russian, "save_window_title")),
                json_escape(tr(russian, "save_confirm_label")),
                json_escape(tr(russian, "text_files_filter")),
                json_escape(tr(russian, "config_files_filter")),
                json_escape(tr(russian, "script_files_filter")),
                json_escape(tr(russian, "config_name_filter")),
                json_escape(tr(russian, "wildcard_files_filter"))
            );
            ok_response(
                tr(russian, "save_window_requested"),
                "info",
                false,
                &commands,
            )
        }
        // Header action. The same button starts and stops a background listener;
        // host state is available in the request payload.
        "start_showcase_background" => {
            // `background_active` is supplied by the host context. Reading it
            // prevents two independent "start" commands from racing each other
            // when a user clicks the header action more than once.
            if extract_json_bool(request, "background_active").unwrap_or(false) {
                let commands = format!(
                    r#"[{{"command_type":"stop_background","payload":{{}}}},{{"command_type":"log_event","payload":{{"severity":"info","message":"{}"}}}}]"#,
                    json_escape(tr(russian, "listener_stopped_log"))
                );
                ok_response(tr(russian, "listener_stopped"), "info", true, &commands)
            } else {
                let commands = format!(
                    r#"[{{"command_type":"start_background","payload":{{"subscriptions":["ui.controls","ui.tables","monitoring.*","filters.*"]}}}},{{"command_type":"log_event","payload":{{"severity":"success","message":"{}"}}}}]"#,
                    json_escape(tr(russian, "listener_started_log"))
                );
                ok_response(
                    tr(russian, "listener_started"),
                    "success",
                    true,
                    // Subscriptions are explicit and user-triggered. The module
                    // is not allowed to autostart background work on discovery.
                    // Keep this list narrow in real modules: subscribe only to
                    // host events you actually use.
                    &commands,
                )
            }
        }
        "stop_showcase_background" => ok_response(
            tr(russian, "listener_stopped"),
            "info",
            true,
            r#"[{"command_type":"stop_background","payload":{}}]"#,
        ),
        "show_notice" => {
            let commands = format!(
                r#"[{{"command_type":"show_dialog","payload":{{"dialog_id":"ui-showcase-rust-notice","buttons":"ok","severity":"info","message":"{}"}}}}]"#,
                json_escape(tr(russian, "dialog_message"))
            );
            ok_response(
                tr(russian, "dialog_requested"),
                "info",
                false,
                // Dialogs are rendered by the host so desktop and browser shells
                // stay visually identical.
                // The module only provides a stable dialog id, severity, buttons and
                // message. It does not create windows or platform-specific UI.
                &commands,
            )
        }
        other => ok_response(
            &format!("{} '{other}'", tr(russian, "action_received_prefix")),
            "info",
            false,
            &format!(
                r#"[{{"command_type":"log_event","payload":{{"severity":"info","message":"{}"}}}}]"#,
                json_escape(tr(russian, "action_received_log"))
            ),
        ),
    }
}

fn simulate_download_progress(
    event_callback: Option<NetStitchEventCallback>,
    event_user_data: *mut c_void,
) {
    for percent in 0..=100_u8 {
        emit_ui_values(event_callback, event_user_data, percent);
        thread::sleep(Duration::from_millis(20));
    }
}

fn emit_ui_values(
    event_callback: Option<NetStitchEventCallback>,
    event_user_data: *mut c_void,
    percent: u8,
) {
    let Some(callback) = event_callback else {
        return;
    };
    let event = format!(
        r#"{{"event":"ui_values","payload":{{"values":{{"showcase-progress":{percent}}}}}}}"#
    );
    unsafe {
        callback(event.as_ptr(), event.len(), event_user_data);
    }
}

fn ok_response(message: &str, severity: &str, refresh: bool, commands: &str) -> String {
    // Host response contract:
    // - ok=true means the ABI call itself succeeded.
    // - result.message/severity become a user-visible status.
    // - refresh=true asks the host to request a fresh module snapshot.
    // - commands is a whitelist of host actions; unsupported commands are ignored
    //   or rejected by NetStitch instead of being executed blindly.
    format!(
        "{{\"ok\":true,\"result\":{{\"message\":\"{}\",\"severity\":\"{}\",\"refresh\":{},\"commands\":{}}}}}",
        json_escape(message),
        json_escape(severity),
        if refresh { "true" } else { "false" },
        commands
    )
}

fn write_response(response: String, out_response: *mut NetStitchAbiBuffer) {
    let mut response = response.into_bytes();
    let buffer = NetStitchAbiBuffer {
        ptr: response.as_mut_ptr(),
        len: response.len(),
    };
    // Transfer ownership to the host. NetStitch will call
    // netstitch_integration_free with the same pointer and length. Do not return
    // a pointer to stack memory or to a temporary string.
    std::mem::forget(response);
    unsafe {
        *out_response = buffer;
    }
}

fn extract_json_string<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    // Demo-only helper: it is enough for known request fields, but a real module
    // should prefer serde_json when it starts handling complex payloads.
    let needle = format!("\"{key}\"");
    let key_pos = source.find(&needle)?;
    let colon_pos = source[key_pos + needle.len()..].find(':')? + key_pos + needle.len();
    let value_start = source[colon_pos + 1..].find('"')? + colon_pos + 2;
    let value_end = source[value_start..].find('"')? + value_start;
    Some(&source[value_start..value_end])
}

fn extract_json_bool(source: &str, key: &str) -> Option<bool> {
    // Same intentionally small parser as extract_json_string. It demonstrates
    // where the value comes from without hiding the ABI behind SDK code.
    let needle = format!("\"{key}\"");
    let key_pos = source.find(&needle)?;
    let colon_pos = source[key_pos + needle.len()..].find(':')? + key_pos + needle.len();
    let value = source[colon_pos + 1..].trim_start();
    if value.starts_with("true") {
        Some(true)
    } else if value.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn request_language_is_russian(request: &str) -> bool {
    extract_json_string(request, "language_code")
        .map(|value| value.to_ascii_lowercase().starts_with("ru"))
        .unwrap_or(false)
}

fn tr(russian: bool, key: &str) -> &'static str {
    // Manifest text is localized through locales/*.ini. Runtime messages are
    // produced by module code, so this sample reads context.language_code and
    // chooses a tiny built-in RU/EN dictionary.
    match (russian, key) {
        (true, "values_reset") => "Rust: значения UI-примера сброшены",
        (false, "values_reset") => "Rust UI showcase values reset to defaults",
        (true, "reset_log") => "Rust UI showcase: элементы управления сброшены",
        (false, "reset_log") => "Rust UI showcase reset controls to defaults",
        (true, "folder_window_requested") => "Rust: открыто окно выбора папки",
        (false, "folder_window_requested") => "Rust UI showcase folder window requested",
        (true, "save_window_requested") => "Rust: открыто окно выбора пути сохранения",
        (false, "save_window_requested") => "Rust UI showcase save window requested",
        (true, "download_simulated") => "Rust: имитация загрузки завершена",
        (false, "download_simulated") => "Rust UI showcase simulated download completed",
        (true, "download_simulated_log") => "Rust UI showcase провёл progress через все фазы",
        (false, "download_simulated_log") => {
            "Rust UI showcase animated progress through all phases"
        }
        (true, "folder_window_title") => "Выберите папку для UI-примера",
        (false, "folder_window_title") => "Choose a folder for the UI showcase",
        (true, "save_window_title") => "Выберите путь сохранения без записи файла",
        (false, "save_window_title") => "Choose a save target without writing a file",
        (true, "folder_confirm_label") => "Выбрать папку",
        (false, "folder_confirm_label") => "Choose folder",
        (true, "save_confirm_label") => "Выбрать путь",
        (false, "save_confirm_label") => "Choose target",
        (true, "folder_status_default") => "Папка не выбрана",
        (false, "folder_status_default") => "No folder selected",
        (true, "save_status_default") => "Путь сохранения не выбран",
        (false, "save_status_default") => "No save target selected",
        (true, "folder_selected_status") => "Папка выбрана",
        (false, "folder_selected_status") => "Folder selected",
        (true, "save_selected_status") => "Путь сохранения выбран; файл не записан",
        (false, "save_selected_status") => "Save target selected; file was not written",
        (true, "text_files_filter") => "Текст (*.txt; *.md)",
        (false, "text_files_filter") => "Text (*.txt; *.md)",
        (true, "config_files_filter") => "Конфиги (*.conf; *.json)",
        (false, "config_files_filter") => "Config (*.conf; *.json)",
        (true, "script_files_filter") => "Скрипты (*.bat; *.cmd)",
        (false, "script_files_filter") => "Scripts (*.bat; *.cmd)",
        (true, "config_name_filter") => "Имена config / config.* / config*",
        (false, "config_name_filter") => "Names config / config.* / config*",
        (true, "wildcard_files_filter") => "Wildcard-маски профилей",
        (false, "wildcard_files_filter") => "Profile wildcard masks",
        (true, "listener_started") => "Rust: listener запущен из хедера модуля",
        (false, "listener_started") => "Rust UI showcase listener started",
        (true, "listener_started_log") => "Rust UI showcase listener запущен",
        (false, "listener_started_log") => "Rust UI showcase listener started",
        (true, "listener_stopped") => "Rust: listener остановлен",
        (false, "listener_stopped") => "Rust UI showcase listener stopped",
        (true, "listener_stopped_log") => "Rust UI showcase listener остановлен",
        (false, "listener_stopped_log") => "Rust UI showcase listener stopped",
        (true, "dialog_requested") => "Rust: открыт стандартный диалог",
        (false, "dialog_requested") => "Rust UI showcase dialog requested",
        (true, "dialog_message") => {
            "Этот Rust-модуль демонстрирует стандартные UI-сущности NetStitch."
        }
        (false, "dialog_message") => {
            "This Rust module demonstrates standard NetStitch UI entities."
        }
        (true, "background_event_received") => "Rust: получено событие host-а",
        (false, "background_event_received") => "Rust UI showcase background event received",
        (true, "background_event_log") => "Rust UI showcase получил событие host-а",
        (false, "background_event_log") => "Rust UI showcase observed host event",
        (true, "unsupported_action") => "Rust: неподдерживаемое действие проигнорировано",
        (false, "unsupported_action") => "Rust UI showcase ignored unsupported action",
        (true, "action_received_prefix") => "Rust: получено действие",
        (false, "action_received_prefix") => "Rust UI showcase action received",
        (true, "action_received_log") => "Rust UI showcase получил действие",
        (false, "action_received_log") => "Rust UI showcase received action",
        (true, "text_input_default") => "редактируемый текст",
        (false, "text_input_default") => "editable",
        (true, "textarea_default") => "Строка 1\nСтрока 2",
        (false, "textarea_default") => "Line 1\nLine 2",
        (true, "path_empty") | (false, "path_empty") => "-",
        _ => "",
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' | '"' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn netstitch_integration_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        // Rebuild the Vec so Rust can free the allocation created in
        // write_response. The capacity is equal to len because write_response
        // returns the exact Vec allocation after converting the response string
        // to bytes.
        unsafe {
            drop(Vec::from_raw_parts(ptr, len, len));
        }
    }
}
