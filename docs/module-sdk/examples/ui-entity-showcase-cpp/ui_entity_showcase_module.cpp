#include <cstddef>
#include <cstdint>
#include <chrono>
#include <cstring>
#include <string>
#include <thread>

#if defined(_WIN32)
#define NETSTITCH_EXPORT __declspec(dllexport)
#else
#define NETSTITCH_EXPORT __attribute__((visibility("default")))
#endif

struct NetStitchAbiBuffer {
    // The module owns this memory until the host calls netstitch_integration_free.
    // Never point this at stack memory or at std::string::data() directly.
    uint8_t* ptr;
    uintptr_t len;
};

using NetStitchEventCallback = void (*)(const uint8_t*, uintptr_t, void*);

// This sample keeps dependencies at zero so it can be copied into a fresh
// module project. For larger payloads, use a real JSON library and keep the ABI
// functions below unchanged. The stable part is the C ABI; JSON fields can grow
// without changing the exported function names.
static std::string json_escape(const std::string& value) {
    std::string out;
    out.reserve(value.size());
    for (char ch : value) {
        switch (ch) {
        case '\\':
        case '"':
            out.push_back('\\');
            out.push_back(ch);
            break;
        case '\n':
            out += "\\n";
            break;
        case '\r':
            out += "\\r";
            break;
        case '\t':
            out += "\\t";
            break;
        default:
            out.push_back(ch);
            break;
        }
    }
    return out;
}

static std::string extract_json_string(const std::string& source, const std::string& key) {
    // Demo-only field extractor for simple host request fields. In real host
    // requests action_id lives in the payload object; searching the whole JSON
    // string is enough for this fixed showcase but is not a replacement for a
    // JSON parser in production code.
    const std::string needle = "\"" + key + "\"";
    auto key_pos = source.find(needle);
    if (key_pos == std::string::npos) return "";
    auto colon_pos = source.find(':', key_pos + needle.size());
    if (colon_pos == std::string::npos) return "";
    auto value_start = source.find('"', colon_pos + 1);
    if (value_start == std::string::npos) return "";
    auto value_end = source.find('"', value_start + 1);
    if (value_end == std::string::npos) return "";
    return source.substr(value_start + 1, value_end - value_start - 1);
}

static bool extract_json_bool(const std::string& source, const std::string& key) {
    // Same intentionally small parser as extract_json_string. It keeps the
    // sample dependency-free while still showing where host context values come
    // from.
    const std::string needle = "\"" + key + "\"";
    auto key_pos = source.find(needle);
    if (key_pos == std::string::npos) return false;
    auto colon_pos = source.find(':', key_pos + needle.size());
    if (colon_pos == std::string::npos) return false;
    auto value_pos = source.find_first_not_of(" \r\n\t", colon_pos + 1);
    return value_pos != std::string::npos && source.compare(value_pos, 4, "true") == 0;
}

static bool request_language_is_russian(const std::string& request) {
    std::string language = extract_json_string(request, "language_code");
    for (char& ch : language) {
        if (ch >= 'A' && ch <= 'Z') {
            ch = static_cast<char>(ch - 'A' + 'a');
        }
    }
    return language.rfind("ru", 0) == 0;
}

static void emit_ui_values(NetStitchEventCallback event_callback, void* event_user_data, int percent) {
    if (!event_callback) return;
    const std::string event =
        "{\"event\":\"ui_values\",\"payload\":{\"values\":{\"showcase-progress\":" +
        std::to_string(percent) + "}}}";
    event_callback(
        reinterpret_cast<const uint8_t*>(event.data()),
        static_cast<uintptr_t>(event.size()),
        event_user_data
    );
}

static void simulate_download_progress(NetStitchEventCallback event_callback, void* event_user_data) {
    for (int percent = 0; percent <= 100; ++percent) {
        emit_ui_values(event_callback, event_user_data, percent);
        std::this_thread::sleep_for(std::chrono::milliseconds(20));
    }
}

static std::string tr(bool russian, const std::string& key) {
    // Manifest text is localized through locales/*.ini. Runtime messages are
    // produced by module code, so this sample reads context.language_code and
    // chooses a tiny built-in RU/EN dictionary.
    if (russian) {
        if (key == "values_reset") return "C++: значения UI-примера сброшены";
        if (key == "reset_log") return "C++ UI showcase: элементы управления сброшены";
        if (key == "folder_window_requested") return "C++: открыто окно выбора папки";
        if (key == "save_window_requested") return "C++: открыто окно выбора пути сохранения";
        if (key == "download_simulated") return "C++: имитация загрузки завершена";
        if (key == "download_simulated_log") return "C++ UI showcase провёл progress через все фазы";
        if (key == "folder_window_title") return "Выберите папку для UI-примера";
        if (key == "save_window_title") return "Выберите путь сохранения без записи файла";
        if (key == "folder_confirm_label") return "Выбрать папку";
        if (key == "save_confirm_label") return "Выбрать путь";
        if (key == "folder_status_default") return "Папка не выбрана";
        if (key == "save_status_default") return "Путь сохранения не выбран";
        if (key == "folder_selected_status") return "Папка выбрана";
        if (key == "save_selected_status") return "Путь сохранения выбран; файл не записан";
        if (key == "text_files_filter") return "Текстовые файлы";
        if (key == "config_files_filter") return "Конфигурационные файлы";
        if (key == "listener_started") return "C++: listener запущен из хедера модуля";
        if (key == "listener_started_log") return "C++ UI showcase listener запущен";
        if (key == "listener_stopped") return "C++: listener остановлен";
        if (key == "listener_stopped_log") return "C++ UI showcase listener остановлен";
        if (key == "dialog_requested") return "C++: открыт стандартный диалог";
        if (key == "dialog_message") {
            return "Этот C++ модуль демонстрирует стандартные UI-сущности NetStitch.";
        }
        if (key == "background_event_received") return "C++: получено событие host-а";
        if (key == "background_event_log") return "C++ UI showcase получил событие host-а";
        if (key == "unsupported_action") return "C++: неподдерживаемое действие проигнорировано";
        if (key == "action_received_prefix") return "C++: получено действие";
        if (key == "action_received_log") return "C++ UI showcase получил действие";
        if (key == "text_input_default") return "редактируемый текст";
        if (key == "textarea_default") return "Строка 1\nСтрока 2";
    }
    if (key == "values_reset") return "C++ UI showcase values reset to defaults";
    if (key == "reset_log") return "C++ UI showcase reset controls to defaults";
    if (key == "folder_window_requested") return "C++ UI showcase folder window requested";
    if (key == "save_window_requested") return "C++ UI showcase save window requested";
    if (key == "download_simulated") return "C++ UI showcase simulated download completed";
    if (key == "download_simulated_log") return "C++ UI showcase animated progress through all phases";
    if (key == "folder_window_title") return "Choose a folder for the UI showcase";
    if (key == "save_window_title") return "Choose a save target without writing a file";
    if (key == "folder_confirm_label") return "Choose folder";
    if (key == "save_confirm_label") return "Choose target";
    if (key == "folder_status_default") return "No folder selected";
    if (key == "save_status_default") return "No save target selected";
    if (key == "folder_selected_status") return "Folder selected";
    if (key == "save_selected_status") return "Save target selected; file was not written";
    if (key == "text_files_filter") return "Text files";
    if (key == "config_files_filter") return "Config files";
    if (key == "listener_started") return "C++ UI showcase listener started";
    if (key == "listener_started_log") return "C++ UI showcase listener started";
    if (key == "listener_stopped") return "C++ UI showcase listener stopped";
    if (key == "listener_stopped_log") return "C++ UI showcase listener stopped";
    if (key == "dialog_requested") return "C++ UI showcase dialog requested";
    if (key == "dialog_message") {
        return "This C++ module demonstrates standard NetStitch UI entities.";
    }
    if (key == "background_event_received") return "C++ UI showcase background event received";
    if (key == "background_event_log") return "C++ UI showcase observed host event";
    if (key == "unsupported_action") return "C++ UI showcase ignored unsupported action";
    if (key == "action_received_prefix") return "C++ UI showcase action received";
    if (key == "action_received_log") return "C++ UI showcase received action";
    if (key == "text_input_default") return "editable";
    if (key == "textarea_default") return "Line 1\nLine 2";
    if (key == "path_empty") return "-";
    return "";
}

static std::string ok_response(
    const std::string& message,
    const std::string& severity,
    bool refresh,
    const std::string& commands
) {
    // Host response contract:
    // - ok=true means the ABI call itself succeeded.
    // - result.message/severity become a user-visible status.
    // - refresh=true asks the host to request a fresh module snapshot.
    // - commands is a whitelist of host actions; unsupported commands are ignored
    //   or rejected by NetStitch instead of being executed blindly.
    return "{\"ok\":true,\"result\":{\"message\":\"" + json_escape(message) +
        "\",\"severity\":\"" + json_escape(severity) +
        "\",\"refresh\":" + (refresh ? "true" : "false") +
        ",\"commands\":" + commands + "}}";
}

static std::string ui_action_response(
    const std::string& action_id,
    const std::string& request,
    bool russian,
    NetStitchEventCallback event_callback,
    void* event_user_data
) {
    if (action_id == "inspect_ui_values") {
        // The module never reaches into the host UI. It asks NetStitch to update
        // host-owned controls through a set_ui_values command.
        //
        // The keys in `values` are entity ids from module.json. This is the usual
        // pattern for module controls: declare the UI in the manifest, then ask
        // the host to update values by stable ids.
        const std::string commands =
            R"([{"command_type":"set_ui_values","payload":{"values":{"showcase-tabs":"ui_entities","showcase-input":")" +
            json_escape(tr(russian, "text_input_default")) +
            R"(","showcase-textarea":")" +
            json_escape(tr(russian, "textarea_default")) +
            R"(","showcase-select":"two","showcase-switch":true,"showcase-progress":68,"showcase-progress-compact":42,"showcase-folder-status":")" +
            json_escape(tr(russian, "folder_status_default")) +
            R"(","showcase-folder-path":")" +
            json_escape(tr(russian, "path_empty")) +
            R"(","showcase-save-status":")" +
            json_escape(tr(russian, "save_status_default")) +
            R"(","showcase-save-path":")" +
            json_escape(tr(russian, "path_empty")) +
            R"("}}},{"command_type":"log_event","payload":{"severity":"success","message":")" +
            json_escape(tr(russian, "reset_log")) + R"("}}])";
        return ok_response(
            tr(russian, "values_reset"),
            "success",
            true,
            commands
        );
    }
    if (action_id == "simulate_download") {
        simulate_download_progress(event_callback, event_user_data);
        const std::string commands =
            R"([{"command_type":"log_event","payload":{"severity":"success","message":")" +
            json_escape(tr(russian, "download_simulated_log")) + R"("}}])";
        return ok_response(
            tr(russian, "download_simulated"),
            "success",
            false,
            commands
        );
    }
    if (action_id == "browse_folder_window") {
        const std::string commands =
            R"([{"command_type":"browse_window","payload":{"target":"showcase-folder-path","status_target":"showcase-folder-status","selected_status":")" +
            json_escape(tr(russian, "folder_selected_status")) +
            R"(","mode":"folder","title":")" +
            json_escape(tr(russian, "folder_window_title")) +
            R"(","confirm_label":")" +
            json_escape(tr(russian, "folder_confirm_label")) +
            R"(","start_dir":""}}])";
        return ok_response(
            tr(russian, "folder_window_requested"),
            "info",
            false,
            commands
        );
    }
    if (action_id == "browse_save_window") {
        const std::string commands =
            R"([{"command_type":"browse_window","payload":{"target":"showcase-save-path","status_target":"showcase-save-status","selected_status":")" +
            json_escape(tr(russian, "save_selected_status")) +
            R"(","mode":"file_save","title":")" +
            json_escape(tr(russian, "save_window_title")) +
            R"(","confirm_label":")" +
            json_escape(tr(russian, "save_confirm_label")) +
            R"(","default_name":"netstitch-showcase","default_extension":"txt","overwrite_policy":"prompt","can_create_directories":true,"filters":[{"name":")" +
            json_escape(tr(russian, "text_files_filter")) +
            R"(","extensions":["txt"]},{"name":")" +
            json_escape(tr(russian, "config_files_filter")) +
            R"(","extensions":["conf","json"]}]}}])";
        return ok_response(
            tr(russian, "save_window_requested"),
            "info",
            false,
            commands
        );
    }
    if (action_id == "start_showcase_background") {
        // Header action used as a toggle. The host tells us whether background
        // work is currently active.
        // Reading background_active prevents double-starting the same listener.
        if (extract_json_bool(request, "background_active")) {
            const std::string commands =
                R"([{"command_type":"stop_background","payload":{}},{"command_type":"log_event","payload":{"severity":"info","message":")" +
                json_escape(tr(russian, "listener_stopped_log")) + R"("}}])";
            return ok_response(
                tr(russian, "listener_stopped"),
                "info",
                true,
                commands
            );
        }
        const std::string commands =
            R"([{"command_type":"start_background","payload":{"subscriptions":["ui.controls","ui.tables","monitoring.*","filters.*"]}},{"command_type":"log_event","payload":{"severity":"success","message":")" +
            json_escape(tr(russian, "listener_started_log")) + R"("}}])";
        return ok_response(
            tr(russian, "listener_started"),
            "success",
            true,
            // Background subscriptions are explicit user-triggered work. The
            // module is loaded on startup, but it does not autostart.
            // Keep this list narrow in real modules: subscribe only to host
            // events you actually use.
            commands
        );
    }
    if (action_id == "stop_showcase_background") {
        return ok_response(
            tr(russian, "listener_stopped"),
            "info",
            true,
            R"([{"command_type":"stop_background","payload":{}}])"
        );
    }
    if (action_id == "show_notice") {
        // Standard dialogs are rendered by NetStitch, not by module-specific UI
        // code, which keeps desktop and browser shells aligned.
        // The module only provides a stable dialog id, severity, buttons and
        // message. It does not create windows or platform-specific UI.
        const std::string commands =
            R"([{"command_type":"show_dialog","payload":{"dialog_id":"ui-showcase-cpp-notice","buttons":"ok","severity":"info","message":")" +
            json_escape(tr(russian, "dialog_message")) + R"("}}])";
        return ok_response(
            tr(russian, "dialog_requested"),
            "info",
            false,
            commands
        );
    }
    return ok_response(
        tr(russian, "action_received_prefix") + " '" + action_id + "'",
        "info",
        false,
        R"([{"command_type":"log_event","payload":{"severity":"info","message":")" +
            json_escape(tr(russian, "action_received_log")) + R"("}}])"
    );
}

extern "C" NETSTITCH_EXPORT int32_t netstitch_integration_call(
    const uint8_t* request_ptr,
    uintptr_t request_len,
    NetStitchEventCallback event_callback,
    void* event_user_data,
    NetStitchAbiBuffer* out_response
) {
    if (!request_ptr || !out_response) return 1;

    // The host passes request bytes for the duration of the call. Copy them into
    // std::string before parsing. Request shape used by this sample:
    // {
    //   "action": "ui_action" | "background_event" | "...",
    //   "payload": { ... host context and UI values ... }
    // }
    std::string request(reinterpret_cast<const char*>(request_ptr), static_cast<std::size_t>(request_len));
    const std::string action = extract_json_string(request, "action");
    const std::string action_id = extract_json_string(request, "action_id");
    const bool russian = request_language_is_russian(request);
    std::string response;

    if (action == "ui_action") {
        response = ui_action_response(action_id, request, russian, event_callback, event_user_data);
    } else if (action == "background_event") {
        // Background events arrive only after start_background. Discovery and
        // manifest loading must not start module background work by themselves.
        const std::string commands =
            R"([{"command_type":"log_event","payload":{"severity":"info","message":")" +
            json_escape(tr(russian, "background_event_log")) + R"("}}])";
        response = ok_response(
            tr(russian, "background_event_received"),
            "info",
            false,
            commands
        );
    } else {
        response = ok_response(tr(russian, "unsupported_action"), "warning", false, "[]");
    }

    auto* bytes = new uint8_t[response.size()];
    std::memcpy(bytes, response.data(), response.size());
    // Ownership moves to the host. It will call netstitch_integration_free.
    // The allocation and free routine must match; here that means new[]/delete[].
    out_response->ptr = bytes;
    out_response->len = static_cast<uintptr_t>(response.size());
    return 0;
}

extern "C" NETSTITCH_EXPORT void netstitch_integration_free(uint8_t* ptr, uintptr_t) {
    // Free the buffer allocated in netstitch_integration_call.
    delete[] ptr;
}
