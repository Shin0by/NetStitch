# NetStitch Integrations

This directory is the temporary in-repository contour for external integration modules.
When a module is ready to move out, its folder can become a separate repository without
changing the main NetStitch UI/runtime contract.

## Layout

- `host/` - generic Rust host used by the core runtime to discover and call modules.
- `<module>/module.json` - module manifest with schema, id, display name, icon label, transport, and platform entrypoint paths.
- `<module>/bin/` - runtime binaries such as Windows `.dll`, Linux `.so`, macOS `.dylib`, and module-owned helper files.
- `<module>/assets/` - module-owned static UI assets such as SVG/PNG action icons.
- `<module>/data/` - portable module storage. SQLite databases, downloaded external content, cache, backups, and generated files stay here.
- `<module>/crates/` - temporary source layout while the module still lives in this repository.

The main application must not store module settings or module data in `storage/netstitch.sqlite3`.
The host passes the module-local storage directory as `integrations/<module>/data/`.

## Discovery

The host looks for modules in:

- `NETSTITCH__INTEGRATIONS_DIR`
- `integrations/` next to the running executable
- `integrations/` under the current working directory

The default transport is a native shared library selected by manifest `library_paths`.
The host checks `<os>-<arch>`, then `<os>`, then `default`, and loads the selected
library through the platform dynamic loader. Windows modules use `.dll`, Linux modules
use `.so`, macOS modules use `.dylib`, and all expose the same C ABI exports:

- `netstitch_integration_call`
- `netstitch_integration_free`

The portable runtime path is the native shared-library transport above; additional transports can
be designed later without changing the main application UI contract.

## UI Contract

Modules do not draw UI directly. They return generic NetStitch UI entity DTOs and action
descriptors. Desktop and browser shells render those entities with the same panels, grids, headers,
footers, tables, path fields, buttons, progress bars, dialogs, and tooltips used by the main
application.

Modules may receive local working data from monitoring: endpoints, selected rows, addresses,
domains, ranges, ports, connection metrics, and local app signature provenance. They must not
receive cloud credentials, cloud account state, user secrets, or direct access to the main SQLite
database.

The public module author guide lives under `docs/module-sdk/`. Use it as the canonical SDK
contract before creating or updating a module.
