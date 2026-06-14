# AGENTS.md / стартовые правила нового не-web проекта

Проект: `NetStitch`
Корень проекта: `C:\Programming\GitHub\NetStitch`
Основная ветка разработки: `development`
Релизная ветка: `release`
Ветка `main` в текущем workflow не используется: не создавать, не пушить и не продвигать её без отдельной прямой команды пользователя.
Namespace env/secrets: `NETSTITCH__...` для project-local значений, `PROVIDER__...` для внешних провайдеров, `LOCAL__...` только для локальной машины.

## 1. Что читать в начале каждой новой сессии

Перед любыми действиями обязательно открыть и учесть:
- `README.md`
- `AGENTS.md`
- `docs/REQUIREMENTS_RU.md`
- `docs/RUNBOOK_RU.md`
- `docs/TESTING_ENV_RU.md`
- `docs/SECURITY_RU.md`
- `docs/ARCHITECTURE_RU.md`
- `tools/README.md`
- `tests/README.md`, если уже существует

После чтения документов всегда уточнить область работы одним плоским списком:
0. Глобальные инфраструктурные решения
1. Core application/runtime
2. CLI/TUI/desktop interface
3. Data/storage/migrations
4. Integrations/API clients
5. Tooling/CI/release
6. Documentation/tests/security

Пользователь может ответить номером. Если выбрано `0`, явно проговорить, что работа идёт по shared/global project contour, и перечислить, какие сервисы, модули или operational-контуры входят в задачу. Если выбрана конкретная область, сначала открыть `README.md` в соответствующем модуле, если он есть. Если задача затрагивает несколько областей, прямо перечислить их до начала работы.

## 2. Базовая структура пустого проекта

Минимальный каркас:
- `README.md` — назначение проекта, быстрый старт, ссылки на docs/tooling/tests.
- `AGENTS.md` — эти правила.
- `docs/REQUIREMENTS_RU.md` — функциональные и operational требования.
- `docs/RUNBOOK_RU.md` — запуск, диагностика, восстановление, типовые инциденты.
- `docs/TESTING_ENV_RU.md` — локальная среда, тесты, логи, ограничения.
- `docs/SECURITY_RU.md` — секреты, threat model, audit/logging, текущие гарантии и риски.
- `docs/ARCHITECTURE_RU.md` — модули, границы ответственности, storage/runtime flow.
- `docs/ROADMAP_RU.md` — отложенные задачи.
- `src/` — исходный код основного приложения.
- `config/` — tracked примеры и schema/config loaders без секретов.
- `storage/` — runtime state, logs, generated data; секреты не коммитить.
- `resources/` — локальные assets/templates/fixtures, если нужны.
- `scripts/` — canonical project entrypoints.
- `tools/` — reusable diagnostics/helpers, с `tools/README.md`.
- `tests/` — tracked regression tests, runner и `tests/README.md`.
- `temp/` или `upload_temp/` — только ignored throwaway артефакты, HAR/log dumps, локальные secret configs; не canonical tooling.
- `.env.example` — безопасные примеры переменных без реальных секретов.
- `.gitignore` — исключить `.env*`, `.local/`, runtime logs, temp dumps, build cache, secrets.

## 3. Tooling discovery layer

Reusable scripts/tests/tools должны быть в git под `scripts/` или `tools/`, не в `temp/`.

Обязательно поддерживать короткий discovery layer:
- repo-wide first lookup: `tools/README.md`
- для набора tooling конкретного модуля: `tools/<module>/README.md`
- формат записей: `что это / когда брать / важные входы или артефакты`
- при добавлении нового reusable entrypoint обновлять соответствующий README в той же работе
- если смысл скрипта не очевиден из имени, добавить короткий header-комментарий в скрипте

## 4. Границы безопасности и окружения

Работать только внутри `C:\Programming\GitHub\NetStitch` и только с ресурсами этого проекта.
Не менять чужие каталоги, чужие compose/VM/service stacks, глобальные системные конфиги, `/etc/*`, systemd, глобальный git/ssh без отдельного согласования.
Не выполнять destructive-команды уровня хоста без явного разрешения.
Project-local bootstrap, ключи и secrets держать внутри ignored `.local/`, `.env.codex.local`, `.env.github.local` или другого local secret contour.
Пароли, private keys, tokens, OAuth/client secrets и production credentials никогда не коммитить.

## 5. Применение изменений

После правок запускать минимально достаточный, но реальный verification path:
- форматирование/линт для затронутого языка
- unit/regression tests
- integration/smoke tests, если затронут runtime, storage, CLI flow, API client или release logic
- build/package команду, если проект собирается в артефакт

Если есть project-owned `scripts/redeploy_local_safe.sh` или аналогичный entrypoint, использовать его для полного применения runtime-изменений.
Fast path допустим только для строго изолированного слоя и должен быть явно описан в runbook.
Если инструмент поддерживает параллельную сборку/тесты, использовать все доступные CPU-ядра по умолчанию.
Если команда важна для задачи и упала только из-за sandbox/network/permission policy, повторить тем же project-owned path с повышенным разрешением, не изобретая обходной путь.

## 6. Тесты

Каждый полезный regression test должен быть tracked:
- лежит в `tests/`
- подключён к общему runner в той же работе
- описан в `tests/README.md` в формате `file / when to run / covers`

Одноразовые diagnostics не оставлять в `tests/` без включения в discovery layer.
Перед commit/push запускать тесты, соответствующие затронутой области.
Если тесты не удалось запустить, явно сообщить причину и остаточный риск.

## 7. Локализация и user-facing строки

Если проект поддерживает i18n/translation layer, перед любым commit/push проверить все затронутые user-facing строки:
labels, prompts, errors, help text, aria/tooltips, notifications, statuses, CLI messages.
Новые или изменённые строки должны быть добавлены во все поддерживаемые локали до commit/push.
Профессиональные термины и бренды можно оставлять на английском, если это устоявшаяся форма, но окружающий текст всё равно локализуется.

## 8. Security, logs, audit

Секреты не логировать и не класть в audit payload.
Ошибки runtime должны иметь correlation/request id, если это применимо.
Audit/event log строить на системных полях, а не локализованном тексте:
- `event_id`
- `created_at`
- `entity_type`
- `action_type`
- `entity_id`
- `actor_id`, если есть пользователь/процесс
- `payload` без секретов

Action names держать системными и стабильными. Причины, детали и old/new значения класть в `payload`.
Core runtime не должен зависеть от внешней сети для базовой работы, если эту зависимость можно убрать локальными assets/data/cache.

## 9. Git workflow

Рабочая ветка по умолчанию: `development`.
Все обычные commit/push делать в `development`.
В `main` не пушить и не создавать её в обычном процессе.
Не откатывать чужие изменения в рабочем дереве. Если дерево dirty, понять, какие изменения относятся к задаче, и работать вокруг остальных.
`anchor commit`, `checkpoint commit` и `stable commit` считать одним intent: commit subject начинается с `Anchor: `, если пользователь не попросил другой формат.

Перед commit/push:
- проверить выбранную область
- запустить релевантные тесты
- проверить i18n, если есть
- убедиться, что новые tooling entrypoints описаны в `tools/README.md` или module catalog
- проверить, что в git не попали secrets/temp/build мусор
- писать commit message с описанием изменений в body: не только subject, но и 2-6 коротких строк, объясняющих что изменилось и зачем; исключение только для явно пользовательского требования сделать однострочный commit

Обычный push в `development`:
1. Работать на ветке `development`.
2. Выполнить минимально достаточную проверку для затронутой области; если менялись release/build/docs/security контуры, включить соответствующие regression tests и `scripts/check_secrets.ps1`.
3. Сделать commit с subject и body-описанием изменений.
4. Push выполнять только в `origin/development`, если пользователь не указал другой разрешённый контур.

После каждого выполненного push вернуть пользователю ровно такой блок:

```text
Коммит: <short_sha>
Tag: <tag_name | пусто>
Дерево чистое.
Ветка: <branch>
Push: <remote>/<branch>
Live-релиз: <release_id | не выполнялся>
Дата/Время: ДД.ММ.ГГГГ ЧЧ:ММ:СС

Память всего: <ram+swap total> / <ram used + swap used>
Физическая память всего: <ram total> / <ram used>
Swap всего: <swap total> / <swap used>
```

Строку `Tag:` указывать всегда сразу после `Коммит:`. Если в этом push создавался, переносился или явно указывался git tag, вписать его имя; если tag не использовался, оставить строку как `Tag: `. Строку `Live-релиз` указывать всегда. Если дерево не чистое, строку после `Tag:` заменить фактическим кратким статусом. Строки памяти брать из project-owned `./scripts/wsl_memory_status.sh` или documented equivalent.

## 10. Release workflow

Release request всегда относится к текущей выбранной области/модулю, а не ко всему репозиторию по умолчанию.
Порядок:
1. Убедиться, что область выбрана явно.
2. Проверить signing status: если production-подпись не настроена и используется `test_certificate`, явно указать в итоговом отчёте, что артефакты test-signed; это не блокирует штатный релиз.
3. Выполнить `scripts\update_release_version.ps1`, чтобы `config/release-version.json` получил full version из локально протестированной portable-папки.
4. Взять base-version из `[workspace.package].version` в `Cargo.toml`, а full-version для релиза из `config/release-version.json`. GitHub tags используются только как fallback, если release-version file отсутствует.
5. В `development` создать release-marker commit с subject `Release: <project-or-module> v<full-version>`.
6. Запушить `origin/development`.
7. Fast-forward продвинуть ветку `release` до этого же commit и запушить `origin/release`; этот push является обязательной частью любой команды на релиз/публикацию релиза и сразу запускает GitHub Actions workflow `Portable Release`.
8. `Portable Release` на push в `release` обязан после успешной сборки создать tag `v<full-version>`, GitHub Release и attached artifacts. Manual `workflow_dispatch` допустим только как явный повтор/аварийный запуск, но не заменяет release-branch push в штатном процессе.
9. Вернуться на `development`, проверить чистое дерево и наличие release-marker commit.

Source of truth для релизной версии:
- base-version: `Cargo.toml` -> `[workspace.package].version`;
- full-version/tag для GitHub Release: `config/release-version.json`, обновлённый из локально протестированного `dist/NetStitch-win64-portable/config/version-manifest.json`;
- git tags на GitHub используются только как fallback/защита от конфликтов;
- generated `dist/**/config/version-manifest.json` описывает уже собранную локальную portable-папку и не является источником версии для GitHub release.

Versioning policy:
- формат версии: `major.minor.patch.revision`;
- `major` повышается только для принципиально новой архитектуры программы;
- `minor` повышается для полноценной новой функции;
- `patch` повышается для изменения API, внутренних механизмов или небольшой доработки существующего функционала;
- `revision` является общим последним разрядом и повышается на `+1` для каждой новой сборки или выпуска независимо от остальных разрядов.

## 11. Documentation policy

Любое изменение процесса запуска, тестирования, release, env/secrets, storage, CI или восстановления отражать минимум в:
- `README.md` короткой ссылкой
- `docs/REQUIREMENTS_RU.md`
- `docs/RUNBOOK_RU.md`
- `docs/TESTING_ENV_RU.md`, если меняется тестовый/локальный контур
- `docs/SECURITY_RU.md`, если меняется auth/secrets/logging/audit/threat model

Definition of Done:
- изменения реализованы
- релевантные docs обновлены
- тесты/линт/build пройдены или явно указан блокер
- secrets/temp artifacts не попали в git
- discovery layer обновлён для новых scripts/tools
- пользователь получил короткий итог с проверками и следующими практичными шагами
