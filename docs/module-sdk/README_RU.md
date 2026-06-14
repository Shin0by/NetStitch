# SDK модулей NetStitch

[English](README_EN.md)

Этот раздел описывает, как писать внешние native-модули для NetStitch. Модуль живёт рядом с portable-приложением, загружается как shared library и общается с host через C ABI + JSON. Расширение Module API добавляет события, фоновые задачи, стандартные диалоги и логирование, но не меняет native ABI: Windows `.dll` и Linux `.so` продолжают использовать тот же C/JSON entrypoint.

Модуль не пишет отдельный desktop UI или web UI. Один declarative `ui_schema` в manifest-е и host-команды из native action handler-а рендерятся NetStitch одинаково в desktop shell и browser shell: панели, подпанели, tabs, header/footer, таблицы, value/status labels, text input/textarea с опциональным `X` через `clear_button` и Enter-применением через `commit_on_enter`, dropdown/select, switch, кнопки, progress bar с опциональными фазами `progress_stages` и стандартные диалоги. Текущие значения controls, включая активную вкладку `tabs`, передаются модулю в `payload.ui_values`; во время долгого `ui_action` модуль может обновлять эти значения live через `IntegrationHostEvent.event = "ui_values"`. Если visual behavior отличается между desktop и web, это считается ошибкой host renderer-а, а не обязанностью модуля.

UI-сущности имеют централизованные дефолты: обычные элементы занимают доступную ширину, panel/grid/tab-контейнеры по умолчанию растут по высоте по содержимому, а footer-сущность является левой частью стандартного footer-а модульного overlay. Host сам добавляет кнопку `Назад` справа; она скрывается только явным `hide_host_back_button: true` у активной `footer`-сущности.

## Быстрый старт

1. Возьмите один из готовых архивов в `docs/module-sdk/packages/`.
2. Распакуйте архив рядом с portable-приложением так, чтобы получился путь `integrations/<module-id>/module.json`.
3. Запустите NetStitch заново: кнопка модуля появится в панели `Модули`.

Для разработки из исходников:

1. Скопируйте один из примеров из `docs/module-sdk/examples/`.
2. Проверьте и измените `module.json` под свой `id`, название, UI-схему и `library_paths`.
3. Соберите shared library в `bin/` под нужную платформу.
4. Обновите архивы через `scripts\package_module_sdk_examples.ps1`, если меняли tracked пример.
5. Установите папку модуля в `integrations/` и перезапустите NetStitch.

Готовые одинаковые примеры:

- [Демонстрационный модуль Rust](examples/ui-entity-showcase-rust/)
- [Демонстрационный модуль C++](examples/ui-entity-showcase-cpp/)

Оба примера показывают один и тот же модуль: `grid`, `text_input`/`textarea` с `clear_button`, `select`, `switch`, progress bar с цветными фазами, скрытые/disabled/readonly controls, `table` с настройками колонок `table_columns`, footer, стандартный dialog, header action `Start/Stop` и background-подписку на события host-а. Разница только в языке реализации native library.

## Runtime layout

```text
integrations/
  ui-entity-showcase-rust/
    module.json
    locales/
      en-en.ini
      ru-ru.ini
    bin/
      ui_entity_showcase_rust.dll
      libui_entity_showcase_rust.so
      libui_entity_showcase_rust.dylib
    assets/
      icon.svg
    data/
      module.sqlite3
```

`data/` принадлежит модулю. Модуль может сохранить туда свою SQLite-БД, кеш, загруженные файлы или результаты анализа. Основная БД NetStitch не хранит runtime-данные внешнего модуля.
`locales/` тоже принадлежит модулю: строки модуля лежат в его `locales/en-en.ini` и `locales/ru-ru.ini`, а не в глобальных `resources/language/*` NetStitch.

## Что получает модуль

Модуль получает только локальные рабочие данные, которые нужны для сценариев автоматизации:

- версия NetStitch и текущий язык UI;
- состояние мониторинга;
- текущие фильтры monitoring UI;
- счётчики таблиц: всего, отображено, выбрано;
- id выбранных и отображённых строк мониторинга;
- выбранные строки мониторинга целиком: приложение, IP, домен, порт, протокол, состояние, счётчики, подписи приложения;
- количество tracked apps и сколько из них включено.

Модуль не получает cloud credentials, Google-данные, токены, пользовательские секреты или прямой доступ к основной SQLite-БД.

## События host-а

Модуль может подписаться на host-события только runtime-командой `start_background`, возвращённой из явного пользовательского `ui_action`, чтобы обновлять свою UI-схему или запускать module-owned обработчики:

- `filters.*` - изменение watcher-owned фильтров `Monitoring`;
- `monitoring.*` - старт/остановка мониторинга, изменение/удаление/добавление строк и выборки;
- `tracked_apps.*` - изменение списка или enabled-состояния `Tracked apps`;
- `ui.controls` / `ui.tables` - события типовых UI controls и таблиц;
- `*` - все dispatch-события host-а.

Подписка не означает автозапуск. При старте NetStitch модуль только загружается и регистрирует manifest; обнаружение модуля не вызывает `ui_action`, не подписывает модуль на события и не запускает background. Фоновые задачи начинаются только когда пользователь явно нажал action модуля. Открытие overlay первого уровня тоже считается явным действием, но host вызовет `ui_action` только если модуль сам объявил action `module.open` или `open`, а модуль в ответ вернул host-команду `start_background`. Стандартная header-кнопка `Остановить` принудительно снимает background-подписки выбранного модуля, останавливает его фоновую работу и возвращает пользователя на главный экран NetStitch; её tooltip: `Прерывает работу модуля и его фоновых процессов`.

## Что может сделать модуль через host

Разрешённые команды host-а:

- `set_filters` - применить `UiFiltersDto` и отобразить значения в фильтрах UI;
- `start_monitoring` / `stop_monitoring` - включить или остановить мониторинг;
- `add_tracked_app` - добавить приложение по локальному пути;
- `set_tracked_app_enabled` - включить/выключить одно приложение;
- `set_all_tracked_apps_enabled` - включить/выключить overlay `Enable all`;
- `delete_tracked_app` - удалить tracked app из активного списка;
- `confirm_monitoring_rows` - подтвердить/снять подтверждение строк мониторинга;
- `delete_monitoring_rows` - удалить строки мониторинга;
- `select_monitoring_rows` - зарезервированная UI-команда выбора строк;
- `start_background` / `stop_background` - управлять module-owned фоновой задачей без автозапуска при старте приложения;
- `set_module_page` - переключать стандартный overlay модуля между страницами UI-схемы;
- `set_ui_values` - выставлять host-owned значения controls и активные вкладки `tabs`;
- `browse_window` - открыть host-owned окно выбора папки/файла/save target и записать путь в UI state модуля;
- `log_event` - записать событие в `system_events` с source равным отображаемому имени модуля;
- `show_dialog` - показать стандартный модульный диалог `OK` или `OK+Cancel` с заголовком/иконкой модуля и залогировать выбранный результат.

Cloud upload/download и CSV import/export не автоматизируются модулями. Эти действия остаются ручными действиями пользователя.

Severity в ответах и логах всегда один из четырёх системных уровней: `info`, `success`, `warning`, `error`.

## Справочник

- [Создание модуля пошагово](CREATING_MODULES_RU.md)
- [Примеры и готовые архивы](EXAMPLES_RU.md)
- [ABI и JSON-контракт](REFERENCE_RU.md)
- [UI-сущности](UI_ENTITIES_RU.md)
- [Host context и данные таблиц](HOST_CONTEXT_RU.md)
- [Команды host-а](COMMANDS_RU.md)
- [Визуальная карта UI-сущностей](assets/ui-entities.svg)
