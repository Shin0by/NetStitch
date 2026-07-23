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
## Tags in CSV and cloud flows

Local CSV ends with a `tags` column; multiple local user tags are separated with `;`. Tags used only to filter downloaded cloud rows are not persisted locally, so a cloud staging CSV has the same final column but leaves it empty.

Tags are assigned to applications locally in the `Tracked apps` panel and accumulate automatically on observed rows. A tag is normalized to upper-case, has 1–16 characters, and accepts only English ASCII letters, digits, `-`, `_`, and `.`; all whitespace is rejected. Non-empty invalid input is shown in red and cannot be added. Assignment works without cloud authorization and does not upload the tag. The square SVG `T` button matches the adjacent `X` button: it is dark without a tag and blue like an active switch when a tag is assigned. Its tooltip explains that tags split one application's data into regions or groups. The modal does not close from its backdrop or after applying a selection; only the explicit `Close` action closes it. The manager deduplicates and combines current application tags, local observation tags, and the cloud catalog. The upper row is only for substring search and adding a missing valid tag through `Add new`; an empty filter shows every available option. The application's current tag is highlighted independently from the search input. Clicking a tag chip selects it, clicking it again clears the selection, and the single `Assign` action applies that state, including removing the application's tag; there is no separate removal action. Applying the state clears the search and restores the full list. Local history uses only rows available to the `Monitoring` contour: rows hidden by `Ignored addresses` no longer preserve a tag option after that tag is unassigned from every application. Ordinary search and header filters do not affect tag history. Author tags are listed first in blue, author tags that are also present in cloud are green, and other cloud tags keep the default color. Any author-owned tag can be removed either from this device only or from this device and the author's cloud namespace; tag links are removed without deleting monitoring rows. In `Monitoring`, the `Tag` column follows the application column and shows about eight characters in a single-line horizontally scrollable field.

Cloud export does not offer manual tag creation or assignment. Only an explicit upload of tagged monitoring rows registers their local tags in the account namespace and associates them with cloud observations. `Clear tags` removes all local tags from rows selected in `Monitoring` without deleting already published cloud associations. The upper export subpanel contains authorization and author-signature controls only; `Private upload` and `Clear tags` share a single nested row inside `Author publications`, immediately above the table. Rows for one application are grouped separately by their exact local tag set. The compact rightmost `Tag` column is only wide enough for its heading and button, with longer content scrolling inside; the `X` next to a tag removes only that tag from the local observations in its group. Cloud import has a separate tag substring filter between author and privacy.
