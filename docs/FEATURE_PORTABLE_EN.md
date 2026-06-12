# Portable Mode

Portable builds run from the unpacked folder and do not require mandatory system-wide installation.

NetStitch is shipped as a cross-platform desktop/runtime flow: the main user workflow supports Windows and Linux, while app connectors are designed with Windows, Linux, and macOS in mind.

## Where To Find It

- Windows: unpacked portable folder -> `NetStitch.exe`.
- Linux: unpacked portable folder -> `NetStitch`.
- Linux desktop launcher: `NetStitch.desktop`.
- User-editable portable files: `storage/`, `apps/`, `apps/icons/`, `language/`, `config/`.

Working files stay next to the application:

- `storage/` - local state and observation database;
- `apps/` - user-editable app connectors;
- `language/` - interface localization files;
- `config/` - editable shipped configuration.

This format is convenient for moving between folders and test environments.

## What Stays Next To The App

- the local monitoring database;
- selected language and interface settings;
- user-editable app connectors;
- NetStitch import, export, and working files.

Portable mode does not require a global NetStitch installation. Advanced Windows network monitoring may require administrator rights and locally supplied traffic-capture runtime files. Linux uses its own portable build with a desktop launcher file.
