pub const GLOBAL_STYLE: &str = r#"
:root {
  color-scheme: dark;

  --color-workbench-bg: #181818;
  --color-window-chrome: #181818;
  --color-panel-bg: #252526;
  --color-panel-header-bg: #2d2d2d;
  --color-panel-subtle-bg: #202020;
  --color-panel-hover-bg: #2a2d2e;
  --color-app-list-bg: #1e1e1e;
  --color-app-row-bg: #2b2b2b;
  --color-app-row-enabled-bg: #2b3338;
  --color-app-row-enabled-hover-bg: #344048;
  --color-app-skeleton-bg: #242424;
  --color-app-skeleton-pulse-bg: #303030;
  --color-control-bg: #1f1f1f;
  --color-control-hover-bg: #2a2d2e;
  --color-control-active-bg: #313131;
  --color-control-border: #3c3c3c;
  --color-control-border-strong: #4a4a4a;
  --color-tooltip-bg: #303030;
  --color-tooltip-border: #5a5a5a;
  --color-indicator-idle: #3a3a3a;
  --color-scrollbar-track: #1e1e1e;
  --color-scrollbar-thumb: #424242;
  --color-scrollbar-thumb-hover: #525252;
  --color-text: #cccccc;
  --color-text-strong: #f0f0f0;
  --color-text-muted: #9d9d9d;
  --color-text-subtle: #858585;
  --color-switch-knob-on: #eeeeee;
  --color-accent: #007acc;
  --color-accent-hover: #0e86d4;
  --color-accent-muted-bg: #04395e;
  --color-accent-border: #3794ff;
  --color-progress-accent: #007acc;
  --color-progress-success: #6a9955;
  --color-progress-warning: #cca700;
  --color-progress-danger: #f48771;
  --color-progress-rust: #d97922;
  --color-progress-muted: #5a5a5a;
  --color-success: #89d185;
  --color-success-bg: #123a2f;
  --color-danger: #f48771;
  --color-danger-bg: #4b1f24;
  --color-warning: #cca700;
  --color-warning-bg: #5a3d00;
  --color-monitoring-active: #c7771a;
  --color-monitoring-active-hover: #dd8a26;
  --color-module-activity-pulse: rgba(255, 255, 255, 0.34);
  --color-transparent: transparent;
  --color-white: #ffffff;

  --radius-window: 0;
  --radius-panel: 3px;
  --radius-control: 3px;
  --radius-pill: 999px;

  --size-panel-gap: 8px;
  --size-panel-stack-gap: 8px;
  --size-header-height: 56px;
  --size-footer-height: 34px;
  --size-tracked-app-height: 52px;
  --size-tracked-app-visible-rows: 4;
  --size-tracked-app-list-height: calc((var(--size-tracked-app-height) * var(--size-tracked-app-visible-rows)) + (5px * (var(--size-tracked-app-visible-rows) - 1)) + 10px + 2px);
  --size-tracked-apps-path-bottom-pad: 0px;
  --size-modules-panel-height: 78px;
  --size-module-button: 36px;
  --size-module-button-pulse-gutter: 3px;
  --size-module-button-slot: calc(var(--size-module-button) + (var(--size-module-button-pulse-gutter) * 2));
  --size-ignored-address-row-height: 30px;
  --size-ignored-address-row-gap: 5px;
  --size-ignored-addresses-visible-rows: 5;
  --size-ignored-addresses-list-height: calc((var(--size-ignored-address-row-height) * var(--size-ignored-addresses-visible-rows)) + (var(--size-ignored-address-row-gap) * (var(--size-ignored-addresses-visible-rows) - 1)) + 10px + 2px);
  --size-ignored-address-ip-column: 230px;
  --size-app-icon-slot: 40px;
  --size-app-icon-image: 32px;
  --size-switch-width: 46px;
  --size-switch-height: 22px;
  --size-switch-knob: 16px;
  --size-close-button: 20px;
  --size-list-row-inline-pad: 3px;
  --size-list-row-action-gap: 2px;
  --size-compact-control: 30px;
  --size-panel-footer-height: 43px;
  --size-header-action-button: 48px;
  --size-control-label-offset: 0px;
  --size-scrollbar: 8px;
  --size-scrollbar-content-gutter: 1px;
  --size-observations-panel-height: clamp(224px, 39vh, 420px);
  --size-tracked-apps-column-max: 640px;
  --size-tracked-apps-column-min: calc(var(--size-tracked-apps-column-max) * 0.4);
  --size-integration-status-left-max: 520px;
  --z-tooltip: 10000;
}

* { box-sizing: border-box; }

* {
  scrollbar-color: var(--color-scrollbar-thumb) var(--color-scrollbar-track);
  scrollbar-width: thin;
}

*::-webkit-scrollbar {
  width: var(--size-scrollbar);
  height: var(--size-scrollbar);
}

*::-webkit-scrollbar-track {
  background: var(--color-scrollbar-track);
}

*::-webkit-scrollbar-thumb {
  border-radius: var(--radius-control);
  background: var(--color-scrollbar-thumb);
}

*::-webkit-scrollbar-thumb:hover {
  background: var(--color-scrollbar-thumb-hover);
}

html,
body {
  width: 100%;
  height: 100%;
  margin: 0;
  background: var(--color-workbench-bg);
  color: var(--color-text);
  font-family: "Segoe UI", "Inter", system-ui, sans-serif;
}

body {
  overflow: hidden;
}

#main,
#netstitch-ui-main,
main[data-ui-entity="app-root"] {
  display: block;
  width: 100%;
  height: 100vh;
  height: 100dvh;
  min-height: 0;
  margin: 0;
  padding: 0;
  overflow: hidden;
  background: var(--color-window-chrome);
}

.shell {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) var(--size-footer-height);
  width: 100%;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  background: var(--color-window-chrome);
}

.shell--controls-disabled [data-ui-action],
.shell--controls-disabled button,
.shell--controls-disabled select,
.shell--controls-disabled textarea,
.shell--controls-disabled input:not([readonly]),
.shell--controls-disabled [role="button"] {
  pointer-events: none;
  cursor: not-allowed;
  opacity: 0.55;
}

.shell--cloud-overlay-active .shell__content {
  pointer-events: none;
  opacity: 0.42;
  filter: saturate(0.7);
}

.shell--cloud-overlay-active .app-chrome--header button:not(#netstitch-ui-confirm-filtered-button):not(#netstitch-ui-unconfirm-filtered-button):not(#netstitch-ui-clear-monitoring-button):not(#netstitch-ui-open-cloud-import-button):not(#netstitch-ui-open-cloud-export-button):not([data-ui-action="clear-observation-search"]):not([data-ui-action="clear-observation-port-search"]),
.shell--cloud-overlay-active .app-chrome--header .header-switch-stack,
.shell--cloud-overlay-active .app-chrome--header .header-action-separator {
  pointer-events: none;
  cursor: not-allowed;
  opacity: 0.45;
  filter: saturate(0.7);
}

.shell--cloud-overlay-active #netstitch-ui-open-cloud-import-button,
.shell--cloud-overlay-active #netstitch-ui-open-cloud-export-button,
.shell--cloud-overlay-active #netstitch-ui-confirm-filtered-button,
.shell--cloud-overlay-active #netstitch-ui-unconfirm-filtered-button,
.shell--cloud-overlay-active #netstitch-ui-clear-monitoring-button,
.shell--cloud-overlay-active [data-ui-action="clear-observation-search"],
.shell--cloud-overlay-active [data-ui-action="clear-observation-port-search"],
.shell--cloud-overlay-active .app-chrome--header .header-filter-block {
  pointer-events: auto;
  cursor: pointer;
  opacity: 1;
  filter: none;
}

.app-chrome {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 9px;
  width: 100%;
  background: var(--color-panel-header-bg);
  color: var(--color-text);
  border-color: var(--color-control-border);
  flex: 0 0 auto;
  overflow: hidden;
}

.app-chrome--header {
  min-height: var(--size-header-height);
  height: auto;
  align-items: flex-end;
  justify-content: flex-start;
  padding: 4px var(--size-panel-gap);
  border-bottom: 1px solid var(--color-control-border);
  z-index: 2;
}

.app-chrome--footer {
  height: var(--size-footer-height);
  justify-content: flex-start;
  padding: 0 clamp(12px, 2vw, 22px);
  border-top: 1px solid var(--color-control-border);
  font-size: 12px;
  color: var(--color-text-muted);
  overflow: visible;
  z-index: 70;
}

.app-footer-panel {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  width: max-content;
  max-width: 100%;
  min-height: 22px;
  padding: 3px 0;
  border: 0;
  border-radius: 0;
  background: var(--color-transparent);
  color: inherit;
  white-space: nowrap;
}

.app-footer-panel--static {
  cursor: default;
  user-select: none;
}

.app-footer-panel + .app-footer-panel {
  margin-left: 6px;
  padding-left: 12px;
  border-left: 1px solid var(--color-control-border);
}

.footer-watcher-status,
.footer-tool-status,
.footer-web-server-status,
.footer-network-status {
  position: relative;
  overflow: visible;
  cursor: help;
}

.footer-web-server-status {
  cursor: pointer;
  user-select: none;
}

.footer-watcher-status__label,
.footer-tool-status__label,
.footer-web-server-status__label,
.footer-network-status__label {
  line-height: 16px;
}

.footer-watcher-status__indicator {
  width: 14px;
  height: 14px;
  border-radius: var(--radius-pill);
  border: 1px solid var(--color-control-border);
  flex: 0 0 14px;
  transform: translateY(0);
}

.footer-watcher-status__indicator--connected {
  background: var(--color-success);
}

.footer-watcher-status__indicator--disconnected {
  background: var(--color-danger);
}

.footer-watcher-status__indicator--inactive {
  background: var(--color-indicator-idle);
}

.app-footer-actions {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  margin-left: auto;
  white-space: nowrap;
}

.app-footer-message-panel {
  flex: 1 1 auto;
  justify-content: flex-start;
  min-width: 0;
  overflow: visible;
  position: relative;
}

.footer-message-label {
  color: var(--color-text);
  flex: 0 0 auto;
  line-height: 18px;
}

.footer-message-text {
  position: relative;
  flex: 1 1 auto;
  min-width: 0;
  max-width: 100%;
  padding: 0;
  line-height: 18px;
  text-align: left;
}

.footer-message-text--success {
  color: var(--color-success);
}

.footer-message-text--warning {
  color: var(--color-warning);
}

.footer-message-text--error {
  color: var(--color-danger);
}

.progress-bar--compact .progress-bar__track {
  height: 6px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-control-bg);
}

.progress-bar--compact .progress-bar__segment,
.progress-bar--compact .progress-bar__remaining {
  height: 100%;
  border-radius: 0;
}

.button[data-tooltip] {
  position: relative;
}

.button--copy-status {
  width: 20px;
  height: 20px;
  min-height: 20px;
  padding: 0;
  flex: 0 0 20px;
}

.app-footer-language-panel {
  flex: 0 0 auto;
  margin-left: 6px;
  margin-inline-start: 6px;
  overflow: visible;
}

.footer-language-label {
  color: var(--color-text);
  line-height: 18px;
}

.language-select-shell {
  position: relative;
  display: inline-flex;
  overflow: visible;
}

.language-select-control {
  position: relative;
  display: inline-grid;
  grid-template-columns: minmax(0, 1fr);
  align-items: center;
  justify-content: center;
  width: max-content;
  min-width: 126px;
  height: 24px;
  margin: 0;
  padding: 2px 8px;
  border-radius: var(--radius-control);
  appearance: none;
  font-size: 12px;
  font: inherit;
  line-height: 18px;
  cursor: pointer;
}

.language-select-control__label {
  min-width: 0;
  overflow: hidden;
  color: var(--color-text);
  text-align: center;
  text-overflow: ellipsis;
  white-space: nowrap;
  pointer-events: none;
}

.language-select-menu {
  position: absolute;
  right: 0;
  bottom: 100%;
  display: grid;
  width: max-content;
  min-width: 126px;
  max-width: min(240px, calc(100vw - 24px));
  max-height: 220px;
  overflow: auto;
  padding: 3px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control) var(--radius-control) 0 0;
  background: var(--color-control-bg);
  z-index: var(--z-tooltip);
}

.language-select-option {
  min-height: 24px;
  padding: 3px 8px;
  border: 0;
  border-top: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-transparent);
  color: var(--color-text);
  font-size: 12px;
  font: inherit;
  line-height: 18px;
  cursor: pointer;
  text-align: center;
  white-space: nowrap;
}

.language-select-option:first-child {
  border-top: 0;
}

.language-select-option:hover,
.language-select-option:focus,
.language-select-option--selected {
  background: var(--color-app-row-enabled-bg);
}

.app-chrome h1 {
  margin: 0;
  font-size: 24px;
  line-height: 28px;
  letter-spacing: 0.02em;
  color: var(--color-text-strong);
}

.app-chrome p {
  margin: 3px 0 0;
  max-width: 660px;
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 16px;
}

.app-chrome__status {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.shell__content {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  overflow: hidden;
  padding: 8px;
  background: var(--color-workbench-bg);
}

.hero {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  padding: 18px 20px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-panel);
  background: var(--color-panel-bg);
}

.hero h1 { margin: 0; font-size: 28px; letter-spacing: 0.02em; }
.hero p { margin: 6px 0 0; color: var(--color-text-muted); max-width: 720px; }

.hero-labels {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  width: 100%;
  gap: 5px;
  justify-content: space-between;
  align-content: flex-end;
  min-height: 0;
}

.hero-labels--hidden {
  display: none;
}

.module-header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  min-width: 0;
}

.module-header-title {
  min-width: 0;
  overflow: hidden;
  color: var(--color-text-strong);
  font-size: 17px;
  font-weight: 700;
  line-height: 20px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.module-header-controls,
.module-header-actions,
.module-header-standard-actions {
  display: flex;
  align-items: center;
  gap: 5px;
  min-width: 0;
}

.module-header-controls {
  flex: 0 0 auto;
  margin-left: auto;
}

.module-header-standard-actions {
  flex: 0 0 auto;
}

.button__icon--module-stop,
.button__icon--module-close,
.button__icon--module-action {
  width: 28px;
  height: 28px;
}

.header-monitor-controls {
  display: flex;
  flex-wrap: nowrap;
  align-items: flex-end;
  gap: 5px;
  flex: 1 1 auto;
  min-width: 0;
}

.header-filter-block {
  display: inline-flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 2px;
  height: var(--size-header-action-button);
  min-width: 0;
}

.header-filter-block__label {
  min-height: 16px;
  padding: 0 var(--size-control-label-offset);
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 600;
  line-height: 16px;
  white-space: nowrap;
}

.header-filter-select,
.header-filter-field,
.header-search-field,
.header-action-button {
  min-height: var(--size-compact-control);
}

.header-filter-field-shell {
  width: 100%;
  min-width: 0;
}

.header-filter-select,
.header-search-field {
  height: var(--size-compact-control);
  min-height: var(--size-compact-control);
  padding: 3px 5px;
  font-size: 12px;
  line-height: 16px;
}

.header-filter-select {
  padding-right: 22px;
}

.header-monitor-controls .header-filter-select--state,
.header-monitor-controls .header-filter-block--state {
  width: min(94px, 9vw);
  min-width: 74px;
  max-width: 94px;
}

.header-monitor-controls .header-filter-select--app,
.header-monitor-controls .header-filter-block--app {
  width: min(170px, 16vw);
  min-width: 132px;
  max-width: 170px;
}

.header-monitor-controls .header-filter-select--port,
.header-monitor-controls .header-filter-block--port {
  width: 66px;
  min-width: 66px;
  max-width: 66px;
}

.header-monitor-controls .header-filter-select--protocol,
.header-monitor-controls .header-filter-block--protocol {
  width: min(68px, 7vw);
  min-width: 64px;
  max-width: 68px;
}

.header-monitor-controls .header-filter-select--visibility,
.header-monitor-controls .header-filter-block--ip,
.header-monitor-controls .header-filter-block--domain {
  width: min(126px, 10vw);
  min-width: 102px;
  max-width: 126px;
}

.header-monitor-controls .header-search-field {
  width: min(126px, 10vw);
  min-width: 102px;
  max-width: 126px;
}

.header-monitor-controls .header-search-field--domain {
  width: min(126px, 10vw);
  min-width: 102px;
  max-width: 126px;
}

.header-port-field-shell {
  width: 66px;
  min-width: 66px;
  max-width: 66px;
}

.header-monitor-controls .header-search-field--port {
  width: 100%;
  min-width: 0;
  max-width: none;
  padding: 3px 20px 3px 5px;
}

.header-action-button {
  white-space: nowrap;
}

.button--icon.header-action-button {
  width: var(--size-header-action-button);
  height: var(--size-header-action-button);
  min-width: var(--size-header-action-button);
  min-height: var(--size-header-action-button);
  max-width: var(--size-header-action-button);
  max-height: var(--size-header-action-button);
}

.header-switch-row {
  display: inline-flex;
  align-items: flex-end;
  gap: 4px;
  min-height: var(--size-header-action-button);
  padding: 0;
  border: 0;
  background: var(--color-transparent);
  color: var(--color-text);
  align-self: flex-end;
}

.header-switch-stack {
  display: flex;
  flex: 0 0 auto;
  flex-direction: column;
  align-items: flex-end;
  justify-content: flex-end;
  gap: 3px;
  margin-left: auto;
  align-self: flex-end;
}

.header-switch-stack .header-switch-row {
  min-height: var(--size-switch-height);
}

.header-switch-row__label {
  max-width: 132px;
  overflow: hidden;
  color: var(--color-text);
  font-size: 12px;
  font-weight: 600;
  line-height: var(--size-switch-height);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.header-switch-row--filter .header-switch-row__label {
  max-width: none;
}

.header-label {
  display: inline-flex;
  align-items: center;
  min-height: 24px;
  padding: 0;
  color: var(--color-text);
  font-size: 13px;
  font-weight: 600;
  line-height: 18px;
  white-space: nowrap;
}

.header-label--accent {
  color: var(--color-accent-border);
}

.header-label--success {
  color: var(--color-success);
}

.header-label--danger {
  color: var(--color-danger);
}

.workspace {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  grid-template-rows: auto minmax(0, 1fr);
  gap: var(--size-panel-gap);
  align-items: start;
  flex: 1 1 auto;
  width: 100%;
  height: 100%;
  min-height: 0;
  min-width: 0;
  max-width: 100%;
}

.column {
  display: flex;
  flex-direction: column;
  gap: var(--size-panel-gap);
  min-width: 0;
  max-width: 100%;
}

.workspace > .column {
  min-width: 0;
  max-width: 100%;
}

.workspace > .column:first-child {
  grid-column: 1;
  grid-row: 1;
  align-self: stretch;
  height: 100%;
  min-height: 0;
}

.workspace > .column:nth-child(2) {
  grid-column: 2;
  grid-row: 1;
  align-self: stretch;
  height: 100%;
  gap: var(--size-panel-stack-gap);
  min-height: 0;
}

.workspace > .observations-card {
  grid-column: 1 / -1;
  grid-row: 2;
  align-self: stretch;
  height: 100%;
  min-height: 0;
}

.card {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 0;
  max-width: 100%;
  padding: 5px;
  border-radius: var(--radius-panel);
  border: 1px solid var(--color-control-border);
  background: var(--color-panel-bg);
}

.tracked-apps-card {
  border-radius: var(--radius-control);
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  height: 100%;
  min-height: 0;
}

.card__header {
  display: flex;
  justify-content: space-between;
  gap: 5px;
  align-items: center;
  min-height: 18px;
  margin: -5px -5px 0;
  padding: 6px 8px;
  border: 0;
  border-bottom: 1px solid var(--color-control-border);
  border-radius: var(--radius-control) var(--radius-control) 0 0;
  background: var(--color-panel-header-bg);
}

.modal--panel .modal__header,
.modal--compact .modal__header {
  display: flex;
  justify-content: space-between;
  gap: 5px;
  align-items: center;
  min-height: 30px;
  margin: -5px -5px 0;
  padding: 5px 8px;
  border: 0;
  border-bottom: 1px solid var(--color-control-border);
  border-radius: var(--radius-control) var(--radius-control) 0 0;
  background: var(--color-panel-header-bg);
}

.tracked-apps-header {
  border-radius: var(--radius-control) var(--radius-control) 0 0;
}

.card h2,
.modal--panel .modal__header h2,
.modal--compact .modal__header h2 {
  margin: 0;
  color: var(--color-text-strong);
  font-size: 17px;
  line-height: 20px;
}

.panel-header-meta {
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 700;
  line-height: 18px;
  white-space: nowrap;
}

.panel-header-meta--success {
  color: var(--color-success);
}

.panel-header-meta--danger {
  color: var(--color-danger);
}

.monitoring-header-meta {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  min-width: 0;
}

.monitoring-header-meta .panel-header-meta {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.monitoring-public-toggle {
  flex: 0 0 auto;
  min-height: var(--size-switch-height);
  align-items: flex-end;
}

.tracked-apps-header-meta {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  min-width: 0;
}

.tracked-apps-card .stack {
  display: grid;
  gap: 5px;
}
.tracked-apps-card > .stack {
  grid-template-rows: auto minmax(0, 1fr) auto;
  min-height: 0;
}
.card h2, .card h3 { margin: 0; color: var(--color-text-strong); }

.title-with-help {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 4px;
}

.help-icon {
  display: inline-grid;
  place-items: center;
  width: 18px;
  height: 18px;
  border-radius: var(--radius-pill);
  min-width: 18px;
  min-height: 18px;
  max-width: 18px;
  max-height: 18px;
  flex: 0 0 18px;
  aspect-ratio: 1 / 1;
  border: 1px solid var(--color-control-border);
  background: var(--color-control-bg);
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 800;
  line-height: 16px;
  position: relative;
  top: 2px;
  overflow: hidden;
  cursor: help;
}

.help-icon__image {
  width: 18px;
  height: 18px;
  display: block;
  filter: none;
  pointer-events: none;
}

.netstitch-floating-tooltip {
  position: fixed;
  top: 0;
  left: 0;
  width: max-content;
  max-width: none;
  margin: 0;
  padding: 6px 8px;
  border: 1px solid var(--color-tooltip-border);
  border-radius: var(--radius-control);
  background: var(--color-tooltip-bg);
  color: var(--color-text);
  font-size: 12px;
  font-weight: 400;
  line-height: 16px;
  text-align: left;
  white-space: normal;
  opacity: 0;
  visibility: hidden;
  pointer-events: none;
  z-index: var(--z-tooltip);
}

.netstitch-floating-tooltip--visible {
  opacity: 1;
  visibility: visible;
}

.netstitch-floating-tooltip--pre {
  white-space: pre;
}

button,
a,
[role="button"],
[data-ui-action] {
  cursor: pointer;
}

.muted { color: var(--color-text-muted); }
.stack { display: flex; flex-direction: column; gap: 12px; }
.tracked-apps-toolbar-spacer {
  min-height: 0;
}

.controls {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.input-box,
.field,
.select,
.input {
  border: 1px solid var(--color-control-border);
  background: var(--color-control-bg);
  color: var(--color-text);
}

.field,
.select,
.input {
  width: 100%;
  border-radius: var(--radius-control);
  min-height: var(--size-compact-control);
  padding: 6px 10px;
  outline: none;
}

.select {
  appearance: none;
  -webkit-appearance: none;
  padding-right: 24px;
  background-color: var(--color-control-bg);
  background-image:
    linear-gradient(45deg, transparent 50%, var(--color-text-muted) 50%),
    linear-gradient(135deg, var(--color-text-muted) 50%, transparent 50%);
  background-position:
    calc(100% - 13px) 50%,
    calc(100% - 8px) 50%;
  background-repeat: no-repeat;
  background-size: 5px 5px, 5px 5px;
  color: var(--color-text);
}

.select option {
  background: var(--color-control-bg);
  color: var(--color-text);
}

.field:focus,
.select:focus,
.input:focus {
  border-color: var(--color-accent-border);
}

.input--apply-pulse {
  animation: input-apply-pulse 180ms ease-out 1;
}

@keyframes input-apply-pulse {
  0% {
    border-color: var(--color-accent-border);
    background: var(--color-control-bg);
  }
  45% {
    border-color: var(--color-accent-border);
    background: var(--color-accent-muted-bg);
  }
  100% {
    border-color: var(--color-control-border);
    background: var(--color-control-bg);
  }
}

.tracked-apps-card .input {
  height: var(--size-compact-control);
  border-radius: var(--radius-control);
  padding: 4px 5px;
}

.field::placeholder,
.input::placeholder {
  color: var(--color-text-subtle);
}

.exe-path-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) var(--size-compact-control);
  gap: 5px;
  align-items: center;
}

.path-input-shell {
  position: relative;
  min-width: 0;
}

.path-input-shell > .input {
  width: 100%;
  padding-right: 30px;
}

.path-input-shell > .header-search-field {
  width: 100%;
  padding-right: 30px;
}

.path-input-shell > .header-search-field--port {
  padding-right: 20px;
}

.path-input-clear {
  position: absolute;
  top: 5px;
  right: 5px;
  display: grid;
  width: var(--size-close-button);
  height: var(--size-close-button);
  min-width: var(--size-close-button);
  min-height: var(--size-close-button);
  place-items: center;
  border: 0;
  border-radius: var(--radius-control);
  background: var(--color-transparent);
  padding: 0;
  cursor: pointer;
  opacity: 0.72;
}

.path-input-clear--port {
  top: 7px;
  right: 3px;
  width: 16px;
  height: 16px;
  min-width: 16px;
  min-height: 16px;
}

.path-input-clear--port .button__icon {
  width: 10px;
  height: 10px;
}

.path-input-clear:hover,
.path-input-clear:focus {
  background: var(--color-control-hover-bg);
  opacity: 1;
  outline: none;
}

.path-input-clear:disabled {
  cursor: default;
  opacity: 0.32;
  pointer-events: none;
}

.path-input-clear .button__icon {
  width: 12px;
  height: 12px;
}

.button-row { display: flex; flex-wrap: wrap; gap: 5px; }

.button {
  border: 1px solid var(--color-control-border);
  background: var(--color-control-bg);
  color: var(--color-text);
  border-radius: var(--radius-control);
  min-height: var(--size-compact-control);
  padding: 6px 10px;
  font-weight: 600;
  cursor: pointer;
  transition: background 120ms ease, border-color 120ms ease, color 120ms ease;
}

.button:hover {
  background: var(--color-control-hover-bg);
  border-color: var(--color-control-border-strong);
}

.button:active {
  background: var(--color-control-active-bg);
}

.button:not(.button--secondary):not(.button--danger):not(.button--warning):not(.button--icon):not(.button--close) {
  background: var(--color-accent);
  border-color: var(--color-accent);
  color: var(--color-white);
}

.button:not(.button--secondary):not(.button--danger):not(.button--warning):not(.button--icon):not(.button--close):hover {
  background: var(--color-accent-hover);
  border-color: var(--color-accent-hover);
}

.button--secondary {
  background: var(--color-control-bg);
  border-color: var(--color-control-border);
  color: var(--color-text);
}

.button--danger {
  background: var(--color-danger-bg);
  border-color: var(--color-danger);
  color: var(--color-text-strong);
}

.button--warning {
  background: var(--color-warning-bg);
  border-color: var(--color-warning);
  color: var(--color-text-strong);
}

.button:disabled {
  cursor: default;
  opacity: 0.55;
  background: var(--color-control-bg);
  border-color: var(--color-control-border);
  color: var(--color-text-muted);
}

.button--monitoring {
  background: var(--color-accent);
  border-color: var(--color-accent);
  color: var(--color-white);
}

.button--monitoring.button--icon {
  background: var(--color-accent);
  border-color: var(--color-accent);
}

.button--monitoring:hover,
.button--monitoring:focus {
  background: var(--color-accent-hover);
  border-color: var(--color-accent-hover);
}

.button--monitoring-active,
.button--monitoring-active:hover,
.button--monitoring-active:focus {
  background: var(--color-monitoring-active);
  border-color: var(--color-monitoring-active-hover);
  color: var(--color-white);
}

.button--icon.button--monitoring.button--monitoring-active,
.button--icon.button--monitoring.button--monitoring-active:hover,
.button--icon.button--monitoring.button--monitoring-active:focus {
  background: var(--color-monitoring-active);
  border-color: var(--color-monitoring-active-hover);
  color: var(--color-white);
}

.button--cloud-active,
.button--cloud-active:hover,
.button--cloud-active:focus,
.button--icon.button--cloud-active,
.button--icon.button--cloud-active:hover,
.button--icon.button--cloud-active:focus {
  background: var(--color-monitoring-active);
  border-color: var(--color-monitoring-active-hover);
  color: var(--color-white);
}

.button--monitoring-active {
  animation: monitoring-button-pulse 1.6s ease-in-out infinite;
}

.button--monitoring-active .button__icon-shell--monitoring {
  transform-origin: center center;
  animation: monitoring-icon-squash-cycle 1.8s infinite;
}

.button--monitoring-active .button__icon-scale--monitoring {
  transform-origin: center center;
  animation: monitoring-icon-scale-cycle 1.8s infinite;
}

.button--monitoring-active .button__icon-rotor--monitoring {
  transform-origin: center center;
  animation: monitoring-icon-rotate-cycle 1.8s infinite;
}

.button--icon {
  width: var(--size-compact-control);
  height: var(--size-compact-control);
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-control-bg);
  border-color: var(--color-control-border);
  border-radius: var(--radius-control);
  font-size: 20px;
  line-height: 0;
}

.button--icon.header-action-button--update-available,
.button--icon.header-action-button--update-available:hover,
.button--icon.header-action-button--update-available:focus {
  background: var(--color-success-bg);
  border-color: var(--color-success);
  color: var(--color-white);
}

.button--square {
  width: var(--size-close-button);
  height: var(--size-close-button);
  min-width: var(--size-close-button);
  min-height: var(--size-close-button);
  max-width: var(--size-close-button);
  max-height: var(--size-close-button);
  flex: 0 0 var(--size-close-button);
  padding: 0;
  display: grid;
  place-items: center;
  border-radius: var(--radius-control);
  line-height: 1;
}

.button--close {
  grid-row: 1;
  background: var(--color-control-bg);
  border-color: var(--color-control-border);
}

.button__icon {
  display: block;
  width: 10px;
  height: 10px;
  filter: brightness(0) invert(1);
  pointer-events: none;
}

.button__icon-fallback {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  color: var(--color-text-strong);
  font-size: 18px;
  font-weight: 800;
  line-height: 1;
  pointer-events: none;
}

.button__icon--module-stop,
.button__icon--module-close,
.button__icon--module-action {
  width: 28px;
  height: 28px;
}

.button__icon-shell,
.button__icon-scale,
.button__icon-rotor {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  pointer-events: none;
}

.button__icon-shell--monitoring,
.button__icon-scale--monitoring,
.button__icon-rotor--monitoring,
.button__icon--monitoring {
  width: 24px;
  height: 24px;
}

.button__icon--confirm-filtered {
  width: 28px;
  height: 28px;
}

.button__icon--unconfirm-filtered {
  width: 28px;
  height: 28px;
}

.button__icon--clear-monitoring {
  width: 28px;
  height: 28px;
}

.button__icon--cloud-sync {
  width: 28px;
  height: 28px;
}

.button__icon--cloud-import {
  width: 28px;
  height: 28px;
}

.button__icon--export-csv {
  width: 28px;
  height: 28px;
}

.button__icon--import-csv {
  width: 28px;
  height: 28px;
}

.button__icon--information {
  width: 28px;
  height: 28px;
}

.button__icon--my-publications {
  width: 24px;
  height: 24px;
}

.header-action-separator {
  display: block;
  flex: 0 0 1px;
  align-self: center;
  width: 1px;
  min-width: 1px;
  height: 44px;
  margin: 0;
  background: var(--color-control-border-strong);
  opacity: 1;
}

.button--mini {
  min-height: 20px;
  padding: 2px 7px;
  font-size: 12px;
  line-height: 14px;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 5px;
  height: var(--size-tracked-app-list-height);
  max-height: var(--size-tracked-app-list-height);
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-gutter: stable;
  border-top: 1px solid var(--color-control-border);
  border-bottom: 1px solid var(--color-control-border);
  padding: 5px var(--size-scrollbar-content-gutter) 5px 5px;
  background: var(--color-app-list-bg);
}

.tracked-apps-card .exe-path-row {
  padding-bottom: var(--size-tracked-apps-path-bottom-pad);
}

.tracked-apps-bulk-row {
  display: grid;
  grid-template-columns: auto var(--size-switch-width) minmax(0, 1fr);
  align-items: center;
  gap: 5px;
  min-height: var(--size-switch-height);
}

.ignored-addresses-card {
  flex: 0 0 auto;
  min-height: 0;
  align-self: stretch;
}

.ignored-addresses-section {
  display: grid;
  gap: 5px;
  min-width: 0;
  min-height: 0;
}

.ignored-addresses {
  display: grid;
  grid-auto-rows: var(--size-ignored-address-row-height);
  align-content: start;
  gap: var(--size-ignored-address-row-gap);
  height: var(--size-ignored-addresses-list-height);
  max-height: var(--size-ignored-addresses-list-height);
  padding: 5px var(--size-scrollbar-content-gutter) 5px 5px;
  border-top: 1px solid var(--color-control-border);
  border-bottom: 1px solid var(--color-control-border);
  background: var(--color-app-list-bg);
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-gutter: stable;
}

.value-label,
.integration-status__label,
.integration-dialog-status-row__label {
  color: var(--color-text);
  font-size: 12px;
  font-weight: 700;
  line-height: 16px;
}

.ignored-addresses__row {
  display: grid;
  grid-template-columns: var(--size-ignored-address-ip-column) 18px minmax(80px, 1fr) var(--size-close-button);
  align-items: center;
  gap: 6px;
  min-width: 0;
  height: var(--size-ignored-address-row-height);
  min-height: var(--size-ignored-address-row-height);
  padding: 3px var(--size-list-row-inline-pad) 3px 5px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-app-row-bg);
}

.ignored-addresses__empty {
  min-height: var(--size-ignored-address-row-height);
  padding: 5px;
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 18px;
}

.ignored-addresses__skeleton {
  height: var(--size-ignored-address-row-height);
  min-height: var(--size-ignored-address-row-height);
  border-radius: var(--radius-control);
  border: 1px solid var(--color-control-border);
  background: var(--color-app-skeleton-bg);
  animation: tracked-app-skeleton-pulse 1s ease-in-out infinite alternate;
}

.ignored-addresses__address {
  display: block;
  width: 100%;
  min-width: var(--size-ignored-address-ip-column);
  max-width: var(--size-ignored-address-ip-column);
  grid-column: 1;
  grid-row: 1;
  justify-self: stretch;
  color: var(--color-text) !important;
  direction: ltr;
  font-size: 12px;
  text-align: left;
  unicode-bidi: plaintext;
}

.ignored-addresses__domain {
  width: 100%;
  min-width: 80px;
  grid-column: 3;
  grid-row: 1;
  color: var(--color-text);
}

.ignored-addresses__help {
  grid-column: 2;
  grid-row: 1;
  justify-self: center;
}

#ignored-addresses-panel-help {
  position: relative;
  top: -2px;
}

.ignored-addresses__row .button {
  grid-column: 4;
  grid-row: 1;
  justify-self: end;
}

.tracked-apps-bulk-label,
.tracked-apps-count-label {
  font-size: 12px;
  line-height: var(--size-switch-height);
  color: var(--color-text);
}

.tracked-apps-count-label {
  justify-self: end;
  white-space: nowrap;
}

.list-item {
  padding: 12px;
  border-radius: var(--radius-control);
  border: 1px solid var(--color-control-border);
  background: var(--color-panel-subtle-bg);
}

.list-item__row { display: flex; justify-content: space-between; gap: 10px; align-items: center; }

.integration-status {
  display: grid;
  gap: 5px;
  min-width: 0;
  padding: 5px;
}

.integration-status--compact {
  padding: 0;
}

.modules-card {
  flex: 1 1 var(--size-modules-panel-height);
  height: auto;
  min-height: var(--size-modules-panel-height);
  max-height: none;
  overflow: visible;
}

.modules-card__header-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
  min-width: 0;
}

.modules-card__order-label {
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 600;
  line-height: 16px;
  white-space: nowrap;
}

.integration-module-grid {
  display: flex;
  flex-wrap: nowrap;
  gap: 6px;
  align-items: center;
  min-width: 0;
  min-height: calc(var(--size-module-button-slot) + var(--size-scrollbar));
  overflow-x: auto;
  overflow-y: visible;
  scrollbar-gutter: stable;
  padding: var(--size-module-button-pulse-gutter) 6px calc(var(--size-module-button-pulse-gutter) + var(--size-scrollbar)) 0;
}

.integration-module-button-shell {
  position: relative;
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 2px;
  padding: var(--size-module-button-pulse-gutter);
}

.integration-module-button {
  width: var(--size-module-button);
  height: var(--size-module-button);
  min-width: var(--size-module-button);
  min-height: var(--size-module-button);
  padding: 0;
  overflow: hidden;
}

.integration-module-button--background-active {
  animation: integration-module-background-pulse 1.6s ease-in-out infinite;
}

.module-action-button--pulse {
  background: var(--color-monitoring-active);
  border-color: var(--color-monitoring-active-hover);
  color: var(--color-white);
  animation: module-action-button-pulse 1.6s ease-in-out infinite;
}

.module-action-button--pulse:hover,
.module-action-button--pulse:focus {
  background: var(--color-monitoring-active);
  border-color: var(--color-monitoring-active-hover);
  color: var(--color-white);
}

@keyframes integration-module-background-pulse {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(255, 255, 255, 0);
    filter: brightness(1);
  }
  50% {
    box-shadow: 0 0 0 3px var(--color-module-activity-pulse);
    filter: brightness(1.12);
  }
}

@keyframes module-action-button-pulse {
  0%,
  100% {
    box-shadow: 0 0 0 0 rgba(255, 255, 255, 0);
    opacity: 1;
    transform: scale(1);
  }

  50% {
    box-shadow: 0 0 0 3px var(--color-module-activity-pulse);
    opacity: 0.96;
    transform: scale(1.02);
  }
}

.integration-module-button-skeleton {
  flex: 0 0 auto;
  width: var(--size-module-button);
  height: var(--size-module-button);
  min-width: var(--size-module-button);
  min-height: var(--size-module-button);
  margin: var(--size-module-button-pulse-gutter);
  border-radius: var(--radius-control);
  border: 1px solid var(--color-control-border);
  background: var(--color-app-skeleton-bg);
  animation: tracked-app-skeleton-pulse 1s ease-in-out infinite alternate;
}

.integration-module-button__icon {
  display: grid;
  place-items: center;
  width: 100%;
  height: 100%;
  font-size: 13px;
  font-weight: 700;
  line-height: 1;
}

.integration-module-button__image {
  display: block;
  width: 80%;
  height: 80%;
  object-fit: contain;
  border-radius: inherit;
  pointer-events: none;
}

.integration-module-reorder-controls {
  display: flex;
  gap: 2px;
  align-items: center;
}

.integration-module-reorder-button {
  width: 18px;
  min-width: 18px;
  max-width: 18px;
  height: var(--size-compact-control);
  min-height: var(--size-compact-control);
  max-height: var(--size-compact-control);
  padding: 0;
  line-height: 1;
  background: var(--color-control-bg);
  border-color: var(--color-control-border);
}

.integration-module-reorder-button--left {
  border-top-right-radius: 0;
  border-bottom-right-radius: 0;
}

.integration-module-reorder-button--right {
  border-top-left-radius: 0;
  border-bottom-left-radius: 0;
}

.integration-module-reorder-button__icon {
  width: 12px;
  height: 12px;
}

.integration-module-reorder-button__icon--right {
  transform: rotate(180deg);
}

.integration-module-dialog {
  width: fit-content;
  min-width: min(620px, calc(100vw - 40px));
  max-width: calc(100vw - 40px);
  max-height: calc(100vh - var(--size-footer-height) - 40px);
  overflow: hidden;
}

.integration-module-dialog--layout .integration-module-dialog__body {
  width: 100%;
}

.integration-module-dialog--layout {
  box-sizing: border-box;
}

.integration-module-dialog--size-fullscreen {
  width: calc(100vw - 40px);
  height: calc(100vh - var(--size-header-height) - var(--size-footer-height) - 40px);
  max-height: calc(100vh - var(--size-header-height) - var(--size-footer-height) - 40px);
}

.integration-module-dialog--align-left {
  justify-self: start;
}

.integration-module-dialog--align-center {
  justify-self: center;
}

.integration-module-dialog--align-right {
  justify-self: end;
}

.integration-module-dialog .modal__body {
  --module-ui-textarea-host-max-height: calc(100vh - var(--size-footer-height) - 116px);
  flex: 0 1 auto;
  align-content: start;
  min-height: 0;
  max-height: calc(100vh - var(--size-footer-height) - 116px);
  overflow-x: hidden;
  overflow-y: auto;
}

.integration-module-dialog--size-fullscreen .integration-module-dialog__body {
  --module-ui-textarea-host-max-height: calc(100vh - var(--size-header-height) - var(--size-footer-height) - 116px);
  flex: 1 1 auto;
  max-height: none;
}

.integration-status-layout--module-menu {
  align-self: start;
  align-items: start;
  width: min(760px, 100%);
  max-width: 100%;
  gap: 5px;
}

.integration-status-layout--module-menu .integration-status__fields {
  gap: 2px;
  align-content: start;
}

.integration-status-layout--module-menu .integration-status__action {
  justify-content: flex-end;
  gap: 3px;
}

.integration-status-layout--module-menu .integration-status__row {
  min-height: 18px;
}

.integration-status-layout--module-menu .integration-status__path,
.integration-status-layout--module-menu .integration-status__provider,
.integration-status-layout--module-menu .integration-status__value {
  height: 18px;
  line-height: 16px;
}

.integration-status-layout--module-menu .integration-status-text {
  line-height: 16px;
}

.module-ui-schema {
  display: grid;
  gap: 6px;
  width: min(760px, 100%);
  max-width: 100%;
  min-width: 0;
  min-height: 0;
  align-self: start;
  align-content: start;
  box-sizing: border-box;
}

.module-ui-schema--dialog-layout {
  width: 100%;
  align-self: stretch;
  justify-self: stretch;
}

.module-ui-schema[hidden] {
  display: none;
}

.module-ui-schema__panel,
.module-ui-schema__nested-subpanel,
.module-ui-schema__row,
.module-ui-schema__actions,
.module-ui-schema__progress {
  display: grid;
  gap: 4px;
  min-width: 0;
  min-height: 0;
  align-content: start;
  box-sizing: border-box;
}

.module-ui-schema__progress--compact-labeled {
  grid-template-columns: max-content minmax(120px, 1fr);
  align-items: center;
  column-gap: 6px;
  width: 100%;
}

.module-ui-schema__progress--compact-labeled > .module-ui-schema__title {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.module-ui-schema__progress--compact-labeled > .progress-bar--compact {
  min-width: 120px;
}

.module-ui-schema__panel {
  padding: 8px;
  border: 1px solid var(--color-control-border);
  background: var(--color-panel-bg);
}

.module-ui-schema__nested-subpanel {
  gap: 5px;
  padding: 5px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-app-list-bg);
}

.module-ui-schema__nested-subpanel-inner {
  display: grid;
  gap: 4px;
  min-width: 0;
  min-height: 0;
  padding: 8px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-app-row-bg);
  box-sizing: border-box;
}

.module-ui-schema__grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 8px;
  width: 100%;
  min-width: 0;
  min-height: 0;
  align-items: start;
}

.module-ui-schema__grid > .module-ui-schema__title,
.module-ui-schema__grid > .module-ui-schema__value,
.module-ui-schema__grid > .module-ui-schema__actions,
.module-ui-schema__grid > .module-ui-schema__button-row {
  grid-column: 1 / -1;
}

.module-ui-schema--size-auto {
  width: fit-content;
  max-width: 100%;
}

.module-ui-schema--size-stretch {
  width: 100%;
  align-self: stretch;
}

.module-ui-schema--size-fullscreen {
  width: calc(100vw - 72px);
  height: calc(100vh - var(--size-footer-height) - 148px);
  max-width: 100%;
  max-height: calc(100vh - var(--size-footer-height) - 148px);
}

.module-ui-schema--align-left {
  justify-self: start;
  text-align: left;
}

.module-ui-schema--align-center {
  justify-self: center;
  text-align: center;
}

.module-ui-schema--align-right {
  justify-self: end;
  text-align: right;
}

.module-ui-schema__actions.module-ui-schema--align-center {
  justify-content: center;
}

.module-ui-schema__actions.module-ui-schema--align-right {
  justify-content: flex-end;
}

.module-ui-schema--scroll-x {
  overflow-x: auto;
  overflow-y: hidden;
}

.module-ui-schema--scroll-y {
  overflow-x: hidden;
  overflow-y: auto;
  max-height: min(360px, 70vh);
}

.module-ui-schema--scroll-both {
  overflow: auto;
  max-height: min(360px, 70vh);
}

.module-ui-schema__table {
  display: grid;
  width: 100%;
  min-width: 0;
  min-height: 0;
  box-sizing: border-box;
  grid-template-rows: auto minmax(0, 1fr) auto;
  gap: 0;
  align-content: stretch;
  align-self: stretch;
  overflow: hidden;
}

.module-ui-schema__table-frame,
.module-ui-schema__table-frame.table-wrap {
  display: flex;
  flex-direction: column;
  grid-row: 2;
  width: 100%;
  height: 100%;
  max-width: 100%;
  min-width: 0;
  min-height: 0;
  box-sizing: border-box;
  overflow: hidden;
  overscroll-behavior: contain;
}

.module-ui-schema__table-body {
  flex: 1 1 auto;
  min-height: 0;
}

.module-ui-schema__table-body.module-ui-schema--scroll-x {
  overflow-x: auto;
  overflow-y: hidden;
}

.module-ui-schema__table-body.module-ui-schema--scroll-y {
  overflow-x: hidden;
  overflow-y: auto;
}

.module-ui-schema__table-body.module-ui-schema--scroll-both {
  overflow: auto;
}

.module-ui-schema__table.module-ui-schema--size-fullscreen .module-ui-schema__table-frame {
  height: 100%;
  max-height: none;
}

.module-ui-schema__table .module-ui-schema__title {
  padding: 6px 8px 0;
}

.module-ui-schema__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  min-width: 0;
  padding: 0;
  border-top: 0;
}

.module-ui-schema__footer-host {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
}

.module-ui-schema__footer-value {
  min-width: 0;
  color: var(--text-muted);
  font-size: 12px;
}

.module-ui-schema__footer > .module-ui-schema__value {
  flex: 1 1 auto;
  min-width: 0;
}

.module-ui-schema__footer > .module-ui-schema__actions {
  flex: 0 0 auto;
  width: auto;
}

.module-ui-schema__row {
  display: grid;
  width: 100%;
  min-width: 0;
  grid-template-columns: minmax(120px, auto) minmax(0, 1fr) auto;
  align-items: center;
  min-height: var(--size-compact-control);
  column-gap: 6px;
  box-sizing: border-box;
}

.module-ui-schema__row--no-title {
  grid-template-columns: minmax(0, 1fr) auto;
}

.module-ui-schema__row--textarea {
  align-items: start;
  min-height: 0;
}

.module-ui-schema__layout-row {
  display: flex;
  flex-wrap: nowrap;
  align-items: center;
  justify-content: flex-start;
  gap: 8px;
  width: 100%;
  min-width: 0;
  min-height: var(--size-compact-control);
  box-sizing: border-box;
  overflow: visible;
}

.module-ui-schema__layout-row--justify-center {
  justify-content: center;
}

.module-ui-schema__layout-row--justify-end {
  justify-content: flex-end;
}

.module-ui-schema__layout-row--justify-between {
  justify-content: space-between;
}

.module-ui-schema__layout-row > .module-ui-schema__row,
.module-ui-schema__layout-row > .module-ui-schema__panel,
.module-ui-schema__layout-row > .module-ui-schema__nested-subpanel,
.module-ui-schema__layout-row > .module-ui-schema__grid,
.module-ui-schema__layout-row > .module-ui-schema__button-row,
.module-ui-schema__layout-row > .module-ui-schema__progress,
.module-ui-schema__layout-row > .module-ui-schema__tabs,
.module-ui-schema__layout-row > .module-ui-schema__table,
.module-ui-schema__layout-row > .module-ui-schema__footer,
.module-ui-schema__layout-row > .module-ui-schema__help,
.module-ui-schema__layout-row > .module-ui-schema__separator {
  flex: 0 1 auto;
  width: auto;
  max-width: 100%;
  margin-top: 0;
  margin-bottom: 0;
}

.module-ui-schema__layout-row > .module-ui-schema--size-stretch {
  flex: 1 1 auto;
}

.module-ui-schema__spacer {
  min-width: 0;
  min-height: 0;
}

.module-ui-schema__layout-row > .module-ui-schema__spacer {
  flex: 1 1 auto;
  align-self: stretch;
}

.module-ui-schema__title h3 {
  margin: 0;
  font-size: 12px;
  line-height: 16px;
}

.module-ui-schema__value,
.module-ui-schema__status {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.module-ui-schema__path {
  width: 100%;
  height: 22px;
}

.module-ui-schema__input-shell {
  position: relative;
  display: grid;
  grid-template-columns: minmax(0, 1fr) var(--size-close-button);
  align-items: center;
  width: 100%;
  min-width: 0;
  align-self: stretch;
  overflow: visible;
}

.module-ui-schema__input,
.module-ui-schema__select {
  width: 100%;
  height: var(--size-compact-control);
  min-height: var(--size-compact-control);
  min-width: 0;
  padding: 3px 5px;
  font-size: 12px;
  line-height: 16px;
}

.module-ui-schema__input-shell > .input {
  grid-column: 1 / -1;
  grid-row: 1;
}

.module-ui-schema__input {
  padding-right: 30px;
}

.module-ui-schema__input-shell--textarea {
  align-items: start;
  grid-auto-rows: auto;
  height: auto;
  min-height: 0;
}

.module-ui-schema__textarea {
  display: block;
  width: 100%;
  max-width: 100%;
  max-height: min(var(--module-ui-textarea-max-height, var(--module-ui-textarea-host-max-height)), var(--module-ui-textarea-host-max-height));
  box-sizing: border-box;
  min-height: 74px;
  overflow: auto;
  resize: vertical;
  line-height: 18px;
  padding: 4px 30px 4px 5px;
}

.module-ui-schema__clear {
  position: absolute;
  top: 50%;
  right: 5px;
  transform: translateY(-50%);
  display: grid;
  place-items: center;
  width: var(--size-close-button);
  height: var(--size-close-button);
  min-width: var(--size-close-button);
  min-height: var(--size-close-button);
  padding: 0;
}

.module-ui-schema__clear--textarea {
  top: 6px;
  transform: none;
}

.module-ui-schema__actions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  width: 100%;
  min-width: 0;
}

.module-ui-schema__action-slot {
  display: flex;
  min-width: 0;
  min-height: var(--size-compact-control);
  align-items: center;
}

.module-ui-schema__action {
  white-space: nowrap;
}

.module-ui-schema__button-row {
  display: grid;
  width: 100%;
  min-width: 0;
  min-height: var(--size-compact-control);
  margin: 8px 0 0;
  padding: 0;
  align-items: center;
  align-self: stretch;
  align-content: center;
  box-sizing: border-box;
  clear: both;
  overflow: visible;
}

.module-ui-schema__button-row-content {
  display: grid;
  width: 100%;
  min-width: 0;
  min-height: var(--size-compact-control);
  align-items: center;
  box-sizing: border-box;
  overflow: visible;
}

.module-ui-schema__button-row > .module-ui-schema__actions {
  min-height: var(--size-compact-control);
  align-items: center;
}

.module-ui-schema__button-row-content > .module-ui-schema__actions {
  min-height: var(--size-compact-control);
  align-items: center;
}

.module-ui-schema__panel > .module-ui-schema__actions,
.module-ui-schema__tabs-body > .module-ui-schema__actions {
  min-height: var(--size-compact-control);
  margin-bottom: 0;
}

.module-ui-schema__panel > .module-ui-schema__button-row,
.module-ui-schema__tabs-body > .module-ui-schema__button-row {
  min-height: var(--size-compact-control);
  margin-bottom: 0;
  padding-top: 0;
  padding-bottom: 0;
  overflow: visible;
}

.module-ui-schema__actions--column {
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  width: auto;
}

.module-ui-schema__actions--column.module-ui-schema--align-center {
  align-items: center;
  justify-content: flex-start;
  justify-self: center;
}

.module-ui-schema__actions--column.module-ui-schema--align-right {
  align-items: flex-end;
  justify-content: flex-start;
  justify-self: end;
}

.module-ui-schema__action--align-left {
  margin-inline-start: 0;
  margin-inline-end: 0;
}

.module-ui-schema__action-slot.module-ui-schema__action--align-left {
  justify-content: flex-start;
}

.module-ui-schema__action--align-center {
  margin-inline-start: 0;
  margin-inline-end: 0;
}

.module-ui-schema__action-slot.module-ui-schema__action--align-center {
  justify-content: center;
}

.module-ui-schema__action--align-right {
  margin-inline-start: 0;
  margin-inline-end: 0;
}

.module-ui-schema__action-slot.module-ui-schema__action--align-right {
  justify-content: flex-end;
}

.module-ui-schema__separator {
  height: 1px;
  background: var(--border);
  margin: 2px 0;
}

.module-ui-schema__help {
  margin: 0;
  color: var(--text-muted);
  font-size: 12px;
}

.ui-entity-table {
  width: 100%;
  min-width: 100%;
  max-width: none;
  border-collapse: collapse;
  background: var(--color-app-list-bg);
  color: var(--color-text);
  font-size: 12px;
  table-layout: fixed;
}

.ui-entity-table th,
.ui-entity-table td {
  padding: 4px 10px;
  border-bottom: 1px solid var(--color-control-border);
  text-align: left;
  vertical-align: middle;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ui-entity-table th {
  padding: 1px 10px;
  background: var(--color-panel-header-bg);
  color: var(--color-text-strong);
  font-weight: 700;
  line-height: 14px;
  height: 18px;
  border-bottom: 1px solid var(--color-control-border-strong);
  white-space: nowrap;
}

.ui-entity-table td {
  padding: 2px 10px;
  line-height: 16px;
}

.ui-entity-table td.module-ui-schema--align-center {
  text-align: center;
}

.ui-entity-table td.module-ui-schema--align-right {
  text-align: right;
}

.module-ui-schema__table-cell-field.path-field {
  display: block;
  width: 100%;
  max-width: 100%;
  min-width: 0;
  height: 20px;
  line-height: 16px;
  padding: 1px 5px;
  box-sizing: border-box;
  overflow-x: hidden;
  overflow-y: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.module-ui-schema__table-cell-field.path-field:focus {
  overflow-x: auto;
  text-overflow: clip;
}

.ui-entity-table tr:last-child td {
  border-bottom: 0;
}

.integration-status-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 8px;
  align-items: stretch;
  width: 100%;
  max-width: 100%;
  min-width: 0;
}

.integration-status__fields {
  display: grid;
  width: 100%;
  max-width: min(100%, var(--size-integration-status-left-max));
  gap: 5px;
  min-width: 0;
}

.integration-status__action {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  justify-content: flex-end;
  gap: 5px;
  min-width: 0;
}

.integration-status__action > .button {
  width: 100%;
  min-width: 0;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.integration-status__main {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.integration-status__main--with-path {
  display: grid;
  grid-template-columns: auto auto minmax(0, 1fr);
  justify-content: stretch;
}

.integration-status__row {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 8px;
  align-items: center;
  min-height: 22px;
  min-width: 0;
}

.integration-status__row--with-action {
  grid-template-columns: auto minmax(0, 1fr) auto;
}

.integration-status__label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.integration-status__path,
.integration-status__provider,
.integration-status__value {
  min-width: 0;
  margin: 0;
  overflow: hidden;
  color: var(--color-text);
  font-size: 12px;
  line-height: 16px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.integration-status-text {
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 700;
  line-height: 18px;
  white-space: nowrap;
}

.integration-status-text--success {
  color: var(--color-success);
}

.integration-status-text--danger {
  color: var(--color-danger);
}

.integration-status .button-row {
  margin-top: 5px;
}

@media (max-width: 700px) {
  .integration-status-layout--module-menu {
    grid-template-columns: minmax(0, 1fr);
  }
}

.tracked-app {
  position: relative;
  display: grid;
  grid-template-columns: var(--size-app-icon-slot) minmax(130px, 1fr) var(--size-switch-width);
  gap: 12px;
  align-items: center;
  min-width: 0;
  height: var(--size-tracked-app-height);
  min-height: var(--size-tracked-app-height);
  max-height: var(--size-tracked-app-height);
  padding: 0 var(--size-list-row-inline-pad) 0 5px;
  overflow: visible;
  cursor: pointer;
  background: var(--color-app-row-bg);
}

.tracked-app--enabled {
  background: var(--color-app-row-enabled-bg);
}

.tracked-app-skeleton {
  min-width: 0;
  height: var(--size-tracked-app-height);
  min-height: var(--size-tracked-app-height);
  max-height: var(--size-tracked-app-height);
  padding: 5px;
  border-radius: var(--radius-control);
  border: 1px solid var(--color-control-border);
  background: var(--color-app-skeleton-bg);
  animation: tracked-app-skeleton-pulse 1s ease-in-out infinite alternate;
}

@keyframes tracked-app-skeleton-pulse {
  from {
    background: var(--color-app-skeleton-bg);
  }

  to {
    background: var(--color-app-skeleton-pulse-bg);
  }
}

@keyframes monitoring-icon-squash-cycle {
  0% {
    transform: translateY(0) scaleY(1);
    animation-timing-function: ease-in-out;
  }

  11.111% {
    transform: translateY(0) scaleY(0.2);
    animation-timing-function: ease-in-out;
  }

  22.222% {
    transform: translateY(0) scaleY(1);
    animation-timing-function: linear;
  }

  38% {
    transform: translateY(0) scaleY(1);
  }

  100% {
    transform: translateY(0) scaleY(1);
  }
}

@keyframes monitoring-icon-rotate-cycle {
  0% {
    transform: rotate(0deg);
  }

  44.444% {
    transform: rotate(0deg);
    animation-timing-function: linear;
  }

  72.222% {
    transform: rotate(360deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

@keyframes monitoring-icon-scale-cycle {
  0% {
    transform: scale(1);
  }

  44.444% {
    transform: scale(1);
    animation-timing-function: cubic-bezier(0.2, 0.78, 0.22, 1);
  }

  58.333% {
    transform: scale(1.2);
    animation-timing-function: cubic-bezier(0.45, 0, 0.2, 1);
  }

  72.222% {
    transform: scale(1);
  }

  100% {
    transform: scale(1);
  }
}

@keyframes monitoring-button-pulse {
  0%,
  100% {
    transform: scale(1);
    opacity: 1;
  }

  50% {
    transform: scale(1.04);
    opacity: 0.92;
  }
}

.tracked-app__main {
  display: grid;
  grid-template-rows: 20px 18px;
  row-gap: 2px;
  align-content: start;
  min-width: 0;
}

.tracked-app__title {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 15px;
  line-height: 20px;
  color: var(--color-text-strong);
}

.path-field {
  display: block;
  width: 100%;
  min-width: 0;
  max-width: 100%;
  height: 22px;
  appearance: none;
  -webkit-appearance: none;
  border: 0;
  border-radius: var(--radius-control);
  background-clip: padding-box;
  background: var(--color-transparent);
  color: var(--color-text-muted);
  font: inherit;
  font-size: 12px;
  line-height: 18px;
  outline: none;
  overflow-x: hidden;
  overflow-y: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  scrollbar-width: none;
}

.path-field:focus {
  background: var(--color-control-bg);
  color: var(--color-text);
  overflow-x: auto;
  text-overflow: clip;
}

.footer-message-text.path-field.footer-message-text--success {
  color: var(--color-success) !important;
}

.footer-message-text.path-field.footer-message-text--warning {
  color: var(--color-warning) !important;
}

.footer-message-text.path-field.footer-message-text--error {
  color: var(--color-danger) !important;
}

.path-field::-webkit-scrollbar {
  width: 0;
  height: 0;
}

.path-field--clickable {
  cursor: pointer;
}

.path-field--content-width {
  width: min(100%, var(--path-field-content-width, 100%));
  max-width: 100%;
  justify-self: start;
  flex: 0 1 auto;
}

.path-field--clickable:hover {
  color: var(--color-text);
  text-decoration: underline;
}

.integration-status__path.path-field,
.integration-dialog-path__input.path-field,
.ignored-addresses__domain.path-field,
.domain-field.path-field {
  color: var(--color-text);
}

.tracked-app__path {
  width: auto;
  max-width: 100%;
  justify-self: start;
  padding: 0;
  text-align: left;
}

.tracked-app__actions {
  position: relative;
  display: block;
  justify-self: end;
  align-self: stretch;
  height: 100%;
  min-width: var(--size-switch-width);
}

.tracked-app__actions--with-delete {
  display: block;
}

.tracked-app__actions--with-delete .button--close {
  position: absolute;
  top: var(--size-list-row-inline-pad);
  right: 0;
}

.tracked-app__action-row {
  position: absolute;
  right: 0;
  bottom: var(--size-list-row-inline-pad);
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.app-icon {
  display: grid;
  place-items: center;
  width: var(--size-app-icon-slot);
  height: var(--size-app-icon-slot);
  min-width: var(--size-app-icon-slot);
  min-height: var(--size-app-icon-slot);
  border-radius: var(--radius-pill);
  border: 1px solid var(--color-control-border);
  background: var(--color-control-bg);
  color: var(--color-text);
  font-weight: 800;
  font-size: 22px;
  letter-spacing: 0.03em;
  overflow: hidden;
}

.app-icon__image {
  display: block;
  width: var(--size-app-icon-image);
  height: var(--size-app-icon-image);
  object-fit: contain;
  object-position: center;
  aspect-ratio: 1 / 1;
}

.state-label {
  display: inline-flex;
  align-items: center;
  padding: 0;
  color: var(--color-text);
  font-size: 12px;
  font-weight: 600;
  line-height: 18px;
}

.state-label--success { color: var(--color-success); }
.state-label--danger { color: var(--color-danger); }
.state-label--warning { color: var(--color-warning); }

.observation-connection {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
}

.connection-metrics {
  display: inline-flex;
  align-items: center;
  gap: 0;
  color: var(--color-text);
  white-space: nowrap;
}

.connection-metrics__success { color: var(--color-success); }
.connection-metrics__failure { color: var(--color-danger); }

.switch {
  position: relative;
  width: var(--size-switch-width);
  height: var(--size-switch-height);
  padding: 0;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-control-bg);
  cursor: pointer;
}

.switch:hover {
  background: var(--color-control-hover-bg);
  border-color: var(--color-control-border-strong);
}

.switch--on {
  background: var(--color-accent);
}

.switch--on:hover {
  background: var(--color-accent-hover);
  border-color: var(--color-control-border);
}

.switch__knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: var(--size-switch-knob);
  height: var(--size-switch-knob);
  border-radius: var(--radius-control);
  background: var(--color-text-muted);
  transition: transform 140ms ease, background 140ms ease;
}

.switch--on .switch__knob {
  transform: translateX(24px);
  background: var(--color-switch-knob-on);
}

.observations-card {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  max-height: none;
}

.observations-card .table-wrap {
  flex: 1 1 auto;
  min-height: 0;
  max-height: none;
  height: 100%;
}

.table-wrap {
  display: flex;
  flex-direction: column;
  height: 100%;
  max-height: none;
  overflow: hidden;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-panel);
  background: var(--color-app-list-bg);
}

.table-header-wrap {
  display: grid;
  grid-template-columns: minmax(0, 1fr) var(--size-scrollbar);
  flex: 0 0 auto;
  min-width: 0;
  background: var(--color-panel-header-bg);
}

.table-header-scroll {
  min-width: 0;
  overflow: hidden;
  background: var(--color-panel-header-bg);
}

.table-header-scrollbar-fill {
  border-bottom: 1px solid var(--color-control-border);
  background: var(--color-panel-header-bg);
}

.table-body-wrap {
  flex: 1 1 auto;
  min-height: 0;
  overflow: auto;
  padding-inline-end: var(--size-scrollbar-content-gutter);
  background: var(--color-app-list-bg);
}

.table-body-wrap::-webkit-scrollbar-track {
  background: var(--color-scrollbar-track);
}

.table-sortable {
  cursor: pointer;
  user-select: none;
}

.table-sortable__content {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  min-width: 0;
  transform: translateX(-8px);
}

.table-sortable__label {
  min-width: 0;
}

.table-sortable__icon {
  position: relative;
  top: 0;
  width: 18px;
  height: 36px;
  flex: 0 0 18px;
  filter: brightness(0) invert(1);
  opacity: 0.28;
  transform: translateZ(0);
  transition: opacity 160ms ease;
}

.table-sortable__icon--asc {
  opacity: 0.85;
}

.table-sortable__icon--desc {
  opacity: 0.85;
}

.table-sortable__icon--idle {
  opacity: 0.28;
}

.observations-table { width: 100%; border-collapse: collapse; min-width: 860px; table-layout: fixed; }
th, td { text-align: left; border-bottom: 1px solid var(--color-control-border); vertical-align: middle; }
th { padding: 1px 10px; background: var(--color-panel-header-bg); color: var(--color-text-strong); font-size: 12px; font-weight: 700; line-height: 14px; height: 18px; }
td { padding: 2px 10px; font-size: 12px; line-height: 16px; }
tr:hover td { background: var(--color-panel-hover-bg); }
.shell:focus { outline: none; }
body.netstitch-observation-selection-modifier .observations-table--body .observation-row,
body.netstitch-observation-selection-modifier .observations-table--body .observation-row .path-field,
body.netstitch-observation-selection-modifier .observations-table--body .observation-row .ip-cell {
  cursor: pointer;
}
.observation-row--confirmed td { background: var(--color-app-row-enabled-bg); }
.observation-row--confirmed:hover td { background: var(--color-app-row-enabled-hover-bg); }
.observations-table--body .observation-row--confirmed .path-field,
.observations-table--body .observation-row--confirmed .path-field:focus,
.observations-table--body .observation-row:hover .path-field,
.observations-table--body .observation-row:hover .path-field:focus {
  background: var(--color-transparent);
  color: var(--color-text);
}
th:nth-child(2), td:nth-child(2) { width: 138px; }
th:nth-child(4), td:nth-child(4) { width: 64px; }
th:nth-child(5), td:nth-child(5) { width: 66px; }
th:nth-child(6), td:nth-child(6) { width: 128px; }
th:nth-child(7), td:nth-child(7) { width: 52px; }
th:nth-child(8), td:nth-child(8) { width: 142px; }
th:nth-child(9), td:nth-child(9) { width: 142px; }
th:nth-child(10), td:nth-child(10) { width: 96px; }

.observation-app-field.path-field,
.observation-ip-field.path-field {
  width: 100%;
  min-width: 0;
  color: var(--color-text);
}

.ip-cell {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.ip-cell__help {
  top: 0;
}

.domain-field {
  width: min(260px, 100%);
  min-width: 120px;
  color: var(--color-text);
}

.table-actions { display: flex; gap: 6px; flex-wrap: nowrap; }

.table-action-button {
  flex: 0 0 20px;
  width: 20px;
  height: 20px;
  min-width: 20px;
  min-height: 20px;
  padding: 0;
}

.table-action-button__icon {
  width: 14px;
  height: 14px;
}

.alert {
  padding: 14px 16px;
  border-radius: var(--radius-panel);
  border: 1px solid var(--color-control-border);
  background: var(--color-panel-subtle-bg);
}

.alert--error { border-color: var(--color-danger); background: var(--color-danger-bg); }
.alert--success { border-color: var(--color-success); background: var(--color-success-bg); }
.small { font-size: 13px; }

.modal-backdrop {
  position: fixed;
  inset: 0 0 var(--size-footer-height) 0;
  display: grid;
  place-items: center;
  padding: 20px;
  background: var(--color-workbench-bg);
  z-index: 50;
}

.modal-backdrop.module-overlay-backdrop {
  inset: var(--size-header-height) 0 var(--size-footer-height) 0;
}

.modal {
  width: min(720px, 100%);
  border-radius: var(--radius-panel);
  border: 1px solid var(--color-control-border);
  background: var(--color-panel-bg);
  padding: 18px;
}

.modal--panel,
.modal--compact {
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding: 5px;
  border-radius: var(--radius-control);
}

.modal--panel .modal__body,
.modal--compact .modal__body {
  gap: 5px;
}

.modal--panel .modal__footer,
.modal--compact .modal__footer {
  flex: 0 0 var(--size-panel-footer-height);
  height: var(--size-panel-footer-height);
  min-height: var(--size-panel-footer-height);
  max-height: var(--size-panel-footer-height);
  align-items: center;
  gap: 5px;
  margin: 0 -5px -5px;
  padding: 6px 8px;
  overflow: hidden;
  border-top: 1px solid var(--color-control-border);
  border-radius: 0 0 var(--radius-control) var(--radius-control);
  background: var(--color-panel-header-bg);
}

.module-host-dialog {
  width: min(520px, calc(100vw - 32px));
}

.module-host-dialog__body {
  display: grid;
  grid-template-columns: 42px minmax(0, 1fr);
  align-items: start;
  padding: 12px 10px;
}

.module-host-dialog__icon {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  border: 1px solid var(--color-control-border);
  background: var(--color-control-bg);
}

.module-host-dialog__icon-image {
  width: 22px;
  height: 22px;
  object-fit: contain;
}

.module-host-dialog__icon-fallback {
  font-weight: 800;
  color: var(--color-text);
}

.module-host-dialog__message {
  min-width: 0;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  color: var(--color-text);
}

.modal__header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 14px;
}

.modal__body { display: grid; gap: 12px; }
.modal__footer {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  justify-content: flex-end;
  margin-top: 16px;
}

[data-ui-entity="panel-footer"] {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 0 0 var(--size-panel-footer-height);
  height: var(--size-panel-footer-height);
  min-height: var(--size-panel-footer-height);
  max-height: var(--size-panel-footer-height);
  overflow: hidden;
}

.panel-footer {
  margin: 0 -5px -5px;
  padding: 6px 8px;
  border-top: 1px solid var(--color-control-border);
  border-radius: 0 0 var(--radius-control) var(--radius-control);
  background: var(--color-panel-header-bg);
  color: var(--color-text-muted);
}

.panel-footer-meta {
  display: inline-flex;
  align-items: center;
  gap: 0;
  min-width: 0;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 16px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.panel-footer-meta__item {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.panel-footer-meta__item + .panel-footer-meta__item {
  margin-left: 6px;
  padding-left: 12px;
  border-left: 1px solid var(--color-control-border);
}

.monitoring-panel-footer {
  justify-content: flex-start;
}

.modal-footer-separator {
  width: 1px;
  align-self: stretch;
  min-height: var(--size-compact-control);
  margin: 0 5px;
  background: var(--color-control-border);
}

.modal--panel .modal__footer,
.modal--compact .modal__footer {
  margin-top: 0;
}

.integration-module-dialog > .modal__footer[data-ui-entity="panel-footer"] {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
}

.integration-module-dialog
  > .modal__footer[data-ui-entity="panel-footer"]
  > .module-ui-schema__footer-host {
  grid-column: 1;
}

.integration-module-dialog
  > .modal__footer[data-ui-entity="panel-footer"]
  > .module-ui-schema__footer-nav {
  grid-column: 2;
  justify-self: end;
}

.modal--panel .modal__footer > .button,
.modal--panel .modal__footer button.button,
.modal--compact .modal__footer > .button,
.modal--compact .modal__footer button.button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: var(--size-compact-control);
  min-height: var(--size-compact-control);
  padding: 4px 10px;
  line-height: 16px;
}

.modal__actions { display: flex; flex-wrap: wrap; gap: 10px; justify-content: flex-end; }
.help-text { color: var(--color-text-muted); font-size: 13px; line-height: 1.5; }
.field-row { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 10px; }

.workspace-overlay-backdrop {
  inset: var(--size-header-height) 0 var(--size-footer-height) 0;
  place-items: stretch;
  padding: var(--size-panel-gap);
}

.workspace-overlay-dialog {
  width: 100%;
  max-width: none;
  height: 100%;
  min-height: 0;
}

.cloud-sync-dialog {
  display: grid;
  grid-template-rows: minmax(0, 1fr);
  width: 100%;
  max-width: none;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  padding: 0;
  border: 0;
  background: var(--color-transparent);
}

.cloud-sync-backdrop {
  inset: var(--size-header-height) 0 var(--size-footer-height) 0;
  place-items: stretch;
  padding: var(--size-panel-gap);
}

.cloud-sync-dialog__body {
  display: grid;
  grid-template-rows: minmax(0, 1fr);
  gap: 8px;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.cloud-sync-refresh-button {
  padding: 4px 10px;
}

.cloud-sync-footer {
  align-items: center;
  justify-content: flex-end;
}

.cloud-sync-footer-status,
.cloud-sync-footer-actions {
  display: flex;
  align-items: center;
  min-width: 0;
}

.cloud-sync-footer-status {
  gap: 6px;
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 16px;
}

.cloud-sync-footer-actions {
  gap: 8px;
}

.cloud-sync-footer-status--static {
  cursor: default;
  user-select: none;
}

.cloud-sync-footer-status > * + * {
  margin-left: 6px;
  padding-left: 12px;
  border-left: 1px solid var(--color-control-border);
}

.cloud-sync-footer-status > .footer-watcher-status__indicator {
  margin-left: 0;
  padding-left: 0;
  border-left: 0;
}

.cloud-sync-footer-status__value {
  min-width: 0;
  overflow: hidden;
  color: var(--color-text-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cloud-sync-footer-identifier {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  max-width: min(460px, 42vw);
  cursor: pointer;
  user-select: none;
}

.cloud-sync-footer-auth {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  max-width: min(260px, 24vw);
  user-select: none;
}

.cloud-sync-footer-identifier__label {
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 16px;
  white-space: nowrap;
}

.cloud-sync-footer-identifier__value {
  min-width: 0;
  overflow: hidden;
  color: var(--color-text-strong);
  font-size: 12px;
  font-weight: 600;
  line-height: 16px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cloud-sync-footer-quota {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  white-space: nowrap;
}

.cloud-sync-footer-quota__label {
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 16px;
}

.cloud-sync-footer-quota__value {
  color: var(--color-text-strong);
  font-size: 12px;
  font-weight: 600;
  line-height: 16px;
}

.cloud-sync-footer-quota__value--exhausted {
  color: var(--color-danger);
}

.cloud-sync-status-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: 8px;
}

.cloud-sync-status-row,
.cloud-sync-panel {
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-panel-bg);
}

.cloud-sync-status-row {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 10px;
  align-items: center;
  padding: 8px;
}

.cloud-sync-label,
.cloud-sync-meta-row dt {
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 16px;
}

.cloud-sync-value,
.cloud-sync-meta-row dd {
  min-width: 0;
  margin: 0;
  color: var(--color-text-strong);
  font-size: 13px;
  line-height: 17px;
  overflow-wrap: anywhere;
}

.cloud-sync-value--error {
  color: var(--color-danger);
}

.cloud-sync-panel {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  gap: 5px;
  min-height: 0;
  padding: 5px;
}

.cloud-sync-panel h3 {
  margin: 0;
  color: var(--color-text-strong);
  font-size: 17px;
  line-height: 20px;
}

.cloud-sync-panel__header {
  min-height: 30px;
}

.cloud-sync-panel-service {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
  min-width: 0;
  color: var(--color-text-muted);
}

.cloud-sync-panel-service .footer-watcher-status__indicator {
  flex: 0 0 auto;
}

.cloud-sync-panel-service__text {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cloud-sync-panel__body {
  display: grid;
  gap: 8px;
  min-height: 0;
  overflow: hidden;
}

.cloud-sync-download-panel__body {
  grid-template-rows: minmax(0, 1fr) minmax(0, 1fr);
}

.cloud-sync-download-subpanel {
  display: grid;
  gap: 7px;
  min-height: 0;
  padding: 7px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-panel-bg);
  overflow: hidden;
}

.cloud-sync-download-apps-subpanel {
  grid-template-rows: auto auto minmax(0, 1fr);
}

.cloud-sync-download-rows-subpanel {
  grid-template-rows: auto auto minmax(0, 1fr);
}

.cloud-sync-upload-panel__body {
  grid-template-rows: auto minmax(0, 1fr);
}

.cloud-sync-upload-auth-subpanel,
.cloud-sync-upload-publications-subpanel {
  display: grid;
  gap: 7px;
  min-height: 0;
  padding: 7px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-panel-bg);
  overflow: hidden;
}

.cloud-sync-upload-auth-subpanel {
  grid-template-rows: auto auto;
}

.cloud-sync-upload-publications-subpanel {
  grid-template-rows: auto minmax(0, 1fr);
}

.cloud-sync-publications-table {
  min-height: 0;
}

.cloud-sync-publications-title {
  color: var(--color-text-strong);
  font-weight: 700;
  min-height: 20px;
}

.cloud-sync-publications-data-table {
  table-layout: fixed;
  min-width: 860px;
}

.cloud-sync-publications-data-table th {
  white-space: nowrap;
}

.cloud-sync-publications-data-table th:nth-child(1),
.cloud-sync-publications-data-table td:nth-child(1) {
  width: calc(100% - 520px);
  min-width: 300px;
}

.cloud-sync-publications-data-table th:nth-child(2),
.cloud-sync-publications-data-table td:nth-child(2),
.cloud-sync-publications-data-table th:nth-child(3),
.cloud-sync-publications-data-table td:nth-child(3),
.cloud-sync-publications-data-table th:nth-child(4),
.cloud-sync-publications-data-table td:nth-child(4),
.cloud-sync-publications-data-table th:nth-child(5),
.cloud-sync-publications-data-table td:nth-child(5) {
  width: 130px;
  text-align: left;
}

.cloud-sync-panel__footer {
  display: flex;
  align-items: center;
  gap: 6px;
  height: var(--size-panel-footer-height);
  min-height: var(--size-panel-footer-height);
  max-height: var(--size-panel-footer-height);
  min-width: 0;
  margin: 0 -5px -5px;
  padding: 6px 8px;
  overflow: hidden;
  border-top: 1px solid var(--color-control-border);
  border-radius: 0 0 var(--radius-control) var(--radius-control);
  background: var(--color-panel-header-bg);
  color: var(--color-text-muted);
}

.cloud-sync-panel__footer > * + * {
  margin-left: 6px;
  padding-left: 12px;
  border-left: 1px solid var(--color-control-border);
}

.cloud-sync-panel__footer-progress {
  flex: 0 1 240px;
  width: 240px;
  min-width: 0;
  height: 18px;
  align-self: center;
}

.cloud-sync-panel__footer-progress .progress-bar--compact .progress-bar__track {
  height: 8px;
  border-radius: 4px;
}

.cloud-sync-panel__footer-progress .progress-bar--compact .progress-bar__segment,
.cloud-sync-panel__footer-progress .progress-bar--compact .progress-bar__remaining {
  height: 8px;
  border-radius: 0;
}

.cloud-sync-panel__footer > .cloud-sync-panel__action {
  flex: 0 0 auto;
  margin-left: auto;
  min-width: 96px;
  height: 28px;
  padding: 0 14px;
}

.cloud-sync-panel__footer > * + .cloud-sync-panel__action {
  margin-left: auto;
}

.cloud-sync-panel__footer-actions {
  display: flex;
  gap: 6px;
  margin-left: auto;
  padding-left: 12px;
  border-left: 1px solid var(--color-control-border);
}

.cloud-sync-panel__footer-actions .cloud-sync-panel__action {
  flex: 0 0 auto;
  min-width: 96px;
  height: 28px;
  padding: 0 14px;
}

.cloud-sync-meta-list {
  display: grid;
  gap: 4px;
  margin: 0;
}

.cloud-sync-meta-row {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: 8px;
}

.cloud-sync-actions {
  justify-content: flex-start;
}

.cloud-sync-auth-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  min-height: 30px;
}

.cloud-sync-auth-user {
  min-width: 0;
  overflow: hidden;
  color: var(--color-text-strong);
  font-size: 13px;
  font-weight: 600;
  line-height: 17px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cloud-sync-app-list {
  min-height: 0;
  max-height: 184px;
  color: var(--color-text-muted);
  font-size: 13px;
  line-height: 17px;
}

.cloud-sync-download-panel__body .cloud-sync-app-list {
  max-height: none;
}

.cloud-sync-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
}

.cloud-sync-app-table {
  min-width: 700px;
}

.cloud-sync-app-table th:nth-child(1),
.cloud-sync-app-table td:nth-child(1) {
  width: 30%;
}

.cloud-sync-app-table th:nth-child(2),
.cloud-sync-app-table td:nth-child(2) {
  width: 30%;
}

.cloud-sync-app-table th:nth-child(3),
.cloud-sync-app-table td:nth-child(3) {
  width: 72px;
}

.cloud-sync-app-table th:nth-child(4),
.cloud-sync-app-table td:nth-child(4) {
  width: auto;
}

.cloud-sync-app-table th:nth-child(5),
.cloud-sync-app-table td:nth-child(5) {
  width: 32px;
  padding-left: 6px;
  padding-right: 6px;
  text-align: center;
}

.cloud-sync-app-table td:nth-child(5) .table-actions {
  justify-content: center;
}

.cloud-sync-app-count-field {
  text-align: right;
}

.cloud-sync-staging-data-table {
  min-width: 1132px;
}

.cloud-sync-staging-data-table th:nth-child(1),
.cloud-sync-staging-data-table td:nth-child(1) {
  width: 140px;
}

.cloud-sync-staging-data-table th:nth-child(4),
.cloud-sync-staging-data-table td:nth-child(4) {
  width: 66px;
}

.cloud-sync-staging-data-table th:nth-child(5),
.cloud-sync-staging-data-table td:nth-child(5) {
  width: 62px;
}

.cloud-sync-staging-data-table th:nth-child(6),
.cloud-sync-staging-data-table td:nth-child(6) {
  width: 148px;
}

.cloud-sync-staging-data-table th:nth-child(7),
.cloud-sync-staging-data-table td:nth-child(7) {
  width: 64px;
}

.cloud-sync-staging-data-table th:nth-child(8),
.cloud-sync-staging-data-table td:nth-child(8) {
  width: 126px;
}

.cloud-sync-staging-data-table th:nth-child(9),
.cloud-sync-staging-data-table td:nth-child(9) {
  width: 32px;
  padding-left: 6px;
  padding-right: 6px;
  text-align: center;
}

.cloud-sync-staging-data-table th:nth-child(10),
.cloud-sync-staging-data-table td:nth-child(10) {
  width: 48px;
}

.cloud-sync-staging-data-table td:nth-child(10) .table-actions {
  justify-content: center;
}

.cloud-sync-app-row strong {
  color: var(--color-text-strong);
  font-weight: 600;
  overflow-wrap: anywhere;
}

.cloud-sync-app-field.path-field {
  color: var(--color-text);
}

.cloud-sync-app-row span {
  overflow-wrap: anywhere;
}

.cloud-sync-empty-cell {
  color: var(--color-text-muted);
}

.cloud-sync-filter-grid {
  display: grid;
  grid-template-columns: minmax(120px, 30fr) minmax(120px, 30fr) minmax(160px, 38fr) 108px var(--size-header-action-button);
  gap: 8px;
  align-items: end;
}

.cloud-sync-filter-block {
  display: inline-flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 2px;
  height: var(--size-header-action-button);
  min-width: 0;
}

.cloud-sync-filter-field-shell {
  width: 100%;
}

.cloud-sync-filter-field-shell > .input {
  height: var(--size-compact-control);
  min-height: var(--size-compact-control);
  padding: 3px 30px 3px 8px;
  font-size: 12px;
  line-height: 16px;
}

.cloud-sync-filter-select {
  width: 100%;
  height: var(--size-compact-control);
  min-height: var(--size-compact-control);
  padding: 3px 5px;
  font-size: 12px;
  line-height: 16px;
}

.cloud-sync-filter-block--visibility,
.cloud-sync-filter-block--visibility .cloud-sync-filter-select {
  min-width: 108px;
}

.cloud-sync-my-publications-button {
  align-self: end;
}

.cloud-sync-download-filter-grid {
  display: grid;
  grid-template-columns: minmax(120px, 1fr) minmax(120px, 1fr) minmax(90px, 130px) minmax(100px, 130px);
  gap: 7px;
  align-items: end;
}

.cloud-sync-nickname-block {
  display: grid;
  grid-template-columns: minmax(180px, 360px) auto;
  gap: 3px 8px;
  align-items: start;
}

.cloud-sync-nickname-row {
  display: grid;
  grid-template-columns: minmax(180px, 360px) auto;
  gap: 8px;
  align-items: center;
  grid-column: 1 / -1;
}

.cloud-sync-nickname-status {
  grid-column: 1;
  min-height: 16px;
  font-size: 12px;
  line-height: 16px;
  white-space: nowrap;
}

.cloud-sync-private-row {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 22px;
  color: var(--color-text-primary);
  font-size: 12px;
  line-height: 16px;
}

.cloud-sync-nickname-status--invalid {
  color: var(--color-danger);
}

.cloud-sync-nickname-status--taken {
  color: var(--color-warning);
}

.cloud-sync-nickname-status--accepted {
  color: var(--color-success);
}

.cloud-sync-staging-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}

.cloud-sync-staging-table {
  min-height: 0;
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 16px;
}

.cloud-sync-app-row--loading td {
  animation: cloud-sync-row-loading 900ms ease-in-out infinite alternate;
}

.cloud-sync-staging-row td {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@keyframes cloud-sync-row-loading {
  from {
    background: var(--color-control-bg);
  }
  to {
    background: var(--color-app-row-enabled-bg);
  }
}

.information-dialog {
  width: min(980px, calc(100vw - 48px));
  height: min(760px, calc(100vh - 48px));
}

.information-dialog__body {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 8px;
  min-height: 0;
  overflow: hidden;
}

.info-panel {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  min-height: 0;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-panel-bg);
  overflow: hidden;
}

.info-panel__header,
.info-panel__footer {
  padding: 6px 8px;
  background: var(--color-panel-header-bg);
}

.info-panel__header {
  border-bottom: 1px solid var(--color-control-border);
}

.info-panel__footer {
  display: flex;
  align-items: center;
  height: var(--size-panel-footer-height);
  min-height: var(--size-panel-footer-height);
  max-height: var(--size-panel-footer-height);
  overflow: hidden;
  border-top: 1px solid var(--color-control-border);
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 15px;
}

.info-panel__footer--success {
  color: var(--color-success);
}

.info-panel__header h3 {
  margin: 0;
  color: var(--color-text-strong);
  font-size: 15px;
  line-height: 18px;
}

.info-panel__body {
  min-height: 0;
  padding: 8px;
}

.info-panel__body--details {
  display: grid;
  min-height: 0;
  overflow: hidden;
}

.info-details-subpanel {
  display: grid;
  grid-template-rows: minmax(0, 1fr);
  min-height: 0;
  overflow: hidden;
}

.info-details-scroll {
  display: grid;
  gap: 8px;
  align-content: start;
  min-height: 0;
  overflow: auto;
  padding-inline-end: var(--size-scrollbar-content-gutter);
  scrollbar-gutter: stable;
}

.info-about-grid {
  display: grid;
  grid-template-columns: 132px minmax(0, 1fr);
  gap: 8px;
}

.info-subpanel {
  min-width: 0;
  padding: 8px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-panel-subtle-bg);
}

.info-group {
  display: grid;
  gap: 5px;
  min-width: 0;
}

.info-logo-panel {
  display: grid;
  place-items: center;
  width: 132px;
  height: 132px;
}

.info-logo {
  width: 112px;
  height: 112px;
  object-fit: contain;
}

.info-meta-list {
  display: grid;
  gap: 6px;
  margin: 0;
}

.info-meta-row {
  display: grid;
  grid-template-columns: 150px minmax(0, 1fr);
  gap: 8px;
  align-items: baseline;
  min-width: 0;
}

.info-meta-row dt {
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 600;
}

.info-meta-row dd {
  min-width: 0;
  margin: 0;
  color: var(--color-text);
  font-size: 13px;
  line-height: 18px;
  overflow-wrap: anywhere;
}

.info-group h4 {
  margin: 0;
  color: var(--color-text-strong);
  font-size: 13px;
  line-height: 16px;
}

.info-group ul {
  display: grid;
  gap: 4px;
  margin: 0;
  padding: 6px 8px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-app-row-bg);
  list-style: none;
  color: var(--color-text);
  font-size: 12px;
  line-height: 17px;
}

.info-group li {
  min-width: 0;
  padding: 0;
}

.info-links-row {
  display: grid;
  gap: 4px;
  margin-top: 8px;
}

.info-link {
  color: var(--color-accent);
  text-decoration: none;
}

.info-link:hover {
  text-decoration: underline;
}

.integration-folder-field {
  display: grid;
  gap: 5px;
  margin-top: 20px;
}

.integration-folder-field-row { margin-top: 0; }
.integration-folder-path-help {
  margin: 0;
  padding-left: var(--size-control-label-offset);
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 16px;
}
.integration-download-actions { justify-content: center; }
.stack--tight { gap: 8px; }

.note-box {
  border-radius: var(--radius-panel);
  border: 1px solid var(--color-control-border);
  background: var(--color-panel-subtle-bg);
  padding: 12px 14px;
}

.integration-dialog-status-list {
  display: grid;
  gap: 5px;
  padding: 5px;
  border-top: 1px solid var(--color-control-border);
  border-bottom: 1px solid var(--color-control-border);
  background: var(--color-app-list-bg);
}

.integration-dialog-status-row {
  display: grid;
  grid-template-columns: minmax(110px, auto) minmax(0, 1fr);
  gap: 8px;
  align-items: center;
  min-height: 30px;
  padding: 5px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-app-row-bg);
}

.integration-dialog-status-row--progress {
  grid-template-columns: minmax(0, 1fr) auto;
  background: var(--color-app-row-enabled-bg);
}

.integration-dialog-status-row__label {
  min-width: 0;
}

.integration-dialog-status-row__value,
.integration-dialog-status-text,
.integration-dialog-path__input {
  min-width: 0;
  overflow: hidden;
  color: var(--color-text);
  font-size: 12px;
  line-height: 18px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.integration-dialog-status-text--success {
  color: var(--color-text-strong);
}

.integration-dialog-status-text--warning {
  color: var(--color-warning);
}

.integration-dialog-status-text--danger {
  color: var(--color-danger);
}

.integration-dialog-progress-main {
  min-width: 0;
}

.integration-dialog-path__input {
  color: var(--color-text);
}

.profile-export-dialog {
  width: 100%;
  max-width: none;
  height: 100%;
  min-height: 0;
}

.profile-export-advanced-dialog {
  display: grid;
  width: min(940px, calc(100vw - 48px));
  height: min(650px, calc(100vh - 48px));
  grid-template-rows: auto minmax(0, 1fr) auto;
}

.profile-export-section {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 10px;
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.profile-export-dialog .modal__body {
  flex: 1 1 auto;
  min-height: 0;
  overflow: hidden;
}

.profile-export-advanced-dialog .modal__body {
  height: 100%;
  min-height: 0;
  overflow: hidden;
}

.profile-export-advanced-body {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  grid-template-rows: auto auto minmax(0, 1fr);
  gap: 10px;
  height: 100%;
  min-width: 0;
  min-height: 0;
}

.profile-export-advanced-intro {
  grid-column: 1 / -1;
  padding-left: var(--size-control-label-offset);
}

.profile-export-advanced-panel {
  display: grid;
  gap: 8px;
  align-content: start;
  min-width: 0;
  min-height: 0;
  padding: 8px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-app-list-bg);
}

.profile-export-advanced-panel--settings {
  grid-row: 2;
  grid-template-rows: auto auto auto auto minmax(0, 1fr);
  align-content: stretch;
  overflow: hidden;
}

.profile-export-advanced-panel--uncovered {
  grid-row: 3;
  grid-template-rows: auto minmax(0, 1fr);
  align-content: stretch;
  overflow: hidden;
}

.profile-export-advanced-panel--uncovered > .profile-export-list {
  min-height: 0;
  margin: 0;
  overflow: auto;
}

.profile-export-advanced-panel--uncovered > #profile-export-advanced-wizard-uncovered {
  min-height: 0;
  overflow: auto;
}

.profile-export-advanced-option {
  display: grid;
  gap: 4px;
  min-width: 0;
  padding: 6px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-app-row-bg);
}

.profile-export-advanced-option > .profile-export-switch-row {
  justify-self: stretch;
  justify-content: space-between;
}

.profile-export-advanced-option > .control-help-text {
  padding-left: var(--size-control-label-offset);
}

.profile-export-field--domains {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  min-height: 0;
}

.profile-export-field--template {
  display: grid;
  gap: 4px;
  min-width: 0;
}

.profile-export-field--manual-domains {
  min-height: 0;
}

.profile-export-domains-input {
  width: 100%;
  height: 100%;
  min-height: 0;
  line-height: 18px;
  overflow: auto;
  resize: none;
}

.profile-export-domain-field-header {
  display: flex;
  align-items: end;
  justify-content: space-between;
  gap: 5px;
  min-width: 0;
}

.profile-export-domain-field-header > .value-label {
  padding-left: var(--size-control-label-offset);
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 600;
  line-height: 16px;
}

.path-input-shell--textarea {
  height: 100%;
}

.path-input-shell--textarea > .profile-export-domains-input {
  padding-right: 30px;
}

.tabs {
  display: grid;
  gap: 0;
  min-width: 0;
}

.tabs__list {
  display: flex;
  flex-wrap: wrap;
  gap: 0;
  align-items: flex-end;
  min-width: 0;
  width: max-content;
  max-width: 100%;
  justify-self: start;
  border: 1px solid var(--color-control-border);
  border-bottom: 0;
  border-radius: var(--radius-control) var(--radius-control) 0 0;
  background: var(--color-panel-header-bg);
}

.tabs__tab {
  position: relative;
  appearance: none;
  min-height: 32px;
  padding: 6px 12px 8px;
  border: 0;
  border-right: 1px solid var(--color-control-border);
  border-radius: 0;
  background: var(--color-panel-header-bg);
  color: var(--color-text);
  font-size: 12px;
  font-weight: 700;
  line-height: 16px;
  text-align: left;
  cursor: pointer;
}

.tabs__tab:hover {
  background: var(--color-control-hover-bg);
}

.tabs__tab--active {
  background: var(--color-panel-header-bg);
  color: var(--color-text-strong);
}

.tabs__tab--active::after {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 3px;
  background: var(--color-accent-border);
}

.tabs__body {
  display: grid;
  gap: 10px;
  min-width: 0;
  height: auto;
  min-height: 0;
  max-height: none;
  overflow: visible;
  padding: 10px;
  border: 1px solid var(--color-control-border);
  border-radius: 0 0 var(--radius-control) var(--radius-control);
  background: var(--color-panel-header-bg);
}

.tabs__panel {
  display: grid;
  gap: 10px;
  min-width: 0;
  align-content: start;
}

.module-ui-schema__tabs {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
  min-height: 0;
  box-sizing: border-box;
  overflow: hidden;
}

.module-ui-schema__tabs-shell {
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  box-sizing: border-box;
  overflow: hidden;
}

.module-ui-schema__tabs-body {
  display: grid;
  align-content: start;
  gap: 8px;
  flex: 1 1 auto;
  min-width: 0;
  min-height: 0;
  box-sizing: border-box;
  overflow: hidden;
}

.module-ui-schema__tabs-body.module-ui-schema--scroll-x {
  overflow-x: auto;
  overflow-y: hidden;
  max-height: min(360px, 70vh);
}

.module-ui-schema__tabs-body.module-ui-schema--scroll-y {
  overflow-x: hidden;
  overflow-y: auto;
  max-height: min(360px, 70vh);
}

.module-ui-schema__tabs-body.module-ui-schema--scroll-both {
  overflow: auto;
  max-height: min(360px, 70vh);
}

.module-ui-schema__tabs-body.module-ui-schema--scroll-y,
.module-ui-schema__tabs-body.module-ui-schema--scroll-both {
  padding-bottom: calc(10px + var(--size-compact-control));
  scroll-padding-bottom: calc(10px + var(--size-compact-control));
}

.profile-export-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(180px, 240px);
  gap: 10px;
}

.profile-export-grid--attach {
  row-gap: 5px;
}

.profile-export-grid--attach > .profile-export-field:first-child {
  grid-column: 1 / -1;
}

.profile-export-grid--attach > .profile-export-field:nth-child(2) {
  grid-column: 1;
}

.profile-export-grid--attach > .profile-export-field:nth-child(3) {
  grid-column: 2;
}

.profile-export-grid--attach > .profile-export-tab-help {
  margin-top: 5px;
}

.profile-export-path-row {
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 5px;
}

.profile-export-path-row .path-input-shell > .input {
  height: var(--size-compact-control);
  border-radius: var(--radius-control);
  padding: 4px 30px 4px 5px;
}

.profile-export-path-row > .button,
.profile-export-field > .input-box.input,
.profile-export-field > .input-box.select {
  height: var(--size-compact-control);
  min-height: var(--size-compact-control);
}

.profile-export-field--domains > .input-box.input.profile-export-domains-input {
  height: 100%;
  min-height: 0;
}

.profile-export-grid--single {
  grid-template-columns: minmax(0, 1fr);
}

.profile-export-field {
  display: grid;
  gap: 5px;
  min-width: 0;
}

.profile-export-field > .value-label {
  padding-left: var(--size-control-label-offset);
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 600;
  line-height: 16px;
}

.profile-export-field--wide {
  grid-column: 1 / -1;
}

.profile-export-switch-row {
  display: flex;
  align-items: center;
  gap: 8px;
  justify-self: start;
  min-width: 0;
  min-height: var(--size-compact-control);
}

.profile-export-switch-row__label {
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 600;
  line-height: var(--size-switch-height);
}

.control-help-text {
  min-width: 0;
  margin: 0;
  padding: 0;
  background: var(--color-transparent);
  color: var(--color-text-muted);
  font-size: 12px;
  font-weight: 400;
  line-height: 16px;
}

.profile-export-tab-help {
  grid-column: 1 / -1;
  align-self: end;
  justify-self: stretch;
  padding-left: var(--size-control-label-offset);
}

.profile-export-preview-panel {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  gap: 10px;
  min-width: 0;
  min-height: 0;
  padding: 10px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-panel-header-bg);
  overflow: hidden;
}

.profile-export-preview-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  justify-content: flex-start;
  min-width: 0;
}

.profile-export-preview-actions__right {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-left: auto;
}

.profile-export-footer {
  justify-content: space-between;
}

.profile-export-feedback,
.profile-export-preview {
  display: grid;
  gap: 8px;
  min-width: 0;
}

.profile-export-preview {
  height: 100%;
  min-height: 0;
  max-height: none;
  overflow: auto;
  align-content: start;
  justify-items: start;
  padding: 5px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-app-list-bg);
}

.profile-export-preview__summary,
.profile-export-preview__group {
  display: grid;
  gap: 4px;
  min-width: 0;
  max-width: 100%;
  justify-items: start;
}

.profile-export-preview__section {
  display: grid;
  gap: 2px;
  justify-items: start;
}

.profile-export-preview__section + .profile-export-preview__section {
  margin-top: 10px;
}

.profile-export-preview__title {
  color: var(--color-text-strong);
  font-size: 12px;
  font-weight: 700;
  line-height: 16px;
}

.profile-export-preview__row {
  display: grid;
  grid-template-columns: 230px minmax(0, 1fr);
  gap: 6px;
  align-items: center;
  min-height: 24px;
  width: max-content;
  max-width: 100%;
  padding: 3px 5px;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-panel-bg);
  color: var(--color-text);
  font-size: 12px;
  line-height: 16px;
}

.profile-export-preview__metric {
  min-height: 18px;
  padding: 0;
  border: 0;
  background: var(--color-transparent);
}

.profile-export-preview__metric-value {
  color: var(--color-text);
  font-weight: 700;
  white-space: nowrap;
}

.profile-export-preview__metric-value--ready {
  color: var(--color-success);
}

.profile-export-preview__metric-value--excluded {
  color: var(--color-accent-border);
}

.profile-export-preview__metric .profile-export-preview__muted {
  color: var(--color-text);
  white-space: nowrap;
}

.profile-export-preview__muted {
  color: var(--color-text-muted);
}

.profile-export-preview__note {
  margin: 0;
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 16px;
}

.profile-export-preview__group--warning > strong,
.profile-export-preview__title--warning {
  color: var(--color-warning);
}

.profile-export-list {
  display: grid;
  gap: 4px;
  margin: 0;
  padding-left: 18px;
  color: var(--color-text);
  font-size: 12px;
  line-height: 18px;
}

.profile-export-list--plain {
  color: var(--color-text);
}

.progress-bar {
  display: grid;
  gap: 5px;
  min-width: 0;
}

.progress-bar__header {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  align-items: center;
  min-width: 0;
  color: var(--color-accent);
  font-size: 12px;
  font-weight: 700;
  line-height: 18px;
}

.progress-bar__label,
.progress-bar__meta {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.progress-bar__meta {
  color: var(--color-accent);
}

.progress-bar__track {
  position: relative;
  height: 6px;
  overflow: hidden;
  border: 1px solid var(--color-control-border);
  border-radius: var(--radius-control);
  background: var(--color-control-bg);
}

.progress-bar__segments {
  position: absolute;
  inset: 0;
  display: flex;
}

.progress-bar__segment {
  height: 100%;
  border-radius: 0;
}

.progress-bar__segment--accent {
  background: var(--color-progress-accent);
}

.progress-bar__segment--success {
  background: var(--color-progress-success);
}

.progress-bar__segment--warning {
  background: var(--color-progress-warning);
}

.progress-bar__segment--danger {
  background: var(--color-progress-danger);
}

.progress-bar__segment--rust {
  background: var(--color-progress-rust);
}

.progress-bar__segment--muted {
  background: var(--color-progress-muted);
}

.progress-bar__remaining {
  position: absolute;
  top: 0;
  right: 0;
  height: 100%;
  border-radius: 0;
  background: var(--color-control-bg);
}

.progress-bar__stages {
  display: grid;
  gap: 8px;
  min-width: 0;
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 14px;
}

.progress-bar__stages span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.progress-bar__stages span:last-child {
  text-align: right;
}

@media (max-width: 820px) {
  .shell__content { padding: 8px; }
  .hero h1 { font-size: 22px; }
  .hero { flex-direction: column; }
  .controls { grid-template-columns: 1fr; }
  .field-row { grid-template-columns: 1fr; }
}
"#;

pub const TOOLTIP_SCRIPT: &str = r#"
(() => {
  if (window.__netstitchTooltipLayerInstalled) {
    return;
  }
  window.__netstitchTooltipLayerInstalled = true;

  const tooltip = document.createElement("div");
  tooltip.className = "netstitch-floating-tooltip";
  tooltip.setAttribute("role", "tooltip");
  document.body.appendChild(tooltip);

  let activeTarget = null;

  function textFor(target) {
    return target?.getAttribute("data-tooltip") || "";
  }

  function hideTooltip() {
    activeTarget = null;
    tooltip.classList.remove("netstitch-floating-tooltip--visible");
  }

  function positionTooltip() {
    if (!activeTarget || !textFor(activeTarget)) {
      hideTooltip();
      return;
    }

    const rect = activeTarget.getBoundingClientRect();
    const gap = 6;
    const padding = 8;
    const tooltipRect = tooltip.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;
    const align = activeTarget.getAttribute("data-tooltip-align") || "center";

    let left;
    if (align === "start") {
      left = rect.left;
    } else if (align === "end") {
      left = rect.right - tooltipRect.width;
    } else {
      left = rect.left + rect.width / 2 - tooltipRect.width / 2;
    }

    if (tooltipRect.width < viewportWidth - padding * 2) {
      left = Math.max(padding, Math.min(left, viewportWidth - tooltipRect.width - padding));
    } else {
      left = padding;
    }

    let top = rect.top - tooltipRect.height - gap;
    if (top < padding) {
      top = rect.bottom + gap;
    }
    if (top + tooltipRect.height > viewportHeight - padding) {
      top = Math.max(padding, viewportHeight - tooltipRect.height - padding);
    }

    tooltip.style.left = `${Math.round(left)}px`;
    tooltip.style.top = `${Math.round(top)}px`;
  }

  function showTooltip(target) {
    const text = textFor(target);
    if (!text) {
      hideTooltip();
      return;
    }
    activeTarget = target;
    tooltip.textContent = text;
    tooltip.classList.toggle(
      "netstitch-floating-tooltip--pre",
      target.classList.contains("footer-message-text") || text.includes("\n")
    );
    tooltip.classList.add("netstitch-floating-tooltip--visible");
    positionTooltip();
  }

  document.addEventListener("pointerover", (event) => {
    const target = event.target.closest("[data-tooltip]");
    if (target) {
      showTooltip(target);
    }
  });

  document.addEventListener("pointerout", (event) => {
    if (activeTarget && !activeTarget.contains(event.relatedTarget)) {
      hideTooltip();
    }
  });

  document.addEventListener("focusin", (event) => {
    const target = event.target.closest("[data-tooltip]");
    if (target) {
      showTooltip(target);
    }
  });

  document.addEventListener("focusout", (event) => {
    if (activeTarget && activeTarget === event.target) {
      hideTooltip();
    }
  });

  function setObservationSelectionModifier(active) {
    document.body.classList.toggle("netstitch-observation-selection-modifier", active);
  }

  document.addEventListener("keydown", (event) => {
    setObservationSelectionModifier(event.ctrlKey || event.shiftKey);
  }, true);

  document.addEventListener("keyup", (event) => {
    setObservationSelectionModifier(event.ctrlKey || event.shiftKey);
  }, true);

  document.addEventListener("pointerover", (event) => {
    setObservationSelectionModifier(event.ctrlKey || event.shiftKey);
  }, true);

  window.addEventListener("blur", () => {
    setObservationSelectionModifier(false);
  });

  window.addEventListener("scroll", positionTooltip, true);
  window.addEventListener("resize", positionTooltip);
})();
"#;

pub(crate) fn initial_window_background_rgba() -> (u8, u8, u8, u8) {
    (24, 24, 24, 255)
}

#[cfg(test)]
mod tests {
    use super::{GLOBAL_STYLE, TOOLTIP_SCRIPT};

    #[test]
    fn theme_uses_flat_workbench_palette_variables() {
        assert!(GLOBAL_STYLE.contains("--color-workbench-bg: #181818;"));
        assert!(GLOBAL_STYLE.contains("--color-window-chrome: #181818;"));
        assert!(GLOBAL_STYLE.contains("--color-panel-bg: #252526;"));
        assert!(GLOBAL_STYLE.contains("--color-panel-header-bg: #2d2d2d;"));
        assert!(GLOBAL_STYLE.contains("--color-app-list-bg: #1e1e1e;"));
        assert!(GLOBAL_STYLE.contains("--color-app-row-bg: #2b2b2b;"));
        assert!(GLOBAL_STYLE.contains("--color-app-row-enabled-bg: #2b3338;"));
        assert!(GLOBAL_STYLE.contains("--color-app-skeleton-bg: #242424;"));
        assert!(GLOBAL_STYLE.contains("--color-app-skeleton-pulse-bg: #303030;"));
        assert!(GLOBAL_STYLE.contains("--color-tooltip-bg: #303030;"));
        assert!(GLOBAL_STYLE.contains("--color-indicator-idle: #3a3a3a;"));
        assert!(GLOBAL_STYLE.contains("--color-scrollbar-track: #1e1e1e;"));
        assert!(GLOBAL_STYLE.contains("--color-scrollbar-thumb: #424242;"));
        assert!(GLOBAL_STYLE.contains("--color-scrollbar-thumb-hover: #525252;"));
        assert!(GLOBAL_STYLE.contains("--color-accent: #007acc;"));
        assert!(GLOBAL_STYLE.contains("--color-switch-knob-on: #eeeeee;"));
        assert!(GLOBAL_STYLE.contains("--color-text: #cccccc;"));
        assert!(GLOBAL_STYLE.contains("--radius-panel: 3px;"));
        assert!(GLOBAL_STYLE.contains("--size-control-label-offset: 0px;"));
        assert!(GLOBAL_STYLE.contains("--size-scrollbar: 8px;"));
        assert!(GLOBAL_STYLE.contains("--size-panel-footer-height: 43px;"));
        assert!(GLOBAL_STYLE.contains("--size-tracked-apps-column-max: 640px;"));
        assert!(GLOBAL_STYLE.contains(
            "--size-tracked-apps-column-min: calc(var(--size-tracked-apps-column-max) * 0.4);"
        ));
        assert!(GLOBAL_STYLE.contains("--z-tooltip: 10000;"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-window-chrome);"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-panel-header-bg);"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-panel-bg);"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-control-bg);"));
        assert!(GLOBAL_STYLE.contains(".header-switch-row"));
        assert!(GLOBAL_STYLE.contains(".header-switch-stack"));
        assert!(GLOBAL_STYLE.contains(".header-switch-stack .header-switch-row"));
        assert!(GLOBAL_STYLE.contains("width: min(126px, 10vw);"));
        assert!(GLOBAL_STYLE.contains(".button__icon--information"));
        assert!(GLOBAL_STYLE.contains(".button--icon.header-action-button--update-available"));
        assert!(GLOBAL_STYLE.contains("padding: 0 var(--size-control-label-offset);"));
        assert!(GLOBAL_STYLE.contains(
            ".profile-export-field > .value-label {\n  padding-left: var(--size-control-label-offset);\n  color: var(--color-text-muted);\n  font-size: 11px;\n  font-weight: 600;"
        ));
        assert!(GLOBAL_STYLE.contains(".shell--controls-disabled [data-ui-action]"));
        assert!(GLOBAL_STYLE.contains(".shell--cloud-overlay-active .shell__content {"));
        assert!(GLOBAL_STYLE.contains(
            ".shell--cloud-overlay-active .app-chrome--header button:not(#netstitch-ui-confirm-filtered-button):not(#netstitch-ui-unconfirm-filtered-button):not(#netstitch-ui-clear-monitoring-button):not(#netstitch-ui-open-cloud-import-button):not(#netstitch-ui-open-cloud-export-button)"
        ));
        assert!(
            !GLOBAL_STYLE.contains(
                ".shell--cloud-overlay-active .app-chrome--header .header-filter-block,\n"
            )
        );
        assert!(
            !GLOBAL_STYLE.contains(".shell--cloud-overlay-active .app-chrome--footer"),
            "cloud overlay must not disable main footer controls"
        );
        assert!(
            GLOBAL_STYLE
                .contains(".shell--cloud-overlay-active #netstitch-ui-open-cloud-import-button,")
        );
        assert!(
            GLOBAL_STYLE
                .contains(".shell--cloud-overlay-active #netstitch-ui-open-cloud-export-button,")
        );
        assert!(
            GLOBAL_STYLE
                .contains(".shell--cloud-overlay-active #netstitch-ui-confirm-filtered-button,")
        );
        assert!(
            GLOBAL_STYLE
                .contains(".shell--cloud-overlay-active #netstitch-ui-unconfirm-filtered-button,")
        );
        assert!(
            GLOBAL_STYLE
                .contains(".shell--cloud-overlay-active #netstitch-ui-clear-monitoring-button,")
        );
        assert!(
            GLOBAL_STYLE.contains(
                ".shell--cloud-overlay-active .app-chrome--header .header-filter-block {"
            )
        );
        assert!(GLOBAL_STYLE.contains("cursor: not-allowed;"));
        assert!(GLOBAL_STYLE.contains("opacity: 0.55;"));
    }

    #[test]
    fn scrollbars_use_tracked_apps_contract_globally() {
        assert!(GLOBAL_STYLE.contains("* {\n  scrollbar-color: var(--color-scrollbar-thumb) var(--color-scrollbar-track);\n  scrollbar-width: thin;\n}"));
        assert!(GLOBAL_STYLE.contains("*::-webkit-scrollbar {\n  width: var(--size-scrollbar);\n  height: var(--size-scrollbar);\n}"));
        assert!(GLOBAL_STYLE.contains(
            "*::-webkit-scrollbar-track {\n  background: var(--color-scrollbar-track);\n}"
        ));
        assert!(GLOBAL_STYLE.contains("*::-webkit-scrollbar-thumb {\n  border-radius: var(--radius-control);\n  background: var(--color-scrollbar-thumb);\n}"));
        assert!(GLOBAL_STYLE.contains(
            "*::-webkit-scrollbar-thumb:hover {\n  background: var(--color-scrollbar-thumb-hover);\n}"
        ));
        assert!(GLOBAL_STYLE.contains("scrollbar-gutter: stable;"));
    }

    #[test]
    fn progress_bar_rounds_only_the_outer_track() {
        assert!(GLOBAL_STYLE.contains(
            ".progress-bar__track {\n  position: relative;\n  height: 6px;\n  overflow: hidden;\n  border: 1px solid var(--color-control-border);\n  border-radius: var(--radius-control);"
        ));
        assert!(
            GLOBAL_STYLE
                .contains(".progress-bar__segment {\n  height: 100%;\n  border-radius: 0;\n}")
        );
        assert!(GLOBAL_STYLE.contains(
            ".progress-bar__remaining {\n  position: absolute;\n  top: 0;\n  right: 0;\n  height: 100%;\n  border-radius: 0;"
        ));
        assert!(GLOBAL_STYLE.contains(
            ".cloud-sync-panel__footer-progress .progress-bar--compact .progress-bar__remaining {\n  height: 8px;\n  border-radius: 0;\n}"
        ));
        assert!(!GLOBAL_STYLE.contains(
            ".progress-bar__remaining {\n  position: absolute;\n  top: 0;\n  right: 0;\n  height: 100%;\n  border-radius: 4px;"
        ));
    }

    #[test]
    fn tabs_use_shared_header_palette_and_active_underline() {
        assert!(GLOBAL_STYLE.contains(".tabs {\n  display: grid;"));
        assert!(GLOBAL_STYLE.contains(".tabs__list {\n  display: flex;"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-panel-header-bg);"));
        assert!(GLOBAL_STYLE.contains(".tabs__tab {"));
        assert!(GLOBAL_STYLE.contains("border-right: 1px solid var(--color-control-border);"));
        assert!(GLOBAL_STYLE.contains(".tabs__tab--active::after"));
        assert!(GLOBAL_STYLE.contains("height: 3px;"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-accent-border);"));
        assert!(GLOBAL_STYLE.contains(".tabs__body {"));
        assert!(GLOBAL_STYLE.contains("overflow: visible;"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-panel-header-bg);"));
        assert!(GLOBAL_STYLE.contains(".control-help-text {"));
        assert!(GLOBAL_STYLE.contains(".profile-export-preview {"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-app-list-bg);"));
        assert!(GLOBAL_STYLE.contains(".profile-export-advanced-dialog {"));
        assert!(GLOBAL_STYLE.contains(".profile-export-advanced-dialog {\n  display: grid;"));
        assert!(GLOBAL_STYLE.contains("width: min(940px, calc(100vw - 48px));"));
        assert!(GLOBAL_STYLE.contains("grid-template-rows: auto minmax(0, 1fr) auto;"));
        assert!(GLOBAL_STYLE.contains(".profile-export-advanced-body {"));
        assert!(GLOBAL_STYLE.contains("grid-template-columns: minmax(0, 1fr);"));
        assert!(GLOBAL_STYLE.contains("grid-template-rows: auto auto minmax(0, 1fr);"));
        assert!(GLOBAL_STYLE.contains(".profile-export-advanced-panel {"));
        assert!(GLOBAL_STYLE.contains(".profile-export-advanced-option {"));
        assert!(GLOBAL_STYLE.contains(".profile-export-advanced-panel--settings {"));
        assert!(GLOBAL_STYLE.contains("grid-template-rows: auto auto auto auto minmax(0, 1fr);"));
        assert!(GLOBAL_STYLE.contains(".profile-export-advanced-panel--uncovered {"));
        assert!(GLOBAL_STYLE.contains(".profile-export-field--domains {"));
        assert!(GLOBAL_STYLE.contains(".profile-export-domains-input {"));
        assert!(GLOBAL_STYLE.contains(
            ".profile-export-field--domains > .input-box.input.profile-export-domains-input"
        ));
        assert!(GLOBAL_STYLE.contains("height: 100%;"));
        assert!(GLOBAL_STYLE.contains("resize: none;"));
    }

    #[test]
    fn information_dialog_uses_panel_layout_contract() {
        assert!(GLOBAL_STYLE.contains(".information-dialog {"));
        assert!(GLOBAL_STYLE.contains("width: min(980px, calc(100vw - 48px));"));
        assert!(GLOBAL_STYLE.contains(".information-dialog__body {"));
        assert!(GLOBAL_STYLE.contains("grid-template-rows: auto minmax(0, 1fr);"));
        assert!(GLOBAL_STYLE.contains(".info-panel {"));
        assert!(GLOBAL_STYLE.contains("grid-template-rows: auto minmax(0, 1fr) auto;"));
        assert!(GLOBAL_STYLE.contains(".info-about-grid {"));
        assert!(GLOBAL_STYLE.contains("grid-template-columns: 132px minmax(0, 1fr);"));
        assert!(GLOBAL_STYLE.contains(".info-panel__body--details {"));
        assert!(GLOBAL_STYLE.contains(".info-details-subpanel {"));
        assert!(GLOBAL_STYLE.contains("grid-template-rows: minmax(0, 1fr);"));
        assert!(GLOBAL_STYLE.contains(".info-details-scroll {"));
        assert!(GLOBAL_STYLE.contains("overflow: auto;"));
        assert!(GLOBAL_STYLE.contains("scrollbar-gutter: stable;"));
        assert!(GLOBAL_STYLE.contains(".info-group {"));
        assert!(GLOBAL_STYLE.contains(".info-group li {"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-app-row-bg);"));
        assert!(GLOBAL_STYLE.contains(".info-panel__footer--success {"));
        assert!(GLOBAL_STYLE.contains("color: var(--color-success);"));
    }

    #[test]
    fn footer_language_select_has_visible_overlay_label() {
        assert!(GLOBAL_STYLE.contains(".app-chrome--footer"));
        assert!(GLOBAL_STYLE.contains("justify-content: flex-start;"));
        assert!(GLOBAL_STYLE.contains(".app-footer-panel + .app-footer-panel"));
        assert!(GLOBAL_STYLE.contains("border-left: 1px solid var(--color-control-border);"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-transparent);"));
        assert!(GLOBAL_STYLE.contains(".footer-watcher-status"));
        assert!(GLOBAL_STYLE.contains(".footer-tool-status"));
        assert!(GLOBAL_STYLE.contains(".footer-web-server-status"));
        assert!(GLOBAL_STYLE.contains(".footer-network-status"));
        assert!(GLOBAL_STYLE.contains(".app-footer-message-panel"));
        assert!(GLOBAL_STYLE.contains("flex: 1 1 auto;"));
        assert!(GLOBAL_STYLE.contains(".app-footer-language-panel"));
        assert!(GLOBAL_STYLE.contains("margin-inline-start: 6px;"));
        assert!(GLOBAL_STYLE.contains(".footer-watcher-status__indicator"));
        assert!(GLOBAL_STYLE.contains("width: 14px;"));
        assert!(GLOBAL_STYLE.contains("height: 14px;"));
        assert!(GLOBAL_STYLE.contains("flex: 0 0 14px;"));
        assert!(GLOBAL_STYLE.contains("transform: translateY(0);"));
        assert!(GLOBAL_STYLE.contains(".footer-watcher-status__indicator--connected"));
        assert!(GLOBAL_STYLE.contains(".footer-watcher-status__indicator--disconnected"));
        assert!(GLOBAL_STYLE.contains(".footer-watcher-status__indicator--inactive"));
        assert!(GLOBAL_STYLE.contains(".footer-language-label"));
        assert!(GLOBAL_STYLE.contains(".language-select-shell"));
        assert!(GLOBAL_STYLE.contains(".language-select-control"));
        assert!(!GLOBAL_STYLE.contains(".app-chrome--footer .input-box"));
        assert!(GLOBAL_STYLE.contains(".language-select-control__label"));
        assert!(GLOBAL_STYLE.contains("text-align: center;"));
        assert!(!GLOBAL_STYLE.contains(".language-select-control__arrow"));
        assert!(GLOBAL_STYLE.contains(".language-select-menu"));
        assert!(GLOBAL_STYLE.contains("bottom: 100%;"));
        assert!(GLOBAL_STYLE.contains(".language-select-option"));
        assert!(!GLOBAL_STYLE.contains(".footer-language-select"));
        assert!(GLOBAL_STYLE.contains("position: absolute;"));
    }

    #[test]
    fn colors_are_declared_in_root_variables_not_inline_rules() {
        let after_root = GLOBAL_STYLE
            .split_once("}\n\n* { box-sizing: border-box; }")
            .map(|(_, rest)| rest)
            .expect("root block terminator");

        assert!(!after_root.contains(": #"));
        assert!(!after_root.contains("rgba("));
        assert!(!after_root.contains("linear-gradient("));
        assert!(!after_root.contains("radial-gradient("));
        assert!(!after_root.contains("box-shadow:"));
        assert!(!after_root.contains("drop-shadow("));
        assert!(!after_root.contains("background: transparent"));
    }

    #[test]
    fn tracked_apps_layout_contract_stays_compact() {
        assert!(
            GLOBAL_STYLE
                .contains("grid-template-rows: auto minmax(0, 1fr) var(--size-footer-height);")
        );
        assert!(GLOBAL_STYLE.contains("--size-panel-gap: 8px;"));
        assert!(GLOBAL_STYLE.contains("--size-panel-stack-gap: 8px;"));
        assert!(GLOBAL_STYLE.contains("--size-tracked-app-visible-rows: 4;"));
        assert!(GLOBAL_STYLE.contains("--size-tracked-app-list-height: calc((var(--size-tracked-app-height) * var(--size-tracked-app-visible-rows)) + (5px * (var(--size-tracked-app-visible-rows) - 1)) + 10px + 2px);"));
        assert!(GLOBAL_STYLE.contains("grid-template-columns: repeat(2, minmax(0, 1fr));"));
        assert!(GLOBAL_STYLE.contains("gap: var(--size-panel-gap);"));
        assert!(GLOBAL_STYLE.contains("grid-template-rows: auto minmax(0, 1fr);"));
        assert!(GLOBAL_STYLE.contains(
            "width: 100%;\n  height: 100%;\n  min-height: 0;\n  min-width: 0;\n  max-width: 100%;"
        ));
        assert!(
            GLOBAL_STYLE.contains(
                ".workspace > .column:first-child {\n  grid-column: 1;\n  grid-row: 1;\n  align-self: stretch;\n  height: 100%;\n  min-height: 0;"
            )
        );
        assert!(GLOBAL_STYLE.contains(".workspace > .column:nth-child(2) {\n  grid-column: 2;\n  grid-row: 1;\n  align-self: stretch;\n  height: 100%;\n  gap: var(--size-panel-stack-gap);\n  min-height: 0;"));
        assert!(GLOBAL_STYLE.contains(".workspace > .observations-card {\n  grid-column: 1 / -1;\n  grid-row: 2;\n  align-self: stretch;\n  height: 100%;\n  min-height: 0;"));
        assert!(GLOBAL_STYLE.contains(".tracked-apps-card {\n  border-radius: var(--radius-control);\n  display: grid;\n  grid-template-rows: auto minmax(0, 1fr) auto;\n  height: 100%;\n  min-height: 0;"));
        assert!(
            GLOBAL_STYLE
                .contains(".tracked-apps-card > .stack {\n  grid-template-rows: auto minmax(0, 1fr) auto;\n  min-height: 0;")
        );
        assert!(GLOBAL_STYLE.contains("height: var(--size-tracked-app-height);"));
        assert!(GLOBAL_STYLE.contains("height: var(--size-tracked-app-list-height);"));
        assert!(GLOBAL_STYLE.contains("--size-scrollbar-content-gutter: 1px;"));
        assert!(
            GLOBAL_STYLE.contains("padding: 5px var(--size-scrollbar-content-gutter) 5px 5px;")
        );
        assert!(GLOBAL_STYLE.contains("padding: 5px;"));
        assert!(GLOBAL_STYLE.contains(".card__header {\n  display: flex;"));
        assert!(GLOBAL_STYLE.contains("align-items: center;"));
        assert!(GLOBAL_STYLE.contains(".panel-header-meta"));
        assert!(GLOBAL_STYLE.contains(".panel-header-meta--success"));
        assert!(GLOBAL_STYLE.contains(".panel-header-meta--danger"));
        assert!(GLOBAL_STYLE.contains(".hero-labels"));
        assert!(GLOBAL_STYLE.contains(".header-label"));
        assert!(GLOBAL_STYLE.contains(".state-label"));
        assert!(GLOBAL_STYLE.contains(".button--warning"));
        assert!(!GLOBAL_STYLE.contains(".badge"));
        assert!(!GLOBAL_STYLE.contains(".chip"));
        assert!(GLOBAL_STYLE.contains(".integration-status-text"));
        assert!(GLOBAL_STYLE.contains("--size-integration-status-left-max: 520px;"));
        assert!(GLOBAL_STYLE.contains(".integration-status-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) auto;"));
        assert!(GLOBAL_STYLE.contains(".integration-status-layout--module-menu {"));
        assert!(GLOBAL_STYLE.contains(
            ".integration-status-layout--module-menu .integration-status__fields {\n  gap: 2px;"
        ));
        assert!(GLOBAL_STYLE.contains(".integration-status-layout--module-menu .integration-status__row {\n  min-height: 18px;"));
        assert!(GLOBAL_STYLE.contains("width: 100%;\n  max-width: 100%;"));
        assert!(GLOBAL_STYLE.contains(".integration-status__fields {\n  display: grid;\n  width: 100%;\n  max-width: min(100%, var(--size-integration-status-left-max));"));
        assert!(GLOBAL_STYLE.contains(".tracked-apps-toolbar-spacer {\n  min-height: 0;"));
        assert!(GLOBAL_STYLE.contains(".integration-status__action {\n  display: flex;\n  flex-direction: column;\n  align-items: stretch;\n  justify-content: flex-end;\n  gap: 5px;"));
        assert!(GLOBAL_STYLE.contains(".integration-status__action > .button {"));
        assert!(
            GLOBAL_STYLE
                .contains("border: 0;\n  border-bottom: 1px solid var(--color-control-border);")
        );
        assert!(
            GLOBAL_STYLE.contains(
                "grid-template-columns: var(--size-app-icon-slot) minmax(130px, 1fr) var(--size-switch-width);"
            )
        );
        assert!(GLOBAL_STYLE.contains("width: var(--size-app-icon-slot);"));
        assert!(GLOBAL_STYLE.contains("width: var(--size-app-icon-image);"));
        assert!(GLOBAL_STYLE.contains("object-fit: contain;"));
        assert!(GLOBAL_STYLE.contains("aspect-ratio: 1 / 1;"));
        assert!(GLOBAL_STYLE.contains("width: var(--size-close-button);"));
        assert!(GLOBAL_STYLE.contains("height: var(--size-close-button);"));
        assert!(GLOBAL_STYLE.contains("min-width: var(--size-close-button);"));
        assert!(GLOBAL_STYLE.contains("min-height: var(--size-close-button);"));
        assert!(GLOBAL_STYLE.contains("max-width: var(--size-close-button);"));
        assert!(GLOBAL_STYLE.contains("max-height: var(--size-close-button);"));
        assert!(GLOBAL_STYLE.contains("flex: 0 0 var(--size-close-button);"));
        assert!(GLOBAL_STYLE.contains("width: var(--size-switch-width);"));
        assert!(GLOBAL_STYLE.contains(".ignored-addresses-card"));
        assert!(GLOBAL_STYLE.contains(
            ".ignored-addresses-card {\n  flex: 0 0 auto;\n  min-height: 0;\n  align-self: stretch;"
        ));
        assert!(GLOBAL_STYLE.contains("--size-modules-panel-height: 78px;"));
        assert!(
            GLOBAL_STYLE.contains(".modules-card {\n  flex: 1 1 var(--size-modules-panel-height);")
        );
        assert!(GLOBAL_STYLE.contains("--color-module-activity-pulse: rgba(255, 255, 255, 0.34);"));
        assert!(GLOBAL_STYLE.contains("--size-module-button: 36px;"));
        assert!(GLOBAL_STYLE.contains("--size-module-button-pulse-gutter: 3px;"));
        assert!(GLOBAL_STYLE.contains("--size-module-button-slot: calc(var(--size-module-button) + (var(--size-module-button-pulse-gutter) * 2));"));
        assert!(GLOBAL_STYLE.contains(".integration-module-grid {\n  display: flex;"));
        assert!(GLOBAL_STYLE.contains("padding: var(--size-module-button-pulse-gutter) 6px calc(var(--size-module-button-pulse-gutter) + var(--size-scrollbar)) 0;"));
        assert!(
            GLOBAL_STYLE.contains(".integration-module-button {\n  width: var(--size-module-button);\n  height: var(--size-module-button);")
        );
        assert!(GLOBAL_STYLE.contains("overflow: hidden;"));
        assert!(
            GLOBAL_STYLE.contains(".integration-module-button__image {\n  display: block;\n  width: 80%;\n  height: 80%;\n  object-fit: contain;")
        );
        assert!(
            GLOBAL_STYLE
                .contains("animation: module-action-button-pulse 1.6s ease-in-out infinite;")
        );
        assert!(GLOBAL_STYLE.contains("@keyframes module-action-button-pulse"));
        assert!(GLOBAL_STYLE.contains(".header-monitor-controls"));
        assert!(GLOBAL_STYLE.contains("--size-tracked-apps-path-bottom-pad: 0px;"));
        assert!(GLOBAL_STYLE.contains("padding-bottom: var(--size-tracked-apps-path-bottom-pad);"));
        assert!(GLOBAL_STYLE.contains(".ignored-addresses-section"));
        assert!(GLOBAL_STYLE.contains(".ignored-addresses {\n  display: grid;"));
        assert!(GLOBAL_STYLE.contains("--size-ignored-address-row-height: 30px;"));
        assert!(GLOBAL_STYLE.contains("--size-ignored-address-row-gap: 5px;"));
        assert!(GLOBAL_STYLE.contains("--size-ignored-addresses-visible-rows: 5;"));
        assert!(GLOBAL_STYLE.contains("--size-ignored-addresses-list-height: calc((var(--size-ignored-address-row-height) * var(--size-ignored-addresses-visible-rows)) + (var(--size-ignored-address-row-gap) * (var(--size-ignored-addresses-visible-rows) - 1)) + 10px + 2px);"));
        assert!(GLOBAL_STYLE.contains("grid-auto-rows: var(--size-ignored-address-row-height);"));
        assert!(GLOBAL_STYLE.contains("align-content: start;"));
        assert!(GLOBAL_STYLE.contains("gap: var(--size-ignored-address-row-gap);"));
        assert!(GLOBAL_STYLE.contains("height: var(--size-ignored-addresses-list-height);"));
        assert!(GLOBAL_STYLE.contains("max-height: var(--size-ignored-addresses-list-height);"));
        assert!(GLOBAL_STYLE.contains("border-top: 1px solid var(--color-control-border);\n  border-bottom: 1px solid var(--color-control-border);"));
        assert!(GLOBAL_STYLE.contains(
            ".value-label,\n.integration-status__label,\n.integration-dialog-status-row__label"
        ));
        assert!(!GLOBAL_STYLE.contains(".integration-dialog-status-row--paths"));
        assert!(!GLOBAL_STYLE.contains(".integration-dialog-paths"));
        assert!(!GLOBAL_STYLE.contains(".integration-dialog-path__label"));
        assert!(GLOBAL_STYLE.contains("color: var(--color-text);\n  font-size: 12px;\n  font-weight: 700;\n  line-height: 16px;"));
        assert!(GLOBAL_STYLE.contains("--size-ignored-address-ip-column: 230px;"));
        assert!(GLOBAL_STYLE.contains(".ignored-addresses__row {\n  display: grid;"));
        assert!(GLOBAL_STYLE.contains("grid-template-columns: var(--size-ignored-address-ip-column) 18px minmax(80px, 1fr) var(--size-close-button);"));
        assert!(GLOBAL_STYLE.contains("min-height: var(--size-ignored-address-row-height);"));
        assert!(GLOBAL_STYLE.contains(".ignored-addresses__address {\n  display: block;\n  width: 100%;\n  min-width: var(--size-ignored-address-ip-column);\n  max-width: var(--size-ignored-address-ip-column);"));
        assert!(GLOBAL_STYLE.contains("color: var(--color-text) !important;"));
        assert!(GLOBAL_STYLE.contains("grid-column: 1;\n  grid-row: 1;"));
        assert!(GLOBAL_STYLE.contains(
            ".ignored-addresses__help {\n  grid-column: 2;\n  grid-row: 1;\n  justify-self: center;"
        ));
        assert!(
            GLOBAL_STYLE
                .contains("#ignored-addresses-panel-help {\n  position: relative;\n  top: -2px;")
        );
        assert!(GLOBAL_STYLE.contains(".ignored-addresses__domain {\n  width: 100%;\n  min-width: 80px;\n  grid-column: 3;\n  grid-row: 1;"));
        assert!(GLOBAL_STYLE.contains(".ignored-addresses__domain {\n  width: 100%;\n  min-width: 80px;\n  grid-column: 3;\n  grid-row: 1;\n  color: var(--color-text);"));
        assert!(GLOBAL_STYLE.contains("direction: ltr;"));
        assert!(GLOBAL_STYLE.contains("text-align: left;"));
        assert!(GLOBAL_STYLE.contains("unicode-bidi: plaintext;"));
        assert!(GLOBAL_STYLE.contains(".ignored-addresses__row .button {\n  grid-column: 4;\n  grid-row: 1;\n  justify-self: end;"));
        assert!(GLOBAL_STYLE.contains(".ip-cell__help {\n  top: 0;"));
        assert!(GLOBAL_STYLE.contains(".integration-folder-path-help"));
        assert!(GLOBAL_STYLE.contains(".path-input-shell {\n  position: relative;"));
        assert!(GLOBAL_STYLE.contains(".path-input-clear {\n  position: absolute;"));
        assert!(
            GLOBAL_STYLE
                .contains(".path-input-clear .button__icon {\n  width: 12px;\n  height: 12px;")
        );
        assert!(GLOBAL_STYLE.contains(".path-field"));
        assert!(GLOBAL_STYLE.contains("max-width: 100%;"));
        assert!(GLOBAL_STYLE.contains("overflow-x: auto;"));
        assert!(GLOBAL_STYLE.contains(".domain-field {\n  width: min(260px, 100%);\n  min-width: 120px;\n  color: var(--color-text);"));
        assert!(GLOBAL_STYLE.contains(".integration-status__path,\n.integration-status__provider,\n.integration-status__value {\n  min-width: 0;\n  margin: 0;\n  overflow: hidden;\n  color: var(--color-text);"));
        assert!(
            GLOBAL_STYLE.contains(".integration-dialog-path__input {\n  color: var(--color-text);")
        );
        assert!(GLOBAL_STYLE.contains(".modal__footer"));
        assert!(GLOBAL_STYLE.contains(".table-wrap {\n  display: flex;\n  flex-direction: column;\n  height: 100%;\n  max-height: none;\n  overflow: hidden;"));
        assert!(GLOBAL_STYLE.contains(".table-header-wrap {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) var(--size-scrollbar);"));
        assert!(GLOBAL_STYLE.contains(".table-header-scrollbar-fill {\n  border-bottom: 1px solid var(--color-control-border);\n  background: var(--color-panel-header-bg);"));
        assert!(GLOBAL_STYLE.contains(
            ".table-body-wrap {\n  flex: 1 1 auto;\n  min-height: 0;\n  overflow: auto;"
        ));
        assert!(GLOBAL_STYLE.contains("padding-inline-end: var(--size-scrollbar-content-gutter);"));
        assert!(GLOBAL_STYLE.contains(".observations-table { width: 100%; border-collapse: collapse; min-width: 860px; table-layout: fixed; }"));
        assert!(GLOBAL_STYLE.contains(
            "body.netstitch-observation-selection-modifier .observations-table--body .observation-row"
        ));
        assert!(GLOBAL_STYLE.contains("cursor: pointer;"));
        assert!(
            GLOBAL_STYLE.contains(
                ".observations-table--body .observation-row--confirmed .path-field:focus"
            )
        );
        assert!(
            GLOBAL_STYLE
                .contains(".observations-table--body .observation-row:hover .path-field:focus")
        );
        assert!(GLOBAL_STYLE.contains("min-width: var(--size-switch-width);"));
        assert!(GLOBAL_STYLE.contains("--size-list-row-inline-pad: 3px;"));
        assert!(GLOBAL_STYLE.contains("--size-list-row-action-gap: 2px;"));
        assert!(GLOBAL_STYLE.contains(".tracked-app__actions {\n  position: relative;\n  display: block;\n  justify-self: end;\n  align-self: stretch;\n  height: 100%;"));
        assert!(
            GLOBAL_STYLE.contains(".tracked-app__actions--with-delete {\n  display: block;\n}")
        );
        assert!(GLOBAL_STYLE.contains(".tracked-app__actions--with-delete .button--close {\n  position: absolute;\n  top: var(--size-list-row-inline-pad);\n  right: 0;"));
        assert!(
            GLOBAL_STYLE.contains(".tracked-app__action-row {\n  position: absolute;\n  right: 0;\n  bottom: var(--size-list-row-inline-pad);\n  display: flex;\n  align-items: center;")
        );
        assert!(GLOBAL_STYLE.contains("height: var(--size-switch-height);"));
        assert!(GLOBAL_STYLE.contains("overflow-x: hidden;"));
        assert!(GLOBAL_STYLE.contains("overflow-y: auto;"));
        assert!(GLOBAL_STYLE.contains(
            ".modal--panel .modal__footer,\n.modal--compact .modal__footer {\n  flex: 0 0 var(--size-panel-footer-height);"
        ));
        assert!(GLOBAL_STYLE.contains("height: var(--size-panel-footer-height);"));
        assert!(GLOBAL_STYLE.contains("[data-ui-entity=\"panel-footer\"] {\n  display: flex;"));
        assert!(GLOBAL_STYLE.contains("flex: 0 0 var(--size-panel-footer-height);"));
        assert!(GLOBAL_STYLE.contains(".switch--on {\n  background: var(--color-accent);"));
        assert!(GLOBAL_STYLE.contains("transform: translateX(24px);"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-accent);"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-switch-knob-on);"));
        assert!(!GLOBAL_STYLE.contains(".tracked-app--active {\n"));
        assert!(!GLOBAL_STYLE.contains(".list-item--active {\n"));
        assert!(GLOBAL_STYLE.contains("cursor: pointer;"));
        assert!(GLOBAL_STYLE.contains(".netstitch-floating-tooltip"));
        assert!(GLOBAL_STYLE.contains("position: fixed;"));
        assert!(GLOBAL_STYLE.contains("width: max-content;"));
        assert!(GLOBAL_STYLE.contains("max-width: none;"));
        assert!(GLOBAL_STYLE.contains(".netstitch-floating-tooltip--pre"));
        assert!(!GLOBAL_STYLE.contains(".help-icon::after"));
        assert!(!GLOBAL_STYLE.contains("content: attr(data-tooltip);"));
        assert!(GLOBAL_STYLE.contains("z-index: var(--z-tooltip);"));
        assert!(TOOLTIP_SCRIPT.contains("getBoundingClientRect"));
        assert!(TOOLTIP_SCRIPT.contains("data-tooltip-align"));
        assert!(TOOLTIP_SCRIPT.contains("netstitch-floating-tooltip--visible"));
        assert!(TOOLTIP_SCRIPT.contains("setObservationSelectionModifier"));
        assert!(TOOLTIP_SCRIPT.contains("netstitch-observation-selection-modifier"));
        assert!(
            TOOLTIP_SCRIPT.contains("window.addEventListener(\"scroll\", positionTooltip, true);")
        );
        assert!(GLOBAL_STYLE.contains("background: var(--color-app-row-bg);"));
        assert!(GLOBAL_STYLE.contains(".tracked-app--enabled"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-app-row-enabled-bg);"));
        assert!(GLOBAL_STYLE.contains(".tracked-app-skeleton {"));
        assert!(GLOBAL_STYLE.contains(".integration-module-button-skeleton {"));
        assert!(GLOBAL_STYLE.contains("@keyframes tracked-app-skeleton-pulse"));
        assert!(
            GLOBAL_STYLE.contains(
                "animation: tracked-app-skeleton-pulse 1s ease-in-out infinite alternate;"
            )
        );
        assert!(!GLOBAL_STYLE.contains("tracked-app-placeholder"));
    }

    #[test]
    fn cloud_sync_dialog_uses_compact_panel_layout() {
        assert!(GLOBAL_STYLE.contains(".button__icon--cloud-sync {"));
        assert!(GLOBAL_STYLE.contains(".button__icon--cloud-import {"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-dialog {"));
        assert!(GLOBAL_STYLE.contains("display: grid;"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-transparent);"));
        assert!(GLOBAL_STYLE.contains("width: 100%;"));
        assert!(GLOBAL_STYLE.contains("max-width: none;"));
        assert!(GLOBAL_STYLE.contains("grid-template-rows: minmax(0, 1fr);"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-panel__header {"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-panel-service {"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-panel__footer {"));
        assert!(GLOBAL_STYLE.contains("height: var(--size-panel-footer-height);"));
        assert!(!GLOBAL_STYLE.contains("min-height: 26px;"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-panel__footer > .cloud-sync-panel__action {"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-download-panel__body {"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-upload-panel__body {"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-auth-row {"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-footer-auth {"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-filter-block {"));
        assert!(GLOBAL_STYLE.contains("grid-template-columns: minmax(120px, 30fr) minmax(120px, 30fr) minmax(160px, 38fr) 108px var(--size-header-action-button);"));
        assert!(GLOBAL_STYLE.contains("min-width: 108px;"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-filter-field-shell > .input {"));
        assert!(GLOBAL_STYLE.contains(".button__icon--my-publications {"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-publications-data-table {"));
        assert!(GLOBAL_STYLE.contains("min-width: 860px;"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-publications-data-table th {"));
        assert!(GLOBAL_STYLE.contains("white-space: nowrap;"));
        assert!(GLOBAL_STYLE.contains("width: 130px;"));
        assert!(!GLOBAL_STYLE.contains(".cloud-sync-scope-control {"));
        assert!(GLOBAL_STYLE.contains(".cloud-sync-app-list {"));
        assert!(!GLOBAL_STYLE.contains(".cloud-sync-browse-panel {"));
        assert!(!GLOBAL_STYLE.contains(".cloud-sync-account-panel {"));
        assert!(!GLOBAL_STYLE.contains(".cloud-sync-auth-grid {"));
    }

    #[test]
    fn header_action_separator_is_simple_vertical_line() {
        assert!(GLOBAL_STYLE.contains(".header-action-separator {"));
        assert!(GLOBAL_STYLE.contains("display: block;"));
        assert!(GLOBAL_STYLE.contains("flex: 0 0 1px;"));
        assert!(GLOBAL_STYLE.contains("align-self: center;"));
        assert!(GLOBAL_STYLE.contains("width: 1px;"));
        assert!(GLOBAL_STYLE.contains("min-width: 1px;"));
        assert!(GLOBAL_STYLE.contains("height: 44px;"));
        assert!(GLOBAL_STYLE.contains("margin: 0;"));
        assert!(GLOBAL_STYLE.contains("background: var(--color-control-border-strong);"));
        assert!(GLOBAL_STYLE.contains("opacity: 1;"));
        assert!(!GLOBAL_STYLE.contains(".header-action-separator {\n  align-self: stretch;"));
        assert!(!GLOBAL_STYLE.contains("background: var(--color-border);"));
    }

    #[test]
    fn input_and_button_hover_styles_do_not_shift_position() {
        assert!(!GLOBAL_STYLE.contains(".button:hover { transform:"));
        assert!(!GLOBAL_STYLE.contains(".input-action-button:hover { transform:"));
        assert!(!GLOBAL_STYLE.contains("translateY(-1px)"));
    }
}
