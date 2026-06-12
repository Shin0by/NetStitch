# Resources Layout

Здесь размещаются локальные assets, templates и fixtures, если они потребуются проекту.

- `connectors/apps/` - tracked шаблоны runtime-коннекторов known apps; packaging копирует их в portable-папку `apps/`, где пользователь сможет добавлять, удалять или отключать `.app`-коннекторы.
- `connectors/icons/` - shipped `.svg` assets and extracted Windows `.ico` fallback assets для встроенных known-app connectors; packaging копирует `.svg` в editable `apps/icons`, где пользовательские `.svg`/`.png` по `icon_key` имеют приоритет.
- `language/` - внешние `.ini` каталоги локализации UI; packaging копирует их в portable-папку `language/`, где пользователь сможет добавлять новые локали вида `en-en.ini`, `ru-ru.ini`, `zh-cn.ini`.
- `runtime/windivert/windows-x86_64/` - tracked WinDivert runtime payload для Windows release packaging; `WinDivert.dll` и `WinDivert64.sys` копируются в корень Windows portable-папки, чтобы UDP/QUIC contour был доступен в GitHub Actions без локального `NETSTITCH__WINDIVERT_DIR`.
- `ui/icons/` - tracked UI SVG assets, embedded into the Dioxus shell with `include_str!`; currently contains the square close button icon for manual tracked-app deletion.
