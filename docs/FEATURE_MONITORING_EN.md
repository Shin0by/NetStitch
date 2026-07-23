# Monitoring And Filtering

NetStitch shows network destinations for selected applications: IPs, domains, ports, protocols, connection state, and request counts.

## What The Table Shows

- the application for each row;
- IP and domain when the domain was verified or imported;
- port, protocol, and connection state;
- network range and address owner when that data is available;
- request counts plus accumulated successful/failed attempt signals.

## Domains And Ranges

NetStitch works with more than individual IPs. When an application exposes a domain name in the connection, NetStitch can show the verified domain next to the IP. Domains are also preserved when imported from trusted CSV data.

For IP addresses, NetStitch can show network range and address owner details. This helps identify whether a row belongs to a CDN, provider, cloud host, or service infrastructure.

Domains and ranges do not replace the original endpoint row. They enrich it and make filtering, confirmation, and later export preparation easier.

## Advanced Monitoring

Regular monitoring shows network destinations available through operating system connection tables. This is enough for many TCP scenarios.

Advanced mode is useful when an application uses network events that OS tables expose only partially:

- UDP and QUIC endpoints;
- short-lived connection attempts;
- domains from HTTP Host, TLS SNI, and QUIC Initial;
- cases where an application has only a local UDP socket, but the real remote addresses need to be seen.

On Windows, advanced mode may require running NetStitch with administrator rights and having the traffic-capture runtime files available in the portable folder. If those rights or files are missing, NetStitch continues in regular mode and shows the data available without advanced capture.

## Where To Find It

- Interface: main window -> `Tracked apps` panel for selecting applications.
- Observations table: main window -> `Monitoring`.
- Filters: main window header (`App`, `IP`, `Domain`, `Port`, `Protocol`, `State`, `Public`).
- The `Domain` filter supports `*` as any number of characters: for example, `*example.com` matches domains with that suffix, and `example*.com` matches domains that start with `example` and end with `.com`.
- The `Monitoring` panel header shows `Rows` and `Displayed`; the main footer no longer has a separate row counter.
- The `Public` switch toggles the table between public and non-public IP addresses. The latest switch state is stored in SQLite `app_settings` under `ui.monitoring.public_ip`.
- After `Public`, a vertical separator precedes the `Tags` and `Connection and count` switches. Both are enabled by default. When enabled, they only change the visual Monitoring table layout, allowing stretchable columns to use the released space. Desktop and browser UI share the behavior, and SQLite `app_settings` persists it under `ui.monitoring.hide_tags` and `ui.monitoring.hide_connection_count`.
- Advanced mode: main window header -> `Advanced mon.`.
- Local portable storage: `storage/`.

Typical workflow:

1. Select an application.
2. Wait for rows to appear.
3. Filter out noise.
4. Confirm rows that are relevant for further work.

Confirmation helps separate the useful data set from background noise.

Monitoring filters do not delete anything from local storage. They only change the visible row set, which can then be confirmed, exported, or used in the next workflow. While scanning is active, new rows are immediately evaluated against the current filters.

Desktop and web UI use the same local NetStitch runtime, so selected applications, filters, and observations stay part of one working state.
