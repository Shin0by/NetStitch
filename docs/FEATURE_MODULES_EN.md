# Modules

Modules extend NetStitch with product-specific workflows without changing the main application. A module can take confirmed monitoring data and turn it into a focused action: analysis, export, profile preparation, resource download, or another guided workflow.

## Where To Find It

- Interface: main window -> `Modules`.
- Module windows open inside NetStitch and use the same desktop and web interface style.
- Portable module files live next to the application under `integrations/`.

## What Modules Can Support

- selected monitoring rows, domains, ranges, ports, protocols, and request counts;
- preview and analysis before applying changes;
- export actions for external tools or product profiles;
- progress, status messages, warnings, and result summaries;
- user-selected folders or files when a workflow needs a local target;
- runtime host-event subscriptions for filters, monitoring, and `Tracked apps`;
- module-owned background tasks that start only after an explicit user action, with no autostart when NetStitch launches;
- standard `OK` and `OK+Cancel` dialogs with the module title and icon;
- Windows, Linux, and macOS support when the module author provides it;
- English, Russian, or additional interface languages.

## How Module Interfaces Work

Module interfaces appear as NetStitch panels, grids, dialogs, tables, buttons, progress bars, and status blocks. This keeps modules familiar: users do not need to learn a separate application for each workflow, and web control can present the same module surface through the local NetStitch runtime.

Modules can show which operating systems and languages they support. If a module does not support the current system, NetStitch can present that clearly instead of leaving the user with a broken action.

Module messages go to the system log with `source` set to the module name; core runtime events use `source = core`. Module ids and display names `core` and `system` are reserved case-insensitively so host/runtime and module events cannot be confused. Message severity is one of four values: `info`, `success`, `warning`, `error`.

Cloud upload/download and CSV import/export remain manual user actions in the main NetStitch interface. A module can analyze selected local rows and store its own results under `integrations/<module>/data/`, but it cannot run cloud or CSV operations on the user's behalf.

## For Module Authors

Modules let authors focus on a specific workflow instead of rebuilding monitoring, localization, table selection, CSV handling, progress UI, and desktop/web presentation. A good module can be small, clear, and useful because NetStitch already provides the surrounding product shell.

The full developer guide is in the [Module SDK](module-sdk/README_EN.md). It documents the unchanged native shared library C/JSON ABI, manifest, UI entities, host context, host event subscriptions, allowed commands, storage boundaries, table column settings through `table_columns`, and two identical Rust/C++ showcase examples with ready Windows/Linux archives.

This makes modules a practical way to share specialized workflows while keeping the main NetStitch interface consistent.
