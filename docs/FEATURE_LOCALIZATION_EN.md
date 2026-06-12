# Localization

NetStitch has a localized interface and ships with English and Russian language files. The selected language is part of the local application state, so desktop and web views use the same language setting.

## Where To Find It

- Interface: bottom bar -> `Language`.
- Portable files: `language/*.ini` next to the application.
- Shipped locales: `language/en-en.ini` and `language/ru-ru.ini`.

## How It Works

- The interface can be switched between available languages.
- Portable language files can be edited or extended.
- Missing translated strings fall back to English.
- Feature names, professional terms, and product names may remain in English when that is the clearer form.

Localization is part of the product surface, not a separate plugin. It applies to desktop UI, web UI, user-facing statuses, tooltips, and dialogs.
