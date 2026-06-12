# Source Layout

Здесь размещается основной код `NetStitch`.

## Workspace Modules

- `netstitch-shared` - общие модели, DTO, IPC-контракты и cloud DTO для Worker/client boundary
- `netstitch-cloud` - отдельный contour community cloud sync helpers: client identity, quota mirror, verified-app upload policy и request integrity; сетевые запросы допускаются только из явных action handlers
- `netstitch-cloud-worker` - Cloudflare Worker + D1 backend contour для community cloud sync; внешние Cloudflare ресурсы называются `netstitch-*`, внутренние D1 tables без префикса
- `netstitch-core` - сервисный слой, SQLite storage и orchestration
- `../integrations/host` - generic host/API для внешних DLL-модулей интеграции
- `../integrations/<module>` - внешний module contour; в текущем репозитории хранится временно до выноса в отдельные репозитории
- `netstitch-watcher` - отдельный backend-процесс мониторинга и локального IPC API
- `netstitch-tool` - optional standalone network tool module для DNS reachability, явного domain lookup и WHOIS enrichment cache; домены мониторинга берутся только из verified HTTP Host, TCP TLS SNI и QUIC Initial SNI path в watcher
- `netstitch-ui` - Dioxus Desktop приложение
