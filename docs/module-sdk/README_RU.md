# SDK модулей NetStitch

Этот раздел описывает, как писать внешние native-модули для NetStitch. Модуль живёт рядом с portable-приложением, загружается как shared library и общается с host через C ABI + JSON. Расширение Module API добавляет события, фоновые задачи, стандартные диалоги и логирование, но не меняет native ABI: Windows `.dll` и Linux `.so` продолжают использовать тот же C/JSON entrypoint.

Модуль не пишет отдельный desktop UI или web UI. Автор описывает один declarative `ui_schema` в manifest-е и возвращает host-команды из native action handler-а, а NetStitch сам рендерит те же типовые сущности в desktop shell и browser shell: панели, подпанели, tabs, header/footer, таблицы, value/status labels, text input с `X`, textarea, dropdown/select, switch, кнопки, progress bar и стандартные диалоги. Текущие значения controls, включая активную вкладку `tabs`, передаются модулю в `payload.ui_values`. Если visual behavior отличается между desktop и web, это считается ошибкой host renderer-а, а не обязанностью автора модуля.

## Быстрый старт

1. Создайте папку `integrations/hello-world/`.
2. Положите туда `module.json`.
3. Соберите shared library в `integrations/hello-world/bin/`.
4. Запустите NetStitch заново: кнопка модуля появится в панели `Модули`.

Минимальные примеры:

- [Rust hello-world](examples/hello-world-rust/)
- [C++ hello-world](examples/hello-world-cpp/)

Оба примера показывают первый уровень меню модуля: выбранные/всего строки Monitoring, последнюю добавленную строку, кнопку `Hello` со стандартным диалогом модуля и второй уровень `Last rows` с таблицей последних строк. `Start/Stop` и `About module` вынесены в типовой header модуля как квадратные header-action кнопки; `Start/Stop` использует `pulse_when_background_active`, поэтому host сам включает пульсацию кнопки, пока background-задача модуля работает.

## Runtime layout

```text
integrations/
  hello-world/
    module.json
    bin/
      hello_world.dll
      libhello_world.so
      libhello_world.dylib
    assets/
      icon.svg
    data/
      module.sqlite3
```

`data/` принадлежит модулю. Модуль может сохранить туда свою SQLite-БД, кеш, загруженные файлы или результаты анализа. Основная БД NetStitch не хранит runtime-данные внешнего модуля.

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
- `log_event` - записать событие в `system_events` с source равным отображаемому имени модуля;
- `show_dialog` - показать стандартный модульный диалог `OK` или `OK+Cancel` с заголовком/иконкой модуля и залогировать выбранный результат.

Cloud upload/download и CSV import/export не автоматизируются модулями. Эти действия остаются ручными действиями пользователя.

Severity в ответах и логах всегда один из четырёх системных уровней: `info`, `success`, `warning`, `error`.

## Справочник

- [ABI и JSON-контракт](REFERENCE_RU.md)
- [UI-сущности](UI_ENTITIES_RU.md)
- [Host context и данные таблиц](HOST_CONTEXT_RU.md)
- [Команды host-а](COMMANDS_RU.md)
- [Визуальная карта UI-сущностей](assets/ui-entities.svg)
