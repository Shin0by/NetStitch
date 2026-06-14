# Примеры и готовые архивы модулей

SDK поставляет два примера одного и того же демонстрационного модуля:

- `docs/module-sdk/examples/ui-entity-showcase-rust/` - Rust;
- `docs/module-sdk/examples/ui-entity-showcase-cpp/` - C++.

Они одинаковые по поведению: показывают controls, `grid`, `tabs`, отдельную вкладку `browse_window` для выбора папки и save target без записи файла, `textarea` с `commit_on_enter`, progress bar с `progress_stages`, таблицы, column-level `table_columns`, стандартный dialog, header action, background-подписку и локализацию RU/EN. Rust-пример дополнительно содержит module-owned SVG-иконку через `icon_path`, а C++-пример намеренно оставлен без `icon_path`, чтобы разработчик видел оба варианта. Это сделано намеренно, чтобы разработчик мог сравнить только языковую часть реализации, не разбираясь в разных сценариях.

## Готовые архивы

Готовые installable-архивы лежат в `docs/module-sdk/packages/`:

- `ui-entity-showcase-rust-windows-x86_64.zip`;
- `ui-entity-showcase-rust-linux-x86_64.zip`;
- `ui-entity-showcase-cpp-windows-x86_64.zip`;
- `ui-entity-showcase-cpp-linux-x86_64.zip`.

Каждый архив содержит папку модуля:

```text
ui-entity-showcase-rust/
  module.json
  locales/
    en-en.ini
    ru-ru.ini
  bin/
    ui_entity_showcase_rust.dll
  Cargo.toml
  Cargo.lock
  README_RU.md
  src/
    lib.rs
```

Для Linux в `bin/` лежит `.so`, для C++ - исходник `ui_entity_showcase_module.cpp` и `CMakeLists.txt`. Папка `locales/` обязательна для этих примеров: `en-en.ini` используется как fallback, а `ru-ru.ini` перекрывает строки при русском UI NetStitch. Все строки примера лежат в этой папке модуля; глобальные `resources/language/*` приложения для строк автора модуля не используются. Rust-пример использует ключи `ui_entity_showcase_rust.*`, включая `ui_entity_showcase_rust.module.display_name`; C++-пример использует ключи `ui_entity_showcase_cpp.*`, включая `ui_entity_showcase_cpp.module.display_name`.

## Установка архива

1. Выберите архив под платформу.
2. Распакуйте его в portable-папку NetStitch:

```text
NetStitch-win64-portable/
  integrations/
    ui-entity-showcase-rust/
      module.json
      bin/
```

3. Перезапустите NetStitch.
4. Откройте панель `Модули` и нажмите кнопку примера.

## Обновление архивов после изменения примеров

Если меняется любой файл в:

- `docs/module-sdk/examples/ui-entity-showcase-rust/`;
- `docs/module-sdk/examples/ui-entity-showcase-cpp/`;
- общий UI-контракт, влияющий на `module.json`;

нужно пересобрать архивы:

```powershell
scripts\package_module_sdk_examples.ps1
```

Это обязательная часть Definition of Done для изменений в SDK-примерах. Пользовательские архивы должны совпадать с исходниками, которые лежат в git.

## Linux-сборка

Скрипт использует отдельный тестовый WSL `NetStitch-Linux-Test`:

```powershell
wsl -d NetStitch-Linux-Test -- uname -a
scripts\package_module_sdk_examples.ps1
```

Не используйте другие WSL-дистрибутивы для SDK packaging. Если нужен другой тестовый distro name, передайте его явно:

```powershell
scripts\package_module_sdk_examples.ps1 -WslDistro NetStitch-Linux-Test
```

## Почему в module.json нет комментариев

`module.json` должен оставаться валидным JSON. Поэтому пояснения к полям находятся в `README_RU.md` рядом с примером и в документах SDK, а не внутри manifest-а.
