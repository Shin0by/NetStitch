<p align="center">
  <img src="resources/branding/images/NetStitch.png" alt="NetStitch logo" width="128">
</p>

<h1 align="center">NetStitch</h1>

<p align="center">
  <img alt="Latest release" src="https://badgen.net/github/release/Shin0by/NetStitch">
  <img alt="Release downloads" src="https://badgen.net/github/assets-dl/Shin0by/NetStitch">
  <img alt="Page views" src="https://hits.sh/github.com/Shin0by/NetStitch.svg?label=views">
</p>

## English

NetStitch shows which network addresses and domains are contacted by selected applications. It is designed to make actual application network destinations visible, separate useful rows from noise, and prepare a clear data set for further work.

### Features

- Observation of network activity for selected applications.
- IP, domain, port, protocol, and connection state display.
- Filtering and confirmation of relevant rows.
- Data import and export for moving work between environments.
- Local storage of working data.
- Cross-platform portable flow for Windows and Linux.
- Portable launch without mandatory system-wide installation.
- Web interface for controlling NetStitch from another machine on the local network.
- Module support for specialized export, analysis, and guided actions.
- Localized interface with editable language files.
- Update hint from GitHub Releases without automatic download or install.

### Feature Guides

- [Portable mode](docs/FEATURE_PORTABLE_EN.md)
- [Web control](docs/FEATURE_WEB_CONTROL_EN.md)
- [App connectors](docs/FEATURE_APP_CONNECTORS_EN.md)
- [Monitoring and filtering](docs/FEATURE_MONITORING_EN.md)
- [Data import and export](docs/FEATURE_DATA_EXCHANGE_EN.md)
- [Modules](docs/FEATURE_MODULES_EN.md)
- [Module SDK](docs/module-sdk/README_RU.md)
- [Localization](docs/FEATURE_LOCALIZATION_EN.md)

### Install And Run

Windows:

1. Unpack the portable archive.
2. Run `NetStitch.exe`.
3. Select applications and start observing.

Linux:

1. For installer testing, install the Linux `.deb` package; it pulls runtime dependencies and adds the desktop launcher.
2. For other Linux distributions, unpack the Linux portable archive and run `./install-desktop-launcher.sh --copy-to ~/.local/opt/netstitch`; install any runtime libraries the script reports as missing.
3. You can also run `NetStitch` directly from the unpacked portable folder.

### Release Builds

Normal development work is pushed to `development`. Push to the `release` branch starts the `Portable Release` workflow, which builds Windows/Linux portable archives and publishes the GitHub Release with attached assets. The `main` branch is not used by the current project workflow. Manual workflow dispatch is reserved for explicit repeat/emergency runs. The workflow derives the base version from `Cargo.toml`, takes the release full version from tracked `config/release-version.json`, and builds folder-first archives without local runtime databases.

When the project version or build revision changes, local verification must refresh Windows portable, Linux portable, and the Linux `.deb` installer. The Linux installer smoke uses only the `NetStitch-Linux-Test` WSL distribution and installs the generated `.deb` through the package manager before UI testing.

## Русский

NetStitch показывает, с какими сетевыми адресами и доменами работают выбранные приложения. Он помогает увидеть фактические сетевые назначения программ, отделить важные строки от шума и подготовить понятный набор данных для дальнейшей работы.

### Возможности

- Наблюдение за сетевой активностью выбранных приложений.
- Отображение IP, доменов, портов, протоколов и состояния соединений.
- Фильтрация и подтверждение нужных строк.
- Импорт и экспорт данных для переноса между рабочими средами.
- Локальное хранение рабочих данных.
- Кроссплатформенный portable-контур для Windows и Linux.
- Portable-запуск без обязательной установки в систему.
- Web-интерфейс для управления NetStitch с другой машины в локальной сети.
- Поддержка модулей для специализированного экспорта, анализа и рабочих сценариев.
- Локализованный интерфейс с редактируемыми языковыми файлами.
- Подсказка о новой версии из GitHub Releases без автоматической загрузки или установки.

### Подробнее о возможностях

- [Portable-режим](docs/FEATURE_PORTABLE_RU.md)
- [Web-управление](docs/FEATURE_WEB_CONTROL_RU.md)
- [App-коннекторы](docs/FEATURE_APP_CONNECTORS_RU.md)
- [Мониторинг и фильтрация](docs/FEATURE_MONITORING_RU.md)
- [Импорт и экспорт данных](docs/FEATURE_DATA_EXCHANGE_RU.md)
- [Модули](docs/FEATURE_MODULES_RU.md)
- [SDK модулей](docs/module-sdk/README_RU.md)
- [Локализация](docs/FEATURE_LOCALIZATION_RU.md)

### Установка и запуск

Windows:

1. Распакуйте portable-архив.
2. Запустите `NetStitch.exe`.
3. Выберите приложения и начните наблюдение.

Linux:

1. Для проверки установочного контура поставьте Linux `.deb` пакет: он подтягивает runtime-зависимости и добавляет desktop-ярлык.
2. Для других Linux-дистрибутивов распакуйте Linux portable-архив и выполните `./install-desktop-launcher.sh --copy-to ~/.local/opt/netstitch`; если скрипт покажет недостающие библиотеки, установите их через package manager своей системы.
3. Также можно запускать `NetStitch` напрямую из распакованной portable-папки.

### Релизные сборки

Обычная разработка пушится в `development`. Push в ветку `release` запускает workflow `Portable Release`, который собирает Windows/Linux portable-архивы и публикует GitHub Release с прикреплёнными assets. Ветка `main` в текущем workflow проекта не используется. Ручной запуск workflow остаётся только для явного повтора или аварийного запуска. Workflow берёт базовую версию из `Cargo.toml`, полный release version из tracked `config/release-version.json` и собирает архивы с папкой в корне без локальных runtime-БД.

При изменении версии проекта или сборочной ревизии локальная проверка должна обновлять Windows portable, Linux portable и Linux `.deb` installer. Linux install smoke выполняется только в WSL `NetStitch-Linux-Test`: сгенерированный `.deb` ставится через package manager до ручной проверки UI.
