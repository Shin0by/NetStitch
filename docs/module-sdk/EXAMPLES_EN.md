# Examples And Ready Packages

The SDK ships two implementations of the same demonstration module:

- `docs/module-sdk/examples/ui-entity-showcase-rust/` - Rust;
- `docs/module-sdk/examples/ui-entity-showcase-cpp/` - C++.

Both examples show the same behavior: `grid`, `tabs`, a dedicated `browse_window` tab for choosing a folder and a save target without writing a file, `text_input`/`textarea` with `clear_button` and `commit_on_enter`, `select`, `switch`, progress bars with `progress_stages`, a simulated-download button that animates the large progress bar from `0` to `100` through its phases, `table_columns`, a standard dialog, a header action, background host subscriptions, and RU/EN localization. The Rust example also has a module-owned SVG icon through `icon_path`; the C++ example intentionally has no `icon_path`, so authors can see both button modes.

## Packages

Ready-to-install archives live in `docs/module-sdk/packages/`:

- `ui-entity-showcase-rust-windows-x86_64.zip`;
- `ui-entity-showcase-rust-linux-x86_64.zip`;
- `ui-entity-showcase-cpp-windows-x86_64.zip`;
- `ui-entity-showcase-cpp-linux-x86_64.zip`.

Unpack one archive next to the portable app so it creates:

```text
integrations/<module-id>/module.json
integrations/<module-id>/bin/...
integrations/<module-id>/locales/...
```

The archives are developer artifacts generated from tracked sources. They must not contain `target/`, `build/`, `.local/`, `temp/`, secrets, runtime `data/`, or local SQLite databases.

## Example Manifest Notes

`module.json` must remain valid JSON. Explanations for fields live in `README_EN.md` near the example and in the SDK docs, not as comments inside the manifest.
