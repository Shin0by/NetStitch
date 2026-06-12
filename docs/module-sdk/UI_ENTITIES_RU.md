# UI-сущности модулей

Модуль не рисует HTML или native controls напрямую. Он описывает UI декларативно в `module.json`, а NetStitch рендерит эти сущности тем же стилем, что основное окно. Один и тот же `ui_schema` используется для desktop shell и browser shell; отдельный web-код от автора модуля не требуется и не должен быть частью обычного module contract.

## Базовая сущность

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

Общие поля:

- `id` - стабильный ключ сущности;
- `page` - необязательная страница модуля; сущность без `page` видна на любой странице, сущность с `page` видна только когда host shell находится на этой странице;
- `entity_type` - тип сущности;
- `title` / `title_key` - заголовок или ключ локализации;
- `tooltip` / `tooltip_key` - tooltip;
- `value` / `value_key` - отображаемое значение или ключ локализации; подходит для `help_text`, `footer`, статичных таблиц и значений по умолчанию;
- `placeholder` / `placeholder_key` - placeholder для текстовых полей;
- `hidden: true` или `visible: false` - не рендерить сущность;
- `disabled: true` - показать сущность отключённой;
- `readonly: true` - запретить редактирование при сохранении обычного вида поля;
- `clear_button: true` - включает встроенную кнопку очистки `X` для `text_input` и `textarea`; по умолчанию кнопки нет, а включённая кнопка автоматически притухает, когда поле пустое, disabled или readonly;
- `commit_on_enter: true` - включает стандартное применение значения по Enter для `text_input` и `textarea`; host диспатчит обычный change-событие, показывает короткую `input--apply-pulse` индикацию и не запускает фильтрацию/проверку на каждый вводимый символ;
- `compact: true` - включает компактный вариант для сущностей, у которых он есть; сейчас используется `progress`, где остаётся только сама шкала;
- `hide_label: true` - скрывает текстовую подпись и stage-labels у `progress`, оставляя доступные `aria-label`/tooltip;
- `progress_stages` - специфическое поле `progress`: массив фаз `{ "color": "accent", "percent": 30, "name": "Queued", "name_key": "my_module.progress.queued" }`. `percent` - правая граница фазы в процентах `0..100`; значения `30`, `40`, `100` дадут три цветных блока шириной `30%`, `10%`, `60%`. `name` / `name_key` задают подпись фазы; в некомпактном режиме снизу отображается текущий процент и имя активной фазы, если оно задано;
  `color` принимает semantic-значения `accent`, `success`, `warning`, `danger`, `rust`, `muted` и синонимы `blue`, `green`, `yellow`, `red`, `orange`, `gray`/`grey`, либо безопасный hex-цвет вида `#2f80ed` / `#2f80edcc`;
- `checked` - начальное состояние `switch`;
- `options` - варианты для `select` / `dropdown` / `tabs`;
- `scroll` - режим прокрутки для `panel`, `subpanel`, `grid`, `tabs`, `table`: `x`, `y`, `both`, `off` или пусто;
- `size` - типовой размер сущности: `auto` / `fit`, `stretch` / `fill`, `fullscreen` / `full`; применяется ко всем `ui_schema`-сущностям, включая обычные `button` / `action_button`; square-кнопки главного header-а модуля остаются отдельной header-сущностью и всегда квадратные;
- `width`, `height`, `min_width`, `min_height`, `max_width`, `max_height` - явные размеры CSS-like значениями (`320px`, `60%`, `calc(100% - 16px)`); host фильтрует небезопасные символы и применяет значения только к контейнеру сущности;
- `align` - выравнивание контейнера и содержимого: `left`, `center`, `right`; если поле не задано, host применяет `left`, чтобы каждая сущность занимала предсказуемое место в layout-е;
- `margin`, `padding` - дополнительные внешние и внутренние отступы CSS-like значениями (`0`, `4px`, `4px 8px`); если поле не задано, используется стандартный compact layout NetStitch без дополнительного inline-отступа;
- `columns`, `rows`, `gap` - специфические поля `grid`: CSS-like значения для `grid-template-columns`, `grid-template-rows` и `gap`, например `repeat(2, minmax(0, 1fr))`, `auto`, `8px`; если `columns` не задано, используется адаптивная сетка `repeat(auto-fit, minmax(180px, 1fr))`;
- `table_columns` - специфическое поле `table`: массив настроек конкретных TSV-столбцов по нулевому `index`; поддерживает `text_field: true` для обёртки ячеек в стандартный ограничивающий `path-field` контейнер, а также `width`, `min_width`, `max_width` и `align`;
- `grid_column`, `grid_row` - placement-поля дочерней сущности внутри `grid`, например `1 / -1`, `2`, `auto`; они применяются к контейнеру любой сущности и нужны для span/позиционирования внутри grid;
- `opacity` - прозрачность любой `ui_schema`-сущности процентом от `0%` до `100%`; если поле не задано, используется `100%`. Значение `0%` удобно для невидимой layout-подпанели, которая работает как разделитель или spacer, но сохраняет размер, `margin`, `padding` и scroll-контракт;
- `children` - вложенные сущности;
- `actions` - кнопки, вызывающие `ui_action`.

## Поддерживаемые entity_type

- `panel` - контейнер панели;
- `subpanel` - вложенная панель;
- `grid` / `layout_grid` - panel-like grid-контейнер для колонок и placement-а дочерних сущностей;
- `tabs` / `tab_view` - вкладки внутри панели; активная вкладка хранится в `payload.ui_values` по `id` сущности;
- `row` - строка;
- `value_label` / `value` - подпись + значение;
- `status_label` / `status` - статус;
- `path_field` - read-only поле пути;
- `input` / `text_input` / `text_field` - однострочное текстовое поле; встроенная кнопка очистки `X` включается через `clear_button: true`, применение по Enter - через `commit_on_enter: true`;
- `textarea` / `text_area` - многострочное текстовое поле; встроенная кнопка очистки `X` включается через `clear_button: true`, применение по Enter - через `commit_on_enter: true`;
- `select` / `dropdown` / `combo_box` - выпадающий список с `options`;
- `switch` / `toggle` - стандартный переключатель NetStitch;
- `help_text` - короткий поясняющий текст;
- `separator` - типовой горизонтальный разделитель для группировки полей;
- `button` / `action_button` - группа action-кнопок;
- `progress` - progress bar; значение `value` задаёт процент `0..100`, `title`/`title_key` задают label, `progress_stages` задаёт цветные фазы `{ color, percent, name }`, `compact: true` делает шкалу компактной, `hide_label: true` скрывает видимые подписи. Сущность поддерживает все общие layout-поля: `size`, `width`, `height`, `min_width`, `min_height`, `max_width`, `max_height`, `align`, `margin`, `padding`, `grid_column`, `grid_row`, `opacity`;
- `table` - табличная область; `value` передаётся как TSV-строка с первой строкой-заголовком, а desktop/browser shell рендерят её одной типовой table-сущностью NetStitch;
- `footer` - footer панели или page-секции с текстом `value`, вложенными action-кнопками и стандартной высотой footer-а.

Если тип неизвестен, NetStitch рендерит его как обычную строку и сохраняет `data-ui-entity`.

## Controls и payload.ui_values

Editable-сущности остаются host-owned. Автор модуля не пишет DOM/JS для desktop или web: host сам хранит текущие значения контролов и передаёт их в `ui_action`:

```json
{
  "payload": {
    "ui_values": {
      "showcase-input": "typed text",
      "showcase-textarea": "line 1\nline 2",
      "showcase-select": "two",
      "showcase-switch": true
    },
    "module_background_active": false,
    "module": {
      "background_active": false
    }
  }
}
```

Ключи в `ui_values` равны `entity.id`. `input`, `textarea`, `select` и `tabs` передаются строками, `switch` передаётся boolean. Если пользователь не менял значение, host может не добавлять его в `ui_values`; модуль должен использовать `value` / `checked` из manifest как fallback.

Пример `select`:

```json
{
  "id": "mode",
  "entity_type": "select",
  "title": "Mode",
  "value": "safe",
  "options": [
    { "value": "safe", "label": "Safe" },
    { "value": "fast", "label": "Fast" }
  ]
}
```

Пример `switch`:

```json
{
  "id": "enabled",
  "entity_type": "switch",
  "title": "Enabled",
  "checked": true
}
```

Пример `progress`:

```json
{
  "id": "download-progress",
  "entity_type": "progress",
  "title": "Download",
  "value": "64",
  "compact": false,
  "hide_label": false,
  "progress_stages": [
    { "color": "accent", "percent": 30, "name": "Queued", "name_key": "my_module.progress.queued" },
    { "color": "rust", "percent": 70, "name": "Processing", "name_key": "my_module.progress.processing" },
    { "color": "success", "percent": 100, "name": "Done", "name_key": "my_module.progress.done" }
  ],
  "width": "100%",
  "min_width": "220px",
  "grid_column": "1 / -1",
  "opacity": "100%"
}
```

Компактный progress bar для footer или плотной строки:

```json
{
  "id": "footer-progress",
  "entity_type": "progress",
  "title": "Upload",
  "value": "42",
  "compact": true,
  "hide_label": true,
  "width": "240px",
  "height": "18px",
  "align": "right"
}
```

Пример `separator`:

```json
{
  "id": "main-separator",
  "entity_type": "separator",
  "width": "100%"
}
```

Пример `grid`:

```json
{
  "id": "settings-grid",
  "entity_type": "grid",
  "title": "Settings",
  "size": "stretch",
  "width": "100%",
  "columns": "repeat(2, minmax(0, 1fr))",
  "gap": "8px",
  "children": [
    {
      "id": "profile-name",
      "entity_type": "text_input",
      "title": "Profile",
      "clear_button": true,
      "width": "100%"
    },
    {
      "id": "mode",
      "entity_type": "select",
      "title": "Mode",
      "width": "100%",
      "options": [
        { "value": "safe", "label": "Safe" },
        { "value": "fast", "label": "Fast" }
      ]
    },
    {
      "id": "notes",
      "entity_type": "textarea",
      "title": "Notes",
      "clear_button": true,
      "grid_column": "1 / -1",
      "width": "100%",
      "height": "96px"
    }
  ]
}
```

Пример `tabs`:

```json
{
  "id": "module-tabs",
  "entity_type": "tabs",
  "value": "summary",
  "scroll": "y",
  "height": "360px",
  "options": [
    { "value": "summary", "label": "Summary" },
    { "value": "details", "label": "Details" }
  ],
  "children": [
    {
      "id": "summary-text",
      "page": "summary",
      "entity_type": "help_text",
      "value": "Visible on Summary tab"
    },
    {
      "id": "details-table",
      "page": "details",
      "entity_type": "table",
      "value": "Key\tValue\nRows\t10"
    }
  ]
}
```

Для `tabs` поле `page` у дочерних сущностей означает не overlay-page модуля, а `value` вкладки. Host сам хранит активную вкладку и передаёт её модулю в `payload.ui_values["module-tabs"]`. Модуль может переключить вкладку host-командой `set_ui_values`.

Пример scrollable table:

```json
{
  "id": "latest",
  "entity_type": "table",
  "title": "Latest rows",
  "scroll": "both",
  "size": "stretch",
  "height": "180px",
  "table_columns": [
    {
      "index": 0,
      "text_field": true,
      "width": "24%"
    },
    {
      "index": 2,
      "text_field": true,
      "width": "38%"
    },
    {
      "index": 3,
      "width": "10%",
      "align": "right"
    }
  ],
  "value": "App\tIP\tDomain\nExample\t203.0.113.10\texample.com"
}
```

`table_columns` применяется только к `table`. `index` начинается с `0` и соответствует позиции ячейки в TSV-строке. Если `text_field: true`, host помещает текст ячейки в тот же однострочный `path-field` контейнер, который используется в основных таблицах NetStitch для путей, IP и доменов: длинный текст не раздвигает колонку и не лезет поверх соседних ячеек, а остаётся внутри своего поля. Это полезно для колонок `App`, `Domain`, путей, URL и других длинных значений. Поля `width`, `min_width`, `max_width` принимают такие же CSS-like размеры, как размеры сущностей; `align` поддерживает `left`, `center`, `right`.

Пример размеров и выравнивания:

```json
{
  "id": "notes",
  "entity_type": "textarea",
  "title": "Notes",
  "size": "stretch",
  "width": "100%",
  "height": "96px",
  "align": "left",
  "margin": "0",
  "padding": "0",
  "placeholder": "Write details"
}
```

Пример невидимой подпанели-разделителя:

```json
{
  "id": "spacer",
  "entity_type": "subpanel",
  "height": "12px",
  "padding": "0",
  "margin": "0",
  "opacity": "0%"
}
```

Header actions модуля всегда квадратные и не используют `size`. Обычные `button` / `action_button` внутри `ui_schema` используют тот же контракт размеров, `align`, `margin` и `padding`, что панели, таблицы и поля ввода. `align` у самой группы выравнивает контейнер кнопок, а `align` у отдельного action - конкретную кнопку внутри группы. Для строки кнопок, которая должна занять всю ширину панели и зарезервировать место под left/center/right-кнопки, используйте `size: "stretch"` или `width: "100%"`.

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

Для action поддерживаются `align: "left"`, `"center"` и `"right"`. Это не меняет размер кнопки, а только её положение внутри группы.

`style` может быть пустым/default для обычной серой кнопки или `primary` / `blue` / `accent` для синей кнопки. Header actions из `header_actions` всегда рендерятся как квадратные кнопки в типовом header-е модуля; ordinary actions внутри `ui_schema` рендерятся в теле панели или footer-е.
`pulse` включает постоянную пульсацию конкретной action-кнопки. `pulse_when_background_active` включает пульсацию только пока background-задача этого модуля активна. Для типовой кнопки `Start/Stop` в header-е обычно используется `pulse_when_background_active: true`, чтобы host сам включал/выключал визуальное состояние и в desktop, и в web.

Клик по action вызывает `ui_action` и передаёт:

- `action_id`;
- текущий `HostContext`;
- выбранные строки мониторинга;
- id отображённых строк;
- текущие фильтры;
- `payload.ui_values` с текущими значениями editable controls;
- `payload.module_background_active` и `payload.module.background_active` с текущим состоянием background-задачи этого модуля.

Фильтры, monitoring status/rows/selection и `Tracked apps` могут отправлять module event, если модуль уже запустил background через `start_background` и указал точные подписки или wildcard-prefix вроде `filters.*`, `monitoring.*`, `tracked_apps.*`. Такие события предназначены для обновления module UI state; они не должны запускать cloud/CSV действия и не включают autostart фоновых задач.

## Страницы и второй уровень

Модуль может описать несколько страниц через поле `page`. По умолчанию открыт `main`. Для перехода модуль возвращает host-команду `set_module_page`:

```json
{
  "command_type": "set_module_page",
  "payload": {
    "page": "last_rows"
  }
}
```

Страница второго уровня остаётся внутри стандартного overlay модуля. Главный header модуля остаётся host-owned: стандартная кнопка `Остановить` прерывает background-задачу/подписки модуля и возвращает пользователя на главный экран NetStitch, а `Закрыть` закрывает overlay без принудительной остановки background-задачи. Навигация на уровень выше выполняется footer-кнопкой `Назад` внутри overlay, если текущая `page` не `main`. Сам модуль описывает содержимое через `panel`, `table`, `footer` и actions. Footer-сущность используется как левая информационная часть стандартного footer-а окна модуля рядом с навигационной кнопкой; отдельную кнопку `Back` в body/footer схемы добавлять не нужно.

## Dynamic placeholders

Host заменяет placeholders в `title`, `tooltip`, `value`, `actions[].label` и `actions[].tooltip` перед рендером:

- `{context.tables.monitoring.selected_rows}` - сколько строк Monitoring выбрано;
- `{context.tables.monitoring.displayed_rows}` - сколько строк Monitoring отображено текущими фильтрами;
- `{context.tables.monitoring.total_rows}` - всего строк Monitoring;
- `{context.tables.monitoring.selected_total}` - краткая запись `selected/total`;
- `{context.module.background_active}` - `true`/`false`, запущена ли background-задача этого модуля;
- `{context.module.background_action_label}` - `Start` или `Stop`;
- `{context.module.background_status}` - человекочитаемый статус listener-а;
- `{context.monitoring.last_row}` - последняя добавленная строка Monitoring в компактном виде;
- `{context.monitoring.latest_rows}` - до 5 последних строк Monitoring для read-only `table`;
- `{context.monitoring.latest_rows_count}` - сколько строк реально попало в `{context.monitoring.latest_rows}`.

`last_row`, `latest_rows` и `latest_rows_count` обновляются как listener-данные только пока background-задача конкретного модуля активна. Если listener остановлен или модуль ещё ни разу не запущен пользователем, host отдаёт пустую таблицу последних строк и count `0`, даже если основная таблица Monitoring продолжает получать snapshot-обновления.

Пример второго уровня:

```json
{
  "id": "latest-panel",
  "page": "last_rows",
  "entity_type": "panel",
  "title": "Latest Monitoring rows",
  "children": [
    {
      "id": "latest-table",
      "entity_type": "table",
      "value": "{context.monitoring.latest_rows}"
    },
    {
      "id": "latest-footer",
      "entity_type": "footer",
      "value": "Rows here: {context.monitoring.latest_rows_count}   Total rows: {context.tables.monitoring.total_rows}"
    }
  ]
}
```

## Диалоги

Модуль не описывает собственные modal layout. Для подтверждений используется host-команда `show_dialog` со стандартным видом NetStitch:

- `buttons: "ok"` - информационный диалог с одной кнопкой;
- `buttons: "ok_cancel"` - подтверждение с `OK` и `Cancel`;
- title и icon всегда берутся из module manifest;
- результат `ok`/`cancel` логируется host-ом как событие модуля и доставляется только модулю-владельцу диалога как `background_event` с `event_type = "ui.dialog_result"`.

## Локализация

Модуль может хранить `display_name_key`, `tooltip_key`, `title_key`, `value_key`, `placeholder_key` и `label_key`. Host загружает их только из `locales/*.ini` в папке этого модуля: сначала `locales/en-en.ini` как fallback, затем при выбранном русском UI добавляет `locales/ru-ru.ini`. Если ключ недоступен в локалях модуля, host использует обычный текст `display_name`, `tooltip`, `title`, `value`, `placeholder` или `label`. Строки автора модуля не добавляются в `resources/language/*` NetStitch.

Формат locale-файла:

```ini
[strings]
my_module.module.display_name=UI Entity Showcase Rust
my_module.entity.note.title=Note
my_module.entity.note.placeholder=Optional text
my_module.entity.help.value=Text shown in the module body
my_module.action.notice.label=Notice
```

Для многострочных значений используйте `\n`; host преобразует его в перенос строки. Значения `value_key` проходят через тот же placeholder-pass, что и обычный `value`, поэтому в локализованном footer/help text можно оставить `{context.tables.monitoring.selected_total}` и другие context placeholders.

## Визуальная карта

Схематичная карта сущностей лежит здесь: [ui-entities.svg](assets/ui-entities.svg).
