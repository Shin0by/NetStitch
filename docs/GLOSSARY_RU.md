# Глоссарий Команд NetStitch

## Назначение

Этот глоссарий фиксирует короткие пользовательские команды и их стандартное действие в проекте.

## Команды сборки

- `компиль`
- `компилируй`
- `скомпилируй`
- `билди`
- `деплой`

Для всех этих слов используется один и тот же action:

1. Выполнить `scripts/compile.ps1`.
2. Скрипт внутри выполняет `scripts/dev_check.ps1` (форматирование, `check`, regression tests).
3. Скрипт затем выполняет release-сборку через `scripts/package_portable.ps1`.
4. После каждой такой компиляции обновляется пользовательская portable-папка `dist\NetStitch-win64-portable`.
5. Отдельно подтвердить наличие тестируемых бинарников: `target/release/NetStitch.exe` и `dist\NetStitch-win64-portable\NetStitch.exe`.

## Завершение работы

Если Codex завершает реализацию, исправление или документационную правку в проекте, перед итоговым ответом нужно выполнить canonical compile/package path:

1. Выполнить `scripts/compile.ps1`.
2. Убедиться, что `dist\NetStitch-win64-portable` обновлён актуальными файлами приложения.
3. Если compile/package не удалось выполнить, явно указать блокер и остаточный риск.

## Команды Git

- `пуш`
- `пушни`
- `запуш`

Для всех этих слов используется один и тот же action:

1. Сделать commit по текущей задаче.
2. Запушить commit в GitHub по умолчанию в `origin/development`, если пользователь явно не указал другой разрешённый контур.
3. Убедиться, что в push не попали лишние файлы, которые должны подтягиваться при развертывании (`target`, temp, локальные секреты, кэши и т.д.).
4. Не пушить и не создавать ветку `main` в обычном процессе.

- `релиз`

Для слова `релиз` используется следующий action:

0. До push в ветку `release` проверить signing status: если настроен self-signed `test_certificate`, продолжать штатный release workflow и явно отметить test-signed статус артефактов в итоговом отчёте.
1. Выполнить `scripts\update_release_version.ps1`, чтобы tracked `config/release-version.json` получил full version из локально протестированной portable-папки.
2. Взять expected full version из `config/release-version.json`; GitHub Actions использует этот файл как основной источник версии, а git tags только как fallback/защиту от конфликтов.
3. Подготовить marker commit в `development` с subject `Release: NetStitch v<full-version>`.
4. Запушить marker commit в `origin/development`.
5. Продвинуть ветку `release` до того же commit, чтобы GitHub Actions сразу начал сборку Windows/Linux portable-архивов.
6. Дождаться GitHub Actions `Portable Release`: push в `release` обязан собрать portable artifacts, создать tag, опубликовать GitHub Release и прикрепить архивы.
7. Manual workflow run использовать только для явно подтверждённого повтора или аварийного запуска, а не вместо штатного push в `release`.
8. Переключиться обратно на `development`.
9. Проверить, что `development`, `release` и release tag указывают на ожидаемый commit, если публикация была выполнена.

Ветка `main` не участвует в release action.

## Примечания

- Эта нормализация относится к локальному проектному workflow и не означает live deployment.
- Если команда невозможна из-за sandbox/network/permissions, повторять тот же canonical path с повышенным разрешением.
