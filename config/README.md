# Config Layout

Здесь хранятся tracked примеры конфигурации, schema/config loaders и безопасные шаблоны без секретов.

- `signing.example.toml` - безопасный шаблон данных для будущего подписывания файлов; реальные значения хранить только в ignored `config/signing.local.toml`, `.local/signing/` или через `NETSTITCH__SIGNING_...`.
- `endpoint_probe_targets.txt` - tracked список network endpoint reachability targets в формате `host:port` или `protocol://host:port`; UI читает его, при snapshot-запросах отправляет watcher-у и target, и protocol, а watcher/tool не держат встроенный список адресов.
