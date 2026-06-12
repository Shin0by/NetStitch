# Storage Layout

Каталог предназначен для runtime state, логов и generated data. Секреты и локальные данные выполнения не должны попадать в git.

Основная БД `netstitch.sqlite3` содержит только core runtime state NetStitch: tracked apps,
observations, export runs, app settings, ignored addresses и core-owned cache. Runtime-данные
внешних модулей в неё не пишутся.

Каждый integration module хранит собственный рабочий SQLite state в portable-контуре
`integrations/<module>/data/module.sqlite3`; рядом с ним остаются скачанные внешние приложения,
cache, backups и generated files модуля.
