# Roadmap NetStitch

## Закрыто в текущем MVP-контуре

- Базовый Rust workspace, runtime entrypoints, tracked regression tests и canonical scripts введены.
- Локальный verification path включает `scripts/check_secrets.ps1`, `cargo fmt --check`, backend checks/tests и UI check.
- Release asset packaging реализован через `scripts/package_release_assets.ps1`, который собирает portable-папку и zip-артефакт.
- CI workflow `.github/workflows/ci.yml` добавляет Windows verification matrix: `dev-check` и `portable-smoke`.
- `Monitoring` поддерживает добавление IP в управляемый ignore-list; список по умолчанию содержит `127.0.0.0/8` и `::1/128`, элементы можно удалить, а отдельный localhost-switch удалён как лишнее состояние.
- Tracked regression test `tests/netstitch-regression/tests/language_files.rs` сверяет shipped `language/*.ini` с базовым `en-en.ini` и fallback-контрактом.
- Панель `Tracked apps` сжимается через `clamp(2/5 max, 50vw, max)`, справа размещены `Модули` и `Ignored addresses`, `Monitoring` вынесена ниже full-width строкой без смены структуры при изменении размера окна, а status/error строки живут в footer `Message`.
- Сохранённая локаль читается из SQLite до первого snapshot refresh, поэтому первый paint UI использует последнюю выбранную локаль без ожидания загрузки приложений.
- Иконки приложений закреплены за квадратным UI-контрактом: slot `40x40`, изображение `32x32`, `object-fit: contain`, `aspect-ratio: 1 / 1`.
- Опциональный browser UI доступен по корню уникального порта `https://<local-ip>:46473/` только при включённом header switch `Web localhost` или стартовом default `NETSTITCH__WEB_UI=1`, не публикует `/web` и app-name path, использует URL из desktop `/v1/web-url` с `#web_key=<derived-key>` fragment, передаёт ключ в `/v1/web-auth` POST-запросом, проверяет same-origin/CSRF header для browser API, пишет audit-события remote browser actions и дублирует основные desktop-панели поверх локального watcher API без замены desktop UI.
- Footer `Message` и клик по footer-блоку `Web server` используют `/v1/web-url`/локальный URL с IP текущей машины, если его удалось определить; default bind/base URL watcher тоже использует этот IP, а если LAN-адрес недоступен, UI явно сообщает fallback на localhost.
- Панель `Модули` стала generic module action surface: основное окно показывает только кнопки найденных модулей, а controls выбранного модуля открываются в overlay первого уровня.
- Native titlebar окна выводит `NetStitch ver. <version>`, а внутренний app header больше не выводит `NetStitch MVP` или subtitle и стал компактнее, чтобы локализованные строки не растягивали chrome/content.
- Централизованный test batch-runner добавлен как `tests/run.ps1`; `scripts/dev_check.ps1` использует `-Suite dev`, а discovery docs описывают `dev`, `regression` и `full` suites.
- Кнопка `Информация` приведена к header icon-density CSV-кнопок, а таблица `Мониторинг` разделяет header и scrollable body: scrollbar начинается с body, а правая header-зона заполняется цветом заголовка.
- UDP endpoint capture для выбранных процессов получил диагностируемый contour: WinDivert FLOW события и WinDivert NETWORK fallback для исходящих UDP-пакетов на любых remote ports, PID matching через Windows UDP owner table по локальному socket, `runtime_status.flow_capture` в snapshot/footer tooltip и unit tests для high-port UDP endpoints без хардкода приложения/IP/порта.
- Регрессии `Monitoring` delete, cloud download pagination с repeated/duplicate page и runtime discovery пользовательских `.app` коннекторов закрыты в коде и regression tests: batch delete удаляет фактические строки SQLite и сообщает backend `deleted`, cloud download двигает offset по размеру страницы Worker-а, а runtime connector parser принимает `apps/*.app` с human-friendly путями и восстанавливает archived connector rows.
- Release governance переведён в release-branch gate: push в `release` собирает Windows/Linux portable artifacts, создаёт tag, публикует GitHub Release и прикрепляет assets; ручной `workflow_dispatch` остаётся только для явно подтверждённого повтора или аварийного запуска.

## Открыто

- Спроектировать и реализовать community cloud sync поверх Cloudflare Workers + D1. Подробная зафиксированная задача, trust-модель, D1/API черновик, JWT-контракт, app-verification правила и подзадачи описаны в `docs/CLOUD_SYNC_PLAN_RU.md`.
  - На 28.05.2026 добавить server-side auth hardening: SVG captcha по модели `ingame.shin0by.com`, одинаковая ошибка для failed OAuth/captcha/rate-limit без раскрытия лишних деталей, лимит не более `3` попыток за `1 час` и понятное уведомление о retry-after.
  - На 06.06.2026 добавить в footer панели `Облако: выгрузка данных` switch `Приватно` с tooltip `Публиковать только для себя`; при включении отправленные строки должны быть доступны для поиска и загрузки только автору строк.
  - Полевое подтверждение текущей portable-сборки: cloud import должен загружать все доступные строки приложения больше одной страницы Worker-а, а `Add to monitoring` должен быть доступен только для явно выбранных staging-строк.
- Довести generic integration host до production-ready внешнего API:
  - довести ABI для native shared-library модулей (`.dll` для Windows, `.so` для Linux, `.dylib` для macOS) до production hardening, включая version negotiation, memory ownership, error codes, progress events и compatibility policy;
  - расширить UI entity API для module-provided surfaces: panels, headers, footers, subpanels, tables, actions, inputs, path fields, progress bars и dialogs;
  - обеспечить dynamic state updates для overlay без перезапуска окна и без прямого доступа модуля к UI runtime;
  - добавить regression coverage для manifest discovery, отсутствующей/битой native library, ABI mismatch, module-local DB path, offline download mock и запрета записи module data в основную SQLite БД.
- Уточнить security gate для внешних native-library модулей: trust model, подпись/источник файла, запрет автозагрузки из сети, ограничение cloud/user secrets и правила diagnostics/logging.
- Перед публичным открытием репозитория выполнить git-history cleanup без сценария утечки секретов: подготовить чистую опубликованную историю из текущего состояния проекта, схлопнуть/переписать промежуточные рабочие коммиты `development` и `release`, удалить устаревшие tags/releases/assets, затем заново опубликовать актуальный release из чистого состояния. Перед выполнением отдельно подтвердить выбранный способ (`orphan clean history` или ограниченный squash), прогнать secret scan и проверить, что ветка `main` не используется.
