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
  "hidden": false,
  "visible": true,
  "disabled": false,
  "readonly": false,
  "clear_button": false,
  "commit_on_enter": false,
  "compact": false,
  "hide_label": false,
  "progress_stages": [],
  "checked": false,
  "scroll": "off",
  "size": "stretch",
  "width": "100%",
  "height": "",
  "align": "left",
  "margin": "",
  "padding": "",
  "columns": "",
  "table_columns": [],
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
- `compact` and `hide_label` - mainly used by `progress`;
- `progress_stages` - progress phases such as `{ "color": "accent", "percent": 30, "name": "Queued" }`;
- `size`, `width`, `height`, `min_width`, `max_width`, `align`, `margin`, `padding`;
- `columns`, `rows`, `gap` for `grid`;
- `table_columns` for `table`;
- `grid_column`, `grid_row`;
- `opacity`;
- `options`, `children`, `actions`.

## Supported Types

- `panel`, `subpanel`, `grid` / `layout_grid`, `tabs` / `tab_view`, `row`;
- `value_label` / `value`, `status_label` / `status`, `path_field`;
- `input` / `text_input` / `text_field`, `textarea` / `text_area`;
- `select` / `dropdown` / `combo_box`, `switch` / `toggle`;
- `help_text`, `separator`;
- `button` / `action_button`;
- `progress`;
- `table`;
- `footer`.

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

`input`, `textarea`, `select`, and `tabs` are strings. `switch` is boolean. If the user did not change a value, the module should fall back to manifest `value` or `checked`.

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
  "align": "left",
  "pulse": true,
  "pulse_when_background_active": false,
  "enabled": true
}
```

Header actions are always square buttons in the module header. Ordinary `button` / `action_button` entities use the same sizing, alignment, margin, and padding contract as other schema entities.

## Localization

Module string keys are loaded only from the module's own `locales/*.ini`: English fallback first, then the selected UI language. Missing keys fall back to plain manifest text. Do not put module strings into the main app `resources/language/*` files.
