# Web Control

NetStitch can expose a web interface for controlling the app from another machine on the local network. This is useful when NetStitch runs on one PC, while tables, tracked applications, import, or export actions are handled from a browser on another device.

Web control does not require a separate server installation: the web page runs on top of the already running NetStitch instance.

## Where To Find It

- Interface: main window -> `Web server` switch.
- Tray menu: `Web server` item.
- The browser address is shown in the app footer.
- The web page is opened from a machine that can reach the local NetStitch network address.

## How To Enable

1. Start NetStitch on the machine that should perform monitoring.
2. Turn on the `Web server` switch.
3. Copy the address from the app footer.
4. Open that address in a browser from another device on the same local network.

The copied link contains the access key after `#`. This fragment is not sent by the normal GET request: the web page sends the key to local NetStitch with a POST request, receives an access cookie, and removes the key from the address bar. If only the IP and port are opened without the key, the key prompt is shown; the key is available in the link copied from the desktop footer.

By default, web access uses HTTPS with a local self-signed certificate from portable storage. The browser may show a certificate trust warning on first open.

## What You Can Do

- view the same main panels as in the desktop interface;
- manage tracked applications;
- work with the `Monitoring` table;
- use CSV import and CSV export through the file system of the machine running NetStitch;
- use cloud import and export through the local NetStitch runtime.

The web interface is not a separate cloud client. All actions go through the locally running NetStitch instance, so the web version mirrors the desktop app state and does not make direct requests to the cloud service.

## Important Boundaries

- The web page controls the same state as the desktop app.
- File selection in the web interface refers to the machine running NetStitch.
- If NetStitch is closed, the web interface is unavailable.
- The web access key is required for browsers that open the address without an existing access cookie.
- Web access is intended for a trusted local network, not for publishing to the internet.

Enable web access only on a trusted local network.
