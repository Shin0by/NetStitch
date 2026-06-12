# Data Import And Export

NetStitch supports moving observations between work environments through CSV and cloud storage. Both paths use the same row merge principle: matching observations are folded together, while new endpoint rows are added to the current data set.

## CSV

CSV is intended for manual data transfer:

- export saves selected monitoring rows;
- import adds rows to the current monitoring set without clearing existing data;
- matching IP, port, and protocol values are merged;
- domains are carried as part of the observation when present;
- application identifiers and origin signatures are preserved when present in the file.

## Domains And Ranges In Export

Domains act as additional evidence for an endpoint row. If a domain was found by monitoring or came from a trusted import, it can be carried into CSV and used in later processing.

Ranges make it possible to work with address blocks instead of only individual IPs. When range data is known for an IP, NetStitch can use it while preparing export data so the user can work with a more stable set of network destinations.

## Where To Find It

- CSV: header buttons `Import CSV` and `Export selected to CSV`.
- Cloud download: `Import Cloud` button -> `Cloud: data download`.
- Cloud upload: `Export Cloud` button -> `Cloud: data upload`.
- Adding cloud rows: `Cloud: data download` -> select rows -> `Add to monitoring`.
- Exporting cloud rows: `Cloud: data download` -> select rows -> `Export CSV`.
- Local portable data: `storage/`; CSV files are saved to or loaded from the path chosen by the user.

## Cloud Storage

Cloud exchange is useful when an observation set should be available on another machine or to another NetStitch user.

- Publishing uploads confirmed rows for the selected application.
- Download shows matching applications and a staging table before adding rows to monitoring.
- Cloud rows can be added to monitoring or exported to CSV only after explicit row selection in the staging table.
- The result counter shows how many rows were selected, imported, and skipped; cloud download/upload progress uses 500-row steps.
- Domains and row data stay in the same logical data set as local monitoring.
- The web interface uses the same local NetStitch runtime as the desktop app.

## Private Mode

Publishing can use either public or private visibility.

- Public rows are part of the shared data set.
- Private rows are available to the account owner and may exist separately from matching public rows in cloud storage.
- During download, the `Private` filter can select `All`, `Public`, or `Private`.
- Privacy is shown only in the cloud import table through the `Private` column: `true` for private rows and `false` for public rows.
- After adding rows to monitoring or exporting them to CSV, the privacy marker is not carried forward.

This keeps the shared data set and personal publications available side by side without leaking cloud privacy into the local monitoring table.
