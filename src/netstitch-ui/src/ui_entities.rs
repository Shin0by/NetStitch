#![allow(dead_code)]

pub mod entity {
    pub const APP_ROOT: &str = "app-root";
    pub const SHELL: &str = "shell";
    pub const APP_HEADER: &str = "app-header";
    pub const MODULE_HEADER: &str = "module-header";
    pub const MODULE_HEADER_ACTIONS: &str = "module-header-actions";
    pub const APP_CONTENT: &str = "app-content";
    pub const APP_FOOTER: &str = "app-footer";
    pub const APP_FOOTER_APPS_PANEL: &str = "app-footer-apps-panel";
    pub const APP_FOOTER_WATCHER_STATUS: &str = "app-footer-watcher-status";
    pub const APP_FOOTER_TOOL_STATUS: &str = "app-footer-tool-status";
    pub const APP_FOOTER_WEB_SERVER_STATUS: &str = "app-footer-web-server-status";
    pub const APP_FOOTER_NETWORK_STATUS: &str = "app-footer-network-status";
    pub const APP_FOOTER_MESSAGE_PANEL: &str = "app-footer-message-panel";
    pub const APP_FOOTER_LANGUAGE_PANEL: &str = "app-footer-language-panel";
    pub const APP_FOOTER_LANGUAGE_SELECT: &str = "app-footer-language-select";
    pub const TRACKED_APPS_PANEL: &str = "tracked-apps-panel";
    pub const TRACKED_APPS_HEADER: &str = "tracked-apps-header";
    pub const IGNORED_ADDRESSES_PANEL: &str = "ignored-addresses-panel";
    pub const OBSERVATIONS_PANEL: &str = "observations-panel";
    pub const INTEGRATION_PANEL: &str = "integration-panel";
    pub const INTEGRATION_MODULE_DIALOG: &str = "integration-module-dialog";
    pub const INTEGRATION_ROOT_DIALOG: &str = "integration-root-dialog";
    pub const PROFILE_EXPORT_DIALOG: &str = "profile-export-dialog";
    pub const CLOUD_SYNC_DIALOG: &str = "cloud-sync-dialog";
    pub const INFORMATION_DIALOG: &str = "information-dialog";
    pub const DOMAIN_CAPTURE_ADMIN_DIALOG: &str = "domain-capture-admin-dialog";
    pub const TABS: &str = "tabs";
    pub const TABS_BODY: &str = "tabs-body";
    pub const HELP_TEXT: &str = "help-text";
    pub const PROGRESS_BAR: &str = "progress-bar";
    pub const DELETE_TRACKED_APP_DIALOG: &str = "delete-tracked-app-dialog";
    pub const DELETE_IGNORED_ADDRESS_DIALOG: &str = "delete-ignored-address-dialog";
    pub const DELETE_OBSERVATION_DIALOG: &str = "delete-observation-dialog";
    pub const CLEAR_MONITORING_DIALOG: &str = "clear-monitoring-dialog";
    pub const TRACKED_APP_LIST: &str = "tracked-app-list";
    pub const TRACKED_APP_ITEM: &str = "tracked-app-item";
    pub const OBSERVATIONS_TABLE: &str = "observations-table";
    pub const OBSERVATION_ROW: &str = "observation-row";
    pub const PANEL: &str = "panel";
    pub const PANEL_HEADER: &str = "panel-header";
    pub const PANEL_FOOTER: &str = "panel-footer";
    pub const SUBPANEL: &str = "subpanel";
    pub const ROW: &str = "row";
    pub const PATH_FIELD: &str = "path-field";
    pub const MODAL: &str = "modal";
    pub const TOOLTIP: &str = "tooltip";
    pub const BUTTON: &str = "button";
    pub const SWITCH: &str = "switch";
    pub const STATUS_LABEL: &str = "status-label";
    pub const VALUE_LABEL: &str = "value-label";
    pub const TABLE_COLUMN: &str = "table-column";
    pub const FILE_PICKER: &str = "file-picker";
}

pub mod control {
    pub const EXE_PATH_INPUT: &str = "exe-path-input";
    pub const OBSERVATION_FILTER_SELECT: &str = "observation-filter-select";
    pub const OBSERVATION_APP_FILTER_INPUT: &str = "observation-app-filter-input";
    pub const OBSERVATION_PORT_FILTER_INPUT: &str = "observation-port-filter-input";
    pub const OBSERVATION_PROTOCOL_FILTER_SELECT: &str = "observation-protocol-filter-select";
    pub const OBSERVATION_SEARCH_INPUT: &str = "observation-search-input";
    pub const OBSERVATION_DOMAIN_FILTER_INPUT: &str = "observation-domain-filter-input";
    pub const INTEGRATION_ROOT_INPUT: &str = "integration-root-input";
    pub const PROFILE_EXPORT_PROFILE_INPUT: &str = "profile-export-profile-input";
    pub const PROFILE_EXPORT_PROFILE_SELECT: &str = "profile-export-profile-select";
    pub const PROFILE_EXPORT_GENERATED_NAME_INPUT: &str = "profile-export-generated-name-input";
    pub const CLOUD_APP_SEARCH_INPUT: &str = "cloud-app-search-input";
}

pub mod action {
    pub const ADD_EXE: &str = "add-exe";
    pub const CLEAR_EXE_PATH: &str = "clear-exe-path";
    pub const CLEAR_OBSERVATION_SEARCH: &str = "clear-observation-search";
    pub const CLEAR_OBSERVATION_DOMAIN_SEARCH: &str = "clear-observation-domain-search";
    pub const CLEAR_OBSERVATION_PORT_SEARCH: &str = "clear-observation-port-search";
    pub const TOGGLE_PUBLIC_IP_FILTER: &str = "toggle-public-ip-filter";
    pub const TOGGLE_ALL_APPS: &str = "toggle-all-apps";
    pub const TOGGLE_TRACKED_APP: &str = "toggle-tracked-app";
    pub const DELETE_TRACKED_APP: &str = "delete-tracked-app";
    pub const CANCEL_DELETE_TRACKED_APP: &str = "cancel-delete-tracked-app";
    pub const CONFIRM_DELETE_TRACKED_APP: &str = "confirm-delete-tracked-app";
    pub const TOGGLE_MONITORING: &str = "toggle-monitoring";
    pub const CONFIRM_FILTERED: &str = "confirm-filtered";
    pub const UNCONFIRM_FILTERED: &str = "unconfirm-filtered";
    pub const CLEAR_MONITORING: &str = "clear-monitoring";
    pub const OPEN_CLOUD_IMPORT: &str = "open-cloud-import";
    pub const OPEN_CLOUD_EXPORT: &str = "open-cloud-export";
    pub const START_CLOUD_GOOGLE_OAUTH: &str = "start-cloud-google-oauth";
    pub const SIGN_OUT_CLOUD: &str = "sign-out-cloud";
    pub const DOWNLOAD_CLOUD_APP_DATA: &str = "download-cloud-app-data";
    pub const TOGGLE_CLOUD_DOWNLOAD_ROW_SELECTION: &str = "toggle-cloud-download-row-selection";
    pub const ADD_CLOUD_DOWNLOAD_ROWS_TO_MONITORING: &str = "add-cloud-download-rows-to-monitoring";
    pub const EXPORT_CLOUD_DOWNLOAD_ROWS_CSV: &str = "export-cloud-download-rows-csv";
    pub const UPLOAD_CLOUD_DATA: &str = "upload-cloud-data";
    pub const SELECT_CLOUD_SCOPE_MINE: &str = "select-cloud-scope-mine";
    pub const IMPORT_CSV: &str = "import-csv";
    pub const EXPORT_CSV: &str = "export-csv";
    pub const OPEN_INFORMATION: &str = "open-information";
    pub const UPDATE_APPLICATION: &str = "update-application";
    pub const CLOSE_INFORMATION: &str = "close-information";
    pub const CLOSE_DOMAIN_CAPTURE_ADMIN: &str = "close-domain-capture-admin";
    pub const CANCEL_CLEAR_MONITORING: &str = "cancel-clear-monitoring";
    pub const CONFIRM_CLEAR_MONITORING_ALL: &str = "confirm-clear-monitoring-all";
    pub const CONFIRM_CLEAR_MONITORING_SELECTED: &str = "confirm-clear-monitoring-selected";
    pub const SELECT_PROFILE_EXPORT_MODE: &str = "select-profile-export-mode";
    pub const ANALYZE_PROFILE_EXPORT: &str = "analyze-profile-export";
    pub const OPEN_PROFILE_EXPORT_ADVANCED_WIZARD: &str = "open-profile-export-advanced-wizard";
    pub const CLOSE_PROFILE_EXPORT_ADVANCED_WIZARD: &str = "close-profile-export-advanced-wizard";
    pub const APPLY_PROFILE_EXPORT: &str = "apply-profile-export";
    pub const BACKUP_PROFILE_EXPORT: &str = "backup-profile-export";
    pub const REVERT_PROFILE_EXPORT: &str = "revert-profile-export";
    pub const CANCEL_PROFILE_EXPORT: &str = "cancel-profile-export";
    pub const TOGGLE_PROFILE_EXPORT_DANGEROUS: &str = "toggle-profile-export-dangerous";
    pub const CLEAR_PROFILE_EXPORT_PROFILE: &str = "clear-profile-export-profile";
    pub const BROWSE_PROFILE_EXPORT_PROFILE: &str = "browse-profile-export-profile";
    pub const TOGGLE_WEB_ACCESS_LOCALHOST: &str = "toggle-web-access-localhost";
    pub const COPY_STATUS_HISTORY: &str = "copy-status-history";
    pub const TOGGLE_OBSERVATION_CONFIRMED: &str = "toggle-observation-confirmed";
    pub const DELETE_OBSERVATION: &str = "delete-observation";
    pub const CANCEL_DELETE_OBSERVATION: &str = "cancel-delete-observation";
    pub const CONFIRM_DELETE_OBSERVATION: &str = "confirm-delete-observation";
    pub const IGNORE_ADDRESS: &str = "ignore-address";
    pub const DELETE_IGNORED_ADDRESS: &str = "delete-ignored-address";
    pub const CANCEL_DELETE_IGNORED_ADDRESS: &str = "cancel-delete-ignored-address";
    pub const CONFIRM_DELETE_IGNORED_ADDRESS: &str = "confirm-delete-ignored-address";
    pub const OPEN_INTEGRATION_MODULE: &str = "open-integration-module";
    pub const CLOSE_MODULE_OVERLAYS: &str = "close-module-overlays";
    pub const CLOSE_INTEGRATION_MODULE: &str = "close-integration-module";
    pub const SET_INTEGRATION_ROOT: &str = "set-integration-root";
    pub const DOWNLOAD_INTEGRATION_PROVIDER: &str = "download-integration-provider";
    pub const CANCEL_INTEGRATION_DOWNLOAD: &str = "cancel-integration-download";
    pub const BROWSE_INTEGRATION_ROOT: &str = "browse-integration-root";
    pub const CLEAR_INTEGRATION_ROOT: &str = "clear-integration-root";
    pub const CANCEL_INTEGRATION_ROOT: &str = "cancel-integration-root";
    pub const USE_INTEGRATION_ROOT: &str = "use-integration-root";
}

pub mod id {
    pub const MAIN: &str = "netstitch-ui-main";
    pub const SHELL: &str = "netstitch-ui-shell";
    pub const APP_HEADER: &str = "netstitch-ui-app-header";
    pub const MODULE_HEADER_CLOSE_BUTTON: &str = "netstitch-ui-module-header-close-button";
    pub const MODULE_HEADER_ACTIONS: &str = "netstitch-ui-module-header-actions";
    pub const APP_CONTENT: &str = "netstitch-ui-app-content";
    pub const APP_FOOTER: &str = "netstitch-ui-app-footer";
    pub const APP_FOOTER_WATCHER_STATUS: &str = "netstitch-ui-app-footer-watcher-status";
    pub const APP_FOOTER_TOOL_STATUS: &str = "netstitch-ui-app-footer-tool-status";
    pub const APP_FOOTER_WEB_SERVER_STATUS: &str = "netstitch-ui-app-footer-web-server-status";
    pub const APP_FOOTER_NETWORK_STATUS: &str = "netstitch-ui-app-footer-network-status";
    pub const APP_FOOTER_MESSAGE_PANEL: &str = "netstitch-ui-app-footer-message-panel";
    pub const APP_FOOTER_LANGUAGE_PANEL: &str = "netstitch-ui-app-footer-language-panel";
    pub const LANGUAGE_SELECT: &str = "netstitch-ui-language-select";
    pub const TRACKED_APPS_PANEL: &str = "netstitch-ui-tracked-apps-panel";
    pub const TRACKED_APPS_HEADER: &str = "netstitch-ui-tracked-apps-header";
    pub const IGNORED_ADDRESSES_PANEL: &str = "netstitch-ui-ignored-addresses-panel";
    pub const OBSERVATIONS_PANEL: &str = "netstitch-ui-observations-panel";
    pub const INTEGRATION_PANEL: &str = "netstitch-ui-integration-panel";
    pub const INTEGRATION_MODULE_DIALOG: &str = "netstitch-ui-integration-module-dialog";
    pub const INTEGRATION_ROOT_DIALOG: &str = "netstitch-ui-integration-root-dialog";
    pub const PROFILE_EXPORT_DIALOG: &str = "netstitch-ui-profile-export-dialog";
    pub const CLOUD_SYNC_DIALOG: &str = "netstitch-ui-cloud-sync-dialog";
    pub const INFORMATION_DIALOG: &str = "netstitch-ui-information-dialog";
    pub const DOMAIN_CAPTURE_ADMIN_DIALOG: &str = "netstitch-ui-domain-capture-admin-dialog";
    pub const PROFILE_EXPORT_ADVANCED_WIZARD_DIALOG: &str =
        "netstitch-ui-profile-export-advanced-wizard-dialog";
    pub const PROFILE_EXPORT_TABS: &str = "netstitch-ui-profile-export-tabs";
    pub const PROFILE_EXPORT_TABS_BODY: &str = "netstitch-ui-profile-export-tabs-body";
    pub const DELETE_TRACKED_APP_DIALOG: &str = "netstitch-ui-delete-tracked-app-dialog";
    pub const DELETE_IGNORED_ADDRESS_DIALOG: &str = "netstitch-ui-delete-ignored-address-dialog";
    pub const DELETE_OBSERVATION_DIALOG: &str = "netstitch-ui-delete-observation-dialog";
    pub const CLEAR_MONITORING_DIALOG: &str = "netstitch-ui-clear-monitoring-dialog";
    pub const TRACKED_APP_LIST: &str = "netstitch-ui-tracked-app-list";
    pub const OBSERVATIONS_TABLE: &str = "netstitch-ui-observations-table";
    pub const EXE_PATH_INPUT: &str = "netstitch-ui-exe-path-input";
    pub const CLEAR_EXE_PATH_BUTTON: &str = "netstitch-ui-clear-exe-path-button";
    pub const OBSERVATION_FILTER_SELECT: &str = "netstitch-ui-observation-filter-select";
    pub const OBSERVATION_APP_FILTER_INPUT: &str = "netstitch-ui-observation-app-filter-input";
    pub const OBSERVATION_PORT_FILTER_INPUT: &str = "netstitch-ui-observation-port-filter-input";
    pub const OBSERVATION_PROTOCOL_FILTER_SELECT: &str =
        "netstitch-ui-observation-protocol-filter-select";
    pub const OBSERVATION_SEARCH_INPUT: &str = "netstitch-ui-observation-search-input";
    pub const OBSERVATION_DOMAIN_FILTER_INPUT: &str =
        "netstitch-ui-observation-domain-filter-input";
    pub const CLOUD_APP_SEARCH_INPUT: &str = "netstitch-ui-cloud-app-search-input";
    pub const CLEAR_OBSERVATION_SEARCH_BUTTON: &str =
        "netstitch-ui-clear-observation-search-button";
    pub const CLEAR_OBSERVATION_DOMAIN_SEARCH_BUTTON: &str =
        "netstitch-ui-clear-observation-domain-search-button";
    pub const INTEGRATION_ROOT_INPUT: &str = "netstitch-ui-integration-root-input";
    pub const PROFILE_EXPORT_PROFILE_INPUT: &str = "netstitch-ui-profile-export-profile-input";
    pub const PROFILE_EXPORT_PROFILE_SELECT: &str = "netstitch-ui-profile-export-profile-select";
    pub const PROFILE_EXPORT_GENERATED_NAME_INPUT: &str =
        "netstitch-ui-profile-export-generated-name-input";
    pub const CLEAR_INTEGRATION_ROOT_BUTTON: &str = "netstitch-ui-clear-integration-root-button";
    pub const ADD_EXE_BUTTON: &str = "netstitch-ui-add-exe-button";
    pub const TOGGLE_ALL_APPS_BUTTON: &str = "netstitch-ui-toggle-all-apps-button";
    pub const TOGGLE_MONITORING_BUTTON: &str = "netstitch-ui-toggle-monitoring-button";
    pub const CONFIRM_FILTERED_BUTTON: &str = "netstitch-ui-confirm-filtered-button";
    pub const UNCONFIRM_FILTERED_BUTTON: &str = "netstitch-ui-unconfirm-filtered-button";
    pub const CLEAR_MONITORING_BUTTON: &str = "netstitch-ui-clear-monitoring-button";
    pub const OPEN_CLOUD_IMPORT_BUTTON: &str = "netstitch-ui-open-cloud-import-button";
    pub const OPEN_CLOUD_EXPORT_BUTTON: &str = "netstitch-ui-open-cloud-export-button";
    pub const START_CLOUD_GOOGLE_OAUTH_BUTTON: &str =
        "netstitch-ui-start-cloud-google-oauth-button";
    pub const SIGN_OUT_CLOUD_BUTTON: &str = "netstitch-ui-sign-out-cloud-button";
    pub const CLOUD_SCOPE_MINE_BUTTON: &str = "netstitch-ui-cloud-scope-mine-button";
    pub const CLOUD_DOWNLOAD_ADD_TO_MONITORING_BUTTON: &str =
        "netstitch-ui-cloud-download-add-to-monitoring-button";
    pub const CLOUD_DOWNLOAD_EXPORT_CSV_BUTTON: &str =
        "netstitch-ui-cloud-download-export-csv-button";
    pub const CLOUD_UPLOAD_BUTTON: &str = "netstitch-ui-cloud-upload-button";
    pub const IMPORT_CSV_BUTTON: &str = "netstitch-ui-import-csv-button";
    pub const EXPORT_CSV_BUTTON: &str = "netstitch-ui-export-csv-button";
    pub const OPEN_INFORMATION_BUTTON: &str = "netstitch-ui-open-information-button";
    pub const UPDATE_APPLICATION_BUTTON: &str = "netstitch-ui-update-application-button";
    pub const CANCEL_CLEAR_MONITORING_BUTTON: &str = "netstitch-ui-cancel-clear-monitoring-button";
    pub const CONFIRM_CLEAR_MONITORING_ALL_BUTTON: &str =
        "netstitch-ui-confirm-clear-monitoring-all-button";
    pub const CONFIRM_CLEAR_MONITORING_SELECTED_BUTTON: &str =
        "netstitch-ui-confirm-clear-monitoring-selected-button";
    pub const PROFILE_EXPORT_MODE_ATTACH_BUTTON: &str =
        "netstitch-ui-profile-export-mode-attach-button";
    pub const PROFILE_EXPORT_MODE_PATCH_BUTTON: &str =
        "netstitch-ui-profile-export-mode-patch-button";
    pub const PROFILE_EXPORT_MODE_MERGE_BUTTON: &str =
        "netstitch-ui-profile-export-mode-merge-button";
    pub const PROFILE_EXPORT_ANALYZE_BUTTON: &str = "netstitch-ui-profile-export-analyze-button";
    pub const PROFILE_EXPORT_ADVANCED_WIZARD_BUTTON: &str =
        "netstitch-ui-profile-export-advanced-wizard-button";
    pub const PROFILE_EXPORT_APPLY_BUTTON: &str = "netstitch-ui-profile-export-apply-button";
    pub const PROFILE_EXPORT_BACKUP_BUTTON: &str = "netstitch-ui-profile-export-backup-button";
    pub const PROFILE_EXPORT_REVERT_BUTTON: &str = "netstitch-ui-profile-export-revert-button";
    pub const PROFILE_EXPORT_CANCEL_BUTTON: &str = "netstitch-ui-profile-export-cancel-button";
    pub const CLEAR_PROFILE_EXPORT_PROFILE_BUTTON: &str =
        "netstitch-ui-clear-profile-export-profile-button";
    pub const BROWSE_PROFILE_EXPORT_PROFILE_BUTTON: &str =
        "netstitch-ui-browse-profile-export-profile-button";
    pub const WEB_ACCESS_LOCALHOST_BUTTON: &str = "netstitch-ui-web-access-localhost-button";
    pub const OPEN_INTEGRATION_MODULE_BUTTON: &str = "netstitch-ui-open-integration-module-button";
    pub const CLOSE_INTEGRATION_MODULE_BUTTON: &str =
        "netstitch-ui-close-integration-module-button";
    pub const SET_INTEGRATION_ROOT_BUTTON: &str = "netstitch-ui-set-integration-root-button";
    pub const DOWNLOAD_INTEGRATION_PROVIDER_BUTTON: &str =
        "netstitch-ui-download-integration-provider-button";
    pub const CANCEL_INTEGRATION_DOWNLOAD_BUTTON: &str =
        "netstitch-ui-cancel-integration-download-button";
    pub const BROWSE_INTEGRATION_ROOT_BUTTON: &str = "netstitch-ui-browse-integration-root-button";
    pub const CANCEL_INTEGRATION_ROOT_BUTTON: &str = "netstitch-ui-cancel-integration-root-button";
    pub const USE_INTEGRATION_ROOT_BUTTON: &str = "netstitch-ui-use-integration-root-button";
    pub const CANCEL_DELETE_TRACKED_APP_BUTTON: &str =
        "netstitch-ui-cancel-delete-tracked-app-button";
    pub const CONFIRM_DELETE_TRACKED_APP_BUTTON: &str =
        "netstitch-ui-confirm-delete-tracked-app-button";
    pub const CANCEL_DELETE_IGNORED_ADDRESS_BUTTON: &str =
        "netstitch-ui-cancel-delete-ignored-address-button";
    pub const CONFIRM_DELETE_IGNORED_ADDRESS_BUTTON: &str =
        "netstitch-ui-confirm-delete-ignored-address-button";
    pub const CANCEL_DELETE_OBSERVATION_BUTTON: &str =
        "netstitch-ui-cancel-delete-observation-button";
    pub const CONFIRM_DELETE_OBSERVATION_BUTTON: &str =
        "netstitch-ui-confirm-delete-observation-button";
}

pub fn tracked_app_item_id(app_id: u64) -> String {
    format!("netstitch-ui-tracked-app-{app_id}")
}

pub fn tracked_app_toggle_id(app_id: u64) -> String {
    format!("netstitch-ui-tracked-app-{app_id}-toggle")
}

pub fn tracked_app_delete_id(app_id: u64) -> String {
    format!("netstitch-ui-tracked-app-{app_id}-delete")
}

pub fn observation_row_id(observation_id: u64) -> String {
    format!("netstitch-ui-observation-{observation_id}")
}

pub fn observation_confirm_id(observation_id: u64) -> String {
    format!("netstitch-ui-observation-{observation_id}-confirm")
}

pub fn observation_delete_id(observation_id: u64) -> String {
    format!("netstitch-ui-observation-{observation_id}-delete")
}

#[cfg(test)]
mod tests {
    use super::{action, control, entity, id};
    use std::collections::BTreeSet;

    #[test]
    fn root_ui_ids_are_stable_and_unique() {
        let ids = [
            id::MAIN,
            id::SHELL,
            id::APP_HEADER,
            id::MODULE_HEADER_CLOSE_BUTTON,
            id::MODULE_HEADER_ACTIONS,
            id::APP_CONTENT,
            id::APP_FOOTER,
            id::APP_FOOTER_WATCHER_STATUS,
            id::APP_FOOTER_TOOL_STATUS,
            id::APP_FOOTER_WEB_SERVER_STATUS,
            id::APP_FOOTER_MESSAGE_PANEL,
            id::APP_FOOTER_LANGUAGE_PANEL,
            id::LANGUAGE_SELECT,
            id::TRACKED_APPS_PANEL,
            id::TRACKED_APPS_HEADER,
            id::IGNORED_ADDRESSES_PANEL,
            id::OBSERVATIONS_PANEL,
            id::INTEGRATION_PANEL,
            id::INTEGRATION_MODULE_DIALOG,
            id::INTEGRATION_ROOT_DIALOG,
            id::PROFILE_EXPORT_DIALOG,
            id::CLOUD_SYNC_DIALOG,
            id::INFORMATION_DIALOG,
            id::DOMAIN_CAPTURE_ADMIN_DIALOG,
            id::PROFILE_EXPORT_ADVANCED_WIZARD_DIALOG,
            id::PROFILE_EXPORT_TABS,
            id::PROFILE_EXPORT_TABS_BODY,
            id::PROFILE_EXPORT_PROFILE_SELECT,
            id::DELETE_TRACKED_APP_DIALOG,
            id::DELETE_IGNORED_ADDRESS_DIALOG,
            id::DELETE_OBSERVATION_DIALOG,
            id::CLEAR_MONITORING_DIALOG,
            id::CANCEL_DELETE_IGNORED_ADDRESS_BUTTON,
            id::CONFIRM_DELETE_IGNORED_ADDRESS_BUTTON,
            id::CANCEL_DELETE_OBSERVATION_BUTTON,
            id::CONFIRM_DELETE_OBSERVATION_BUTTON,
            id::CANCEL_CLEAR_MONITORING_BUTTON,
            id::CONFIRM_CLEAR_MONITORING_ALL_BUTTON,
            id::CONFIRM_CLEAR_MONITORING_SELECTED_BUTTON,
            id::DOWNLOAD_INTEGRATION_PROVIDER_BUTTON,
            id::CANCEL_INTEGRATION_DOWNLOAD_BUTTON,
            id::EXE_PATH_INPUT,
            id::CLEAR_EXE_PATH_BUTTON,
            id::OBSERVATION_FILTER_SELECT,
            id::OBSERVATION_APP_FILTER_INPUT,
            id::OBSERVATION_PORT_FILTER_INPUT,
            id::OBSERVATION_PROTOCOL_FILTER_SELECT,
            id::OBSERVATION_SEARCH_INPUT,
            id::CLOUD_APP_SEARCH_INPUT,
            id::CLEAR_OBSERVATION_SEARCH_BUTTON,
            id::CLEAR_INTEGRATION_ROOT_BUTTON,
            id::TOGGLE_ALL_APPS_BUTTON,
            id::TOGGLE_MONITORING_BUTTON,
            id::CONFIRM_FILTERED_BUTTON,
            id::UNCONFIRM_FILTERED_BUTTON,
            id::CLEAR_MONITORING_BUTTON,
            id::OPEN_CLOUD_IMPORT_BUTTON,
            id::OPEN_CLOUD_EXPORT_BUTTON,
            id::START_CLOUD_GOOGLE_OAUTH_BUTTON,
            id::SIGN_OUT_CLOUD_BUTTON,
            id::CLOUD_SCOPE_MINE_BUTTON,
            id::CLOUD_DOWNLOAD_ADD_TO_MONITORING_BUTTON,
            id::CLOUD_DOWNLOAD_EXPORT_CSV_BUTTON,
            id::CLOUD_UPLOAD_BUTTON,
            id::IMPORT_CSV_BUTTON,
            id::EXPORT_CSV_BUTTON,
            id::OPEN_INFORMATION_BUTTON,
            id::UPDATE_APPLICATION_BUTTON,
            id::WEB_ACCESS_LOCALHOST_BUTTON,
            id::CLOSE_INTEGRATION_MODULE_BUTTON,
        ];
        let unique = ids.iter().copied().collect::<BTreeSet<_>>();

        assert_eq!(unique.len(), ids.len());
        assert!(ids.iter().all(|value| value.starts_with("netstitch-ui-")));
    }

    #[test]
    fn dynamic_ui_ids_keep_parent_entity_key() {
        assert_eq!(
            super::tracked_app_item_id(42),
            "netstitch-ui-tracked-app-42"
        );
        assert_eq!(
            super::tracked_app_delete_id(42),
            "netstitch-ui-tracked-app-42-delete"
        );
        assert_eq!(
            super::observation_confirm_id(7),
            "netstitch-ui-observation-7-confirm"
        );
        assert_eq!(
            super::observation_delete_id(7),
            "netstitch-ui-observation-7-delete"
        );
    }

    #[test]
    fn ui_entities_cover_core_interactive_controls() {
        let controls = [
            control::EXE_PATH_INPUT,
            control::OBSERVATION_FILTER_SELECT,
            control::OBSERVATION_APP_FILTER_INPUT,
            control::OBSERVATION_PORT_FILTER_INPUT,
            control::OBSERVATION_PROTOCOL_FILTER_SELECT,
            control::OBSERVATION_SEARCH_INPUT,
            control::CLOUD_APP_SEARCH_INPUT,
            control::INTEGRATION_ROOT_INPUT,
            control::PROFILE_EXPORT_PROFILE_INPUT,
            control::PROFILE_EXPORT_PROFILE_SELECT,
            control::PROFILE_EXPORT_GENERATED_NAME_INPUT,
        ];
        let actions = [
            action::ADD_EXE,
            action::CLEAR_EXE_PATH,
            action::CLEAR_OBSERVATION_SEARCH,
            action::CLEAR_OBSERVATION_PORT_SEARCH,
            action::TOGGLE_ALL_APPS,
            action::DELETE_TRACKED_APP,
            action::CANCEL_DELETE_TRACKED_APP,
            action::CONFIRM_DELETE_TRACKED_APP,
            action::TOGGLE_MONITORING,
            action::CONFIRM_FILTERED,
            action::UNCONFIRM_FILTERED,
            action::CLEAR_MONITORING,
            action::OPEN_CLOUD_IMPORT,
            action::OPEN_CLOUD_EXPORT,
            action::START_CLOUD_GOOGLE_OAUTH,
            action::SIGN_OUT_CLOUD,
            action::DOWNLOAD_CLOUD_APP_DATA,
            action::TOGGLE_CLOUD_DOWNLOAD_ROW_SELECTION,
            action::ADD_CLOUD_DOWNLOAD_ROWS_TO_MONITORING,
            action::EXPORT_CLOUD_DOWNLOAD_ROWS_CSV,
            action::UPLOAD_CLOUD_DATA,
            action::SELECT_CLOUD_SCOPE_MINE,
            action::IMPORT_CSV,
            action::EXPORT_CSV,
            action::OPEN_INFORMATION,
            action::UPDATE_APPLICATION,
            action::CLOSE_INFORMATION,
            action::CLOSE_DOMAIN_CAPTURE_ADMIN,
            action::CANCEL_CLEAR_MONITORING,
            action::CONFIRM_CLEAR_MONITORING_ALL,
            action::CONFIRM_CLEAR_MONITORING_SELECTED,
            action::SELECT_PROFILE_EXPORT_MODE,
            action::ANALYZE_PROFILE_EXPORT,
            action::OPEN_PROFILE_EXPORT_ADVANCED_WIZARD,
            action::CLOSE_PROFILE_EXPORT_ADVANCED_WIZARD,
            action::APPLY_PROFILE_EXPORT,
            action::REVERT_PROFILE_EXPORT,
            action::CANCEL_PROFILE_EXPORT,
            action::TOGGLE_PROFILE_EXPORT_DANGEROUS,
            action::CLEAR_PROFILE_EXPORT_PROFILE,
            action::BROWSE_PROFILE_EXPORT_PROFILE,
            action::TOGGLE_WEB_ACCESS_LOCALHOST,
            action::COPY_STATUS_HISTORY,
            action::TOGGLE_OBSERVATION_CONFIRMED,
            action::DELETE_OBSERVATION,
            action::CANCEL_DELETE_OBSERVATION,
            action::CONFIRM_DELETE_OBSERVATION,
            action::IGNORE_ADDRESS,
            action::DELETE_IGNORED_ADDRESS,
            action::CANCEL_DELETE_IGNORED_ADDRESS,
            action::CONFIRM_DELETE_IGNORED_ADDRESS,
            action::OPEN_INTEGRATION_MODULE,
            action::CLOSE_MODULE_OVERLAYS,
            action::CLOSE_INTEGRATION_MODULE,
            action::DOWNLOAD_INTEGRATION_PROVIDER,
            action::CANCEL_INTEGRATION_DOWNLOAD,
            action::CLEAR_INTEGRATION_ROOT,
            action::USE_INTEGRATION_ROOT,
        ];
        let entities = [
            entity::TRACKED_APPS_PANEL,
            entity::TRACKED_APPS_HEADER,
            entity::APP_HEADER,
            entity::MODULE_HEADER,
            entity::MODULE_HEADER_ACTIONS,
            entity::APP_CONTENT,
            entity::APP_FOOTER,
            entity::APP_FOOTER_APPS_PANEL,
            entity::APP_FOOTER_WATCHER_STATUS,
            entity::APP_FOOTER_TOOL_STATUS,
            entity::APP_FOOTER_WEB_SERVER_STATUS,
            entity::APP_FOOTER_MESSAGE_PANEL,
            entity::APP_FOOTER_LANGUAGE_PANEL,
            entity::APP_FOOTER_LANGUAGE_SELECT,
            entity::IGNORED_ADDRESSES_PANEL,
            entity::OBSERVATIONS_PANEL,
            entity::INTEGRATION_MODULE_DIALOG,
            entity::INTEGRATION_ROOT_DIALOG,
            entity::PROFILE_EXPORT_DIALOG,
            entity::CLOUD_SYNC_DIALOG,
            entity::INFORMATION_DIALOG,
            entity::DOMAIN_CAPTURE_ADMIN_DIALOG,
            entity::TABS,
            entity::TABS_BODY,
            entity::HELP_TEXT,
            entity::PROGRESS_BAR,
            entity::DELETE_TRACKED_APP_DIALOG,
            entity::DELETE_IGNORED_ADDRESS_DIALOG,
            entity::DELETE_OBSERVATION_DIALOG,
            entity::CLEAR_MONITORING_DIALOG,
        ];

        assert!(controls.iter().all(|value| !value.is_empty()));
        assert!(actions.iter().all(|value| !value.is_empty()));
        assert!(entities.iter().all(|value| !value.is_empty()));
    }
}
