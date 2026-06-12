# План community cloud sync

## Статус

Задача в реализации. Выполнен slice Cloudflare Worker/D1 + desktop overlay: Google OAuth + captcha, upload/download quota, app catalog search, обязательный публичный `Никнейм` для upload, download staging-table, импорт выбранных cloud-строк в Monitoring через CSV-import storage path и CSV export выбранных staging-строк. Password/email register-login больше не является активным MVP-путём.

Preflight внешней инфраструктуры должен быть выполнен до кода:
- Cloudflare account создан в отдельном нейтральном project contour.
- Obsolete Worker `ns-sync` удалён; рабочий внешний Cloudflare Worker называется `netstitch-sync`, чтобы все Cloudflare-сущности проекта были явно маркированы `netstitch-*`.
- D1 database создана с именем `netstitch`.
- D1 binding к Worker должен использовать имя переменной `DB`.

Живые значения Cloudflare (`workers.dev` URL, D1 database id, account id, API tokens) не коммитятся. Для локальной работы использовать ignored файл вроде `.env.cloudflare.local`.

Naming policy: внешние Cloudflare/D1 ресурсы маркируются `netstitch` (`netstitch-sync`, D1 `netstitch`). Внутри D1 таблицы называются коротко (`users`, `observations`, `app_catalog`) без `netstitch_` prefix.

## Цель

Добавить внешний community-cloud контур для NetStitch:
- загрузка наблюдений во внешний сервис методом append/upsert;
- скачивание community-наблюдений по приложению из списка приложений, сформированного на стороне сервиса;
- upload разрешён только для verified applications;
- данные из CSV/cloud-import не считаются локально доверенными по умолчанию;
- домены не считаются authority и должны проходить server-side verification.

Выбранный backend для MVP: Cloudflare Workers + D1.

Cloud sync должен быть отдельным модулем, полностью отвечающим за отправку данных во внешний сервис и получение данных из него. Он не должен быть скрытым расширением CSV import/export, external integration export или UI слоя. Исключение: расширение работы с приложениями/app identity, потому что upload policy зависит от verified application proof.

Существующий код добавления/удаления `Tracked apps` должен быть дополнен, а не заменён новым параллельным контуром. При добавлении приложения core/app layer должен пытаться собрать verification data и сохранить её вместе с tracked app identity. Исключение приложения из `Tracked apps` не должно удалять исторические данные приложения, observations или app-verification evidence: они должны оставаться пригодными для CSV export, cloud upload policy и будущего анализа. Это требует заменить текущую hard-delete семантику remove-app на archive/untrack flow с отдельной явной операцией purge, если она когда-либо понадобится.

Storage invariant: уже полученные monitoring observations не принадлежат lifecycle удаления приложения из активного списка. Если данные уже были получены мониторингом, только пользователь отдельным явным действием решает удалить эти observations. Удаление/исключение приложения из `Tracked apps` может остановить дальнейший мониторинг и скрыть приложение из активного списка, но не должно удалять ранее собранные данные.

## Области Работы

0. Глобальные инфраструктурные решения
1. Core application/runtime
2. CLI/TUI/desktop interface
3. Data/storage/migrations
4. Integrations/API clients
6. Documentation/tests/security

Primary область реализации: `4. Integrations/API clients`.

Если работа идёт по `0`, это shared/global project contour: Cloudflare Worker, D1 schema, локальная app-verification модель, API-auth, import/export trust policy, docs и tests.

## Ключевые Решения

- `app_name`, `exe_name`, `process_name`, path, product version, file version, original filename и `exe_sha256` не являются authority.
- `exe_sha256` можно хранить только как diagnostic/anomaly evidence, потому что hash меняется при обновлении файла.
- Authority для Windows: `app_id + stable app identity key`, где key строится из валидного Authenticode leaf SPKI при наличии и стабильных VersionInfo `CompanyName`/`ProductName`.
- MVP identity для signed Windows apps: стабильный `appsig_v1_*` key, включающий валидную Authenticode цепочку и стабильные VersionInfo поля.
- Ротация сертификатов или появление Authenticode у того же продукта создаёт другой stable app identity key и не смешивается со старым `publisher_key`.
- Upload в cloud разрешён для любых добавленных приложений, если клиент может сформировать stable identity; неподтверждённые/импортированные строки отправляются как `CloudImportUntrusted`.
- Cloud принимает display name от authenticated upload payload только как catalog/display metadata, а не как trust authority.
- CSV/cloud-import строки в локальной БД всегда получают external trust marker.
- Импорт из cloud не равен локальному наблюдению; export-ready поведение должно требовать отдельной политики подтверждения.
- JWT защищает client identity, scope, TTL и целостность payload, но не доказывает, что клиент честно наблюдал endpoint.
- Cloud download/upload должен иметь лимиты запросов, например `3 запроса за 6 часов` отдельно для загрузки из облака и отправки в облако.
- Cloud-side quota state является authority, а локальная SQLite хранит только mirror/cache для отображения лимита, расхода и времени сброса без обязательного roundtrip при каждом render.
- UI должен показывать пользователю текущий лимит и сколько запросов потрачено для download/upload.
- Cloud overlay должен открываться отдельной кнопкой в верхнем toolbar. Кнопка должна использовать тот же compact icon-only стиль, что CSV import/export, располагаться после separator и перед CSV-кнопками.
- При открытии cloud overlay разрешено сразу выполнить стартовый refresh: проверить доступность cloud, получить quota для upload/download и загрузить список приложений из server-side catalog. Это действие считается инициированным пользователем, потому что пользователь явно открыл overlay.
- Cloud overlay состоит из двух основных панелей: `Облако: загрузка данных` и `Облако: выгрузка данных`. Ручной кнопки `Обновить` внутри overlay нет; пока overlay открыт, статус/лимиты/списки обновляются по таймеру `10s` как продолжение явного действия открытия overlay.
- Список приложений должен поддерживать поиск по имени приложения и публичной подписи автора. Публичная подпись для отправки - обязательное поле `Никнейм` в панели upload, не заполняемое из Google. Ник не может быть пустым, допускает только английские буквы, цифры и `!.@#$%^&*()_+=-~`, максимум `32` символа. Worker нормализует ник в case-insensitive форму для уникальности (`lower_ascii`) и привязывает его к конкретной Google OAuth identity через необратимый provider subject HMAC с server-side secret pepper; другой Google-пользователь не может публиковать под уже занятым ником. Разные Google accounts являются разными авторами и могут иметь разные ники. Если владелец меняет ник на свободный валидный ник, уже опубликованные им строки должны отображаться и искаться по новому нику, потому что endpoint rows связаны со стабильным author id, а не с копией текста ника. Публикация видна в поиске именно под текущим ником/источником.
- Cloud download/search фильтры повторяют главный header `Monitoring`: `Приложение`, `IP`, `Порт`, `Протокол`, но вместо `Статус` используется `Автор`. `Автор` - публичный ник/подпись автора публикации. `Протокол` - dropdown/select; остальные фильтры - текстовые поля. Текстовые поля не запускают поиск во время набора: значение применяется клавишей `Enter`, после чего поле делает одну короткую pulse-анимацию и запускает поиск; пустая строка сбрасывает фильтр. Для приложения нужен autocomplete: первые `5` вариантов показываются сразу, полный список доступен через scroll, а выбор должен сохранять `app_id`.
- Download доступен без регистрации: пользователь может получить общий community-срез по выбранному `app_id`.
- Upload доступен только после входа через Google OAuth в cloud overlay. Пользовательский пароль и email-подтверждение письмом в MVP не используются; Worker получает verified identity от Google OAuth/OIDC, создаёт или находит пользователя в D1 и выдаёт session/client key.
- Google OAuth flow должен использовать Authorization Code + PKCE/state. Worker создаёт одноразовый `state`, хранит его в D1 с TTL вместе с безопасной UI-локалью для рендера наших browser-страниц, принимает callback только один раз, валидирует Google token/userinfo server-side, затем связывает `provider = google`, необратимый provider subject HMAC, optional email HMAC/email_verified и локальный client public key. Google email/name/raw subject не сохраняются в D1/cloud как обратимые значения и не становятся публичной подписью; текущий клиент может получить email/name в OAuth session response и сохранить их только локально как Windows DPAPI-защищённые display blobs.
- Captcha используется дополнительно перед началом auth/registration flow. Активный бесплатный вариант MVP: собственный server-issued SVG challenge по локальной модели проекта: алфавит `23456789`, длина `6`, искажённые segment glyphs, decoy/noise layers, `challenge_id + answer`, purpose binding, TTL `300s`, минимальное время решения `5s`, максимум `3` попытки, refresh pool `6` challenge на purpose/scope с round-robin `next_index`, очистка pool после verify/failure и failure bucket с lockout/backoff. Сгенерированная картинка хранится как набор SVG tile data URI `10 x 3`, а browser-страница собирает их JavaScript-ом с микрозадержками, tiny-overlap и subpixel-jitter, чтобы не было одной цельной inline-картинки в исходном HTML. Страница авторизации показывает увеличенную на 20% капчу и ввод ответа в одном компактном столбце под ширину капчи, icon-only кнопку обновления и голубую кнопку отправки капчи рядом с вводом, клик по капче открывает 3x zoom overlay с размытием окружения. Google OAuth URL не присутствует в HTML до прохождения капчи: после POST формы Worker проверяет капчу, переводит D1 `oauth_states.status` в `captcha_ok` и только затем отдаёт server-generated Google OAuth ссылку через `302 Location`. Ответ хранится в D1 только как salted SHA-256 hash. Проверка captcha выполняется именно Worker/D1, без доверия к клиенту.
- На 28.05.2026 запланировать server-side hardening auth: Google OAuth + PKCE/state, captcha перед регистрацией/входом, одинаковая ошибка для отказа/истечения auth flow без раскрытия лишних деталей, ограничение попыток входа/регистрации не более `3 за 1 час` с машинно-читаемым retry-after и user-facing уведомлением.
- После успешного входа UI сохраняет в SQLite только безопасные account/session metadata: `cloud.auth.provider = google`, provider subject hash, локальный session/client state и display-only Google email/name в Windows DPAPI-защищённом storage-контуре. Публичный ник вводится отдельно перед upload, локально запоминается как `cloud.upload.nickname` для следующего запуска и хранится на Worker как author profile после публикации. Пользовательские пароли, Google email/name/raw subject, Google tokens, OAuth client secret и provider credentials в SQLite открытым текстом не пишутся.
- В cloud overlay должна быть кнопка `Выход` вместо switch `Оставаться в системе`. После успешного Google OAuth пользователь не должен повторно проходить вход после перезапуска программы, если локальная DPAPI-защищённая session/client state ещё валидна. Кнопка `Авторизация` блокируется при активной сессии, `Выход` блокируется без активной сессии. При выходе NetStitch немедленно удаляет локальные session/client credentials, безопасные account metadata и obsolete `cloud.auth.login`/`cloud.auth.email`/`cloud.auth.password.dpapi.v1`/`cloud.auth.stay_signed_in`, сбрасывает состояние аккаунта в overlay.
- Авторизованный пользователь может сузить download scope: общий срез по приложению (`app_id`) или только публикации конкретного автора (`app_id + author_user_id`/public author signature). Также overlay должен показывать список приложений, которые публиковал текущий пользователь, и давать поиск по приложению/публичной подписи автора.
- В UI списка cloud-приложений не показывать OS/platform и не показывать `verified`. При одинаковых именах различать строки по publisher/vendor, короткому `app_id`, endpoint/submission counts и датам обновления.
- Cloud observations имеют retention не больше 1 года. Строка endpoint-а (`app_id + ip + port + protocol`) удаляется из D1, если её `last_seen_ms`/`last_observed_at` старше retention cutoff. Удаление старых строк является штатной maintenance-политикой сервиса, не пользовательским destructive purge.
- Client identifier должен быть privacy-preserving hash identifier: NetStitch локально собирает утверждённые machine/OS signals, например hardware/Windows-version contour, нормализует их и отправляет только hash/HMAC identifier, без сырых серийных номеров, имени пользователя, hostname или секретов.
- Окно `Информация` должно показывать этот identifier в footer верхней панели `О программе`; пока cloud identity не создана, отображается состояние `Идентификатор: не создан`.
- NetStitch не должен выполнять никаких запросов к cloud без явного действия пользователя. Запрещены cloud-запросы на startup/bootstrap, snapshot refresh, render path, открытие окна `Информация`, фоновый polling лимитов, автоматический upload/download и silent retry. Любой cloud request должен начинаться с явного UI/CLI действия вроде `Проверить лимит`, `Загрузить из облака` или `Отправить в облако`.
- Открытие cloud overlay считается явным user action и может запускать один стартовый refresh статуса, лимитов и списка приложений. Пока overlay открыт, разрешён bounded refresh статуса/лимитов/каталога по таймеру `10s`; ручной кнопки `Обновить` внутри overlay нет. Отдельные explicit действия `Отправить` и `Загрузить` выполняют upload/download. Повторный render/update overlay без нового открытия, таймера overlay или кнопочного действия не должен порождать network request.

## Cloud API MVP

Минимальные endpoints:
- `GET /v1/health`
- `GET /v1/apps`
- `GET /v1/users/me/apps`
- `POST /v1/auth/google/start`
- `GET /v1/auth/google/callback`
- `POST /v1/auth/session/poll`
- `GET /v1/observations?client_identifier=<id>&app_id=<app_id>`
- `GET /v1/observations?client_identifier=<id>&app_id=<app_id>&author_user_id=<user_id>`
- `POST /v1/observations/append`
- `GET /v1/client/quota`

Auth start request body:
- `client_identifier`;
- `client_public_key_jwk`;
- `captcha_token` или `captcha_challenge_id + captcha_answer`;
- `redirect_mode = browser_callback | device_poll`;
- optional `return_to`.

Google OAuth storage:
- хранить в D1/cloud только `provider = google`, необратимый stable provider subject hash, email hash если Google вернул email, `email_verified` как metadata и локальный user/client mapping;
- не хранить Google email, display name/avatar/raw subject как обратимые значения в D1/cloud; desktop может хранить Google email/name только локально как Windows DPAPI-защищённые display blobs;
- не хранить Google access token дольше exchange/verification step, если для текущей функции он больше не нужен;
- OAuth `state`, PKCE verifier/challenge, captcha result и device polling code одноразовые, с TTL `5-10 минут`, used-at marker и cleanup.

Upload body должен содержать:
- `app_id`;
- `platform`;
- verified publisher proof;
- observations batch;
- optional diagnostic metadata;
- domain raw values only as secondary evidence.

Сервер должен:
- проверять JWT;
- выдавать одинаковый auth error для отклонённого/истёкшего OAuth flow или failed captcha/rate-limit без раскрытия лишних деталей;
- ограничивать auth attempts по client identifier + IP bucket + Google subject, если он уже известен, до `3 за 1 час`;
- требовать/проверять server-issued captcha challenge для auth/onboarding policy и не принимать client-generated challenge;
- требовать авторизованный client/user для upload;
- для upload требовать одновременно `Authorization: Bearer <session_token>` и `x-netstitch-jwt` с ES256 подписью активным client key;
- разрешать download без авторизации, кроме scope `author_user_id`, где author id всё равно берётся из server-side catalog/session policy, а не из произвольного display name;
- проверять `body_sha256`;
- проверять replay через `jti`;
- проверять quota/rate-limit по `client_id`, operation kind и rolling window;
- принимать `app_id + publisher proof` как identity metadata, upsert-ить `app_catalog` из upload payload и отклонять только явно `blocked` app ids;
- отклонять private/local/bogon IP;
- нормализовать endpoint;
- выполнять upsert по `app_id + ip + port + protocol`;
- писать submission/audit row;
- проверять домены отдельно, не доверяя `domain_raw`.
- не отдавать и периодически удалять observations старше retention cutoff.

Отправленные клиентом observations должны добавляться к уже имеющимся данным облака через append/upsert. Cloud storage не очищает существующие данные по запросу клиента и не принимает client-side replace semantics.

Retention MVP: `365 days` от `last_seen_ms` строки. При append/upsert новый `last_seen_ms` продлевает срок жизни только если observation снова реально прислана verified upload-ом. Cleanup можно выполнять лениво в Worker перед/после write/read запросов с небольшим `LIMIT`, а позже вынести в Cron Trigger, чтобы не делать большой delete в одном request.

## JWT Контракт

Рекомендуемый MVP алгоритм: `ES256`.

Обязательные claims:
- `iss = netstitch-client:<client_id>`
- `sub = <client_id>`
- `aud = netstitch-cloud`
- `scope`
- `iat`
- `nbf`
- `exp`
- `jti`
- `body_sha256`
- `client_version`

Header:
- `alg`
- `kid`
- `typ = JWT`

Transport:
- session token передаётся в `Authorization: Bearer <session_token>`;
- signed request JWT передаётся отдельно в `x-netstitch-jwt`, чтобы session token и асимметричная подпись не смешивались;
- upload JWT TTL не больше 15 минут;
- Worker хранит `jti` в replay cache до `exp`.
- Rust helper `netstitch-cloud` генерирует ES256 P-256 key material, экспортирует public JWK для регистрации client key и подписывает request JWT с `body_sha256`.

Минимальные scopes:
- `observations:append`
- `observations:read`
- `apps:read`
- `apps:read:self`
- `quota:read`
- `client:auth:start`
- `client:auth:complete`

## D1 Schema Draft

Таблицы:
- `app_catalog`
- `users`
- `oauth_states`
- `captcha_challenges`
- `clients`
- `client_keys`
- `user_clients`
- `client_quota_windows`
- `jwt_replay_cache`
- `observations`
- `observation_submissions`
- `domain_verifications`
- `audit_events`
- `auth_challenges`
- `auth_attempt_windows`

Ключ observation:
- `UNIQUE(app_id, ip, port, protocol)`
- индекс для cleanup: `(last_seen_ms)` или `(expires_at_ms)`;
- optional stored `expires_at_ms = last_seen_ms + 365 days`.

Promotion в `community_verified` не должен происходить от одного клиента. Нужна отдельная quorum/reputation policy.

Quota draft:
- `client_id`
- `operation_kind = upload | download`
- `window_started_at`
- `window_ends_at`
- `limit_count`
- `used_count`

Current MVP policy: `limit_count = 0` means unlimited upload/download operations; `window = 6 hours` remains in the quota mirror for future bounded modes.

Observation retention:
- `retention_ms = 365 days`;
- delete predicate: `last_seen_ms < now_ms - retention_ms` или `expires_at_ms <= now_ms`;
- cleanup должен удалять только `observations`/derived domain rows, не удаляя `app_catalog`, users, client keys или audit rows.

## Локальная SQLite Model

Нужны признаки:
- `source_kind = local_monitoring | csv_import | cloud_import`
- `trust_level = local_observed_verified_app | cloud_import_untrusted | csv_import_untrusted | community_verified | blocked_or_suspect`
- `app_identity_status = verified_local | verified_remote | unknown`
- `tracked_visibility = active | untracked | archived`
- `remote_app_id`
- `cloud_observation_id`

Для cloud quota/client identity нужны локальные mirror/cache поля или таблица:
- `cloud_client_identifier`
- `cloud_client_identifier_status`
- `quota_operation_kind`
- `quota_window_ends_at`
- `quota_limit_count`
- `quota_used_count`
- `quota_synced_at_ms`
- `cloud.auth.provider` в `app_settings`;
- `cloud.auth.provider_subject_hash` в `app_settings`;
- `cloud.auth.email.dpapi.v1` и `cloud.auth.display_name.dpapi.v1` в `app_settings`, если provider вернул email/name; значения должны быть только Windows DPAPI-защищёнными blobs и удаляться при `Выход`;
- obsolete `cloud.auth.stay_signed_in` должен удаляться при явном выходе и больше не управляет чтением сохранённой OAuth-сессии;
- локальный session/client secret только в защищённом storage-контуре; Google access token, OAuth client secret и пользовательские пароли в SQLite не сохранять.

CSV import следует отделить от доверенного переноса домена:
- домен из CSV сохраняется как external/user-provided evidence;
- verified/domain-trusted статус должен быть отдельным полем;
- cloud upload не берёт CSV-imported строки как verified source.
- CSV и локальные monitoring observations не являются носителем авторства cloud-публикации: в них нельзя писать публичный ник, Google email, Google subject, user id, client id или другие пользовательские данные.

Cloud download должен добавлять полученные строки в локальный список `Monitoring` тем же накопительным storage-path, что CSV import: существующая таблица не очищается, дубли объединяются, новые endpoints добавляются. Отличие от CSV import фиксируется только в source/trust markers (`cloud_import_untrusted` вместо `csv_import_untrusted`) и optional `cloud_observation_id`.

Перед добавлением в `Monitoring` cloud download показывает staging-таблицу выбранных строк. Требования к таблице:
- колонки endpoint/app/domain/port/protocol/connection/requests/first seen/last seen;
- выбор всех строк, снятие всего выбора и toggle отдельной строки;
- поиск/фильтрация через cloud header-фильтры `Приложение`, `IP`, `Порт`, `Протокол`, `Источник`, где `Источник` является публичным ником/подписью автора в server-side query/catalog;
- действие `Добавить в мониторинг`, использующее тот же core import/upsert path, что CSV import, но только для явно выбранных staging-строк; пустой выбор не означает `все строки`;
- действие `Экспорт CSV`, использующее существующий CSV serializer/export flow для явно выбранных staging-строк без обязательного добавления в `Monitoring`;
- кнопки `Добавить в мониторинг` и `Экспорт CSV` неактивны, пока нет выбранных staging-строк;
- результат импорта возвращает и отображает `requested/imported/skipped`, чтобы выбранный объём не мог тихо обрезаться на размере страницы;
- progress bar облачной загрузки и выгрузки использует шаги по 500 строк, например 1600 строк отображаются как 4 шага.
- при импорте staging-строк в `Monitoring` и при CSV export author signature/search metadata отбрасывается; сохраняются только endpoint/app/provenance/trust поля, допустимые для monitoring model.

## Модульная Граница

Новый cloud sync contour должен иметь собственную границу ответственности, например отдельный crate `src/netstitch-cloud` или изолированный module boundary с тем же уровнем самостоятельности:
- Cloudflare Worker/D1 API DTO and client mapping;
- append/upload batches;
- remote app list fetch;
- remote observations fetch by `app_id`;
- JWT request signing support;
- client identifier derivation/formatting boundary or shared helper with core app identity;
- quota fetch/update mirror;
- response validation and error mapping;
- cloud-specific trust/source mapping;
- no direct UI rendering;
- no external integration export writes;
- no direct filesystem side effects except project-local config/key material через утверждённые storage boundaries.

Соседние модули используют cloud sync только через явный service boundary:
- `netstitch-core` отвечает за SQLite persistence, local observation upsert и trust flags.
- Embedded watcher runtime отдаёт local HTTP routes для desktop/browser UI.
- `netstitch-ui` показывает controls and statuses.
- App verification/app identity может жить рядом с connector/core layer, потому что она нужна не только cloud sync, но cloud upload policy её использует.

Cloud sync module must not spawn background tasks on construction and must not perform network I/O from snapshot/render helpers. Network access is allowed only from explicit command handlers.

Получение данных:
1. cloud sync получает список remote apps из сервиса;
2. UI показывает список remote apps с поиском по имени и пользователь выбирает `app_id`;
3. пользователь выбирает scope: общий community-срез приложения или, после входа, срез публикаций текущего пользователя;
4. cloud sync получает observations по `app_id` или `app_id + author_user_id`;
5. core добавляет их в Monitoring через cumulative import/upsert path;
6. imported rows получают `source_kind = cloud_import` и недоверенный trust marker.

Поиск приложений:
- базовый search работает по server-side `app_catalog` и не требует входа;
- после входа доступен дополнительный список `Мои приложения`, построенный по submissions текущего пользователя;
- совпадающие display names не объединяются: выбор всегда идёт по `app_id`;
- в UI строки приложения показывать publisher/vendor, короткий `app_id`, endpoint/submission counts и last updated/uploaded; OS/platform и `verified` не показывать.

Отправка данных:
1. пользователь входит в cloud account через Google OAuth в overlay;
2. core выбирает локальные observations только для locally verified applications;
3. cloud sync проверяет локальный quota mirror и при необходимости запрашивает cloud quota;
4. cloud sync формирует batch append request;
5. cloud sync подписывает request JWT client key, привязанным к вошедшему user/client;
6. Worker валидирует app proof/client proof/user session/quota и upsert-ит строки в D1;
7. local state сохраняет upload result/batch status and quota mirror, но cloud upload не должен менять локальную таблицу Monitoring как будто это export-confirmation.

## Windows App Verification

Для Windows реализовать локальный слой Authenticode verification:
- `WinVerifyTrust`/Authenticode chain validation;
- extraction of signer certificate chain;
- SPKI SHA-256 для leaf certificate;
- subject/issuer как diagnostic constraints;
- optional diagnostic file metadata без использования как authority.

Локальный NetStitch должен определять `app_id` только из trusted connector catalog, а не из имени exe.

Интеграция с существующим app lifecycle:
- manual add path flow после валидации executable запускает app verification attempt;
- connector bootstrap/discovery после определения `connector_id/app_id` запускает app verification attempt для найденного executable;
- toggle enabled/disabled не пересчитывает verification data и не должен проходить через add/upsert path;
- remove/untrack tracked app исключает приложение из активного списка `Tracked apps`, но не удаляет observations, app identity и app verification evidence;
- сохранённые observations для untracked app остаются доступны для CSV export и, если приложение было verified, для cloud upload policy;
- повторное добавление того же verified app должно переиспользовать сохранённую app identity/evidence, если она ещё актуальна, или пересчитать её без потери historical observations;
- hard purge, если будет введён, должен быть отдельной destructive-командой с явным предупреждением и не должен быть действием обычной кнопки удаления из `Tracked apps`;
- если verification временно не удалась, приложение остаётся в `Tracked apps`, но получает `app_identity_status = unknown/unverified` и не допускается к cloud upload;
- runtime snapshot должен отдавать verification summary, чтобы UI мог показывать, какие apps можно выгружать в cloud.

Пример локальных storage полей или отдельной таблицы:
- `tracked_app_id`
- `app_id`
- `platform`
- `verification_method`
- `verification_status`
- `signature_chain_valid`
- `leaf_spki_sha256`
- `subject`
- `issuer`
- `checked_at_ms`
- `diagnostic_metadata_json`

`exe_sha256`, product/file version и original filename допускаются только внутри `diagnostic_metadata_json`.

## Subtasks

1. Cloudflare preflight: Worker, D1, binding `DB`, ручной `/v1/health` smoke.
2. Создать Worker/D1 проектный contour без live secrets в repo.
3. Спроектировать и применить D1 schema migrations.
4. Реализовать Worker endpoints и basic request validation.
4.1. Реализовать server-side auth captcha/rate-limit: SVG challenge pool/rotation, `3 за 1 час`, одинаковая auth-ошибка для failed OAuth/captcha/rate-limit.
4.2. Реализовать Google OAuth/OIDC auth: `/v1/auth/google/start`, `/v1/auth/google/callback`, PKCE/state, одноразовый device/browser poll flow, D1 `oauth_states`, provider user mapping и session/client key issue.
4.3. Удалить password/email register-login из активного MVP UI/API.
4.4. Заменить `Оставаться в системе` на явный `Выход`: сохранять local session/client state после Google OAuth, блокировать `Авторизация` при активной сессии, блокировать `Выход` без активной сессии, очищать auth metadata/obsolete credentials при выходе.
5. Добавить JWT verification, `jti` replay cache, `body_sha256`, scopes and client key lookup.
6. Добавить app catalog upsert из authenticated upload payload и migration path для удаления старой таблицы разрешённых издателей.
7. Добавить server-side domain verification через DNS-over-HTTPS или отложенную job-политику.
8. Добавить shared DTO в `netstitch-shared`.
9. Добавить локальную SQLite migration для source/trust/app identity markers.
10. Добавить client identifier derivation и local/cloud quota mirror.
11. Добавить Windows Authenticode verification service.
12. Дополнить существующий add/remove tracked-app lifecycle сбором app verification data и заменить ordinary remove на archive/untrack без удаления observations/evidence.
13. Добавить отдельный cloud sync module/crate для отправки и получения данных.
14. Подключить cloud sync к core через явный service boundary, где core владеет local Monitoring upsert.
15. Реализовать cloud account sign-in в overlay через Google OAuth без хранения пользовательского пароля, Google access token или provider secrets в логах/SQLite/коммитах.
16. Привязать upload к authenticated user/client key; download оставить публичным.
17. Реализовать upload only selected Monitoring observations with required public nickname/source signature, Google-bound ownership, nickname change propagation to existing cloud rows, case-insensitive uniqueness and no email/display-name storage.
18. Реализовать cloud apps list/search by app name + public author signature and import by `app_id`, включая общий срез и срез по текущему автору.
19. Реализовать cloud search filters с применением текстовых полей по `Enter`, protocol dropdown, author/nickname text search и app autocomplete `5` visible options + scroll.
20. Реализовать staging-таблицу cloud download с выбором строк, импортом выбранного в `Monitoring` и CSV export выбранного; без fallback `пустой выбор = все строки`.
21. Реализовать D1 retention cleanup для observations старше 365 дней.
22. Добавить UI/CLI controls рядом с CSV import/export с отображением quota limit/used/reset.
23. Добавить i18n строки в `resources/language/en-en.ini` и `resources/language/ru-ru.ini`.
24. Добавить tests: DTO, JWT validation, quota/client-id lifecycle, app verification lifecycle, storage trust flags, upload selection filtering, nickname validation/ownership/case-insensitive uniqueness/change propagation, no user data in CSV/monitoring rows, cloud search Enter-apply/autocomplete, cloud import trust, staging table selection, запрет cloud add/export без явного выбора строк, проверку `requested/imported/skipped`, progress bar шагами по 500 строк, CSV export from cloud staging, retention cleanup, Worker API smoke.
25. Обновить README, runbook, testing, security, architecture and tools discovery docs.

## Открытые Решения

- Оставить MVP на `ES256` или выбрать Ed25519 после проверки совместимости Rust/Workers/JWT tooling.
- Регистрация client key: anonymous self-registration, invite/admin approval или staged moderation.
- Quorum для `community_verified`: 2, 3 или больше независимых active clients.
- Domain verification: synchronous in append route или queued/background maintenance Worker.
- Политика блокировки злоупотреблений для app catalog: только явный `blocked` статус в D1 или дополнительная moderation policy.
- Показывать ли trust/source badges в UI сразу в MVP или оставить только machine-readable fields.
- Какие cloud-import строки можно предлагать внешним интеграциям без локального наблюдения.
- Нужна ли отдельная destructive purge-команда для полного удаления historical observations/evidence, и кто имеет доступ к ней.
- Какие именно machine/OS signals входят в client identifier, чтобы он был достаточно стабильным, но не раскрывал чувствительные hardware данные.
- Должен ли Windows version участвовать в стабильном identifier или храниться только как отдельный diagnostic/version field, чтобы обновление Windows не меняло client identity.

## Локальные Env Placeholders

Пример для ignored `.env.cloudflare.local`:

```env
NETSTITCH__COMMUNITY_API_URL=https://<worker-name>.<workers-subdomain>.workers.dev
PROVIDER__CLOUDFLARE__D1_DATABASE_NAME=<database-name>
PROVIDER__CLOUDFLARE__D1_DATABASE_ID=<database-id>
PROVIDER__CLOUDFLARE__D1_BINDING=DB
```

Cloudflare API token, account token, Google OAuth client secret, mail provider credentials and recovery codes must not be stored in git.
