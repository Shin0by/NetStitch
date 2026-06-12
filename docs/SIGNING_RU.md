# Подписывание файлов NetStitch

## Назначение

Этот документ фиксирует данные, которые нужны для будущего подписывания Windows-артефактов NetStitch, и границу безопасности для секретов.

Tracked-шаблон находится в `config/signing.example.toml`. Реальный файл с локальными значениями должен быть ignored, например `config/signing.local.toml` или `.local/signing/signing.toml`.

## Что нужно от пользователя

Для production-подписи нужны следующие данные:
- Юридическое имя издателя ровно как в code-signing сертификате.
- Публичный сайт или страница поддержки проекта, если есть.
- Email для поддержки или security contact, если его можно показывать в release-документации.
- Тип подписи: `certificate_store`, `pfx_file`, `hardware_token` или временно `test_certificate`.
- SHA-1 thumbprint сертификата, если сертификат установлен в Windows Certificate Store или доступен через hardware token.
- Store scope сертификата: обычно `CurrentUser` или `LocalMachine`.
- Путь к локальному `.pfx/.p12`, только если используется PFX-файл; сам файл должен лежать в `.local/signing/` или другом ignored-контуре.
- URL timestamp-сервера провайдера сертификата, желательно RFC3161.
- Подтверждение, какие артефакты подписывать: минимум `NetStitch.exe`, native runtime libraries under `libs\`, а также project-owned integration `.dll` under `integrations\`, если они входят в Windows portable-пакет.

## Текущее решение

Пока production-сертификата нет, используем только self-signed test certificate для локальной разработки и проверки signing pipeline.

Это означает:
- test certificate допустим для `development` и локальной portable-проверки
- test certificate не считается production trust
- Windows SmartScreen и чужие машины могут не доверять такой подписи
- перед продвижением ветки `release` и созданием релиза нужно остановиться и решить вопрос с production-сертификатом

Release gate:
- если пользователь просит `релиз`, сначала проверить signing method
- если method остаётся `test_certificate`, явно сообщить, что production-сертификат не настроен
- не продвигать ветку `release` и не создавать release record без отдельного подтверждения пользователя на test-signed релиз или без настройки production signing
- предпочтительные production варианты: Azure Trusted Signing, сертификат в Windows Certificate Store, hardware token или PFX из защищённого secret storage

## Что нельзя передавать в tracked-файлах

Эти данные нельзя коммитить и нельзя записывать в `config/signing.example.toml`:
- private key
- PFX/P12 password
- hardware token PIN
- provider API token
- production credentials
- закрытые сертификаты или key-файлы

Для секретов использовать только local contour:
- `config/signing.local.toml`
- `.local/signing/`
- переменные окружения вида `NETSTITCH__SIGNING_...`

## Предпочтительный Windows-вариант

Предпочтительный production-вариант для Windows:
- сертификат установлен в Windows Certificate Store или доступен через hardware token
- в config указан только thumbprint
- timestamp выполняется через provider RFC3161 URL
- пароль/PIN вводится локально или берётся из защищённого локального хранилища, но не из git

PFX-файл допустим для локальной разработки или CI только если он хранится в защищённом secret storage. В рабочем дереве проекта PFX должен лежать только в ignored `.local/signing/`.

## Самоподписанный сертификат

Самоподписанный сертификат нужен только для разработки:
- проверить, что signing command подписывает `NetStitch.exe`, native runtime libraries и project-owned integration `.dll`
- проверить, что verification step видит подпись и publisher
- проверить, что portable packaging берёт уже подписанные project-owned binaries

Самоподписанный сертификат не должен маскироваться под production-сертификат. В config он должен быть явно обозначен как `method = "test_certificate"` и `allow_test_certificate_for_development_only = true`.

Локальный signing entrypoint:
- `scripts/sign_windows.ps1` - прочитать `config/signing.local.toml`, создать или переиспользовать self-signed code-signing certificate и подписать `NetStitch.exe` плюс project-owned native libraries
- `scripts/sign_windows.ps1 -SkipMissing` - подписать только существующие артефакты
- `scripts/sign_windows.ps1 -EnsureTrustedForCurrentUser` - дополнительно импортировать публичную часть test-сертификата в `CurrentUser\Root` и `CurrentUser\TrustedPublisher`, чтобы локальная машина доверяла test-подписи

Флаг `-EnsureTrustedForCurrentUser` меняет пользовательские certificate stores Windows, поэтому его использовать только после явного решения пользователя. По умолчанию скрипт создаёт/использует сертификат в `CurrentUser\My`, но не добавляет его в доверенные корни.

Первичное создание сертификата пишет в `CurrentUser\My` и может требовать запуска вне sandbox/с обычным доступом к Windows crypto store. Если сертификат не добавлен в доверенные хранилища, `Get-AuthenticodeSignature` может показывать `UnknownError`: это означает, что файл подписан, но self-signed root не доверен текущей системе.

## Какие файлы подписывать

Подписываем project-owned binaries:
- `NetStitch.exe`
- native runtime libraries under `libs\`
- project-owned integration `.dll` under `integrations\`, если они поставляются в Windows portable-пакете

Для этих же project-owned binaries должен быть заполнен Windows Version Resource:
- `CompanyName = NetStitch`
- `ProductName = NetStitch`
- `FileDescription = NetStitch Desktop` для UI
- `FileDescription = NetStitch Watcher` для watcher library
- `FileDescription = NetStitch Network Tool` для tool library
- `OriginalFilename` соответствует имени файла
- `LegalCopyright = Copyright (c) 2026 NetStitch`

Не переподписываем third-party runtime:
- `WinDivert.dll`
- `WinDivert64.sys`

WinDivert берётся из локального trusted WinDivert контура и сохраняет vendor-origin. Если потребуется отдельная проверка vendor signature, это должно быть отдельным verification step, а не переподписью NetStitch-сертификатом.

## Минимальная проверка после подписи

После появления signing entrypoint нужно проверять:
- подпись присутствует на каждом project-owned `.exe` и `.dll`
- publisher совпадает с ожидаемым именем из `config/signing.local.toml`
- timestamp присутствует и валиден
- unsigned third-party binaries не были случайно переподписаны
- portable-пакет содержит уже подписанные `NetStitch.exe` и project-owned native libraries
