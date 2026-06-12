use std::ffi::c_void;
use std::slice;

#[repr(C)]
pub struct NetStitchAbiBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn netstitch_integration_call(
    request_ptr: *const u8,
    request_len: usize,
    _event_callback: Option<unsafe extern "C" fn(*const u8, usize, *mut c_void)>,
    _event_user_data: *mut c_void,
    out_response: *mut NetStitchAbiBuffer,
) -> i32 {
    if request_ptr.is_null() || out_response.is_null() {
        return 1;
    }
    let request_bytes = unsafe { slice::from_raw_parts(request_ptr, request_len) };
    let request = String::from_utf8_lossy(request_bytes);
    let action = extract_json_string(&request, "action").unwrap_or_default();
    let action_id = extract_json_string(&request, "action_id").unwrap_or_default();
    let response = match action {
        "ui_action" => ui_action_response(action_id, &request),
        "background_event" => background_event_response(&request),
        _ => format!(
            "{{\"ok\":false,\"error\":\"unsupported action: {}\"}}",
            json_escape(action)
        ),
    };
    write_response(response, out_response);
    0
}

fn ui_action_response(action_id: &str, request: &str) -> String {
    match action_id {
        "say_hello" => {
            ok_response(
                "Rust demo: Hello world dialog opened",
                "success",
                false,
                r#"[{"command_type":"show_dialog","payload":{"dialog_id":"hello-rust-hello","buttons":"ok","severity":"success","message":"Hello world"}}]"#,
            )
        }
        "toggle_listener" => {
            if json_bool(request, "module_background_active") {
                ok_response(
                    "Rust demo listener stopped",
                    "info",
                    true,
                    r#"[
                      {"command_type":"stop_background","payload":{}},
                      {"command_type":"log_event","payload":{"severity":"info","message":"Rust demo listener stopped"}}
                    ]"#,
                )
            } else {
                ok_response(
                    "Rust demo listener started",
                    "success",
                    true,
                    r#"[
                      {"command_type":"start_background","payload":{"subscriptions":["monitoring.rows_added","monitoring.rows_changed","monitoring.selection_changed","monitoring.started","monitoring.stopped"]}},
                      {"command_type":"log_event","payload":{"severity":"success","message":"Rust demo listener started"}}
                    ]"#,
                )
            }
        }
        "show_last_rows" => ok_response(
            "Rust demo: opened latest rows page",
            "info",
            true,
            r#"[{"command_type":"set_module_page","payload":{"page":"last_rows"}}]"#,
        ),
        "about" => ok_response(
            "Rust demo: about dialog opened",
            "info",
            false,
            r#"[
              {"command_type":"show_dialog","payload":{"dialog_id":"hello-rust-about","buttons":"ok","severity":"info","message":"Hello World Rust uses ui_schema entities, selected Monitoring row payloads, start_background/stop_background, log_event, show_dialog, and set_module_page."}},
              {"command_type":"log_event","payload":{"severity":"info","message":"Rust demo about dialog requested"}}
            ]"#,
        ),
        other => ok_response(
            &format!("Rust demo action '{other}' is not implemented"),
            "warning",
            false,
            "[]",
        ),
    }
}

fn background_event_response(request: &str) -> String {
    let event_type = extract_json_string(request, "event_type").unwrap_or("background_event");
    ok_response(
        &format!("Rust demo listener received {event_type}"),
        "info",
        true,
        "[]",
    )
}

fn ok_response(message: &str, severity: &str, refresh: bool, commands: &str) -> String {
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
    std::mem::forget(response);
    unsafe {
        *out_response = buffer;
    }
}

fn extract_json_string<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\"");
    let key_pos = source.find(&needle)?;
    let colon_pos = source[key_pos + needle.len()..].find(':')? + key_pos + needle.len();
    let value_start = source[colon_pos + 1..].find('"')? + colon_pos + 2;
    let value_end = source[value_start..].find('"')? + value_start;
    Some(&source[value_start..value_end])
}

fn json_bool(source: &str, key: &str) -> bool {
    let needle = format!("\"{key}\"");
    let Some(key_pos) = source.find(&needle) else {
        return false;
    };
    let Some(colon_pos) = source[key_pos + needle.len()..]
        .find(':')
        .map(|offset| key_pos + needle.len() + offset)
    else {
        return false;
    };
    source[colon_pos + 1..].trim_start().starts_with("true")
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch == '\\' || ch == '"' {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn netstitch_integration_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        unsafe {
            drop(Vec::from_raw_parts(ptr, len, len));
        }
    }
}
