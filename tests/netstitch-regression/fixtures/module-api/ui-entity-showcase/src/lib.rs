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
    let request =
        String::from_utf8_lossy(unsafe { slice::from_raw_parts(request_ptr, request_len) });
    let action = extract_json_string(&request, "action").unwrap_or_default();
    let action_id = extract_json_string(&request, "action_id").unwrap_or_default();
    let response = match action {
        "ui_action" => ui_action_response(action_id, &request),
        "background_event" => ok_response(
            "UI showcase background event received",
            "info",
            false,
            r#"[{"command_type":"log_event","payload":{"severity":"info","message":"UI showcase observed host event"}}]"#,
        ),
        _ => ok_response(
            "UI showcase ignored unsupported action",
            "warning",
            false,
            "[]",
        ),
    };
    write_response(response, out_response);
    0
}

fn ui_action_response(action_id: &str, request: &str) -> String {
    match action_id {
        "inspect_ui_values" => ok_response(
            "UI showcase values reset to defaults",
            "success",
            true,
            r#"[{"command_type":"set_ui_values","payload":{"values":{"showcase-tabs":"ui_entities","showcase-input":"editable","showcase-textarea":"Line 1\nLine 2","showcase-select":"two","showcase-switch":true,"showcase-folder-status":"No folder selected","showcase-folder-path":"-","showcase-save-status":"No save target selected","showcase-save-path":"-"}}},{"command_type":"log_event","payload":{"severity":"success","message":"UI showcase reset controls to defaults"}}]"#,
        ),
        "browse_folder_window" => ok_response(
            "UI showcase folder window requested",
            "info",
            false,
            r#"[{"command_type":"browse_window","payload":{"target":"showcase-folder-path","status_target":"showcase-folder-status","selected_status":"Folder selected","mode":"folder","title":"Select folder","confirm_label":"Select","start_dir":""}}]"#,
        ),
        "browse_save_window" => ok_response(
            "UI showcase save window requested",
            "info",
            false,
            r#"[{"command_type":"browse_window","payload":{"target":"showcase-save-path","status_target":"showcase-save-status","selected_status":"Save target selected; file was not written","mode":"file_save","title":"Select save target","confirm_label":"Select","default_name":"netstitch-showcase","default_extension":"txt","overwrite_policy":"prompt","can_create_directories":true,"filters":[{"name":"Text files","extensions":["txt"]},{"name":"Config files","extensions":["conf","json"]}]}}]"#,
        ),
        "start_showcase_background" => {
            if extract_json_bool(request, "background_active").unwrap_or(false) {
                ok_response(
                    "UI showcase listener stopped",
                    "info",
                    true,
                    r#"[{"command_type":"stop_background","payload":{}},{"command_type":"log_event","payload":{"severity":"info","message":"UI showcase listener stopped"}}]"#,
                )
            } else {
                ok_response(
                    "UI showcase listener started",
                    "success",
                    true,
                    r#"[{"command_type":"start_background","payload":{"subscriptions":["ui.controls","ui.tables","monitoring.*","filters.*"]}},{"command_type":"log_event","payload":{"severity":"success","message":"UI showcase listener started"}}]"#,
                )
            }
        }
        "stop_showcase_background" => ok_response(
            "UI showcase listener stopped",
            "info",
            true,
            r#"[{"command_type":"stop_background","payload":{}}]"#,
        ),
        "show_notice" => ok_response(
            "UI showcase dialog requested",
            "info",
            false,
            r#"[{"command_type":"show_dialog","payload":{"dialog_id":"ui-showcase-notice","buttons":"ok","severity":"info","message":"This module demonstrates standard NetStitch UI entities."}}]"#,
        ),
        other => ok_response(
            &format!("UI showcase action '{other}' received"),
            "info",
            false,
            r#"[{"command_type":"log_event","payload":{"severity":"info","message":"UI showcase received action"}}]"#,
        ),
    }
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

fn extract_json_bool(source: &str, key: &str) -> Option<bool> {
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
