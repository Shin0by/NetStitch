#include <cstdint>
#include <cstring>
#include <string>

#if defined(_WIN32)
#define NETSTITCH_EXPORT __declspec(dllexport)
#else
#define NETSTITCH_EXPORT __attribute__((visibility("default")))
#endif

struct NetStitchAbiBuffer {
    uint8_t* ptr;
    uintptr_t len;
};

using NetStitchEventCallback = void (*)(const uint8_t*, uintptr_t, void*);

static std::string json_escape(const std::string& value) {
    std::string out;
    out.reserve(value.size());
    for (char ch : value) {
        if (ch == '\\' || ch == '"') {
            out.push_back('\\');
        }
        out.push_back(ch);
    }
    return out;
}

static std::string extract_json_string(const std::string& source, const std::string& key) {
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

static bool json_bool(const std::string& source, const std::string& key) {
    const std::string needle = "\"" + key + "\"";
    auto key_pos = source.find(needle);
    if (key_pos == std::string::npos) return false;
    auto colon_pos = source.find(':', key_pos + needle.size());
    if (colon_pos == std::string::npos) return false;
    auto value_pos = source.find_first_not_of(" \r\n\t", colon_pos + 1);
    return value_pos != std::string::npos && source.compare(value_pos, 4, "true") == 0;
}

static std::string ok_response(const std::string& message, const std::string& severity, bool refresh, const std::string& commands) {
    return "{\"ok\":true,\"result\":{\"message\":\"" + json_escape(message) +
        "\",\"severity\":\"" + json_escape(severity) +
        "\",\"refresh\":" + (refresh ? "true" : "false") +
        ",\"commands\":" + commands + "}}";
}

static std::string ui_action_response(const std::string& action_id, const std::string& request) {
    if (action_id == "say_hello") {
        return ok_response(
            "C++ demo: Hello world dialog opened",
            "success",
            false,
            R"([{"command_type":"show_dialog","payload":{"dialog_id":"hello-cpp-hello","buttons":"ok","severity":"success","message":"Hello world"}}])"
        );
    }
    if (action_id == "toggle_listener") {
        if (json_bool(request, "module_background_active")) {
            return ok_response(
                "C++ demo listener stopped",
                "info",
                true,
                R"([{"command_type":"stop_background","payload":{}},{"command_type":"log_event","payload":{"severity":"info","message":"C++ demo listener stopped"}}])"
            );
        }
        return ok_response(
            "C++ demo listener started",
            "success",
            true,
            R"([{"command_type":"start_background","payload":{"subscriptions":["monitoring.rows_added","monitoring.rows_changed","monitoring.selection_changed","monitoring.started","monitoring.stopped"]}},{"command_type":"log_event","payload":{"severity":"success","message":"C++ demo listener started"}}])"
        );
    }
    if (action_id == "show_last_rows") {
        return ok_response(
            "C++ demo: opened latest rows page",
            "info",
            true,
            R"([{"command_type":"set_module_page","payload":{"page":"last_rows"}}])"
        );
    }
    if (action_id == "about") {
        return ok_response(
            "C++ demo: about dialog opened",
            "info",
            false,
            R"([{"command_type":"show_dialog","payload":{"dialog_id":"hello-cpp-about","buttons":"ok","severity":"info","message":"Hello World C++ uses ui_schema entities, selected Monitoring row payloads, start_background/stop_background, log_event, show_dialog, and set_module_page."}},{"command_type":"log_event","payload":{"severity":"info","message":"C++ demo about dialog requested"}}])"
        );
    }
    return ok_response("C++ demo action '" + action_id + "' is not implemented", "warning", false, "[]");
}

extern "C" NETSTITCH_EXPORT int32_t netstitch_integration_call(
    const uint8_t* request_ptr,
    uintptr_t request_len,
    NetStitchEventCallback,
    void*,
    NetStitchAbiBuffer* out_response
) {
    if (!request_ptr || !out_response) return 1;
    std::string request(reinterpret_cast<const char*>(request_ptr), static_cast<std::size_t>(request_len));
    const std::string action = extract_json_string(request, "action");
    std::string response;
    if (action == "ui_action") {
        response = ui_action_response(extract_json_string(request, "action_id"), request);
    } else if (action == "background_event") {
        response = ok_response(
            "C++ demo listener received " + extract_json_string(request, "event_type"),
            "info",
            true,
            "[]"
        );
    } else {
        response = "{\"ok\":false,\"error\":\"unsupported action: " + json_escape(action) + "\"}";
    }
    auto* bytes = new uint8_t[response.size()];
    std::memcpy(bytes, response.data(), response.size());
    out_response->ptr = bytes;
    out_response->len = static_cast<uintptr_t>(response.size());
    return 0;
}

extern "C" NETSTITCH_EXPORT void netstitch_integration_free(uint8_t* ptr, uintptr_t) {
    delete[] ptr;
}
