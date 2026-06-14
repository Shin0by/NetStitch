# Module UI Entities

Modules do not render HTML or native controls directly. A module describes UI declaratively in `module.json`, and NetStitch renders the same schema in the desktop and browser shells.

## Base Entity

```json
{
  "id": "status-row",
  "page": "main",
  "entity_type": "value_label",
  "title": "Selected rows",
  "title_key": "my_module.entity.selected_rows.title",
  "value": "0",
  "value_key": "my_module.entity.selected_rows.value",
  "tooltip": "How many Monitoring rows are selected",
  "tooltip_key": "my_module.entity.selected_rows.tooltip",
  "placeholder": "",
  "placeholder_key": "my_module.entity.selected_rows.placeholder",
  "hidden": false,
  "visible": true,
  "disabled": false,
  "readonly": false,
  "clear_button": false,
  "commit_on_enter": false,
  "compact": false,
  "hide_label": false,
  "hide_host_back_button": false,
  "progress_stages": [],
  "checked": false,
  "scroll": "off",
  "size": "stretch",
  "width": "100%",
  "height": "",
  "min_width": "",
  "min_height": "",
  "max_width": "",
  "max_height": "",
  "align": "left",
  "margin": "",
  "padding": "",
  "columns": "",
  "table_columns": [],
  "rows": "",
  "gap": "",
  "grid_column": "",
  "grid_row": "",
  "opacity": "100%",
  "options": [],
  "children": [],
  "actions": []
}
```

Common fields:

- `id` - stable entity key;
- `page` - optional module page or tab page;
- `entity_type` - entity type;
- `title` / `title_key`, `tooltip` / `tooltip_key`, `value` / `value_key`, `placeholder` / `placeholder_key`;
- `hidden: true` or `visible: false` - do not render;
- `disabled`, `readonly`;
- `clear_button: true` - enables the embedded clear `X` for `text_input` and `textarea`;
- `commit_on_enter: true` - commits text input on Enter without per-keystroke app-state updates;
- `compact` and `hide_label` - mainly used by `progress`; compact progress keeps the visible title as a single-line label to the left of the mini bar when `hide_label` is false;
- action/button labels render as a single line at normal control widths; use `width`, `min_width`, and `max_width` when a module needs a wider command button;
- `hide_host_back_button: true` - only for `footer`: hides the standard host-owned `Back` button when the module fully replaces it with custom footer actions; by default the button is shown and remains the rightmost item in the module footer;
- `progress_stages` - progress phases such as `{ "color": "accent", "percent": 30, "name": "Queued" }`;
- `size`, `width`, `height`, `min_width`, `max_width`, `align`, `margin`, `padding`;
- `columns`, `rows`, `gap` for `grid`;
- `table_columns` for `table`;
- `grid_column`, `grid_row`;
- `opacity`;
- `options`, `children`, `actions`.

## Predictable Layout Defaults

The host applies defaults to empty layout fields before rendering in both desktop and browser shells. A normal entity without boilerplate sizing still gets a stable result:

- every entity, including an unknown future type, gets `size = "stretch"`, `width = "100%"`, `min_width = "0"`, `opacity = "100%"`, and `align = "left"`;
- `panel`, `subpanel`, `nested_subpanel`, `grid`, `tabs`: `height = "auto"`, `min_height = "0"`, `scroll = "off"`; by default the panel uses the common `100%` width and grows vertically only from its content;
- `grid`: also `columns = "repeat(auto-fit, minmax(180px, 1fr))"` and `gap = "8px"`;
- `button` / `action_button`: `size = "stretch"`, `width = "100%"`, `min_width = "0"`, `margin = "8px 0 0"`, `padding = "0"`; multiple actions render as one horizontal row unless `button_layout` is set;
- `separator`, `help_text`, `table`, `progress`, and `footer`: also `min_height = "0"`.

Explicit fields always override defaults. For internal scrolling, use `scroll: "y"` or `"both"` together with an explicit `height` or `max_height`; otherwise prefer the default `scroll: "off"` so important controls do not end up hidden inside a small accidental scroll region.

Row-like entities (`row`, `value`, `status`, `path_field`, `input`, `textarea`, `select`, `switch`) use the common stretch contract and NetStitch's standard compact control height. Minimal JSON with `id`, `entity_type`, `title`, and `value` is usually enough: the host supplies width, safe `min-width`, alignment, and opacity.

If a row-like entity has no `title`, the host removes the empty label column and lets the main control/value use the available width. So `{"id": "q", "entity_type": "text_input"}` remains a predictable one-column field instead of a narrow control in the left label column.

`grid.children` may contain any normal UI entity from this reference. For action rows, large tables, textareas, and other controls that should span the whole grid, set `grid_column: "1 / -1"`. Without placement, a child occupies one adaptive grid column.

## Supported Types

Canonical JSON values and aliases available to modules:

```json
"entity_type": "panel"
"entity_type": "subpanel"
"entity_type": "nested_subpanel"
"entity_type": "grid"
"entity_type": "layout_grid"
"entity_type": "tabs"
"entity_type": "tab_view"
"entity_type": "row"
"entity_type": "value_label"
"entity_type": "value"
"entity_type": "status_label"
"entity_type": "status"
"entity_type": "path_field"
"entity_type": "input"
"entity_type": "text_input"
"entity_type": "text_field"
"entity_type": "textarea"
"entity_type": "text_area"
"entity_type": "select"
"entity_type": "dropdown"
"entity_type": "combo_box"
"entity_type": "switch"
"entity_type": "toggle"
"entity_type": "help_text"
"entity_type": "separator"
"entity_type": "button"
"entity_type": "action_button"
"entity_type": "progress"
"entity_type": "table"
"entity_type": "footer"
```

- `panel`, `subpanel`, `nested_subpanel`, `grid` / `layout_grid`, `tabs` / `tab_view`, `row`;
- `nested_subpanel` is a ready two-layer container: the outer subpanel uses the darker list surface, the inner subpanel uses the lighter row surface like rows in `Tracked apps`, and `children` render inside the inner layer;
- `value_label` / `value`, `status_label` / `status`, `path_field`;
- `input` / `text_input` / `text_field`, `textarea` / `text_area`;
- `select` / `dropdown` / `combo_box`, `switch` / `toggle`;
- `help_text`, `separator`;
- `button` / `action_button`;
- `progress` - can be updated live during a long `ui_action` through `IntegrationHostEvent { event: "ui_values" }` by explicitly naming this entity id;
- `table`;
- `footer` - bottom informational/action row for the module window or active page section, with `value`, child action buttons, and standard compact height. In the module window the host automatically places `footer` in the bottom row next to the navigation button, so the schema does not provide a separate footer container, height, or top spacing. By default the host-owned `Back` button is always shown on the right; `hide_host_back_button: true` is used only when the module provides replacement footer actions.

Unknown types are rendered as a plain row with `data-ui-entity` preserved.

## Editable Controls

Editable values are host-owned. The module receives current values through `payload.ui_values`:

```json
{
  "payload": {
    "ui_values": {
      "showcase-input": "typed text",
      "showcase-textarea": "line 1\nline 2",
      "showcase-select": "two",
      "showcase-switch": true,
      "showcase-tabs": "details"
    }
  }
}
```

`input`, `textarea`, `select`, `tabs`, `path-field`, `value-label`, and `status-label` are strings. `switch` is boolean. If the user did not change a value, the module should fall back to manifest `value` or `checked`. Display entities `path-field`, `value-label`, and `status-label` can be updated by host commands such as `set_ui_values` or `browse_window`, which lets modules show selected paths and statuses without creating custom UI.

## Progress

```json
{
  "id": "download-progress",
  "entity_type": "progress",
  "title": "Download",
  "value": "64",
  "compact": false,
  "hide_label": false,
  "progress_stages": [
    { "color": "accent", "percent": 30, "name": "Queued" },
    { "color": "rust", "percent": 70, "name": "Processing" },
    { "color": "success", "percent": 100, "name": "Done" }
  ],
  "width": "100%",
  "grid_column": "1 / -1"
}
```

Compact progress with a visible label:

```json
{
  "id": "mini-progress",
  "entity_type": "progress",
  "title": "Mini progress bar",
  "value": "42",
  "compact": true,
  "hide_label": false,
  "width": "100%",
  "min_width": "260px"
}
```

## Table Columns

`table_columns` configures TSV columns by zero-based `index`:

```json
{
  "id": "latest",
  "entity_type": "table",
  "scroll": "both",
  "height": "180px",
  "table_columns": [
    { "index": 0, "text_field": true, "width": "24%" },
    { "index": 2, "text_field": true, "width": "38%" },
    { "index": 3, "width": "10%", "align": "right" }
  ],
  "value": "App\tIP\tDomain\nExample\t203.0.113.10\texample.com"
}
```

`text_field: true` wraps cell text in the same constrained field used by main NetStitch tables, so long app names, domains, paths, or URLs do not resize neighboring columns.

## Actions

```json
{
  "id": "say_hello",
  "label": "Hello",
  "tooltip": "Show selected rows",
  "style": "primary",
  "pulse": true,
  "pulse_when_background_active": false,
  "enabled": true
}
```

Header actions are always square buttons in the module header. Ordinary `button` / `action_button` entities use the same sizing, alignment, margin, and padding contract as other schema entities. By default, multiple actions render as one horizontal row (`button_layout: "row"`); `align` on the entity aligns the whole row (`left`, `center`, `right`). For a vertical list, set `button_layout: "column"`. For complex layouts, use a regular `grid` so the module author explicitly describes the columns without a hidden button-placement mode.

Three centered buttons:

```json
{
  "id": "provider-actions",
  "entity_type": "action_button",
  "align": "center",
  "actions": [
    { "id": "a", "label": "A", "style": "primary" },
    { "id": "b", "label": "B", "style": "primary" },
    { "id": "c", "label": "C", "style": "primary" }
  ]
}
```

The host treats every `ui_action` as potentially long-running: the UI shell must remain responsive while the native handler runs, and an action result is not applied after the module's host-owned Stop/Close flow invalidates that action.

## Localization

Module string keys are loaded only from the module's own `locales/*.ini`: English fallback first, then the selected UI language. Missing keys fall back to plain manifest text. Do not put module strings into the main app `resources/language/*` files.
