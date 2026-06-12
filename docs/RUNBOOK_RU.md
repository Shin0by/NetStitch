# Runbook NetStitch

## Текущее состояние

Проект находится в bootstrap-состоянии. Production runtime, service management и recovery automation ещё не введены.

## Базовый порядок работы

1. Открыть `README.md`, `AGENTS.md` и обязательные документы из `docs/`.
2. Уточнить область работы по стандартному списку из `AGENTS.md`.
3. Для reusable helpers сначала смотреть `tools/README.md`.
4. Для проверки покрытия и test entrypoints сначала смотреть `tests/README.md`.

## Локальный запуск

Текущий локальный runtime-контур:
- `scripts/run_ui.ps1` - запуск desktop UI; приложение поднимает embedded watcher runtime внутри основного процесса
- `scripts/compile.ps1` - canonical compile action: verification + release build + обновление portable-папки + Windows zip portable в `dist\release-assets`
- `scripts/dev_check.ps1` - форматирование, сборка и тесты workspace
- `tests/run.ps1` - централизованный test batch-runner; `-Suite dev` используется внутри `scripts/dev_check.ps1`, `-Suite regression` запускает tracked regression tests, `-Suite full` запускает расширенный пакет unit/regression/UI-contract/browser-parity тестов
- `scripts/check_secrets.ps1` - high-confidence проверка tracked и неignored untracked text files на private key blocks и известные token formats; запускается внутри `dev_check.ps1`
- `scripts/package_portable.ps1` - сборка core portable-папки с `NetStitch.exe`, native runtime libraries в `libs\`, `apps\`, `language\`, `storage\` и WinDivert runtime-файлами в корне portable-папки; внешние integration-модули не включаются по умолчанию и добавляются только явным `-PackageModules <id>` или `NETSTITCH__PACKAGE_MODULES` (`*` явно включает все repo-local modules); WinDivert берётся из `NETSTITCH__WINDIVERT_DIR`, `NETSTITCH__WINDIVERT_DLL` или tracked fallback `resources\runtime\windivert\windows-x86_64`; portable runtime lifecycle остаётся native-only; packaging ставит `storage\clear-system-events-on-next-start`, чтобы следующий запуск очищал только старую историю `system_events`
- `scripts/update_release_version.ps1` - обновляет tracked `config\release-version.json` из локально протестированного `dist\NetStitch-win64-portable\config\version-manifest.json`; это обязательный шаг перед push в `release`, чтобы GitHub Release получил ту же версию, которую пользователь проверял локально
- `scripts/package_portable_linux.sh` - сборка Linux core portable-папки `dist/NetStitch-linux64-portable` с `NetStitch`, `NetStitch.desktop`, app icon, native `.so` runtime libraries в `libs/`, `apps/`, `language/`, `storage/`; внешние integration-модули не включаются по умолчанию и добавляются только явным `--package-modules <id>` или `NETSTITCH__PACKAGE_MODULES` (`*` явно включает все repo-local modules); запускается из Linux/WSL, использует отдельный `target/linux-portable`, чтобы не смешивать Windows и Linux cargo-кэш, наследует общий package revision из `NETSTITCH__PACKAGE_REVISION` или Windows portable manifest и подходит для WSLg GUI-smoke перед Linux release; packaging ставит `storage/clear-system-events-on-next-start`, чтобы следующий запуск очищал только старую историю `system_events`
- `config\endpoint_probe_targets.txt` - editable список network endpoint probe targets в формате `host:port` или `protocol://host:port`; desktop/browser UI перечитывают его при refresh и передают targets в embedded watcher snapshot requests как `endpoint_probe_target` + `endpoint_probe_protocol`, а native network tool library получает targets только от caller-а и поддерживает IPv4, bracketed IPv6 и домены
- `scripts/package_release_assets.ps1` - сборка Windows portable-папки и zip release asset `NetStitch-win64-portable-<version>.zip` в `dist\release-assets`; в корне zip лежит папка `NetStitch-win64-portable\`, если `-Version` не передан, имя zip берётся из `modules.app.full_version` portable manifest
- `scripts/package_release_assets_linux.sh` - сборка Linux portable-папки и tar.gz release asset `NetStitch-linux64-portable-<version>.tar.gz` в `dist/release-assets`; в корне архива лежит папка `NetStitch-linux64-portable/`
- `.github/workflows/release.yml` - GitHub portable release workflow: push в ветку `release` запускает workflow `Portable Release`. Он собирает Windows zip и Linux tar.gz на GitHub runners, берёт базовую версию из `[workspace.package].version`, а full-version/tag - из tracked `config\release-version.json`, который синхронизируется с локально протестированной portable-сборкой. Если release-version file отсутствует, workflow падает обратно на tags fallback. Затем workflow передаёт revision в оба package script-а и после успешной сборки создаёт tag, GitHub Release и public assets. Manual workflow dispatch остаётся только для явно подтверждённого повтора/аварийного запуска. Generated `dist\...\config\version-manifest.json` хранит версию уже собранной portable-папки и не коммитится; tracked source-of-truth для чистого GitHub runner - `config\release-version.json`.
- `scripts/setup_github_push.ps1` - bootstrap project-specific GitHub push contour: создаёт отдельный deploy key `~/.ssh/netstitch_github_deploy`, обновляет `known_hosts` официальными GitHub host keys из `api.github.com/meta`, настраивает repo-local `core.sshCommand` и печатает public key для GitHub Deploy key; `-UseHttpsPort443` включает fallback через `ssh.github.com:443`, `-TestGitHub` проверяет auth path
- `scripts/sign_windows.ps1` - локальная development-подпись project-owned Windows `.exe` через self-signed `test_certificate`; без `-EnsureTrustedForCurrentUser` не меняет доверенные certificate stores
- `scripts/load_cloudflare_env.ps1` - загружает ignored `.env.cloudflare.local` в текущий PowerShell-процесс перед Cloudflare checks/deploy без вывода secret values
- `scripts/wrangler.ps1` - запускает project-local Wrangler из `.local\wrangler` без глобального PATH; перед live-командами использовать `scripts\load_cloudflare_env.ps1`
- `scripts/check_cloud_config.ps1` - проверяет live Cloudflare Worker config, D1 binding `DB`, Google OAuth vars и наличие Worker secrets без вывода secret values
- при каждом packaging `dist\NetStitch-win64-portable` очищается от устаревших publish-файлов; пользовательское `storage\` сохраняется, shipped connector `.app` manifests и `.svg` иконки обновляются в `apps\` и `apps\icons\`, пользовательские `.app`, `.svg` и `.png` в этих папках не удаляются
- packaging обновляет shipped-файлы `language\en-en.ini` и `language\ru-ru.ini`, но не удаляет пользовательские `.ini` локали из `language\`
- `resources/connectors/apps/*.app` - tracked шаблоны runtime-коннекторов; при packaging копируются в `dist\NetStitch-win64-portable\apps`
- `resources/language/*.ini` - tracked shipped-локали UI; при packaging копируются в `dist\NetStitch-win64-portable\language`, пользователь может добавить свои файлы вида `zh-cn.ini`, а отсутствующие ключи будут браться из fallback `en-en`
- выбранная локаль UI хранится в SQLite `storage\netstitch.sqlite3` в таблице `app_settings` с ключом `ui.language`; если значение отсутствует или указывает на несуществующую локаль, UI использует fallback `en-en`
- bulk `Enable all` хранится в SQLite `app_settings` с ключом `ui.enable_all_overlay`; он не меняет `tracked_apps.enabled`, но monitoring loop и UI snapshot считают все приложения эффективного включёнными, пока overlay активен
- tray-галка `Запоминать положение` хранится в SQLite `app_settings` как `ui.remember_window_placement`; при включении UI сохраняет `ui.window_x`, `ui.window_y`, `ui.window_width`, `ui.window_height` и `ui.window_hidden` только при завершении приложения. При следующем старте восстанавливается геометрия, но окно всегда открывается видимым; hidden-state используется только для hide/show flow внутри текущей tray-сессии
- если `Запоминать положение` выключено, tray-действия показа окна центрируют окно на текущем мониторе и возвращают его к минимальному размеру вместо последней пользовательской геометрии
- web-доступ для browser shell хранится в SQLite `app_settings` с ключом `web.localhost_enabled`; switch `Web localhost` в header включает/выключает корень уникального порта локального watcher; default bind/base URL использует локальный IP текущей машины и fallback на `127.0.0.1:46473` только если IP определить нельзя; default URL выглядит как `https://<local-ip>:46473/#web_key=<derived-key>`; `/web` и app-name сегмент не публикуются; endpoint `/v1/web-url` возвращает фактический URL для footer/clipboard. Ключ выводится только как fragment после `#`, browser JS передаёт его в `/v1/web-auth` POST-запросом, получает HttpOnly SameSite cookie и удаляет fragment из адресной строки; если открыть только IP и порт без ключа, browser shell показывает окно ввода ключа. HTTPS использует app-owned самоподписанный сертификат `storage\web-server.cert.pem` и private key `storage\web-server.key.pem`; браузер может потребовать ручное доверие сертификату.
- tray menu дублирует desktop-доступ к `Web сервер`: check-item `Web сервер` меняет persisted setting `web.localhost_enabled` тем же `/v1/settings` контуром, что и header switch
- Пользовательский ignore-list хранится в `ignored_addresses`, по умолчанию содержит `127.0.0.0/8`, `::1/128` и точные CIDR-правила для обнаруженных локальных IP текущей машины (`/32` или `/128`), а UI применяет к отображению и `Confirm filtered` только правила из этого списка; локальный IP определяется последовательными IPv4/IPv6 route-probe попытками по 8 target-адресам на семейство с лимитом `10s` на попытку, затем фильтруются special/transition адреса вроде Teredo `2001:0::/32` и 6to4 `2002::/16`; отдельного localhost-switch больше нет; удаление правила идёт через modal-подтверждение с пояснением для localhost/local-IP правил
- footer `DNS` показывает кэшируемый backend network probe: зелёный индикатор означает первый успешный endpoint target, красный - успешный target ещё не найден или все проверки упали; tooltip содержит полный список endpoint probe addresses и статусы, а проверка выполняется фоном, чтобы snapshot/UI не блокировались
- footer `Message` читает последние строки из SQLite `system_events`, а не из отдельного runtime-списка. При старте туда записываются build/current/app, watcher, tool, web server и DNS статусы, затем connector/module diagnostics и пользовательские/runtime события. UI показывает последние 10 сообщений, copy-history копирует последние 100, одинаковые подряд строки увеличивают `repeat_count`, а таблица ротируется до 9999 строк.
- После compile/portable packaging первый запуск применяет marker `storage\clear-system-events-on-next-start`: очищает только `system_events` и удаляет marker, не трогая monitoring observations, tracked apps, настройки, cloud session или другие runtime-таблицы. Это нужно, чтобы свежая тестовая сборка не смешивала новые события со старой историей сообщений.
- `docs/GLOSSARY_RU.md` - нормализация коротких пользовательских команд (`компиль`, `билди`, `деплой`, `пуш`, `релиз` и др.)
- `docs/SIGNING_RU.md` и `config/signing.example.toml` - безопасный контур данных для будущего подписывания файлов; реальные сертификаты, PFX, PIN и пароли не хранить в git
- `netstitch-core` при bootstrap запускает `netstitch-connectors` и добавляет найденные known apps в `Tracked apps` как disabled, не перезаписывая уже существующие строки
- для versioned connector apps core сохраняет одну строку продукта по `connector_id`: при появлении новой versioned-папки путь строки обновляется на новый executable, enabled/disabled состояние сохраняется, а stale-строки старых версий удаляются из `tracked_apps`; их наблюдения не удаляются, а отвязываются от активного приложения и остаются историческими строками
- `netstitch-core` при bootstrap не добавляет manual-приложения по локальным путям: новая SQLite БД стартует без пользовательских приложений по умолчанию. Connector discovery может добавить найденные known apps только как disabled connector-managed строки; manual apps появляются после явного добавления пользователем и отображаются сверху по новизне
- при ручном добавлении executable core пытается извлечь его Windows shell/icon и положить в `apps\icons\manual_<path-hash>.png`; hash берётся от полного нормализованного пути, поэтому одинаковые имена файлов из разных папок не перетирают иконки друг друга
- если рядом с exe есть папка `apps/` или задан `NETSTITCH__CONNECTORS_DIR`, connector discovery читает пользовательские `.app` manifests оттуда; удаление файла или `enabled = false` отключает соответствующий connector, а при следующем bootstrap connector-managed строка с отсутствующим `connector_id` удаляется из SQLite `tracked_apps`; manual-приложения без `connector_id` не удаляются
- UI по умолчанию работает с embedded watcher runtime; browser shell/API при включённом web-доступе публикуется на локальном IP текущей машины (`https://<local-ip>:46473/#web_key=<derived-key>`) и fallback на `https://127.0.0.1:46473/#web_key=<derived-key>`, если IP определить нельзя; `NETSTITCH__UI_USE_MOCK=1` включает mock-only режим для изолированной отладки UI
- при запуске UI сначала показывает окно с локальным cold-start snapshot и только после первого paint выполняет bootstrap embedded watcher runtime, SQLite и discovery
- Header-фильтры `Monitoring` (`Приложение`, `IP`, `Домен`, `Порт`, `Протокол`, `Статус`, `Публичные`) хранятся в watcher как единый `UiFiltersDto`. Если фильтр меняется в desktop или browser UI, watcher публикует новое значение в следующих `snapshot`; второй клиент должен догонять это состояние без ручного refresh и без собственных независимых filter-signals. Состояние switch `Публичные` дополнительно сохраняется в SQLite `app_settings` как `ui.monitoring.public_ip`, чтобы оно переживало перезапуск.
- Desktop header содержит отдельные кнопки CSV import/export для `Monitoring`: import читает CSV с колонками таблицы мониторинга и дополняет текущую SQLite-таблицу без очистки; export сохраняет выбранные для экспорта строки. При импорте нескольких файлов повторяющиеся endpoints объединяются по `IP + port + protocol`, новые endpoints добавляются.
- CSV import/export переносит `app_connector_id`, `cloud_app_id` и поля подписи приложения. При диагностике cloud upload важно смотреть provenance самой observation-строки: импортированная подпись не заменяется локальной подписью найденного exe, а только используется для проверки, можно ли считать строку verified.
- Browser shell выбирает executable/profile/CSV пути через runtime-host файловый браузер: `/v1/filesystem/browse` показывает каталоги машины, где запущен `NetStitch.exe`, `/v1/filesystem/read-text` читает выбранный CSV для импорта, а `/v1/filesystem/write-text` сохраняет CSV export. Это намеренно не native browser file picker, потому что браузер мог быть открыт с другой машины в LAN.
- Cloud overlay открывается только явной кнопкой в desktop header. Открытие overlay выполняет live-запросы к `netstitch-sync` для health/quota/app-list, пока overlay открыт эти данные обновляются автоматически каждые 10 секунд; каждая refresh-операция имеет общий deadline `9.5` секунд и устаревший refresh-result не должен перетирать более свежую авторизацию; ручной кнопки refresh внутри overlay нет. Startup, render, snapshot refresh и окно `Информация` cloud-запросы не выполняют. Для upload пользователь явно запускает Google OAuth sign-in из overlay; Worker проверяет server-issued SVG captcha/rate-limit, state-bound browser-verification proof и OAuth callback, рендерит промежуточные browser-страницы на текущем языке UI, после чего UI сохраняет только безопасные account metadata, DPAPI-защищённый local session/client key state и DPAPI-защищённые Google name/email display blobs до явного выхода. Google name/email могут вернуться в текущий ответ авторизации только для отображения в UI; в D1/cloud сохраняются только HMAC-значения subject/email через `NETSTITCH_IDENTITY_PEPPER`, а в CSV, monitoring rows и логах Google name/email не сохраняются. Browser-verification выполняется на промежуточной странице перед Google redirect и включает SHA-256 proof-of-work, короткий memory-walk, canvas/WebGL/Worker/timing/interaction signals и pointer percent risk fields. Кнопка `Выход` в overlay удаляет local auth/session state, DPAPI-защищённые Google name/email blobs и obsolete login/password keys. Пользовательские пароли, Google access token и provider secrets не должны попадать в SQLite, portable storage или логи.
- Desktop UI при старте выполняет один короткий фоновый запрос к GitHub Releases API для проверки новой версии. Этот запрос не нужен для базовой работы NetStitch, не блокирует UI, не использует secrets и при ошибке просто оставляет кнопку `Информация` в обычном состоянии. Кнопка `Обновить программу` в окне `Информация` только открывает GitHub Releases в системном браузере.
- Полная очистка облачной D1 выполняется только явной operator-командой `scripts\clear_cloud_database.ps1 -Yes`. Скрипт удаляет все данные, пришедшие от клиентов или созданные вокруг клиентских действий: пользователей, идентификаторы клиентов, сессии, ключи, авторов, каталог приложений, наблюдения, OAuth/captcha/rate-limit/browser-verification состояния, quota windows, audit/cache rows. Сохраняются только схема, миграции и служебные Cloudflare/D1 metadata, чтобы база оставалась чистой и готовой к работе.
- Cloudflare Worker/D1 bootstrap на новом аккаунте: создать Worker subdomain, Google OAuth redirect `https://netstitch-sync.<workers-subdomain>.workers.dev/v1/auth/google/callback`, D1 database `netstitch`, ignored `src\netstitch-cloud-worker\wrangler.local.toml` на базе `wrangler.example.toml` с binding `DB`, затем выполнить `scripts\load_cloudflare_env.ps1; scripts\wrangler.ps1 d1 migrations apply netstitch --remote --config src\netstitch-cloud-worker\wrangler.local.toml`, загрузить Worker secrets `NETSTITCH_GOOGLE_CLIENT_SECRET` и `NETSTITCH_IDENTITY_PEPPER`, выполнить `scripts\check_cloud_config.ps1`, `scripts\check_cloud_worker.ps1`, `scripts\wrangler.ps1 deploy --config src\netstitch-cloud-worker\wrangler.local.toml` и live smoke `tests\cloud_auth_smoke.ps1 -BaseUrl https://netstitch-sync.<workers-subdomain>.workers.dev`. GitHub Actions values хранить как `PROVIDER__CLOUDFLARE__API_TOKEN` secret, `PROVIDER__CLOUDFLARE__ACCOUNT_ID` variable и provider/project OAuth/D1 variables/secrets по `.env.example`; live values не коммитить.
- Cloud upload nickname хранится локально как обычная публичная настройка `cloud.upload.nickname` в SQLite `app_settings`, подставляется при следующем запуске и обновляется при вводе валидного значения. Это не auth secret и не часть monitoring/CSV rows; logout из Google-сессии его не удаляет.
- Cloud upload отправляет confirmed/not-exported observations: строки с совпавшей verified подписью группируются как `VerifiedUpload`, импортированные или неподтверждённые строки группируются как `CloudImportUntrusted`. D1 migration `0007_observation_publisher_keys.sql` добавляет `publisher_key` в primary key observations для provenance/write-conflict boundary, а public cloud catalog/download наружу агрегирует строки по общему Monitoring/CSV ключу `app_id + IP + port + protocol`; приватный/author-scoped срез остаётся отдельным namespace. `0012_client_quota_operations.sql` добавляет техническую таблицу для группировки страниц/пачек одной пользовательской upload/download операции по `operation_id`. После изменения Worker/D1-схемы нужно сначала применить миграции к `NetStitch`, затем деплоить Worker и запускать `tests\cloud_auth_smoke.ps1`.
- Browser shell cloud import/export не делает внешних запросов из JavaScript. Веб-страница обращается только к локальным routes embedded watcher runtime (`/v1/cloud/apps`, `/v1/cloud/observations`, `/v1/cloud/quota`, `/v1/cloud/local-author`, `/v1/cloud/upload`), а runtime уже выполняет разрешённые cloud-запросы как native-прокси явного действия пользователя. При диагностике web cloud сначала проверять локальные runtime routes и только затем внешний Worker.
- Одноразовая миграция локальных observation-строк на cloud app identity должна выполняться через documented embedded maintenance action после его фиксации. Операция не должна удалять строки или перезаписывать уже импортированную подпись: она заполняет только пустые `cloud_app_id`/`app_signature_*` по текущим tracked apps. Ключ приложения строится из всех стабильных доказательств: валидный Authenticode leaf SPKI, если он есть, плюс Windows VersionInfo `CompanyName` и `ProductName`; значения нормализуются в нижний регистр, версии, путь и имя exe не используются.
- Browser shell обновляется секционно: background refresh сверяет сигнатуры блоков `snapshot` и перерисовывает только изменившиеся регионы (`Tracked apps`, `Monitoring`, `Модули`, footer/status и т.д.). Если web начинает мерцать целиком или не догоняет изменения из desktop, в первую очередь проверять `snapshot.filters`, `snapshot.observed_endpoints` и browser-контур `snapshotSyncDelta/applySnapshotSyncDelta` в embedded watcher/browser-shell runtime.
- Embedded watcher runtime выбирает backend по платформе: Windows использует TCP owner tables и пытается включить WinDivert FLOW backend для UDP/QUIC endpoints, Linux basic backend читает `/proc/net` и сопоставляет socket inode с PID. Если WinDivert DLL не найдена или нет админ-прав, Windows runtime продолжает работать в TCP-only режиме и пишет debug-диагностику. Если выбранное приложение использует UDP socket без remote endpoint в стандартной Windows UDP table, ожидание NetStitch - выявить все remote endpoints этого процесса через elevated WinDivert FLOW/packet contour, без хардкода конкретной игры, IP или порта.
- native network tool module подключается как linked library: embedded watcher runtime вызывает library фоновыми one-shot проходами, если SQLite setting `ip_enrichment.enabled` не выключен. Library actions `once`, `lookup`, `lookup-domain` и `probe-endpoint` доступны runtime-у через native ABI; `probe-endpoint` не имеет встроенных адресов, требует targets от caller-а и проверяет доступность endpoint target, а не только DNS. Watcher-managed enrichment/probe jobs используют общий runtime lock, поэтому одновременно выполняется не больше одного native tool-call. Сетевые ошибки DNS/WHOIS не считаются отказом monitoring path; таблицы просто продолжают показывать `Unknown`, пока кеш не заполнен
- `NETSTITCH__WEB_UI=1` задаёт только стартовый default для browser shell на корне уникального порта, если в SQLite ещё нет `web.localhost_enabled`; после первого сохранения управлять режимом надо switch-ом `Web localhost`; default bind/base URL embedded runtime использует локальный IP текущей машины и fallback на `127.0.0.1:46473`, `/web` и app-name сегмент не используются, а страница должна показывать рабочие панели вместо raw JSON dump; footer copy берёт HTTPS URL с web-key fragment из `/v1/web-url`
- для ручной диагностики без UI отдельный portable helper больше не предусмотрен; использовать desktop/browser UI, tracked tests и будущий documented embedded maintenance entrypoint, когда он будет утверждён
- новый automation entrypoint нужно отдельно утвердить и описать до использования
- полноценный автономный CLI workflow без UI больше не является текущим runtime-контуром; ручная работа выполняется через desktop/browser UI, а headless сценарий остаётся открытым решением до появления app-owned entrypoint
- `NETSTITCH__WINDIVERT_DLL` можно указать на конкретный `WinDivert.dll`, если он не лежит в корне portable-папки рядом с app и не находится через стандартный DLL search path
- `NETSTITCH__INTEGRATIONS_DIR` можно указать на папку с runtime-модулями. Если переменная не задана, host ищет `integrations\` рядом с exe и в рабочей папке.
- Внешние интеграции обнаруживаются по manifest-файлам `integrations\<module>\module.json`. Панель `Модули` показывает только кнопки найденных модулей, сохраняет их порядок в `ui.modules.order`, а первый уровень overlay получает controls от выбранного модуля через generic UI entity API. Manifest выбирает native library через `library_paths` по ключам `<os>-<arch>`, `<os>`, затем `default`: `.dll` для Windows, `.so` для Linux и `.dylib` для macOS при общем C/JSON ABI.
- В overlay модульного экспорта обычная последовательность для доменов такая: выбрать строки `Monitoring`, открыть модульный overlay/дополнительные настройки, добавить доказанные домены из выбранных строк или ввести домены вручную, применить настройки и запустить анализ. Preview/analyze не должны писать файлы; фактическая запись выполняется только явным module action.
- Если нужно экспортировать не отдельные IP, а диапазоны, в `Дополнительных настройках` включается switch `Использовать диапазоны вместо IP`. `Анализ` учитывает текущий switch даже до применения настроек и показывает реальные export targets: для строк с валидным cached range используются диапазоны, для остальных остаётся экспорт по одиночным IP. Статистика `Анализ` разделена на группы `Адреса`, `Домены` и `Диапазоны`; в каждой группе показываются только `Выбрано`, `Будет добавлено` и `Пропущено`.
- Если в `Дополнительных настройках` нужно создать правила для непокрытых `protocol + port`, сначала выполнить `Анализ`, затем выбрать длинный `Шаблон правила` в верхней панели настроек. Панель `Не покрыто правилами профиля:` находится снизу и заполняет оставшуюся высоту оверлея, а панель настроек всегда занимает всю ширину окна.
- Текущее полевое наблюдение для UDP game traffic: выбранный процесс может логировать remote endpoints вида `34.79.201.95:64293`, `34.79.201.95:64318` или `34.79.201.95:64337`, при этом `Get-NetTCPConnection`/TCP owner tables показывают только TCP endpoints вроде `104.18.124.108:443`, `104.18.125.108:443`, `34.182.171.49:443`, `34.149.135.127:8000`, а UDP table показывает только локальный socket `0.0.0.0:64090` без remote. Это не доказательство TCP `:443` к `34.79.201.95`; это generic целевой кейс для UDP/FLOW захвата всех remote UDP endpoints выбранного процесса.
- Overlay модуля показывает module-provided status без badge-статусов, detected/export paths как read-only input-like поля и progress-блок после старта module download; все длинные пути отображаются через общий однострочный path-field с горизонтальной прокруткой. Открытие overlay и ввод пути не запускают тяжёлую проверку файловой системы; root validation выполняется только при явном module action. Backend публикует прогресс через generic integration endpoints, desktop/browser UI стартуют загрузку неблокирующим action и дальше опрашивают progress, чтобы watcher API оставался доступным во время скачивания.
- Выбранные пути, provider state, download/cache state и module UI state сохраняются только в module-local SQLite `integrations\<module>\data\module.sqlite3`; основная SQLite БД `storage\netstitch.sqlite3` не содержит runtime-данные внешних модулей.
- `cargo run -p netstitch-connectors --example discover` показывает known-app connector manifests, платформенные process aliases/discovery sources и реально найденные executable paths на текущей машине; discovery читает источники текущей ОС, но не включает найденные приложения автоматически

Особенность текущего verification path:
- `scripts/dev_check.ps1` закрепляет процесс на последних 20 логических CPU и оставляет первые свободными, чтобы не забивать всю машину во время локальной проверки

Нормализованный compile/deploy action из глоссария:
- запуск `scripts/compile.ps1`
- внутри него: `scripts/dev_check.ps1`, release-сборка через `scripts/package_portable.ps1`, обновление `dist\NetStitch-win64-portable`
- проверка наличия `target/release/NetStitch.exe` для локального тестирования
- core portable-папка должна содержать `NetStitch.exe`, `libs\` с platform native libraries, `WinDivert.dll`/`WinDivert64.sys` в корне для расширенного Windows-мониторинга, `apps\*.app`, `apps\icons\`, `language\*.ini` и `storage\` с `netstitch.sqlite3`, `exports\`; bundled integration modules не входят без явного `-PackageModules`/`NETSTITCH__PACKAGE_MODULES`, а при opt-in каждый модуль живёт в `integrations\<module>\` с `module.json`, `bin\`, `assets\`, `data\`; для UDP/QUIC packaging указать `NETSTITCH__WINDIVERT_DIR` или `NETSTITCH__WINDIVERT_DLL`; если основной `target\release` занят запущенным exe, использовать `scripts/package_portable.ps1 -TargetDir target-release-verify`
- version policy: NetStitch использует формат `major.minor.patch.revision`. `major` повышается для принципиально новой архитектуры программы; `minor` - для полноценной новой пользовательской функции; `patch` - для изменения API, внутренних механизмов и небольших доработок существующих функций; `revision` - общий последний разряд, который повышается на `+1` для каждой новой сборки/выпуска независимо от того, какой предыдущий разряд изменился. Базовая версия `major.minor.patch` хранится в `[workspace.package].version` в `Cargo.toml`; full-version для portable/release хранится в `config\release-version.json` и generated `dist\...\config\version-manifest.json`.
- перед итоговым ответом после любой завершённой работы Codex нужно выполнить `scripts/compile.ps1`, чтобы пользователь всегда мог тестировать актуальную portable-выкладку в `dist\NetStitch-win64-portable`

Нормализованный push/release action из глоссария:
- `пуш/пушни/запуш` -> commit + push в `origin/development` по умолчанию
- после push итоговый блок всегда содержит строку `Tag:` сразу после `Коммит:`; если в текущем push создавался, переносился или был явно указан git tag, в строке пишется его имя, иначе строка остаётся пустой как `Tag: `
- `релиз` -> marker commit в `development`, push в `origin/development`, затем fast-forward/merge `release` до того же commit для запуска portable artifact build и публикации; ветка `main` не участвует в текущем workflow и не создаётся. GitHub workflow `Portable Release` на push в `release` собирает Windows/Linux portable archives, создаёт tag, публикует GitHub Release и прикрепляет assets после signing gate.
- перед `релиз` обязательно пройти signing gate: если настроен только self-signed `test_certificate`, сначала явно сообщить пользователю, что production-сертификат не решён, и не продвигать `release`/не публиковать assets без отдельного подтверждения или настройки production signing
- base-version для релиза берётся из `[workspace.package].version` в `Cargo.toml`; full-version/tag берётся из tracked `config\release-version.json`, который обновляется из локального `dist\...\config\version-manifest.json` перед release commit. Git tags остаются fallback и защитой от конфликта уже существующего tag.

Обычный push в `development`:
1. Проверить, что рабочая ветка `development`: `git status --short --branch`.
2. Выполнить релевантные тесты; для изменений tooling/release/docs минимум `cargo test -p netstitch-regression package_app_docs`, `scripts\check_secrets.ps1` и `git diff --check`.
3. Если менялся пользовательский runtime или packaging, выполнить `scripts\compile.ps1`; для Linux packaging отдельно проверить `scripts/package_portable_linux.sh` из Linux/WSL контура.
4. Сделать commit с subject и body-описанием изменений.
5. Выполнить `git push origin development`.
6. Вернуть пользователю стандартный post-push блок из `AGENTS.md`.

Релизный build/publish процесс:
1. Работать из актуальной `development`.
2. Пройти signing gate из `docs/SIGNING_RU.md`.
3. Выполнить `scripts\update_release_version.ps1` после локального `scripts\compile.ps1`, чтобы `config\release-version.json` совпал с протестированной portable-версией.
4. Взять expected full version из `config\release-version.json`; workflow использует этот файл и должен опубликовать tag `v<full-version>`.
5. Создать release-marker commit в `development`: `Release: NetStitch v<full-version>`.
6. Push marker commit в `origin/development`.
7. Fast-forward продвинуть только `release` до marker commit и выполнить `git push origin release`; этот push должен сразу инициировать GitHub Actions деплой.
8. Дождаться GitHub Actions `Portable Release`: он должен собрать `NetStitch-win64-portable-<full-version>.zip` и `NetStitch-linux64-portable-<full-version>.tar.gz`, проверить наличие Windows `WinDivert.dll`/`WinDivert64.sys`, Windows native DLL под `libs\` и Linux native `.so` под `libs/`, создать tag `v<full-version>`, опубликовать GitHub Release и прикрепить архивы.
9. Manual workflow `Portable Release` с `publish_release=true` использовать только для явно подтверждённого повтора или аварийного запуска, если штатный release-branch push не смог быть использован.
10. Вернуться на `development` и проверить, что `origin/development` и `origin/release` указывают на ожидаемый commit; `main` не проверяется и не используется.

Операционный протокол push в GitHub по SSH:
- основной bootstrap: выполнить `scripts/setup_github_push.ps1`; скрипт создаст project-specific ключ `~/.ssh/netstitch_github_deploy`, обновит `known_hosts` и настроит repo-local `core.sshCommand`
- добавить напечатанный public key в Deploy keys проектного репозитория с write access
- после добавления ключа выполнить `scripts/setup_github_push.ps1 -TestGitHub`; ожидаемый ответ GitHub: успешная аутентификация без shell access
- затем выполнять обычный `git push origin development`
- если `github.com:22` недоступен, повторить `scripts/setup_github_push.ps1 -InstallKnownHosts -ConfigureRepo -UseHttpsPort443 -TestGitHub`; это переключает repo-local SSH на `ssh.github.com:443`
- если ошибка звучит как `Host key verification failed`, сначала обновить `known_hosts` через `scripts/setup_github_push.ps1`; не полагаться только на `ssh-keyscan` из старого Windows OpenSSH
- если ошибка звучит как `Permission denied (publickey)`, проблема уже не в host key, а в Deploy key: проверить, что GitHub получил именно public key из `~/.ssh/netstitch_github_deploy.pub` и что у него есть write access

Текущий UI runtime-path дополнительно включает:
- system tray menu для быстрого старта/остановки сканирования и завершения приложения
- hide-to-tray при сворачивании, если включён toggle `Скрывать свернутое`; состояние toggle сохраняется в SQLite setting `ui.hide_when_minimized`
- folder-prompt для module-owned локальной папки, если tray-flow требует экспортный target, а путь ещё не настроен
- embedded watcher runtime стартует и останавливается внутри lifecycle `NetStitch.exe`; UI не ищет и не запускает отдельный watcher helper рядом с приложением
- при закрытии окна, tray `Выход` или fallback-завершении окна UI штатно останавливает embedded runtime и освобождает project-owned files; отдельного child process handle для watcher нет
- `scripts/package_portable.ps1` перед сборкой и обновлением dist проверяет, не запущен ли `NetStitch.exe` из target/portable. WinDivert runtime-файлы копируются только если отличаются по SHA256, поэтому загруженный драйвер `WinDivert64.sys` не блокирует упаковку, когда в portable уже лежит та же версия файла.

## Embedded watcher runtime

Portable runtime поставляет native-only контур. Мониторинг, локальный API, browser shell и runtime state живут внутри процесса `NetStitch.exe`; network tool подключается как native library из `libs\`.

Базовый пользовательский цикл теперь выполняется через desktop/browser UI:
- добавить или включить `Tracked apps`
- запустить или остановить monitoring через header/tray action
- дождаться наблюдений в `Monitoring`
- подтвердить нужные строки, выполнить CSV export/import или экспорт через интеграционный модуль
- закрыть окно или tray `Выход`, чтобы основной процесс штатно остановил embedded runtime

Диагностика без UI пока не имеет нового canonical portable entrypoint. Если headless/automation workflow понадобится снова, его нужно отдельно зафиксировать в requirements/runbook/tests как app-owned entrypoint в native-only runtime boundary.

Ограничения embedded runtime:
- UDP/QUIC endpoints всё равно требуют Windows, админ-права и доступный WinDivert runtime
- network enrichment/probe выполняется через linked native library и не должен блокировать UI
- посторонние watcher-процессы из старых локальных сборок не являются частью текущего portable lifecycle

После появления полного production flow здесь должны быть дополнительно описаны:
- обязательные переменные окружения
- локальные зависимости
- безопасный redeploy path
- signing entrypoint и verification path для подписанных release-артефактов

## Signing gate перед релизом

Текущий default signing mode: self-signed `test_certificate` только для разработки.

Перед любой командой пользователя `релиз`:
- открыть `docs/SIGNING_RU.md` и signing config
- проверить, не остался ли `method = "test_certificate"`
- если production-сертификат не настроен, остановиться до push в `release` и до публикации release assets
- предложить варианты: Azure Trusted Signing, certificate store/hardware token, PFX из secret storage или явное подтверждение test-signed релиза
- без этого подтверждения не создавать GitHub Release и не публиковать release assets

Локальная test-подпись:
- заполнить ignored `config/signing.local.toml`
- выполнить `scripts/compile.ps1`
- выполнить `scripts/sign_windows.ps1`
- если нужно, чтобы текущая Windows-машина доверяла test-подписи, отдельно выполнить `scripts/sign_windows.ps1 -EnsureTrustedForCurrentUser`
- помнить, что доверие к self-signed сертификату локально и не решает production trust для чужих машин

## Диагностика

Пока доступны только базовые проверки структуры репозитория и tracked-файлов. По мере появления runtime следует документировать:
- где смотреть логи
- как собирать диагностические артефакты
- как воспроизводить типовые сбои

Минимальный smoke-check для текущего UI:
- запустить UI
- убедиться, что окно появляется сразу, а таблицы/статусы могут догружаться динамически после подключения к watcher
- убедиться, что UI показывает live-статус watcher и не наполняется синтетическими mock-IP без включения `NETSTITCH__UI_USE_MOCK`
- убедиться, что discovered connector apps в `Tracked apps` появляются disabled и включаются только вручную
- убедиться, что manual apps находятся выше connector apps, connector apps отсортированы по алфавиту, а bulk-кнопка включает/выключает весь список без отладочной кнопки переключения первой строки
- для диагностики известных приложений сначала запустить `cargo run -p netstitch-connectors --example discover` и проверить, что manifest содержит Windows/macOS/Linux aliases/sources, а ожидаемые приложения текущей ОС находятся до bootstrap UI/watcher
- для диагностики runtime-коннекторов запустить discover с `NETSTITCH__CONNECTORS_DIR=resources\connectors\apps` и проверить, что `.app` manifests дают ожидаемый список коннекторов
- убедиться, что окно можно свернуть и скрыть в tray
- запустить `NetStitch.exe` повторно под той же Windows-учётной записью и убедиться, что второй процесс не создаёт новый runtime, а тихо восстанавливает/показывает первое окно, включая случай свёрнутого окна
- проверить tray menu `Скрывать свернутое`, `Начать/Остановить сканирование`, `Выход`
- после закрытия окна или tray `Выход` проверить, что `NetStitch.exe` из target/portable завершён и не держит project-owned binaries/native libraries
- при повторной сборке/packaging проверить, что активный `NetStitch.exe` из `target\release` или `dist\NetStitch-win64-portable` явно блокируется или закрыт пользователем; watcher helper artifacts не должны участвовать в preflight
- при пустом module-owned path проверить, что tray flow сначала запрашивает папку и валидирует корректный root через integration layer
- при проверке UDP/QUIC запустить watcher от администратора и убедиться, что в корне portable-папки доступен WinDivert runtime (`WinDivert.dll` + driver/SYS из одного релиза)
- если ожидаемый UDP endpoint из лога игры не появился в `Monitoring`, проверить `runtime_status.flow_capture`: backend started/error, есть ли UDP events до фильтрации, matched/dropped counters, local socket и PID matching. Сравнивать `netstat`/`Get-NetTCPConnection` можно только для TCP; стандартная Windows UDP table не показывает remote endpoint для socket `0.0.0.0:64090`.
- для endpoint-таблицы проверить фильтр `Failed attempts` и убедиться, что строки со state `attempting/failed` попадают в выборку и остаются доступными для ручного подтверждения

## Восстановление

До появления runtime восстановление сводится к восстановлению репозитория из git и повторному локальному bootstrap по актуальным документам.

## Типовые инциденты

- Отсутствует обязательная документация для новой области работы.
- Новый reusable script добавлен без обновления discovery layer.
- В индекс попали временные или секретные файлы.
