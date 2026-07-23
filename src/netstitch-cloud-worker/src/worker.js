const JSON_HEADERS = {
  "content-type": "application/json; charset=utf-8",
  "cache-control": "no-store",
};

const RETENTION_MS = 365 * 24 * 60 * 60 * 1000;
const QUOTA_LIMIT = 0;
const QUOTA_WINDOW_MS = 6 * 60 * 60 * 1000;
const SESSION_TTL_MS = 30 * 24 * 60 * 60 * 1000;
const OAUTH_STATE_TTL_MS = 10 * 60 * 1000;
const OAUTH_START_LIMIT = 5;
const OAUTH_START_WINDOW_MS = 15 * 60 * 1000;
const AUTH_ATTEMPT_LIMIT = 3;
const AUTH_ATTEMPT_WINDOW_MS = 60 * 60 * 1000;
const CAPTCHA_ALPHABET = "23456789";
const CAPTCHA_LENGTH = 6;
const CAPTCHA_REFRESH_POOL_SIZE = 6;
const CAPTCHA_TILE_COLUMNS = 10;
const CAPTCHA_TILE_ROWS = 3;
const CAPTCHA_TTL_MS = 300 * 1000;
const CAPTCHA_MIN_SOLVE_MS = 5 * 1000;
const CAPTCHA_MAX_ATTEMPTS = 3;
const CAPTCHA_FAILURE_WINDOW_MS = 15 * 60 * 1000;
const CAPTCHA_FAILURE_LOCKOUT_THRESHOLD = 3;
const CAPTCHA_FAILURE_LOCKOUT_MS = 5 * 60 * 1000;
const CAPTCHA_FAILURE_BACKOFF_BASE_MS = 2 * 1000;
const CAPTCHA_FAILURE_BACKOFF_MAX_MS = 30 * 1000;
const BROWSER_VERIFICATION_TTL_MS = 120 * 1000;
const BROWSER_VERIFICATION_MAX_ATTEMPTS = 3;
const BROWSER_VERIFICATION_POW_DIFFICULTY = 2;
const BROWSER_VERIFICATION_MEMORY_SIZE_KIB = 128;
const BROWSER_VERIFICATION_MEMORY_ROUNDS = 2;
const BROWSER_VERIFICATION_MIN_SIGNAL_SCORE = 6;
const BROWSER_VERIFICATION_SUSPICIOUS_SCORE_THRESHOLD = 40;
const BROWSER_VERIFICATION_HOSTILE_SCORE_THRESHOLD = 90;
const BROWSER_VERIFICATION_TARPIT_MS = 350;
const OBSERVATION_BATCH_LIMIT = 65535;
const TAG_MAX_LENGTH = 16;
const TAGS_PER_OBSERVATION_LIMIT = 32;
const TAGS_PER_USER_LIMIT = 100;
const CLEANUP_LIMIT = 200;
const JWT_AUDIENCE = "netstitch-cloud";
const JWT_CLOCK_SKEW_SECONDS = 30;
const UPLOAD_SCOPE = "observations:append";
const RESERVED_AUTHOR_SIGNATURES = [
  "admin",
  "administrator",
  "anon",
  "anonymous",
  "cloud",
  "cloudflare",
  "google",
  "mod",
  "moderator",
  "netstitch",
  "netstitchcloud",
  "netstitchsupport",
  "official",
  "owner",
  "root",
  "support",
  "system",
];
const NETSTITCH_LOGO_DATA_URI = [
  "data:image/png;base64,",
  "iVBORw0KGgoAAAAN",
  "SUhEUgAAApoAAAKa",
  "CAYAAACeDPn/AAAA",
  "CXBIWXMAAAsTAAAL",
  "EwEAmpwYAAAKBGlU",
  "WHRYTUw6Y29tLmFk",
  "b2JlLnhtcAAAAAAA",
  "PD94cGFja2V0IGJl",
  "Z2luPSLvu78iIGlk",
  "PSJXNU0wTXBDZWhp",
  "SHpyZVN6TlRjemtj",
  "OWQiPz4gPHg6eG1w",
  "bWV0YSB4bWxuczp4",
  "PSJhZG9iZTpuczpt",
  "ZXRhLyIgeDp4bXB0",
  "az0iQWRvYmUgWE1Q",
  "IENvcmUgMTAuMC1j",
  "MDAwIDI1LkcuZDIw",
  "ZTQ2NiwgMjAyNS8x",
  "Mi8wOC0yMDo1MDoy",
  "MSAgICAgICAgIj4g",
  "PHJkZjpSREYgeG1s",
  "bnM6cmRmPSJodHRw",
  "Oi8vd3d3LnczLm9y",
  "Zy8xOTk5LzAyLzIy",
  "LXJkZi1zeW50YXgt",
  "bnMjIj4gPHJkZjpE",
  "ZXNjcmlwdGlvbiBy",
  "ZGY6YWJvdXQ9IiIg",
  "eG1sbnM6eG1wTU09",
  "Imh0dHA6Ly9ucy5h",
  "ZG9iZS5jb20veGFw",
  "LzEuMC9tbS8iIHht",
  "bG5zOnN0RXZ0PSJo",
  "dHRwOi8vbnMuYWRv",
  "YmUuY29tL3hhcC8x",
  "LjAvc1R5cGUvUmVz",
  "b3VyY2VFdmVudCMi",
  "IHhtbG5zOnN0UmVm",
  "PSJodHRwOi8vbnMu",
  "YWRvYmUuY29tL3hh",
  "cC8xLjAvc1R5cGUv",
  "UmVzb3VyY2VSZWYj",
  "IiB4bWxuczpkYz0i",
  "aHR0cDovL3B1cmwu",
  "b3JnL2RjL2VsZW1l",
  "bnRzLzEuMS8iIHht",
  "bG5zOnBob3Rvc2hv",
  "cD0iaHR0cDovL25z",
  "LmFkb2JlLmNvbS9w",
  "aG90b3Nob3AvMS4w",
  "LyIgeG1sbnM6eG1w",
  "PSJodHRwOi8vbnMu",
  "YWRvYmUuY29tL3hh",
  "cC8xLjAvIiB4bWxu",
  "czp0aWZmPSJodHRw",
  "Oi8vbnMuYWRvYmUu",
  "Y29tL3RpZmYvMS4w",
  "LyIgeG1sbnM6ZXhp",
  "Zj0iaHR0cDovL25z",
  "LmFkb2JlLmNvbS9l",
  "eGlmLzEuMC8iIHht",
  "cE1NOkRvY3VtZW50",
  "SUQ9ImFkb2JlOmRv",
  "Y2lkOnBob3Rvc2hv",
  "cDo0MmJmN2QwZC02",
  "YTFlLTNiNDEtYTY2",
  "Yi1lNzAxZDExODRk",
  "NDciIHhtcE1NOklu",
  "c3RhbmNlSUQ9Inht",
  "cC5paWQ6MWJkODYx",
  "OGUtOGVjZC1lYzQ3",
  "LTljYTYtMzY2NmE5",
  "ODU5ZTg1IiB4bXBN",
  "TTpPcmlnaW5hbERv",
  "Y3VtZW50SUQ9IjA5",
  "ODcwQTRERDUzM0I2",
  "QTRGRUNDNjk0NTU5",
  "NjgxMEQ4IiBkYzpm",
  "b3JtYXQ9ImltYWdl",
  "L3BuZyIgcGhvdG9z",
  "aG9wOkNvbG9yTW9k",
  "ZT0iMyIgeG1wOkNy",
  "ZWF0ZURhdGU9IjIw",
  "MjYtMDQtMjVUMTk6",
  "MzY6MzYrMDM6MDAi",
  "IHhtcDpNb2RpZnlE",
  "YXRlPSIyMDI2LTA0",
  "LTI1VDIxOjM3OjA0",
  "KzAzOjAwIiB4bXA6",
  "TWV0YWRhdGFEYXRl",
  "PSIyMDI2LTA0LTI1",
  "VDIxOjM3OjA0KzAz",
  "OjAwIiB0aWZmOklt",
  "YWdlV2lkdGg9Ijgz",
  "MiIgdGlmZjpJbWFn",
  "ZUxlbmd0aD0iMTI0",
  "OCIgdGlmZjpQaG90",
  "b21ldHJpY0ludGVy",
  "cHJldGF0aW9uPSIy",
  "IiB0aWZmOlNhbXBs",
  "ZXNQZXJQaXhlbD0i",
  "MyIgdGlmZjpYUmVz",
  "b2x1dGlvbj0iNzIv",
  "MSIgdGlmZjpZUmVz",
  "b2x1dGlvbj0iNzIv",
  "MSIgdGlmZjpSZXNv",
  "bHV0aW9uVW5pdD0i",
  "MiIgZXhpZjpFeGlm",
  "VmVyc2lvbj0iMDIz",
  "MSIgZXhpZjpDb2xv",
  "clNwYWNlPSI2NTUz",
  "NSIgZXhpZjpQaXhl",
  "bFhEaW1lbnNpb249",
  "IjgzMiIgZXhpZjpQ",
  "aXhlbFlEaW1lbnNp",
  "b249IjEyNDgiPiA8",
  "eG1wTU06SGlzdG9y",
  "eT4gPHJkZjpTZXE+",
  "IDxyZGY6bGkgc3RF",
  "dnQ6YWN0aW9uPSJz",
  "YXZlZCIgc3RFdnQ6",
  "aW5zdGFuY2VJRD0i",
  "eG1wLmlpZDpjMDNh",
  "ZDk0ZS1jNTYwLWU3",
  "NDktYmI3ZS0yNWJh",
  "YTkzYzljOTYiIHN0",
  "RXZ0OndoZW49IjIw",
  "MjYtMDQtMjVUMjA6",
  "MTk6MzErMDM6MDAi",
  "IHN0RXZ0OnNvZnR3",
  "YXJlQWdlbnQ9IkFk",
  "b2JlIFBob3Rvc2hv",
  "cCAyNy41IChXaW5k",
  "b3dzKSIgc3RFdnQ6",
  "Y2hhbmdlZD0iLyIv",
  "PiA8cmRmOmxpIHN0",
  "RXZ0OmFjdGlvbj0i",
  "Y29udmVydGVkIiBz",
  "dEV2dDpwYXJhbWV0",
  "ZXJzPSJmcm9tIGlt",
  "YWdlL2pwZWcgdG8g",
  "aW1hZ2UvcG5nIi8+",
  "IDxyZGY6bGkgc3RF",
  "dnQ6YWN0aW9uPSJk",
  "ZXJpdmVkIiBzdEV2",
  "dDpwYXJhbWV0ZXJz",
  "PSJjb252ZXJ0ZWQg",
  "ZnJvbSBpbWFnZS9q",
  "cGVnIHRvIGltYWdl",
  "L3BuZyIvPiA8cmRm",
  "OmxpIHN0RXZ0OmFj",
  "dGlvbj0ic2F2ZWQi",
  "IHN0RXZ0Omluc3Rh",
  "bmNlSUQ9InhtcC5p",
  "aWQ6MDY0ZWVkYjYt",
  "Y2ZkYy1iYjQ5LWEx",
  "NWQtMDMwMTY0YmFh",
  "NWJhIiBzdEV2dDp3",
  "aGVuPSIyMDI2LTA0",
  "LTI1VDIwOjE5OjMx",
  "KzAzOjAwIiBzdEV2",
  "dDpzb2Z0d2FyZUFn",
  "ZW50PSJBZG9iZSBQ",
  "aG90b3Nob3AgMjcu",
  "NSAoV2luZG93cyki",
  "IHN0RXZ0OmNoYW5n",
  "ZWQ9Ii8iLz4gPHJk",
  "ZjpsaSBzdEV2dDph",
  "Y3Rpb249InNhdmVk",
  "IiBzdEV2dDppbnN0",
  "YW5jZUlEPSJ4bXAu",
  "aWlkOjFiZDg2MThl",
  "LThlY2QtZWM0Ny05",
  "Y2E2LTM2NjZhOTg1",
  "OWU4NSIgc3RFdnQ6",
  "d2hlbj0iMjAyNi0w",
  "NC0yNVQyMTozNzow",
  "NCswMzowMCIgc3RF",
  "dnQ6c29mdHdhcmVB",
  "Z2VudD0iQWRvYmUg",
  "UGhvdG9zaG9wIDI3",
  "LjUgKFdpbmRvd3Mp",
  "IiBzdEV2dDpjaGFu",
  "Z2VkPSIvIi8+IDwv",
  "cmRmOlNlcT4gPC94",
  "bXBNTTpIaXN0b3J5",
  "PiA8eG1wTU06RGVy",
  "aXZlZEZyb20gc3RS",
  "ZWY6aW5zdGFuY2VJ",
  "RD0ieG1wLmlpZDpj",
  "MDNhZDk0ZS1jNTYw",
  "LWU3NDktYmI3ZS0y",
  "NWJhYTkzYzljOTYi",
  "IHN0UmVmOmRvY3Vt",
  "ZW50SUQ9IjA5ODcw",
  "QTRERDUzM0I2QTRG",
  "RUNDNjk0NTU5Njgx",
  "MEQ4IiBzdFJlZjpv",
  "cmlnaW5hbERvY3Vt",
  "ZW50SUQ9IjA5ODcw",
  "QTRERDUzM0I2QTRG",
  "RUNDNjk0NTU5Njgx",
  "MEQ4Ii8+IDx0aWZm",
  "OkJpdHNQZXJTYW1w",
  "bGU+IDxyZGY6U2Vx",
  "PiA8cmRmOmxpPjg8",
  "L3JkZjpsaT4gPHJk",
  "ZjpsaT44PC9yZGY6",
  "bGk+IDxyZGY6bGk+",
  "ODwvcmRmOmxpPiA8",
  "L3JkZjpTZXE+IDwv",
  "dGlmZjpCaXRzUGVy",
  "U2FtcGxlPiA8L3Jk",
  "ZjpEZXNjcmlwdGlv",
  "bj4gPC9yZGY6UkRG",
  "PiA8L3g6eG1wbWV0",
  "YT4gPD94cGFja2V0",
  "IGVuZD0iciI/PpDM",
  "GjoAAcvLSURBVHic",
  "7J0HnGNV9cdP+rQt",
  "sLSlF2mCgHSlKwhK",
  "lw5/KYoUC4LSRDpY",
  "6FURAQsqHUR6F5Am",
  "vZel7i5bWLbv9LT/",
  "53fuO2/uhJm8m0ky",
  "eUnOF58p+5K8ZN59",
  "99xTfieSz+dJURRF",
  "URRFUSpNtOLvqCiK",
  "oiiKoihqaCqKoiiK",
  "oijVQg1NRVEURVEU",
  "pSqooakoiqIoiqJU",
  "BTU0FUVRFEVRlKqg",
  "hqaiKIqiKIpSFdTQ",
  "VBRFURRFUaqCGpqK",
  "oiiKoihKVVBDU1EU",
  "RVEURakKamgqiqIo",
  "iqIoVUENTUVRFEVR",
  "FKUqqKGpKIqiKIqi",
  "VAU1NBVFURRFUZSq",
  "oIamoiiKoiiKUhXU",
  "0FQURVEURVGqghqa",
  "iqIoiqIoSlVQQ1NR",
  "FEVRFEWpCmpoKoqi",
  "KIqiKFVBDU1FURRF",
  "URSlKqihqSiKoiiK",
  "olQFNTQVRVEURVGU",
  "qqCGpqIoiqIoilIV",
  "1NBUFEVRFEVRqoIa",
  "moqiKIqiKEpVUENT",
  "URRFURRFqQpqaCqK",
  "oiiKoihVQQ1NRVEU",
  "RVEUpSqooakoiqIo",
  "iqJUBTU0FUVRFEVR",
  "lKqghqaiKIqiKIpS",
  "FdTQVBRFURRFUaqC",
  "GpqKoiiKoihKVVBD",
  "U1EURVEURakKamgq",
  "iqIoiqIoVUENTUVR",
  "FEVRFKUqqKGpKIqi",
  "KIqiVAU1NBVFURRF",
  "UZSqoIamoiiKoiiK",
  "UhXU0FQURVEURVGq",
  "ghqaiqIoiqIoSlVQ",
  "Q1NRFEVRFEWpCmpo",
  "KoqiKIqiKFVBDU1F",
  "URRFURSlKqihqSiK",
  "oiiKolQFNTQVRVEU",
  "RVGUqqCGpqIoiqIo",
  "ilIV1NBUFEVRFEVR",
  "qoIamoqiKIqiKEpV",
  "UENTURRFURRFqQpq",
  "aCqKoiiKoihVQQ1N",
  "RVEURVEUpSqooako",
  "iqIoiqJUBTU0FUVR",
  "FEVRlKqghqaiKIqi",
  "KIpSFdTQVBRFURRF",
  "UaqCGpqKoiiKoihK",
  "VVBDU1EURVEURakK",
  "amgqiqIoiqIoVUEN",
  "TUVRFEVRFKUqqKGp",
  "KIqiKIqiVAU1NBVF",
  "URRFUZSqoIamoiiK",
  "oiiKUhXU0FQURVEU",
  "RVGqghqaiqIoiqIo",
  "SlVQQ1NRFEVRFEWp",
  "CmpoKoqiKIqiKFVB",
  "DU1FURRFURSlKqih",
  "qSiKoiiKolQFNTQV",
  "RVEURVGUqqCGpqIo",
  "iqIoilIV1NBUFEVR",
  "FEVRqoIamoqiKIqi",
  "KEpVUENTURRFURRF",
  "qQpqaCqKoiiKoihV",
  "QQ1NRVEURVEUpSrE",
  "q/O2iqIoiqIoVaWF",
  "iLb0tq2IaAMiGkNE",
  "nxHRf4noKW97vdYH",
  "2sxE8vl8rY9BURRF",
  "URTFhY09w/Ig774L",
  "C4noDiJ6yDM8p1b5",
  "GBULNTQVRVEURQk7",
  "hxLRVZ4Xs1wmE9HR",
  "RPQIEaUr8H5KEdTQ",
  "VBRFURQlrOznGZiL",
  "Ven9jyGia4iot0rv",
  "3/SooakoiqIoShi5",
  "zDMER4PjiOhKIsqM",
  "0uc1DWpoKoqiKIoS",
  "JpbxvJh71OCzf+oZ",
  "nEqFUENTURRFUZSw",
  "gKrx62tkZAo3ejmc",
  "C2p4DA2D6mgqiqIo",
  "ihIWauXJtDmAiOYT",
  "0bE1Po6GQA1NRVEU",
  "RVHCwDmebFFYuISI",
  "biGiCbU+kHpGDU1F",
  "URRFUWrNt4no1JG+",
  "ePvtt6cllliCUqkU",
  "rbjiinTssRVzRu5D",
  "RO971e/KCNAcTUVR",
  "FEVRaskSRPQiEa1U",
  "yosmTpxIc+fOpf7+",
  "/qL7bbDBBvTKK69Q",
  "BbiciH5WiTdqJtTQ",
  "rC82I6IZRDSl1gei",
  "KIqiKBXiViLa23Xn",
  "VVZZhaZMmUK5XM75",
  "A+LxOLW3t9P8+Ui9",
  "LIu7vEKh6eW+UbOg",
  "hma4aLX6tkrv1kSR",
  "/WdYvVyxvTyKx6oo",
  "iqIoldCvvNh151gs",
  "RtFolDKZDBuPuA0i",
  "EomQ2Dq439raSl1d",
  "XeUc8xtE9H3PC6sE",
  "oIZmONiQiLYpZbAN",
  "Q7e30kLysnY5UBRF",
  "UcLuXMG85cTYsWOp",
  "r6+PQ+UwMrPZLLW1",
  "tQUajTBO4f20jU3c",
  "h4ezs7OznOP/mRdO",
  "V4qgxUC1I05EBxMR",
  "RshLFTAyQRsR/Y2I",
  "elSWQVEURQk5F7ju",
  "uOGGG9KiRYvYyITh",
  "CE8mDEYXzyQMUhiW",
  "eB3Afby2u7ubn3v+",
  "+efL6VyETSmCejRH",
  "nyQRHUFEV4xS3gs8",
  "nHNG4bMURVEUxZUD",
  "ieifLjtee+219MMf",
  "/pArymE0StgcxiI8",
  "lXjOBbweNk86nfYN",
  "T3g1Yazi/Xp7RxwI",
  "VIH3IqihObocU4PV",
  "jyYuK4qiKGEDRtlY",
  "lx1hUCYSCTYwW1pa",
  "qKenx38e+ZpBhmZh",
  "LicMTNg+MFKTyaRv",
  "eIIybKJ/eXPtZyN9",
  "g0ZFQ+ejwypE9ECN",
  "XOy7EdF/iOgbNfhs",
  "RVEURSlkPVcjE15I",
  "IMagGJkAj128mYUF",
  "Q3iNVKwjFA9jUwxQ",
  "bKutthqNgD2J6Gqo",
  "Lo3kxY2MGprV5wQi",
  "+oiIdqzhMaxBRI+q",
  "sakoiqKEgN+77HT+",
  "+ecHamRWAjE2YYDC",
  "+zl58mQ2OO+6CwHB",
  "ktidiP5IRMtW50jr",
  "Ew2dV49lQtKztZAj",
  "iehPtT4IRVEUpSkZ",
  "5/URD0Qqy0cDhObh",
  "NZVbKRz61a9+RWed",
  "dVapb6cpaxbq0awO",
  "hxPR1EobmS+88AJN",
  "nYq3LYurvcp0RVEU",
  "RanF/BjIl770pVEz",
  "MgGMSzvMDkMTn/+b",
  "3/yGdtppp5GkrCFd",
  "7igiWpeaHPVoVpaU",
  "58U8rNw3uvvuu2mf",
  "ffZhzTD7pMcKT/JS",
  "ll9++XIMT/VsKoqi",
  "KKMJGpA4xcLhWZQ8",
  "ylI6AI0EW9BdCofw",
  "nFS1i+H7/vtoeV4y",
  "Oc+72bTzrRqalWNd",
  "z8hER58Rs/LKK3Nr",
  "LfxdMNBgUEoXBOSQ",
  "4KTHfTyHfbAhWXoE",
  "sgw4+dcnojfLOV5F",
  "URRFcWQ7InosaKdd",
  "dtmFnnjiiXLF1EsC",
  "BiY2zK3Q1xQw7+I5",
  "zLHjxo0rp4Xl9UR0",
  "CDUhamhWhp+Uq4vZ",
  "0dHBCcny9xCBWZFe",
  "kFtZXYmhKfexjSBp",
  "+iIiOr6c41YURVEU",
  "R/5CRIcG7SROltG0",
  "T8TIxDwKT6aIwuN5",
  "iSjKYwmzj4Drm9HY",
  "1BzN8pOabyjHyET7",
  "LAwqrJZw8uJEtqUY",
  "xHiUk19Oemmnhfvi",
  "6cS///znPy/l439B",
  "RN8d6bEriqIoSglh",
  "80NdxNkxB2J+w3w3",
  "WuAzbWeNzMMyJ0tU",
  "UVLYRsjBzVgjoR7N",
  "8kLl53pyBiVz8MEH",
  "04033sjGocvqCCut",
  "YsiKCwbnz372M7r0",
  "0ktdD2UWES3turOi",
  "KIqiVCtsPmbMGK5N",
  "wFwGg86O9I0UzLPF",
  "GOn7l3Fc1zeTZ1M9",
  "miMDLSRfG4mR+emn",
  "n3JnAxiZ0goryIgE",
  "ko853CarLBicv//9",
  "7+mee+7hz3JgKa2K",
  "UxRFUaqMU/0ConsS",
  "oRPPZljBMY5Aa1M8",
  "m7AjmgI1NEvnb55E",
  "UMm/HcRn0XFA3O6V",
  "lG7ACY+Vn+SV7Lnn",
  "nvTyyy+7vhxFTIqi",
  "KIpSLQLrAY477riS",
  "epeHgd13353OOOOM",
  "kbz0qmZx8mjovDSO",
  "8IzMkll22WVp3rx5",
  "vFpDngeMQinigeE5",
  "gqrxL2D3bIXB6RqW",
  "9xjv9Z5VFEVRlEq3",
  "nEQUMDBsLpXmmBsx",
  "h1XC6KxW6FzAse67",
  "777sTELNxTLLoF+L",
  "E9c3QwhdPZpVNjIR",
  "woYhOXfuXDYmcRJK",
  "wjFWbjAIRSuzXCQE",
  "j88TGaS1117b9eVl",
  "yTIpiqIoSjnzi8gK",
  "iapKvXg2Md/efvvt",
  "dNhhh7Fz57PPPnN9",
  "6cHNEEJXQ7O0cHlJ",
  "7LHHHrTrrrvyYIEx",
  "CY+jXVFeympKxGOH",
  "22CwirdU3hOD9d13",
  "33U93DNL/X6KoiiK",
  "UglDE81HMJdh3pIc",
  "TYCahrAj+aTQ/vy/",
  "//s/diaVYGxe1egh",
  "dDU03YxMrDpKYvz4",
  "8fTvf//blx+ScDkM",
  "QBk48G7CAMVtuUgx",
  "kFTqAbkPg9eBjT25",
  "JkVRFEWpJDsH7bDl",
  "llt+wemC+VN6jocd",
  "NE6Bsfnkk0/S9773",
  "vVKMzSgR/ZqInOPt",
  "9YbmaFYhXC5uf9zK",
  "IBHpIVmt2fu65KEE",
  "VaaLUYkTXXQ2pX2W",
  "FAg5AIP6727fUlEU",
  "RVEqk585lEi7rRkd",
  "5hxNHKdIMcn7bbLJ",
  "JhxOX2GFFVzf5k6v",
  "D/wcajDU0Bye3Yjo",
  "3647S8/xFVdcsSoD",
  "oVwce8VORhfMqh6I",
  "oiiK0kz8iIh+X805",
  "sNqG5EiPbeutt6a/",
  "//3vpRibtxLRfjhk",
  "aiA0dF4BI1MYqZE5",
  "Gmy//fYuu61ERM4j",
  "QlEURVHKzc8coRZl",
  "6EEY/Uc/+pGrpjXY",
  "pxHlBtXQHDpXsSQj",
  "84c//GGojUzw6KOP",
  "uu6q1eeKoihKpdgq",
  "aIef/OQnVEuCim3L",
  "4d5776UbbkCnameO",
  "JKLLqIFQQ/OLnswX",
  "SgmXH3vssXT99ZDC",
  "CjfI8XRcNaqhqSiK",
  "olSK5YN2KMHjV5ec",
  "euqppb7kGG9rCDRH",
  "c4SFPzAyr7vuOjrr",
  "rLO+UOATxhxNvP/6",
  "669HL7/8isvuregE",
  "VtUDUhRFURodpGJN",
  "qfb8V26OZrXnXzBC",
  "8flNiOhFqnPUozlC",
  "ncxrrrmGzj33XJYq",
  "KtfIHA1wjG+88brr",
  "7urVVBRFUcrlmlof",
  "QBiIWAowJdIQ+Zpq",
  "aBpPZkk6mUcccQT9",
  "9re/5fsltHisOZmM",
  "s0GshqaiKIpSDqcR",
  "0Y5BOyEyWGuqmaMp",
  "HlXobMLYhMZ2iTUj",
  "V1Kd0+yG5salejIP",
  "P/xw9maKLiVc4UEa",
  "l2FAROHPOON0l93V",
  "0FQURVFGyjeJ6GyX",
  "HS+//HJqdGKxGBub",
  "aNyyYMECWmklCLw4",
  "82MiOpDqmGbO0VyW",
  "iF4moqVdX3D66afT",
  "OeecM6IPq7UxGosl",
  "2PuaSiW4VWUAUJ1N",
  "jc6RKYqiKA3GGa5t",
  "jdvb26mnp6eqBxNk",
  "55Q7Pwelz8ViMXZK",
  "weGDeRi3yy23HH38",
  "8ceuH7GQiNYmoulU",
  "h4TfFVc9I/Oq0TIy",
  "wwA8sBhMjqF+NExf",
  "qvpHpSiKojQgzlEx",
  "tGFudLLZLIfgxeCF",
  "YTpt2jRXfWsw1rNZ",
  "YLvUHdEmNjIhZeSs",
  "k1nPRiYYQRKyhs8V",
  "RVGUkeA0f5x//vl1",
  "UUxbiRzPSEFBEIzP",
  "xx9/3O8q6MBunu1S",
  "d9HGODUXJRmZOAFu",
  "vPFGuvbaa6kRMKup",
  "fCkXijuqe0SKoihK",
  "gwFJnpZiO7z22mvU",
  "2dlJf/nLX3heGg15",
  "oVoTjUb9Xu5idOJ2",
  "lVVW8Ws+SjA2v091",
  "RDN5NCeW6smEkXnS",
  "SSdRIyAnMk7szTff",
  "vCLdHBRFURSlgJlB",
  "Oyy11FI0btw4mjFj",
  "BjUD0WjU926KoYnn",
  "cB/G56qrrlrK2x2G",
  "umSqI5rF0BxTqpGJ",
  "nMwzzzyTWltbG2K1",
  "hZM6HjcO7Jdeesnl",
  "JV+t9jEpiqIoDYfT",
  "hDlhwgQuAqp1oexo",
  "EIlEuD5CvJi4FYMT",
  "WtyTJ0+mCy64oJS3",
  "vMpzntUFzVJ1/g8i",
  "Osh1Z4TKkZeJyjDJ",
  "qejvRyH2yKn1YEok",
  "UrxyymbTfII7/t2R",
  "TFL/CTSKoijKaLEi",
  "EU0utsPMmTN5bl1h",
  "hRVc56JQV527kMvl",
  "2KYQTyY2RBrtY5sy",
  "ZQr/Jo78m4j2oDqg",
  "GQzNHxHR70t5ge3i",
  "lsfl/k5hkTeSw3Bs",
  "hbUNET1Z5UNTFEVR",
  "msjQ/Oyzz9jIWn55",
  "0wa92lHDWhua+Xze",
  "NzCBXQAFR5YowsC7",
  "WaLUE0Txz6WQ0+jF",
  "QOuOxMgsPDEbIVlZ",
  "TuwS22ChIEgNTUVR",
  "FKWiyByL+ajcyvNy",
  "DcVqO9zyXi5mMekj",
  "AI1rpLiVUBwEOZxn",
  "iOgxCjGNnByxTKl9",
  "QuvdmCyG5IbIqmqX",
  "XXZxeZlKHCmKoigV",
  "BXPR9Ol1qT1eNWKe",
  "ZxOG5xJLLNFQ/dAb",
  "1dBc0vvxnQ2lRjYy",
  "bdc9NhidTz7p5KjU",
  "ynNFURSlFKYEhc5R",
  "dd4M864rGcuDid9k",
  "3rx59Pzzz7u+fA0i",
  "+gmFmEY0NMd5RqZz",
  "kqxUYzc6dnigq6vL",
  "5SXt1TweRVEUpSF5",
  "KmgHiJWroTlAobD7",
  "1772NSqBKzzbJ5Q0",
  "oqEJI3Mv150XW2wx",
  "18KYunfLw9AsMRdG",
  "rwKKoihKqawZtMNt",
  "t902KhXn9ULOq0q3",
  "81aTSXSDrv8QeqMZ",
  "mqcS0QGuO++xxx40",
  "f/58agYkN7MR2n0p",
  "iqIooWatoB3efffd",
  "iii6NAIR63ew52lU",
  "om+5pXMG4AFhDaE3",
  "krzRN4joUded0V5y",
  "xRVXpI6ODuru7g40",
  "wMp18dc6RAB5I3hu",
  "c7mMf1I7/u3Vq6ko",
  "iqKUQjbIkSUpa5Vw",
  "foS96jxXwneENxNz",
  "dSqV8kXeSzw+uEWd",
  "y9ZHg0bxaMZKdRvD",
  "yMSJLq7qRkeKgcJg",
  "9CqKoigNTdTV+Gog",
  "Z1dFiHrzdF9fn//b",
  "jBuH5obOnEwho1EM",
  "zau8yquSZARw69oC",
  "S0TcR7rVGngypcmP",
  "hs8VRVGUWmJrSw4X",
  "YRut+bXa7x/1FF9c",
  "Nvwmcl8kCRcu7Kaz",
  "z4ZkphPneBHe0NAI",
  "oXPkZTr/BeCOhpSA",
  "uKddja5ad/YpFzlh",
  "4YoXNHSuKIqiVIG8",
  "a91AYRc+G1cjr9ad",
  "hapNzjNT8nnnwuVJ",
  "LgVZo0V9W09Eu5Vi",
  "ZCIf0z6xxcisdyOy",
  "FEroCqQoiqIoDW3E",
  "1dO8PWaMcwgdEd4T",
  "KCTUs4W1bCl5mZtu",
  "uin19/f7rnrbyGyG",
  "E10Sim2tLkVRFEWp",
  "Bc0w71byt4rH49TZ",
  "2cmFzI6cD9uUQkA9",
  "h87/7Xk0nbDbL9p5",
  "EK7V1/Xu9ZQe5+LJ",
  "xX3HfqpqkSqKoigV",
  "D53LvCxzkYbOh0MK",
  "eY29UoL291+J6DCq",
  "MfVqPV1YipEpMgq4",
  "bWlp8b2ZqDhvlq5A",
  "wDaqMTAdW1xpv3NF",
  "URTFlbZS5qN6d+KM",
  "Brlcjuds2Cy4v8UW",
  "W7i+9FAi+i7VmHr8",
  "C+NH+4XrziussMKg",
  "cHmhBzMsVeGjQaE4",
  "LlqAOaCGpqIoiuJK",
  "4JwhTg5tIuIO7BgY",
  "mohGPvPMM3XVMaje",
  "DM2lS/nRNttsM/r0",
  "00/5DyOhYrvqGjpV",
  "eNwMhmahQY37//3v",
  "f11eqoamoiiK4krg",
  "nHHPPfc0VY1EpVi0",
  "aBEb5iLP6MhSRHQM",
  "1ZB6MzSv8n60QI48",
  "8kh64YUX+A+CP4x4",
  "NQtzG5ppRWV/dxjd",
  "U6ZMcXnZitU8JkVR",
  "FKWh2DBoh5dffpnT",
  "1lxrJIJ0NqtN7XWy",
  "c74Ottgs2I499ljX",
  "N7iMiEpqnN6sxUDH",
  "E9EFLjvCwNx88835",
  "BBDrv4Tk2SGp9zwS",
  "u4+qsPTSS9PMmTOD",
  "XjrL8yQriqIoShDv",
  "E9GXiu2wxhpr0Acf",
  "fMBzNDxzEmkcrhio",
  "XJ3NalPLFpZ5988+",
  "yatEH3XqxdBcgog+",
  "d91ZTj7kM8BzV4nv",
  "2IiGZmtrK/d5D6AX",
  "u1b36BRFUZQGYS4R",
  "LVZshwkTJtDcuXN9",
  "QzOo6lwNzdyw/7b6",
  "6qvTpEnQZ3diVSL6",
  "mEaZerGenPMy11ln",
  "Hf8ExCoJxmZYTsaw",
  "AV1RB2rmblcURVHq",
  "jo6gHdD6WQw0R5k9",
  "ZRjefx8O5HAXBtWD",
  "oQkZo71ddkSC8dtv",
  "v82eOvFAwpiqE69t",
  "VRlKqN3xd6mHc0RR",
  "FEUJB4kKOTmUEroe",
  "OrIjEW1Mo0w9hM6n",
  "eV2AAhE3vMgYweDE",
  "ykn6mjdz6HyopOoS",
  "clfVJawoiqJUvM+5",
  "PQ9p6HxoXAqW77rr",
  "37Trrk7y4i8S0SY0",
  "ioTd0FyXiN5w2REV",
  "bDAoZaWUTCbZyp83",
  "b57maBZgt99UQ1NR",
  "FEWphaEp3YHU0Czf",
  "0IzHo5ROOzvUvkJE",
  "b9IoEXbryUnD8Ygj",
  "juATVjQxRdIIfUHD",
  "ciIqiqIoijLYcCw3",
  "2qiQ18YzRwceuH8o",
  "czXD7tH8BxEdFLQT",
  "vJe2EPtICDJI691g",
  "tXu826tEx79/fX95",
  "RVEUZbQInFREf1KE",
  "xyUSGeSxDPJ4NqtH",
  "M+alDJZouB9CRNfT",
  "KFD3Hs3111+/aQTX",
  "y6XWg1FRFEVRbOFx",
  "rTovHzEu8ZvutNNO",
  "ofNqhtmjiY40k4vt",
  "gPaS6GVeiWKfZvBo",
  "ioC9/V0cjfT6/vKK",
  "oihK6DyaMhfJPKQe",
  "zaEJmqch4yhyjrgt",
  "4XgQMb6BmtijOTFo",
  "h+nTp9Muu+yiOR5l",
  "GJ6KoiiKUgskfK6U",
  "B37DtrY2P4Vwjz32",
  "CJVXs649mjA04dX8",
  "xje+QV1dXWV9WLN4",
  "NO0OQVp1riiKotTC",
  "owkkRzOoBWXQ42b3",
  "aLa0tPBtb2+vH+Et",
  "4Zi+SUSPURUJ81Ji",
  "SpChueyyy/L27rvv",
  "1r0hWAvwm02dOtXV",
  "6FcURVGUiqEezcrQ",
  "19fHRiYKo2Fk4jdF",
  "WmFYvJph/ws/FbTD",
  "8ssvz5b75MlFbVLF",
  "o9AgnzFjRkXSGBRF",
  "UZSmB9rXzqiDqDLA",
  "BpLuQMjThMGJaK8j",
  "axDRUlRF6t7QBGK5",
  "/+xnP6v28TQUOsgV",
  "RVGU0da+tkPCWmNR",
  "GZB+AJkoVPHDwwlW",
  "XXVV15efR01uaOZc",
  "jc1LL72UQ+nSKcjV",
  "qJLcxXI2DJhiW5hy",
  "NOU+TsjPPvvM5eXL",
  "VP8IFUVRlGYwNCVc",
  "Ls1V6h27in6orVzw",
  "OxXbgBiXtj72xx9/",
  "7PoRhxLREmUf6HDH",
  "T+EGLZKOLuUF06ZN",
  "8w0q+QNLomwzM9zJ",
  "XuskakVRFKV5DE3H",
  "ugClQmy66aY1z9UM",
  "u6EJ/lSqev1zzz03",
  "qOqq0LvZrKhRqSiK",
  "olQJFI2uVGyHmTNn",
  "cl2ArX6iKVzV5YUX",
  "XnDddW8iGtushmbJ",
  "rZJgwa+++upsYOIk",
  "Rs/zZkcHs6IoilJr",
  "YGwqo8uxxx7ruutJ",
  "zWxolmxsTpo0SVtb",
  "FfFoykpy1qxZLi9d",
  "sprHpSiKojRfVE09",
  "mqPDZZdd5rrrKdXw",
  "ataToVmysXndddfp",
  "ieyhv4GiKIpSS+3r",
  "iRMnsnND07hGnx/+",
  "8Ieuu+7S7IamGJtX",
  "u+z4/e9/nyZMmFD9",
  "I6pT1PhUFEVRRlOS",
  "cO7cuYMe6zw0OsDx",
  "VquioHo0NMmrRHeK",
  "+c6ePVtXT0UGc+Gg",
  "Hwa11hVFUZSy0ZqJ",
  "0QcSSLCDzj//fJfd",
  "ETpfr6KfT/VJvhTZ",
  "o8MP/wHfit6UbXSl",
  "UikaDWqts4l8Vft7",
  "N4p+maIoilI/8kb2",
  "PCROoOH0Jl11rMNO",
  "1NO7rLS+ZuH7D7fF",
  "Ygk29045BSmYThij",
  "qVLHR/XLHUR0kcuO",
  "11xzLd8OZVhBTb8e",
  "TtRqgBN9+vTpLrsa",
  "FXxFURRFGaG8EXSu",
  "HeccpYLAxoEKTwld",
  "mI6pZFFQPRua4Hgi",
  "cvrl7rzzTsu6j/GP",
  "jlv8AcTT2egMtYLS",
  "/BhFURRlNIqBlltu",
  "OVq0aNHoHZHiIyo8",
  "q6yyCjniXD0URCNY",
  "WE4h9N122813XcOq",
  "lx6rzeLNtA1K+75j",
  "vkxHdY5KURRFaSA+",
  "DNphzpw5o3Mkio8d",
  "zZ08uehawOZC9Luh",
  "CtAIhuY1RHSby47f",
  "/e536yanoxrY+TD2",
  "Y0VRFEWpAIGay5A3",
  "GipPU6ke+L0RwRXO",
  "PPPMivaubwZD09mr",
  "efPNN/sntYTLpRqr",
  "GRAjW3rBA8cwxphq",
  "H5ui1EkO2ooB/+7y",
  "nKI0Kq1BO/T09GjK",
  "1ihjz/m4Peecc1xf",
  "umclPr9RmoDP9lon",
  "nRe0YzKZpL6+Pt+V",
  "3EyV14WdgUT+yQGV",
  "N1IakTZvxY5tHPSk",
  "recxYSasiXNJq9AB",
  "saeF3r+DtJc4v5L3",
  "b88TUb/3vvLcU9b2",
  "Zg2+q6KMBi1BO/T2",
  "9o7OkSg+SBOEUy2R",
  "SHABNAzPa6+9lg4/",
  "/HByKAo6zbvejZhI",
  "A3nzcKFfELTT3Xff",
  "7edr2hZ+EEErsHJ/",
  "x2oXJNkpA3ChS/XZ",
  "JptsQs8/j3mxKC9i",
  "16oeoKJUb+Lb0tq2",
  "wnqz1gdFRG8Q0X8t",
  "43NqrQ9IUSrAdGvB",
  "NiTLLrssh88xB0lI",
  "17UaunAerhfPaGSY",
  "dLXRsr/gT4OTrb+/",
  "1z+etrY21xqNnYno",
  "vnI+v5EMTfAAEe0Y",
  "tJPkKhh3coQr0OUP",
  "MBxBJ3TYT3gpggL2",
  "wF5jjTXovffeC3r5",
  "B0S0evWPUlEqxhhP",
  "C+4Sqg8meylA99f6",
  "QBSlDOZ70YFhGT9+",
  "PC1YsMB39kDLGlFG",
  "YNsjQ2lNDve43u2Y",
  "fJWPXyK3orSDW1Sh",
  "O0Z0HySincr5/EbJ",
  "0bSrpAJBW0qc5FJ9",
  "3qxhc7BwoZNHvGJ6",
  "WooyCpzghXrqxcgk",
  "L8R+n7dYdtYfUZSQ",
  "EdgBpb+//wti7Ep1",
  "Qcgc2DYPbKDTTz/d",
  "5eU7Bi0ems3QDOyz",
  "Kj0/9eQerK3VJLm8",
  "SmOzDBH9i4ic+qyF",
  "FFzUXyOig2p9IIoy",
  "AiRveViQIwhKSV1T",
  "ykPSFERLXOQdzz33",
  "XNe3+GY5n99ohmav",
  "580oyq677uonxzb7",
  "ie743cOdF6AopnvV",
  "1US0BzVG2P8fRPSL",
  "Wh+IolTaphCPWtjT",
  "zRqJiJeGIB5k8XDa",
  "kkcBXFXO5zeaoYkL",
  "9FsuO1ar52g96mkq",
  "SgMYmbgQ7kaNlwqk",
  "xqZSTzhNKjr3jD4w",
  "8MXQlA6JiGjut99+",
  "Li9fCiUdzWhorkdE",
  "PyKiG7yKzayXl+VU",
  "HTVmzBjfq9kMJ/1w",
  "nstm9uYqDcHEBjUy",
  "bWPza7U+CEWpFM3s",
  "5KkV4kWGJxO/ObRM",
  "5blbbrnF9W12bRRD",
  "cy0igrDTZUR0DxG9",
  "g45VRISStHzBhjym",
  "3xPRAUS0fKnfZa21",
  "1vL/AM1SEDSUR9NR",
  "ViJs54miCKNmZM6c",
  "OZO3Yv/u8twIKCts",
  "pShhwi4C0mKg0QHe",
  "S8z38GBKCF3yNfF4",
  "6tSprovewBzcWhV5",
  "JIfQsQsUda02LS0t",
  "nsxP3lnDq5GQlaSj",
  "ke2cyKEoo8iJRLR7",
  "OW8ADdl77rmHO2TN",
  "mDGDn+vq6mJRaRQt",
  "YOUPZDIUGRa7qAFe",
  "Aki0gKWWWora29tp",
  "4sSJrBeIBe1Xv/pV",
  "WmGFFco5zPVdG1Io",
  "StgpNC7V0By9YiDM",
  "97heibSRGJzbb/9N",
  "eu+9SS5vBRvuP2Ez",
  "NI/xvJMhNrayflvG",
  "cggy2KotyB4kCDuU",
  "PpmcaA6oR1MJG0uU",
  "anjdddddtO+++w4y",
  "FkcT0bDDuNtggw3o",
  "lVdeKeXlvyOiP7o0",
  "pVCUMGO3QR5KF1Oe",
  "a6Z+6JEqN4SR98Dn",
  "yAJZPhfPv//++65v",
  "MyJDs1oGxGJEdFOY",
  "jUxc9KUlUzPj6NFU",
  "Q1MJG1eVOt533333",
  "mhmZhRp2r776Kl/k",
  "HRPxhUurdWyKUis0",
  "T7P2xGJRbknpwNkj",
  "6axWjc5Aoa8AhWdj",
  "r732GpSjWe7JHvQ7",
  "htWjiRQCCQ8Wod9F",
  "iFdRRolxXgeSQJB7",
  "tOKKK1JYQBs4eBRs",
  "b85WW21FTzzxRCnf",
  "vay+w4pSRQINCtFz",
  "lDEg1c/2v9u3xZRS",
  "GqUzUBDlfr+g18di",
  "EU75mT9/oaum5mOl",
  "fH60CuGs0BuZe+65",
  "JxtZOLklR6GZ0PwY",
  "pc5B+MaJ1VZbjcKA",
  "TIiFDRJw7XnyySfp",
  "7rvvdn2rRtAJVRqT",
  "Nped1IMZPjKZXCnR",
  "Hufrr1BpCyu0RuZ2",
  "223HKycxMsXAQui8",
  "GarOhxvcKtiu1CEn",
  "u+wUj8cH5SPVEtsj",
  "Y3thpBBxjz2c7Uet",
  "QFfCypYuxXdAHRzh",
  "ApcktAaVv0+YDc3j",
  "iGhvCgH4sbbZZhuu",
  "BEWoChf25557jm/h",
  "HoZxKZ5MaTDfrOiA",
  "V+qMsS4XusMPP5yN",
  "OBibYUHChIJEUuT2",
  "2GOPdfUabVitY1SU",
  "Mggcl1B40DknnDma",
  "YNttt3XZfQfvOjzq",
  "hmYrEV1MNTAojzji",
  "CM7BgvGIiziMy69/",
  "/ev0zDPP+BNNa2sr",
  "38cG6RK5sEupf7OF",
  "zgUNYSh1iNNq+rrr",
  "rvMXmWE4z6XiHF4D",
  "iaBIQaLkif/xjygq",
  "p6p4FBRlFDDi1EV4",
  "7733+FZ1NMMXOsef",
  "ob8fXbwrfw2q1HL/",
  "yEq8CRL3H3zwQXrr",
  "rbfogw8+oI8//pg+",
  "++wz1rhDbpNoQUkh",
  "C7CNRJlQpL0SwmbY",
  "7P3lYg/jE0UweFyY",
  "N9WoiJSBjcobKXWG",
  "0wUOC0yMc2D0cmur",
  "lSvjDMciqTuFYX1c",
  "h7B43nTTTYPebh8i",
  "urx6R6soI2JC0A5z",
  "5qD/ihI24vGoZ2w6",
  "G/07unZh5Pen8oH4",
  "+iWuhiQEjLu7u7/w",
  "by5eBzu3aahqM9Go",
  "A0MZj/ZnQJAZj3Fb",
  "CWrtNQnKM7V1y0Zw",
  "rLV3CSmKwchFFGG3",
  "3QanidfayLTHZ7Fj",
  "wb8hV3P69OkuxnYH",
  "EXVW9igVpSwCtQLF",
  "8QPEm2nPXbaHczhN",
  "TRv1hlYGGJkgEonR",
  "/vvvTzfdBHXKQI30",
  "4/EnHS1P1b6uO668",
  "8souUjrKKKLFQEod",
  "sRQRreGSByYGnUip",
  "hB14YHGsJbSs1PC5",
  "EjaQQleUwk5boJlr",
  "JMJ2/cHfpYTe587X",
  "oEpcgZ2qIBGq1nwM",
  "RVHKwOnCVthxpNbR",
  "BhdEaq2E66MamkrY",
  "WDJoh88//3zQY0lz",
  "U2oLvMqSM17CNWiP",
  "0TI0N3TRzrr33nv9",
  "fCklXD1m1fBX6ohA",
  "4+qFF17gC6Yddqsn",
  "+TJ4FnbddVeXXdXQ",
  "VOrSo1kYGtc5qPaI",
  "rrikHJ5xxhkuLzti",
  "tDoDHeuSnzlu3Dgu",
  "6AHDfV49eB3Kodrf",
  "L+jvOFQBlXREcpyI",
  "G/sPpNQDuAoWdX+g",
  "kOall14alI8chhxN",
  "FzAu5Zgd9T+1S5AS",
  "JrqCHE+QF5QaDVFi",
  "KCzWFVxyNJuFfJU7",
  "A9kGP247Ojp8my2A",
  "TYjoxWp6NJ2LgCAp",
  "1MxGZhhQT6ZS53wt",
  "yMgEr7322qCGDGGR",
  "NwpCjtHuh+6AejWV",
  "MBHYplgim7Y3s14W",
  "go1Mzrpmgs7Ozope",
  "g8oxNJ0+YLnllquL",
  "ZHxFUUKN0/XGnrRK",
  "8NaHyptZwvVSDU0l",
  "TAQuBNWoDCfQHIam",
  "uI1j+DwchiaqKMVF",
  "riiKUq3rzV133TXo",
  "sYih1wM4TkzCuFYi",
  "T2q//fZzeZkamkpY",
  "WHckkTVQDxGHRifr",
  "LQBsvd+rr766InJz",
  "5eZoziWixYrtcPfd",
  "d3NvcZxI5YqiV/tk",
  "DHr/cg3l0RpMIxFk",
  "V4mjULCltSHvZQoR",
  "/ZeInvK2WdS8YKkd",
  "WE04YcIEmjsXl6X6",
  "QzyacsHHY0fvz5gm",
  "1tNsKRg3W8E54/1b",
  "umD8YFNtvepxlIsC",
  "jcyD8J7h/JZW0LIg",
  "LJwn1Qh1Yyhdcfux",
  "S46mXINgq0n1ueM1",
  "aCtvfA3//iM0oNYh",
  "ojeDdkLiL6rM4Jbt",
  "6+ujclBD0w01NOuu",
  "Z/cvHT1Tk4joMm9A",
  "v07NxXZE9FjQTiVK",
  "c4QWyV1z/C7fJqIH",
  "qLlAEdRhrjUCFr8h",
  "oke8MeRUbaU48w8i",
  "OihoJ0kPsbt1QWlB",
  "HFFqaNbG0BzqNSUs",
  "djGH/a4aoXOnkA0q",
  "J6XlmqIozMZQ4SGi",
  "BVD+KiH8CaHy36Pe",
  "xXvtgdQ8OOtnNopo",
  "cgk4xdgbhCWI6FYi",
  "mj8CIxOc4i1Y4B3/",
  "aRWOr1lZzUVTEZ0B",
  "C7sCAa3hqD22Sofo",
  "msJuu+666ypyfR7p",
  "X/i3LieVrFIcpToU",
  "pRkMzBe8++V6Q/9J",
  "RFdScxB4ITv2WCit",
  "1Te4XtpyLgcccIDL",
  "yw71QsiNzs+h901E",
  "e1fo/dArHnIoB1fo",
  "/ZoZjM/2YjvMmjWL",
  "ZsyYMcjDNpwXThl9",
  "JHQuCwAsesEvfvGL",
  "mhma6wTlZoLtttuO",
  "UqmUVpkpzc43iOi9",
  "ChmYhfyYiKahvTc1",
  "LrhGfStop2uuuYYa",
  "AXgTkGqE21tvhfOO",
  "mr0oCEUmtxHRRVV4",
  "b2g+/s3L3YTBrlS5",
  "MFgQQ1PljcKBRJ5t",
  "pQ7cX7hwoWsqi+RG",
  "V8zQdDqp4NHs7e3l",
  "+5pnoTQh63rFCI+6",
  "9Ocug2WJ6N9BOTJ1",
  "jNP1RkSg6xUJHyL6",
  "gw3ezRIm4EY1NI/w",
  "UkWcKlvLAB7hv3i5",
  "rqtU+bMaEafzD17N",
  "4TrUKbVFDP+2tjY/",
  "Z1aKgypxDozE0LzU",
  "RWYEwqywktEVSA1N",
  "pYlAFfCJRPTGKBsA",
  "JxHRHa4yI3XEjtRE",
  "vYYLJ97jjjvO5eUQ",
  "vGu0RDd4Gq8e5e+F",
  "c+0jb/wq7nmzaxXb",
  "Yc6cOXz75ptvDmqg",
  "oPJG4cFWu5CwOaiV",
  "obmxSz6Q5EtBwgDV",
  "5loMpDQJJ3gtAc+r",
  "0efv6XmAnHvQ1gEw",
  "oIty7bXXUqNy1VWB",
  "ijGN6NX8W41zJ8/z",
  "0l2Q9qIUZ7DK9xDA",
  "OwbZsQ8++IAfFxqa",
  "ssBSam9sooujXVOD",
  "5yoh3F6qvJFTb3N7",
  "ZS4CxEGUu6op9/XV",
  "dt+rvFFD811PQ24p",
  "Cg/XE9EhVP9tJ58J",
  "2mns2LGufXnrEsfx",
  "+StPvqeeWdebsJyt",
  "61HgQSI6mog+rvWB",
  "hJTlkSlXbAcUAcFT",
  "ttNOO9Grr77qF56I",
  "F82eG1XeKHz2QWtr",
  "q99GvAjIk2ytlEdz",
  "S1cJA0kshZFpu2KV",
  "uqXRQrKVzMO8vVJG",
  "5i677EITJ06kzTbb",
  "jJ5//vly3upgzzNU",
  "zzh56RrZyASO58HW",
  "1Bj5mFeN9DfChCih",
  "WRSinn/++ZU4Lg2n",
  "F+fTICMc1zOxDQRb",
  "XUGNyXADLXQHWorN",
  "gaV4NDtwTQ/aCSfV",
  "559/7rdTwy0STIOS",
  "9dWjGXqPJlb1fxz5",
  "kTUcTt59F3bYYQd6",
  "9FHUDBkK9eWWW245",
  "mjIFjYKazrPZ5VUG",
  "DwuMiZNOCoyu1zVj",
  "xoxxrf4c72msNk2o",
  "/PDDD6c///nPfocZ",
  "ACeHFDLgeRignZ0V",
  "aZ40y7sOIhdaGXyN",
  "+V7QTlBTQFgWfxuJ",
  "dIqhqZ2BKLT2AXBM",
  "f9xruLERrYaEQWFb",
  "Ka0qawgaKQes3FZr",
  "+UoYmTvuuCNfSB97",
  "7DG/EtMeKxg/2OAJ",
  "QL5zk3k2NwwyMsF5",
  "59UqHXb0KMFju2Wd",
  "ejJHZGTCawlBaRgw",
  "gghOi2wOxg88Mnhu",
  "hRVWKPdYl/KiFxd6",
  "RX+KoWj7QWGo7j9q",
  "SNYHaCdezvWn4oam",
  "jUyaqEBXwosd0miw",
  "SayStHsr97Lzxzbf",
  "fHOe+J5++mm+lckR",
  "xiQ2LNJkkwsxJkyk",
  "oHz5y19uFmPT6XxD",
  "kYFOVj4XU/2lnlxV",
  "6rUKm3jEMEZkfhHH",
  "htzK2JIF26effsqP",
  "l10WimBlARXrhV7x",
  "n+JoaBZKGg21uFbC",
  "ya9//WvXnufVNzRF",
  "ikNCFjoB1AcPPvgg",
  "ffbZZ0G7rUREK1Jz",
  "cgwRdbqEh4qx6667",
  "8uT3v//9j2+RYA2v",
  "i2yinygXXzFAYWBi",
  "osQ+kyZNGmnOc70Z",
  "m+gE49zSrtHZbz+n",
  "TpNrhKwgzSUnM1rq",
  "gnjFFVf0vZaFMjkY",
  "JzJ+RDLKbq2HDdc6",
  "jCHH37QY5zdBwwQX",
  "3nbZyR6rI+nBrdQO",
  "FHE5sH4lDM1hrVXh",
  "iiuu4FsZ6PDOyEBX",
  "wgvy/5Zeeumg3SZj",
  "V2ou0AHrJiK6rJw3",
  "ueeee3hiwy3AeJBc",
  "pULNMrvXtYwbqc6U",
  "TltihMJD4+iNrjdj",
  "cwVvYVOULbbYYlBu",
  "XiNz221ojuNEWYuh",
  "MGtkwkCEkWnn+sGY",
  "xLiw2+eJQSndTuR5",
  "7CP7YhzdcsstlTh3",
  "Gr1hgguBDghcqwRZ",
  "BAAtBqoPHJtHJIbr",
  "fhctIV/KaV9MnDKo",
  "ZXORN1Jqx/Tp0ysW",
  "HmkgjkdkFs6kkb4B",
  "Lq5YbO22227+QJU+",
  "smJkSvhPxo0UMgD8",
  "m4wd/Ds0aUUaBCDn",
  "7LXXXhuJsRl2nU2n",
  "TjDTpk1rGq+I5Oo6",
  "cGExmZF6zclEjtim",
  "m27K9+1CU/H2S5qJ",
  "GJz4t8J5B/+OolTb",
  "yJHxiHSWMjmpiXun",
  "rxm0w/LLQwWJvjBe",
  "1dCsD0qw4bYsx9B0",
  "ypeS8J6cTJgY7cqy",
  "YlsQ5b6+0Rku18Xl",
  "93EsNijay7SBONCr",
  "3L2gnDdZe+21aaWV",
  "VhrkoQS4D28Mxoqd",
  "W2YPZPHC2K+Rf7fv",
  "Szgefb5LNDavDrGx",
  "2eJaaCV5q83SEGKv",
  "vZw7Me5ADZCTCQ8m",
  "ZIvQaQ6LNRkTkmoC",
  "MBbslBPJxxTs+4W5",
  "nPb74XMwVg88EMO/",
  "7N7pUOdopknJyT4o",
  "TKeTa2OzjN8wUGg3",
  "2bUAQfbVcceZRjw1",
  "NTQrpFmm1ABpERbA",
  "BGpsMAleSUT/hAZ4",
  "OTqYCM8hl3K0OOec",
  "c7g7zgiMzVMpfPzU",
  "Zaf111/f1+ltBvBd",
  "X3zxRdfdwyR4LsS8",
  "XGfnWDUMQ/TH3mOP",
  "Pap7ZJ4RhHF74403",
  "VqJTzZEQYPEaOTQD",
  "zV4o2vBEIkQ33ogs",
  "spGlWFbM0LzssrLS",
  "2BQlDIUJPy7nTTo6",
  "OuiRRx7xvSri4R8N",
  "j/tvf/tbuuiii0o1",
  "Ns/xQq1hYRWvwCKQ",
  "119/3feQNEOOpuQe",
  "OrbbXDZkIVz0w8Ys",
  "9cNSXvTwww+zx340",
  "wHkkqSlyXiEqUSEp",
  "pEYn0D4os/mEUmOi",
  "0Qjrozuw5JCvd0zM",
  "N9L+AfqZSn1i9zYt",
  "woiFHBuxMMFm/Pjx",
  "7A1B/hcmKzvPsjAM",
  "Xs2/IYrxLr744lKN",
  "zV+EJNS3pKsnDqFU",
  "YBeANAPw3v7sZz9z",
  "3R2/Zdk6PhUAcwes",
  "471LedFRRx1FP/jB",
  "D76QelJtj6acU/jc",
  "9957j29feOGFct4a",
  "4+tGT0y/EdksKK3q",
  "7bffVvugzslmMSZG",
  "3qo6Wsn8C6U+mT9/",
  "vstujXihHLFYNLjg",
  "ggvYy4SuLZIjJt7L",
  "0fa2oegIn3fppZeO",
  "xLN5pOd9qVVu33jP",
  "MEK7v0D23XdfvpU2",
  "t82Qoy3hXMd2cJIv",
  "WGtjc1lvEbO7y87o",
  "iY3zFmlYV199NZ/T",
  "o6UqIMV2UmhkyyKh",
  "UKiMhglg/5CmM1SC",
  "GUE7TJgwgZtSKPVN",
  "CeNwhaoYmo6q8UpI",
  "cWxvN+K8xUYRixYw",
  "GcKLeeKJJ/p6fnaB",
  "gXgxJdF6NJAJElx+",
  "+eV09tlnl/oWexLR",
  "Q0R0BtXGyHSqdEHo",
  "2M7LRLpCM2CfX1/5",
  "yldcXwZ9x5u9c70W",
  "RuZVpWhM4u960003",
  "0cknn+wv0mDgjUax",
  "CMaPfJ7ocdoatjg2",
  "3C/htx/K2HTKP64z",
  "pnjSd8MC6bw333xz",
  "9I5IqTiYxjIZJ4mj",
  "IW1Gl1nw+0E7nHpq",
  "GGsKFFccvSRhlkwZ",
  "Cd8ZSbj89NNP52ry",
  "BQsW+IYkeinb8iqY",
  "oETjb7SKVWCASFUt",
  "7qP/8/77Y24rmTM9",
  "g3O0dErh8XI+0COP",
  "hPPVgN9YUhUaHVtb",
  "tcRJGxf9N7wQ7mix",
  "rmfgliRkDm88Fm92",
  "OoRjWk9FwHkkFex2",
  "nrWA3/7jjz8ux4N+",
  "eY2M/mqTdtFqVuoX",
  "k8KCKN75VTE0N/bk",
  "RoryzjvvuHy4ElJ6",
  "e3tddktR44CLfUlN",
  "siG2DqFoVHgLUvUM",
  "Q10MTSkAKtSKGy3E",
  "g4rPhyD17rs7RS0L",
  "2WEUihi+RkT/KUWn",
  "FIazXWAlhkEFqoTr",
  "BvmuEC4vkQvL1YV1",
  "5FjPsC2pEvlrX/sa",
  "5xfbDT6wiBgtJNWl",
  "cKwWjmOMdXg2cR8L",
  "zhHQiCH0xV1axSr1",
  "zw03QJSl9MrzSEBu",
  "5bEumnZ2O7CREvY8",
  "q7B/v6DjK/bvbW1t",
  "3A4xgG6v33cj8N9S",
  "JkJM6iXmPIaKTTbZ",
  "ZKRVn2hHczQRza7Q",
  "obR4v/vvvXaJDXWN",
  "qCYi7i+GJm6PPvpo",
  "9gKOgMne3/VRqAhV",
  "6BAneEbUPqW+EL3H",
  "kZ9Zr6ClrIjJO4LV",
  "n6loawx6ghxS7e3t",
  "bKQXjuFmHtNhIsh+",
  "kFSSsWPHuiwa0K55",
  "TCmG5m1BuVOoAIXX",
  "pNwTJuwnXCMbmqi2",
  "dPBq9jeIV3OsJ8ge",
  "2HUGv9lqq63GXrN6",
  "L3ZbffXVy9H2PNGr",
  "zHdK5i0g4RmX2EpO",
  "HBWkGrhZkU444jkX",
  "wXKEJNEhqgzO9rp+",
  "YXOuNCpgP8/IRCpE",
  "SbS0tIyq5zIk4wut",
  "2JajxgH5BUUnOMl1",
  "VUMznARdW+W6Iw1H",
  "gt6uMFoeZGh+5umB",
  "DcsyyyzDHRzU0CyO",
  "Gpqhys2816V9pEzg",
  "MsnXO2gDV6Zn9inL",
  "MClmlIyxjMtTqEyk",
  "t3Uzg+uHeDOlxSKe",
  "Q/FKBQ01/F3/4N1O",
  "dViw7eIZmCMqFBQj",
  "U/KZ6xn8LRAZ6uyE",
  "M8eJ1YjoI2oM8q4p",
  "PWpohpNS7AfHsRop",
  "xdB0OoEqoXUW9hOu",
  "kQ1NTFZSSFKEdIO0",
  "ofwNEf0yaCepQMXv",
  "0ggToYReYbRVyDDB",
  "D4JKJ7vaKe55MCty",
  "ssMohmg2/gbNbmja",
  "uqzyWO5X8G9aCC4K",
  "8wsWFbgGxIIcEK4e",
  "aixyG8GjKYtRCMyL",
  "zmsAhxDR9dQYqKHZ",
  "BKFzmUccr8WD/rBl",
  "a69or9L6x9FbByOi",
  "EQjMzZQCmsKeyPWM",
  "yLPAaMPkft1115Ur",
  "RB31jI42a8Pjis0c",
  "G220Eed1NbuRCewK",
  "aFE1EIUD/G0xYVdB",
  "Zi7pGZQrWdvEcoxM",
  "nHOyiBNvbCN0dpKw",
  "cAl/g0Zp21hyrrVS",
  "fyD6gPmjBAZdI4qN",
  "8HVc37HEA1BChqMh",
  "1ShLzyF7sdo88MAD",
  "/JuINFEjGJpimOCC",
  "AWMTnVcgih1WFl98",
  "cW55NlqC3WEHRgwW",
  "CEBksyRBX4yc3Xbb",
  "bdRaNo4EGGFbbLGF",
  "/zdtJLUA8cxhLvzy",
  "l79cketQnaDtJ5uA",
  "nKcNXUIK2aDzIlrO",
  "CST5XurRrG8awZCq",
  "JOJBw2TYKIso0dmU",
  "FAkYKvBqltnPuSog",
  "FIx2nsh5a4S0hWr8",
  "LeENlPCzqH7A2Ln3",
  "3nvL7WJTFXbZZRfa",
  "c889/b8lxhjywkdL",
  "kL3a4DvId3n33Xdd",
  "XrIWNQaBdgJE+JX6",
  "BvOGyPahc9eoGppn",
  "nXUW3zbChUJpCr7u",
  "svoWw1ta0TUC8p3s",
  "Puy4RaWseMpqzXbb",
  "bedrk+LCBkNEry0D",
  "fz+EmcXAFCUEWxFB",
  "cuXFY7jKKqtQGEBD",
  "g/vuu2+QR0RucfyN",
  "kqcnBVpNtnDfykWD",
  "WKlv7HbKd9xxR8nn",
  "RbScE+jRRx91Xj2L",
  "GO5wW9gJOv5afz8J",
  "jQ73ubU+vpAQeE4/",
  "/vjjfAtPJiaORpoI",
  "BTHeJO9PCp5q1Up2",
  "s802499YfnsxliRc",
  "owyA87Ewd9j+ewLx",
  "PHzyySf8u66//vo1",
  "O158PgzkYrn8jfI3",
  "tmWnmohA1frJkycP",
  "O8/YXaCUMIPoSaSU",
  "zmRfdjU0kfRdFIjs",
  "aqK+UkcExrTWWmst",
  "PxQpk2OjeDWLAUMT",
  "OX74rmUWCTmBylzk",
  "i+LCpTlc1eX111/3",
  "5XdGC+jPyrhptIXa",
  "UEiKR5PlE0PGLNDT",
  "5KBootQBslgYScvq",
  "YqMi6XoCNcNErDRM",
  "R6CiwNgST5pMkM2w",
  "4pZJEt8VXU5wf4Tt",
  "K4cFHtOOjg7/vREi",
  "D2M+YaOCVARp8Yh0",
  "CeRNVhqE6/H+H33U",
  "KBKRbsg1QyJLDjTC",
  "RcWpcl7tg/pH5kJp",
  "/evyEvtBWcsvO9dL",
  "UeoAp4a7MpjsBP9G",
  "x87vA7iF11GMkmuv",
  "vXZE77vDDjvwRINU",
  "BBjxWA1LnqGkJuj1",
  "Y3SQv6/kwd5///2+",
  "l+KrX/0qF4eNBJwn",
  "YmBJgagtMN9MEzHO",
  "6YkTA4OB4HNqAkPz",
  "z3/+c1Ms1JuF/Aj/",
  "lsMJtm8MybOgF9vS",
  "GoEf1AThk1oynPfN",
  "9cRwNKYaoZvF3KBW",
  "edLtyu5K0+gXS7uI",
  "wW7CIOeFfX7ZGo4w",
  "IG3jRX4rea5wBWyn",
  "JYiB2YR5baNOYXcr",
  "8V7L31wE/QX83XDu",
  "jx8/not55G+JfEu8",
  "D/od2xEtyU+U80aK",
  "ByRftJl+48MPP5yu",
  "ueaaoN1vJ6K9qb55",
  "MijvHQV+Tz6J3Yqj",
  "9kFtCR6jJjUkEjFj",
  "2/Ga7f9Ro+VoY9ne",
  "D6W+cWxNuGUzhM8P",
  "OOAA/z4m32Y4v8U4",
  "gMEhhoN4pcSYxH0Y",
  "H2JU4mKDDVJEXV1d",
  "gwwV8VqKVxjvYUsV",
  "SQWyGCdKdcHvjL8B",
  "NjH+8TcQ+S787eTv",
  "C8QIhZYpiorQUx3h",
  "8GnTpvFzWFDY5wPA",
  "39lesMjYQS5uoyNj",
  "Bpx++ukuL0Gbz3rn",
  "S0E74JxphohQMxFx",
  "v14vVbahefHFFw/y",
  "Wij1zUUXXdQshmbg",
  "Bf6SSy7xPXbNcpGU",
  "7ytGpG1IFlapi8Ei",
  "+4nBYV8PJCQOIwP3",
  "xQgVb6mGy0cX8WDK",
  "30+80rKwkPu2coXd",
  "jajwvQDODfs8gYdT",
  "3l/eA/siN7TRsb3D",
  "K6ywQrMYmuOCdpgz",
  "Z47aBw2ELCRLdUxF",
  "y5WBaQZvTzPw8MMP",
  "N0s3C6cLvIT9QDPk",
  "mhWGvoebHLAPwqdi",
  "OIqhASPDliTC74d/",
  "g5Eh74V/E8NVJ5/R",
  "RTzIdlch+++BDX9X",
  "kSIqlEYTz7MsImQf",
  "O01CPkfOCVuPttGR",
  "c7+9vd1ld1jeL1L9",
  "E+iqluuCUt/YDkX8",
  "PW+88caKGJoTXFYq",
  "di7XcHpYhatje189",
  "AStHub8vtM4cWJXq",
  "n2fhjAnaaa+99vK9",
  "Ps2QQ1j4He1zp1Cn",
  "cTjs887WwBwqb9j+",
  "vEpcB+RvVa2tXB3d",
  "oPevNvZvbN8vzD+2",
  "tTkLW7AWXlPsx8Od",
  "G2LMNguXXnqpy25P",
  "UGMQeOJqsV9jgD9h",
  "LDaQb3/77UgxdndM",
  "DVcMlC/VwrWfL9wP",
  "FBYVDLe/MjLKnazh",
  "dXDURG2EP9hfiOjQ",
  "oJ3EE9Ms4fN6ptqT",
  "WbnjK+g6p+dYY+B4",
  "npxGROdSffM1Inom",
  "aCe7PWox1A4I93kr",
  "+ftSALj44ovT7Nmz",
  "g952HnbFnWi5OT9K",
  "Y9BkE90tLjvpxU8Z",
  "LbRzV/1TgiB+I+Rn",
  "Bubro/GDXRym1C+I",
  "Skj0BX9PFH864OeR",
  "RCthmOiFsP5pMkPz",
  "KdeJowm7fSiKUiKY",
  "Ay+//HLX3ZvC0Lzw",
  "wgvZQGmGHN1mIOul",
  "wEietgPxYobmZkGv",
  "tlvG6YpbqUMWEdGD",
  "QTudeuqpfKsXSkVR",
  "igEvzw9+8AOXXc9C",
  "DR3VP4GFoY888gjf",
  "avevxikWteXpXF5W",
  "zND8StCr33jjjdKO",
  "Ugk9MKYWLlzosutK",
  "1BgEqgifeOKJpYjT",
  "KorSpJTgbGkEb2Zr",
  "UNMLgPmksFGAUp/Y",
  "2rulvKyYoYkk36I8",
  "+ywKdw0i5KzU/4kk",
  "K9Am0NIsKXyuOUZK",
  "tdEczfpm+eWXbyZD",
  "02kOEMF+u5GDUp+I",
  "coZci0pNJxtq79WD",
  "XvTBBx/4F0CdhBsD",
  "XAwcDc1G0NJ0vuBD",
  "rkQnekVRivHMM4EF",
  "2ADig42gXr9lqY0g",
  "lPpHGjG4qAh84bUj",
  "aSs1adIkvhWNwWKr",
  "bldDtNaTebXlS0ZD",
  "fqDc17///vsuhxJ4",
  "ftQJSDb5DRGdUmwn",
  "9C0+4ogjRu+ompSw",
  "F1zV+vpU7d+nyYoB",
  "y/r721E8CSc6ejQb",
  "wZvpZGjut99+fGvr",
  "bMtjm1qPK8X97yCF",
  "XbbGrivRkYi1z507",
  "VwXXG/BE+/TTT112",
  "XY4aB6d2SLoiVxRF",
  "Ol5J3qHkHkJT0JFG",
  "MTQDo1p33nkn/1bD",
  "tTFVmouhDM3AEjGU",
  "tktyKFCDszHAAsKB",
  "pahxcLrwb7jhhtU/",
  "EkVRQo8dvRMv8/33",
  "3+/yUlRavk71zzou",
  "NgI8XqK7qFXnjUE5",
  "dt5QhmZJvmzb4FTq",
  "F/wNofTvcDJh+R7u",
  "OKc78P9fErTTFVdc",
  "MTpHoyhKqL2ZUuAi",
  "xiZuN910U5e3+C01",
  "Sdh86tSpfutRdUIp",
  "NFKDoTAnU0+m+kfC",
  "w/Pnz3fZfTw1kVfT",
  "cSJRFKWBsfMMcR/G",
  "VJN1A3IyNM855xy+",
  "FcO8r69vNI5LGUVK",
  "tflGZGja+ZlqZDZW",
  "iynHPM0VqHFwmgCS",
  "yWT1j0RRlFBiezAl",
  "ZI7b4447rtkMzQOC",
  "dnj44YcH/VaoVlbq",
  "n5FUmw9naDplNRcK",
  "d2rlWGOA1eeMGTNc",
  "dp1IjcMsCCkE7XT0",
  "0UePztEoihI6xLiE",
  "0SSpYogCnX322S4v",
  "P40aA2hsB1ZGTpky",
  "hW8lzSDsihJK9YmW",
  "Wk121113DbnKU+of",
  "XDgdDc1lqbG4yUVP",
  "U1GU5sSW6ZFq8xIk",
  "XhrFm+mkn4nfSDQX",
  "RQJRaW4KfdpLBr0A",
  "BSOFemIuxUAqERNu",
  "5IIwc+bMZqs8d54I",
  "7AunrNK1RaUb6tVo",
  "7r9PUIpV2FOwMMfZ",
  "TUrwe6255prNZmju",
  "FrTD+eef7+ev4jdq",
  "aWmh3t5G0KhvbnLe",
  "3xOMpFFPtNw+59oi",
  "rTGQE+ezzz5z2X1p",
  "aiycJoKxY8f6RqXk",
  "q0jDAkVRGhc7XC76",
  "kA899JDLS//iqVvU",
  "O2NcPJrnnXee36oQ",
  "10gYmepkahxGqp8e",
  "LTchVA3NxkAuoLNm",
  "IWWx6TyaPUR0b9BO",
  "6BI0VMqInv/aq1tp",
  "bMRYQrhcjE7tBjR8",
  "MxeRNhpJFxklfJR7",
  "DR+xvJGgHYIaA7kw",
  "NKlH02lCuPDCCwed",
  "88O1VVMUpbGQdBlJ",
  "nylBhLypDE3baSFq",
  "HWofNAblzHPRUquJ",
  "USxif6C2mGosHHM0",
  "l6HG4ynXi6ic/5K3",
  "oue/ojQ+kp8Nw2np",
  "pZeumKJFoxiau+66",
  "K9/K9RG/E7yZuhCv",
  "f8qVsyw0NNcKesG7",
  "77476LFOso2BrNQ/",
  "+eQTl5MJbcgarbpj",
  "qstO7e3tfr/jkSZG",
  "K4pSf8DIlPlun332",
  "cXnJE9Q4BCrSoBUn",
  "qs3ld8I1UsLnSv1T",
  "jqRlobGwXNALpk2b",
  "9oWQoa5Y6h/xTHd1",
  "dfk6aAGsSI2FU2ho",
  "8cUXH1QQ1Cw6cUE5",
  "mJqjqTQDCJnjfL74",
  "4oubKWwOx4LTIBZZ",
  "I/mt5PdS6ptCZ0q5",
  "hiYqy4qyaNEivyBI",
  "Dc3GARcIyUF6++23",
  "XV7yZWpCQ3O11Vbj",
  "Wzt8rlWVitLYyGKy",
  "RA/dU810bcQCXKSM",
  "cH3E/cLmLkp9U6nQ",
  "eWCvKKkgs4shGsE1",
  "HnaPTbnH53L8EvIo",
  "TI8YhsA0i0a7mKJQ",
  "at111/V/KzEww6Cj",
  "We75iYm02Fbr81+p",
  "b8J+fQ1C5jiMdcd2",
  "tGkiepma5Nr4/PPP",
  "+9cKcUQVag4r4cUu",
  "cB1qi8eTlM0aNQH5",
  "G5dCoWFZ+xGt1AQs",
  "IMSr+d5777m8xFmt",
  "uA5YjIiMBTkM8+fP",
  "51sxNG3PRiMstBRF",
  "KY5MsBtssIHL7q9R",
  "4xAo1H7AAYEt0JU6",
  "ZkDWMuI3LyiFEXW7",
  "1z7njQe8c8ingcHZ",
  "hKHzeUT0ZjFjc/z4",
  "8Xy744471mWluY5T",
  "RalM6HynnXZyecl/",
  "qTFYD70qgnaaPHny",
  "6ByNUhOkMYkUvzp6",
  "qX1DsWSftk5YjXsi",
  "pdNpNjQdPZqNFjp3",
  "yqdaYYUVfCPTDi0r",
  "itK4SAQDG5QnHHAS",
  "JG6k/uaKUjFDc9C7",
  "6MnVUOAiivA5+tnP",
  "mTMnaPclvK1RGFHi",
  "vuQiKYrSuDRxY5Jf",
  "Be1w3XXXjc6RKDXF",
  "LgKHneBApqwWlIWe",
  "TfVyNg4tLS1824QF",
  "QTA0F7jsKMUx4uEI",
  "QzGQoijVpQnnuVWJ",
  "aNmgnc4999zRORol",
  "FOc+5r+2tjbX1s7m",
  "NeUamoW9z5X6RMR1",
  "u7u72Xh65513mi1P",
  "E0lGR7vsKKHzevJy",
  "hL2qV1HCThPK+TmF",
  "zadOdep1odQx4lyR",
  "PvZjxgQqYYJF/utL",
  "/UD1aDYm0k5RvHOT",
  "Jjl1TluDGosbXXe0",
  "tTPrxdhUFGXkyDzX",
  "2dnpsrtTIme9G5ow",
  "MuutMFIZGbadt/ba",
  "a7u8ZNKIq86l4taO",
  "12vosL6qJm3sx7Zs",
  "QRNWnpcEfjcYmzj3",
  "ka8i+rK1Qhd7Sj1T",
  "7vk7Gos9WYw3SVoR",
  "cqh+GLSTqHAE/f6a",
  "x17fYJ5DuLyvr6eU",
  "Fqy+8kKk4AQJHK2Y",
  "XGXA2WFzPdHCzXB/",
  "J3lc+Pwqq6xCH374",
  "YdDbfoJdqXFYx5M5",
  "cp4UYWRKrmYt0fGl",
  "NDJB88toRRUw/y29",
  "9NLcijmAGS75jSFm",
  "eyJ62CWn3yVPXRfC",
  "4SZo/EQiMa+PvVGm",
  "cRxvOxDRI7hT9uyk",
  "J1BjAl00hxDRyi5t",
  "S+uI4113lJ6+UhSk",
  "KErjA4MKqhwOTCSi",
  "cdTg+ZlAF7nNQV9f",
  "X6nRa1/JZURniCSF",
  "Ko0LjKeXX3bqoLYh",
  "NQYdRHSoq5SHhMrV",
  "yFSU5kDysvv7+11f",
  "ciTVLzsH7XD33Xez",
  "4VHC76HUOSUWj/aW",
  "XQxkf7DSmDSZoXmF",
  "y06//vWvKZVK+ee9",
  "XRSkKErjIkLtjhqC",
  "4Lw69WrimDcO2mn/",
  "/fcfUd9rpX47A43U",
  "sTJijybQE6yxcTQ0",
  "N6L654cu3kwwZcoU",
  "HnQibaQeTUVpDiSV",
  "CNEMFME4chXVHye6",
  "7NTT0+OnECmNTbTM",
  "DngjfqUtc6QnWmPy",
  "4osvNoOhuZTrZHDc",
  "ccexkYmJBoNOCoEU",
  "RWlsxJuTSCT48cMP",
  "B9bJCAcQ0U+ofkDO",
  "/SkuO8pCWx1OzUHW",
  "y89MJpMuuw/Kp9As",
  "XmVYoKXpUBC0Vh0X",
  "BLV7RqZT/PvKK6/k",
  "WzEw1aOpKM2BGFMS",
  "QsTjEoTKr/CKg+qB",
  "w1122n777fk62Nvb",
  "q8VATUDcSxfBuQ9F",
  "Ggem2A9GlKNpezPt",
  "gTfUysbV4ymvH25T",
  "ymMkfx8YUa+88orL",
  "23+V6hMYmd912fH8",
  "88/3C4DkVvVjw9PZ",
  "KOybUv/XfwkT43ig",
  "KbjqqujQWNK1BtGT",
  "MNNKRBe77Pif//yH",
  "r4Pi4dXzv7HHXzrd",
  "R7FYhBcVO+20U0ka",
  "mkCXIkpRXnrppUYN",
  "n19IRN9z3fmkk06q",
  "7tEoihJapCmDeO+Q",
  "n4gJ+Pnnn3d9i93r",
  "wNjc0nXRDScECiHV",
  "iGwOol6jHmyXXHJJ",
  "SdJG/PpSP7DwxArL",
  "alOpDg1aEHQGEf3C",
  "decJEyZU92gURQk1",
  "ducvqbTG7ZZbOstN",
  "khc9+SMRLU3hxOQG",
  "BfDLX/7S730tEU2l",
  "8cmXZuuVZ2gqzUWD",
  "eTTXJaKHiOhM1xdc",
  "cMEFNHfu3OoelaIo",
  "oQbGFIwrSZfB43Q6",
  "zduYMSWlqO/peTaX",
  "oHABT+saQTvZeanD",
  "dZVTGo+89zcuIR/X",
  "73M+VAtKjKKi72QX",
  "Qki+CvI0MOD4DQtW",
  "N/I4aNUT3AJJV03V",
  "ZLiiFpxY8+fPp44O",
  "6JkXZTEimk/h5Qjv",
  "Al/S4grnnfQ0DzON",
  "npBf7+NfJ+Pq/j7V",
  "/n1lfIlurq08gY4p",
  "EC/fZZddSnnL24jo",
  "aCJyajM0CpyAqHjQ",
  "TshNRdoAwHfHbwDB",
  "dtUTrm9yDkWtGGP4",
  "m4utF8CgC3bh7DQQ",
  "HxiGEsRqlQY5AV99",
  "9VWXXUuKIY0yfyOi",
  "q0s1MrGAwoU17Eam",
  "oiijMxHbha+ipwtj",
  "a9dddy31LfcmomuI",
  "aBmqPWNcjEyA7yxh",
  "c3x3O6VAaWyi7lqa",
  "X7BEC19llipFaGlp",
  "aQgPg1JxPc0tQ+zJ",
  "PLjUF40fP54HVXd3",
  "d8N7CxVFCUYMK7sQ",
  "Bt48qawewXViDy9n",
  "s9bSR05FkZttthl/",
  "f/muUhyi3szmIBKJ",
  "+CoD5Rqa3UHvAA+P",
  "oOGg5uCFF15w2W1b",
  "CmdOZsmdOdZaay3q",
  "6urii6qKsiuKYrda",
  "xPVAvJm2kYnHaE9b",
  "IqhGv8W7VtUC2AC/",
  "d9kRFfa25I2kFakd",
  "0BxEIhG/DWsAfUGG",
  "ZmC+yJJLLsm3cqKB",
  "SoQWVYerthT77R99",
  "9FGXi8lmIaumbPEq",
  "y0tyM0CI+P333/eN",
  "Sw0NjQ6NPv5Vx7M4",
  "4g3ErXjI7PsSqgVi",
  "3NhzULWR60HholO8",
  "nLhFxxR4OEdgbCIa",
  "9AYR/ZRGH6fE0uuu",
  "u84PmdvYBUHl6Ggr",
  "tdXJDLp+tLa2sjdz",
  "2WWXdfm4TwufKJyE",
  "P3c1NOXggZ5Ijc3n",
  "n39Or732Wr2Ez+EZ",
  "OIqIbnLtXy5e2/33",
  "35+efPJJnkzEk6kX",
  "SkWpPuIhxK1I5ki7",
  "10IDz9azlLFaa3Ac",
  "MDIBbiXFrEQuJ6Ib",
  "kblDo4dTxOeoo3BJ",
  "VZqV7u5uHnfXXIO0",
  "4tLE2ocyNN8Meod1",
  "162Vh1+pJkErmkce",
  "ecTlbban2udjvuZd",
  "PBGScgKSHRdddBHd",
  "csstgzyYmMCwaWhI",
  "UaoP0rLgNREPixiT",
  "eA5eTITt5HnbAxqW",
  "hSCOEd5MeH9Qie4Y",
  "ZixkfyL66yjlbaK1",
  "UaCL6q677tL0IYUw",
  "N2688cYla2gOZWhO",
  "D3qHoVynOhE3Pmg5",
  "FnJDs+TK8lmzZrGR",
  "ecIJJ9DNN9/sS3WJ",
  "XJdo5YVlIlOURkVk",
  "UyQNC7d2RAFjE321",
  "pQDFFgoPQ3qLXRwE",
  "+R98H9w6yMIV6yJU",
  "bWMTle+BHHxwybWU",
  "SgOSd7fzAg3NL7g8",
  "C9l22y/WfKih2fg8",
  "/fTTfKEP4EtEtBrV",
  "xsgs+WoIr8Pxxx9P",
  "t912G3tGkGMloTvJ",
  "XREPi6Io1UX0+TAO",
  "YbjJog/jVLRs8W8A",
  "YxLPhaXqGdcPMXjt",
  "wiAUFY4wjL67V5Hu",
  "lBQ3AsYS0XkuO4pu",
  "ptLctLa2uuyGQTyg",
  "6j+MoflM0Ltsuumm",
  "fKtenuais7OTnnkm",
  "8PQA36Q6kC8Chx9+",
  "ON166608gWGywkQh",
  "94Ek+SuKUl3sXEzc",
  "l/EoYuDi6YQxCsNN",
  "/j0scxGOS1JtJL8b",
  "BrLkmo4wj3S3Kno2",
  "93XZCalyYTDkldqC",
  "c8AxbXLIYo4RCQTa",
  "1X44ANUZbPwcTfzN",
  "Q5inOSL5IrDNNtvQ",
  "ww8/zPdFlF3Oa7vY",
  "QHTjFEWpHhhjMMbg",
  "wZTIAsYj9Gw32mgj",
  "OvbYY+naa6/lgj3p",
  "TCeVsGEInYuuJI5d",
  "Qv7SSUWKnMo0Nitd",
  "IOR03XznnXdcO8Eo",
  "DUwul6O993bKtBgy",
  "Kl62hThCoVqlziih",
  "IGgHGj1Q5V7yybf0",
  "0kvTf/87MB7gNREh",
  "2kLJFExmamgqyugY",
  "ahiHGI8Yh9/5zne4",
  "QA+RFFQ9oygFaS6y",
  "+EVINwxGJpDcUanO",
  "tZUrYGTie0lXnd12",
  "g+04opzNSgE1jkCr",
  "F4a9pCcozU0kEqED",
  "DjhgRPmZ/PohQoP5",
  "UsRrAVZxQaseNUbr",
  "H/wNp0+fTksssUTQ",
  "rpugoVCVD2cVz02P",
  "9mnOFJ67YQOTk0ye",
  "ossnx1x43PZ3sXPW",
  "Cu/Xw9hTY766hPmc",
  "B3Ke4pzH+X/qqafS",
  "SSedxOHnp556ivbd",
  "d1824iRv2p5vEErH",
  "89hXqtDl3B/KmAUY",
  "H7Yep3gdC7HD9uUg",
  "Y1COCVEUpCOVyM88",
  "CaRyQdJlYOLoUGLs",
  "heNUx21jjO+I9Xe0",
  "5xT7OcfPgJb2rMIn",
  "wz8DKaEBF+nHHnvM",
  "Zde9qnwoE70VfqCR",
  "+dlnn3FlObwhtuhz",
  "GMFkZPcSlvZ2dss7",
  "/BsqWVFoIAUThV5Y",
  "uV9Pgt1KcwMDU85l",
  "eC+PO+44Pr+RQy0e",
  "QNvIxDktRUMwTCXF",
  "xTYYxbiTHEMZ+zK+",
  "pMpdxg8WebjFZ9jy",
  "SdUARvMIch8vIyIn",
  "fZkiIJ/dqTopzNdK",
  "pTrYf/PCyJ7Ly4cy",
  "Mvm9hjiZMEqLji4M",
  "SHsw24nZw1EPXhUl",
  "mB/84Ad09dVQESoK",
  "TrY1iWh+FQ4BK6ar",
  "XXQyYWTCWENXi7PO",
  "OovCjEw60N5buHCh",
  "73GRIgMzueJCgPFn",
  "PJmYEHt7YYzilQO5",
  "tIUeFNuTE1bUGK4u",
  "YTcaZDGFggM0TcA5",
  "f8kll9BvfvMbXwxd",
  "xgOwc8dlLGC/5Zdf",
  "ntZcc02OuiyzzDKc",
  "44nKb1wLFi1aRJ98",
  "8glNnjyZ5s6d679O",
  "FnR25yFbrqgSv12h",
  "R9N+vsTwPyJFkH7p",
  "GuGh4HUDfaSLNGaZ",
  "M2fOF55Xj2bjejTz",
  "w7wHnl977bXprbfe",
  "Cnqbd4lo7aH+Yag8",
  "DZz1RkPCwdDUE625",
  "cMzTXIqItiKiuyv8",
  "8eNLEWPHJAEh9iuu",
  "uILCjhQQYFIEduhb",
  "xhiuA7iLf+vrS1M0",
  "itwvTL4Yi4MvEoNf",
  "F24jQ1HEW3/uuefS",
  "mDFjOHR+wQUX+J5L",
  "hMXFkymLJzHSFl98",
  "cdpzzz25UGiTTTbx",
  "dTjxOoTV8RqEqXEf",
  "tzNmzOAc7X/+85/c",
  "EUzGiRiDthOlkmOn",
  "MCRpRytKCM9v7F0D",
  "R6K0sZ+LkQmGMjKV",
  "xiVf5DzHefrXv6KH",
  "QCDDymMO5dHEGbZ4",
  "sXfDalFWhOrRbD5Q",
  "ibj66qsH7XYnEe1Z",
  "4Y++0euc4cSXv/xl",
  "evfdd/2QdJixdQJx",
  "H5qlEuJD8dImm2zE",
  "HhpMTMiTfe6553gM",
  "mlyv7rr3ONTLcdYr",
  "YV9swAhcbbXV6Pnn",
  "n6fTTjuNLrzwQj63",
  "JSxu54ghbQTjAOE8",
  "VKNDpgweONkP40by",
  "OfEekrspucuilIJz",
  "DvrAaKt3991383vC",
  "mC3s/VyJgqPCfOrC",
  "FBf8e4nV3ccT0UUl",
  "HsZcIlosaKeJEyey",
  "B3go6vX60ujkqzy+",
  "HefPQ4joeldD84Wg",
  "PBBoab700ku+F8Yu",
  "YBgONTQbh1//+tec",
  "qO/AikOJt46Qn5aS",
  "CA/vBipWxWsQduSC",
  "LWFyWy7lkEMOoXPP",
  "PZsnWFTaYqKcNm0a",
  "e2tvuukm3gchdLuT",
  "SmFXlbBPCGE/vnon",
  "7IYmzlcYl4stthin",
  "54gxKfOKHL8UyKE4",
  "CJEKpJpgvGB/LM5E",
  "QULyLCXvEvtIOBz3",
  "RcJMckNfe+01OuWU",
  "U+jRRx/1owli+FUi",
  "9WS46IL9PIxiiWg4",
  "As1ip6R5Ivo2Ed3n",
  "smOxuVoNzcYc3/ki",
  "c0QJiy00a/loyPcY",
  "4gAvJqLjir3b+eef",
  "TyeffLJ/cC6ufzU0",
  "Gwd4M99++22Xi8yJ",
  "RHRBhSrMhzyBhwLF",
  "PyuuuOKg864ejE27",
  "AMGuHse25lqr0U9/",
  "+lPaZ599fCFoPP+v",
  "f/2Ln8+kzarTDv8V",
  "Gp1hJuzHV++E3dDE",
  "efuzn/2M/vznP3N4",
  "2/YqwngUAxOGJcJ4",
  "e+yxB48DIF5IvAcW",
  "YsjD/Oijj2j+/PmD",
  "/g1GLHI4EWpHlACR",
  "OWnSgDGCIrsHH3yQ",
  "F9Gvv/46P4bxWgmP",
  "ZpChKc6ab3zjG67p",
  "SWASEW1BRLMd9v2E",
  "iFYK2mm55Zbj1ILh",
  "UEOzcQ3N4f6+GHuI",
  "EASQLpZyOZSh+V0i",
  "uj3oXQvd/kETuRqa",
  "jdf7fKutkIY5suTg",
  "EkBl+a1EtKPrC4aq",
  "tA77RCveFtv7Ivlb",
  "bHQSjEWi9dZbl0OL",
  "u+66K/dqR8jxjTfe",
  "oJ123GWQoSlVuNKZ",
  "JOwTQtiPr94J+/mP",
  "41tqqaW4YEdC4LLY",
  "ghEGI/MrX/kK/f3v",
  "f6dVV12V/008/DAI",
  "EXL/05/+xKoYMDDh",
  "GZS5SQxFvA/GCx4j",
  "DWWLLbagHXbYgXbc",
  "cUc2PCWPE/+O/FBs",
  "0n6x3NQbF48mGEGa",
  "z21EtE/APj8iot+7",
  "vFnQPK2GZmMbmoW2",
  "HUDxL6IMATxWrCvg",
  "UIYmCjmGTtCwkAnQ",
  "1i0rhhqajcXBBx/M",
  "3gdHUfWny/iofxDR",
  "QWHXyYxGjTgzEJkW",
  "2xMSiQ2u7MuLnp9X",
  "WBfJ5SkRT1AarklM",
  "ijHzfF6KfKwiCHzO",
  "t7/9bY4sYNIFN9xw",
  "A/3whz/0P088JBJt",
  "KHUCURSb0RhTIl0k",
  "hiWAEQkDEIva22+/",
  "ncaNG8feFeRn4ryG",
  "9w0eyDvuuCPw+CXN",
  "S4w5EVWHx2avvfai",
  "Qw89lL7+9a/7bWhR",
  "Zfv973+fc9IljC+F",
  "SUDGYjVUHUr8vX9O",
  "RJcM82+Qs3ByyeJa",
  "Am9wEIWGiNIYhmbU",
  "GxeFIv2O5/bZRHRG",
  "KYYmf27Qu9rJ1S4H",
  "ooZmY4EQFkLUY8eO",
  "DdoV1mjgcmgYjvG0",
  "45yw1RBGnwHNPjEo",
  "MSkBo9VnNPswVGIx",
  "8TSacYMhGIuYBZuv",
  "oZk2E21ba5uZdKPG",
  "0wlkwsT2y1/+kn7y",
  "k59wpS6Mz2effZa9",
  "MDIBuuRPy3sqSq0M",
  "TZlD4FFEuFoMQpz7",
  "6623Hj3++ON+e0fs",
  "M3PmTJYsQx42jFE8",
  "djn+QkF30c2Uzj4w",
  "NDGmMJbgGcW//+IX",
  "v+Bwve1tRD6laGGK",
  "FmclgUGNzy+B5Yho",
  "+hDP/4mIBlagFZij",
  "1dAMH/kyzz+7baog",
  "hd6y6AvgW0RkejqX",
  "YGgGdg7AQMOEJv1b",
  "NUez+bjqqqsGedGG",
  "AWfp8kT0eYlvv7FX",
  "mOZs+DrkkVQNqWSV",
  "iacwR9J4T4zhh0kE",
  "oTvkiOG4MQY/nTqV",
  "PSWoJMeWyWSpo6Od",
  "32/+/IW88pNFnXTi",
  "Eg8QNM4uvfRS1gyE",
  "vAtCingvu7NQEDpp",
  "KLU0NMXogxGJcYz7",
  "OI+xoQXlyiuvzM/j",
  "HEeYHMVAKIhDtTnG",
  "QJBRZo+Bws5BMmal",
  "bzqeRzj9zDPPpPXX",
  "X5///fe//z2deOKJ",
  "gyIm0jAB4f5qsPXW",
  "W9MTTzzhuvtdnkE5",
  "q6DV5F9cXrzBBhtw",
  "XqoLamg2pkcz5mk5",
  "i2MC5zrGm6PUFWSz",
  "TJ5JCYbmO0S0lqt0",
  "DNDQefMB9QFMAg4c",
  "TkTXlfDWy3ganE5d",
  "MPbbbz/2bNQau2uJ",
  "HbLG4y232Iy22XZb",
  "+sZ229Eaa6zBBqa8",
  "Rjwl8ORgwvzwww85",
  "BxZFAZMmTeIJNp5o",
  "9UN2dihcxh0WflAD",
  "eOWVV+j6668f1KZP",
  "2voVQycNpdYeTRkr",
  "QOR+kJN54IEH8hjA",
  "mEGvc4x3GKQYB1Ls",
  "E4RLVzDZB5+DHE+E",
  "5/fee28655xz2KDF",
  "mDzooIN4nOJzccwY",
  "d3hcrWJDFEhhEenI",
  "HVZXtoleAaVTF6BS",
  "5mc1NBvToxm3FBpK",
  "PP/uJ6LvFNthOEMz",
  "0N0O/TKs8uxk7WKo",
  "odmYQBZknXXWCdrt",
  "Ga860pV/EdEeLjvC",
  "u7HZZptRGLDlVGAU",
  "wnOJSfHII4+kdb68",
  "hh/GljFn53fhFq+B",
  "B8cPz7W305NPPMHd",
  "UZ7877O+vqbITchn",
  "SV4NbuFhfuCBBzh3",
  "TSZOMX6LoZOGUmtD",
  "UzyKknLyne98hxeQ",
  "yNGE8Yf7hx12mF+J",
  "XorQ+VCGpl2II+PQ",
  "PgaMR4wxeHWOP/54",
  "OuaYYziHcZddduHu",
  "QlKwFLLfXvQ173Rt",
  "bIH0p1L6rquh2ZiG",
  "ZsxL+5K5Bc85jq9f",
  "EdFviu0wnPX3VNA7",
  "Q4ZBpCEcY/hKA/KX",
  "vzhFZr5ORIHWqMcJ",
  "rkYm2HzzzSkMYKKC",
  "B0SkWE444QQuJvjj",
  "H//IHsw8RSmTRWlP",
  "lCIoHMpH+DYWT/Jz",
  "eB28NBKC58VbXx9t",
  "tNFG3KcdXkqE2qWK",
  "VjqjiJEpk+Xf/vY3",
  "TmmRtnoyIStKmJH2",
  "ktL8A7fQiYUnEzmY",
  "EFb/3ve+N8hglGgA",
  "bl0Qj6nkQhduMrkO",
  "tHw1k+7s2bM5bxMG",
  "Jj4LXYUgLj9azhMs",
  "PkvgQiK60tXIxLWl",
  "WqF/pb6IeE4LoYRF",
  "VKC9OJxHE6WsHwa9",
  "2J7Agixq9Wg2Jggp",
  "YXUvXogiXODpahZj",
  "XSJ6w/WzsRIPy0Wy",
  "vb2Vurp6aP/996Wz",
  "zz6bVlllFTb4RFBa",
  "wuliKKKfOYBxygLt",
  "Xn4aJjnpZCJhDPay",
  "ZPP8XRG6Qwiv0EMj",
  "XlFseB88RvhPxqhL",
  "2FBRah06F68iCnBQ",
  "7IOJb8qUKVykI143",
  "8dJjPzEGR+pZtMeR",
  "jE+8N0LiMm4lzxn3",
  "MZ4RzkcUBceEgkg7",
  "SlEtzjvvPM4RrTQj",
  "8U6qR7NxPZpZy9CE",
  "3qxjfiYmmdxIDE3n",
  "ynPxqgSddGpoNi43",
  "33wzy4MEgDP2S0Q0",
  "P6BXKuSQAtl+++25",
  "i0dYWHHF5dl7CV0+",
  "8fTDiMS4gGfz0f88",
  "Th9//DHnX6JCFhqY",
  "GNyoFse+LYk4ffWr",
  "X6U111yTPaBbbrkl",
  "6wrCIMX7xBMp9u7A",
  "iEQf6EsuucQvaLBD",
  "5+KVkfFo920uhk4a",
  "SjGqbUiJNwULVhh5",
  "77//PhuSOK9h1OGx",
  "nOs4x235sFKUFQql",
  "+MTQlJC5KDpIhyF8",
  "voTQbekX5G2i+xjG",
  "O8b0aPDvf/+bdttt",
  "t4q9HwoSpdWkGprN",
  "PT4j3nkvzUCwQZfW",
  "QT/zLhfvuSkZHyES",
  "ssNEKIUKw6EelcYO",
  "nzsYmhO8E/JvRaSM",
  "nIzMa6+99gtGZpAh",
  "VYlcKvkMu9AHkxO6",
  "lEBTFBMTWkFiPHz4",
  "4bv8u6CHMrqUBE2E",
  "GORvvf2ubxzCU7zd",
  "dtvxZAZx9v7+Xmpp",
  "QavJbvrtb3/NHlRU",
  "xZpjGXyMEnrEpI1j",
  "gbFaD52RlOYF5y0M",
  "TIB8SCzAcP6iGKew",
  "C5mMJXsxJZ57WVjZ",
  "nkrbwLSvE7aBKt5L",
  "eU8poBNdT9HbRJQA",
  "oWy0q4Txi7xR5JIO",
  "1xu8kuy+++4VM/ih",
  "wVvKMev83fgRhbhn",
  "aOKzsDkYmU5h8yCP",
  "5lwiWqzYi5EzhklM",
  "wg3loCdq/YK/P4wp",
  "tHcLAH3P1yeieQXP",
  "I+5efKUScK5U29CU",
  "HC7xqkio4fTTT+dJ",
  "B+MAk+OTTz5J5557",
  "LsuS2NJfrlqzsh8M",
  "RITecYs81PPP/x1L",
  "kIjXBZPe7373O+4S",
  "BAO3v9+E72yZCsl3",
  "c/H46PhTajnRSV4k",
  "PufFF1/kNrdoQvDj",
  "H/94kEh60DHKGIIx",
  "iPMfUQC8XqJuMiZk",
  "nBQWO8j4E6NTxpQc",
  "o4T2RcAd3lbkkJbQ",
  "NrIiv1O5lNrhp1jn",
  "GJfXK+GXF7Pvyzhx",
  "rL94NminYrNzYDwA",
  "FwNQO5FsJQzgooti",
  "FQdWGKZdmnPyUa0K",
  "W0QLUyaZCRMm0K23",
  "3spFAjj/P/30U/Zs",
  "fOtb3+JKeBiIkkvm",
  "auRif0yQmExgZIpg",
  "NQohYGxC6UGKfPCe",
  "8PycfPLJgyYeHJ+E",
  "FyX8p8V6StjBOYrz",
  "HwU3SB1BXiYWcGLQ",
  "BSEFQWJM4v2wiSdS",
  "8DtzWRXjkgImr7f/",
  "XYr8MC5l0SYC7dhe",
  "eOGFUTMyAY4BEZ1y",
  "0DQ2pRBZVIlzZN11",
  "US4RSNbFyATFzrhA",
  "gUSs5tTIVADCxI6r",
  "LrSpWtx6/FsiOsfl",
  "hZiEahkCxkUe8kKQ",
  "c8LkAqMS3xmJ+qgO",
  "h0Eo+n7Yz3FFyIgQ",
  "NDw38h3l95TKcfSc",
  "RU4YJmH8Gz4DBRNo",
  "nSevkcpdkUqSoglF",
  "CTMwBjF2UASEEDq6",
  "XSFMLb3Hg7DVFcQI",
  "xOuw2dI9ItkihqRt",
  "yErDBTtFRjw7Ep4X",
  "w1XGlEt9QqVxaJIx",
  "LCuuuGLo+94rtQFj",
  "Tc6Na665xuUlV7u+",
  "dzSgMCNQ4gho/peC",
  "0DmkMhxYlogO8O7v",
  "QkQnu37GvffeS7VC",
  "Qudoh3fPPfewN3/6",
  "9Ok8BlAYIP3NEaqz",
  "20+6emHt/DEZT6Jr",
  "ZmubPf/8i5y7CU8K",
  "QnaoREfSNjyeuFDI",
  "66R7kGhtKkqYwTmP",
  "c3iLLbagyy+/3E89",
  "wYLJZfwUduTC+S9t",
  "LIFteMpYtsHr7H+X",
  "94MBKyF9WUBiw1iT",
  "9ru18BBKPmspQNMX",
  "VfKKUojIhMmiCx3m",
  "KpWfGZSjie4sM4Le",
  "QCa0cgebTob1Dc6j",
  "r3zlK/Tyyy+7nAto",
  "R7k2Eb3qtacMRFqe",
  "Dke1czS58jsep3/9",
  "61+0zTbb0J133sme",
  "RPGkSO6keBQxEeG+",
  "a9haJj6p/hMvyoCq",
  "Azww+B3Qoq+Xllpq",
  "CbrjjjtYYgXdhBYt",
  "6mIjGIaneHTsSVYF",
  "25WwFyOguA2C7Og6",
  "J0YdvPKuY8gO/UmE",
  "AHnTaNGKKATaWKLS",
  "WsLfyKtGQc8bb7zB",
  "Gx5LK0oxLO3iIFx/",
  "bK+pGKO1crQgPxtS",
  "akHAuESaTbHuaZqj",
  "GW7yozD+ZM6aOHGi",
  "64JkRa/uoixDE+RH",
  "K2dOT9T6Rs4jJPBj",
  "5ezA5V6leUW6/4xG",
  "MRBacUE0+owzzqAr",
  "r7zSD03Ld5eJydbl",
  "s3sqB72/9CYXeSKZ",
  "zKTYDr3SMxkUIRkv",
  "J4rxkK/17W9/m3p6",
  "+rjyHaFHMXZlhSpi",
  "2MXQ8afUuhgBShJX",
  "XHEFL+ZcFpc2skDD",
  "eQ5PP9Js0A8dRiuK",
  "FDEWxMAUTUwJm+Mz",
  "Pv/8c06HgUoEFst4",
  "DOQ12KChiX0lV1uM",
  "0VqGol0+26VFrxqa",
  "4SY/SlXnohvrwGQi",
  "Wtn1/YMMTXxi0QQv",
  "aZlXCR0npX6Rv/9a",
  "a61Fr7/+ekWLdsQ7",
  "EbRPNQ1NFOL86Ec/",
  "op///Od0//33+wUH",
  "OP/x3iKPYh+nXUUe",
  "hF0Ra2MXH4CxYzto",
  "4UKTc4Yh09rawjqm",
  "3/zmDjwJwtuK/ug4",
  "NnhhpbVl0HHo+FNq",
  "OdHB6wjDEIs4WXDB",
  "YMS565KjKYoQ//d/",
  "/8cFcquuuip7RCWF",
  "RdJZbC+lfCeMG4xj",
  "5HJivMydO5eNXXTZ",
  "evPNN/3PsFNZ7IKg",
  "UsZ5pUFRIjoXFcvJ",
  "dPFOqaEZbvKjoGMr",
  "ecuOn/Uzz1nkRFCM",
  "MzBPc6utttKTTPEv",
  "wO+++y7ddNNNFXvf",
  "TTfdtCKDTI5vuM02",
  "VO3iGYQTdtppJ57A",
  "4M2EkQnEyJQcLjGG",
  "xbMiHhMZG1LZamv1",
  "FR6fhN+lCAGIFIsc",
  "I8LmknsWiaAqPUPf",
  "/e7enDcKD9BJJ53k",
  "ezQx0eL45H2LbYpS",
  "S6DpCFkw8d5L1x8Z",
  "V3a1uH1fDExcJ9Ax",
  "66qrrmIPpow9aQlr",
  "SxnZ7wGwILPHPTqi",
  "/PSnP6X//e9/9Ne/",
  "/tWvwJVFn4xJGZcS",
  "PQAyxu3rSTXH13Cd",
  "WxAFwjG45mS6Xh/s",
  "+1pUVDny3vk03FZt",
  "ZOFUQgqkc34miJb7",
  "Zr/5zW+08lwZBPKG",
  "Sqm4HgqcU4cccggX",
  "vYwGds9kHDsmJzyH",
  "8DQ8JEceeSRPPCId",
  "JJp6dv6jDFK7w4hU",
  "09peACk2sC/keAxD",
  "EfvboXfJOSuUZZFN",
  "wnqoRH3llVdozz33",
  "pA033PALXU8UJcwg",
  "P9JO75CQtuhnisEp",
  "CzAZg3jNcccdRw89",
  "9BDrzMKwxDjCv8n4",
  "cbkWSeMRkTKS8Ybx",
  "9Oyzz3L+KD5L+o7b",
  "Ri7Gn0Q0JKQuoXpQ",
  "7YVc4fsjVI5UIzUE",
  "FVdkPoLj0IFuInrZ",
  "+c0dQuc7ENFDgW9S",
  "sIIbCepVaQzkfELu",
  "IBL7RwoMTHgpXAk6",
  "/1w7W4jYsyRFw8hE",
  "a0mEo6UiFYjcie2J",
  "tD9HxoTdplVWjYX7",
  "yoQouZRiQNpeFPsY",
  "h1rl4piXXXZZ9iZj",
  "0karOng0xbNaK/1R",
  "pTEYjRwxuymCLOIK",
  "ezDLfcldhuICumdJ",
  "EY8sqnDuFy4agz5f",
  "0mFEz1a8l3IfKUEo",
  "AITChujZSqtZGMWy",
  "SMQYxn3J5XTNM60l",
  "rqlHwwm96/xdHvka",
  "Lwpk7nE8jt8Q0a8q",
  "aWi2etZrUST3pRz0",
  "RG0M5HxChec777wz",
  "SOanlDAaqkNLoVxD",
  "U/bBxCCyQDDWkKMF",
  "IxOIF1PCcjLZySQn",
  "k5vt6bCxu43YizMx",
  "HEVkXbyrMtHaBq79",
  "XWyjVu5vvPHG7N1B",
  "RxVUxmOSk+NRlLAb",
  "mnIuy7gqXJiJMYr9",
  "kUe5/fbb+95OuxWl",
  "vWB0cYJg3NkGrByP",
  "3Jexic+BBxVFj3he",
  "xjyQ3FIgntRSCgJr",
  "iRqatSVf4/PDzjt2",
  "4DtEZHLIHAkagViK",
  "vRj0JltvvXUpn6k0",
  "AZ988gmLuJcKdCih",
  "pzfaSFhOJiZMEtAF",
  "/eCDD76Qf4X9JK9L",
  "JihMOIWhc9uLKIYl",
  "3guhdEyCMkHaxiuw",
  "W0hKCN4OlwtyPAL2",
  "f+2111jsGoUV4mXR",
  "1BalHpDJTs7zofIy",
  "xZhDFTUaJsjCTBZx",
  "eJ14MGUsuxiaGJOS",
  "0iLjXY4D74Pjwvvi",
  "MxDh+Pvf/87FStLq",
  "0pY4A+IJLSwQVJSw",
  "gsKxauRnung0AYS6",
  "Tiu2AxKO4cEqB10R",
  "NQb2+YSk/Pfee88X",
  "EncBLR3Rw7tUyvVo",
  "2tWjoidmh+pkEgR2",
  "kYJtxNmeQ/HErLLK",
  "KpwzueWWW3JrPfwm",
  "YsjCEEQYDgVUSN6H",
  "lh9aWUq7O/GuDmco",
  "Fnp6cOwSxkMxA4xk",
  "qZBXlLBXvRaqLhR6",
  "OXFuYyGKtBwU5kn3",
  "K+jIwgiEgDr+3S6m",
  "K9SUHQ7xPsr3lDEq",
  "1wGMbVwXRDcX/z5t",
  "2jQ66KCD6JlnnvG9",
  "nng9ZJBwHOrRVFzJ",
  "h+D8gHNohRXQJboo",
  "rxHRBtUwNL9FRA8G",
  "7VRueE5P1Mag8ByA",
  "9iSMHhfefvttTugP",
  "0nyshqEphQUw7mSS",
  "MlXe3YMqX7Gf3T8c",
  "Ex0mPAmV4d9gUELb",
  "cscdd2Sh6HHjxg1Z",
  "uWlXdOIzoN0Hj+R9",
  "993HYUGkEMBTOtRk",
  "NVT1p20o41hg3D71",
  "1FOhuIgp9c1onEOy",
  "mCscb3ZID/Ji6MQl",
  "RiEMTMgSQZgdYwUN",
  "CzDeMA5lzNjh96Ac",
  "TTF0pREJxrmMJ8lh",
  "g8ErEQyMf6SpQMNW",
  "PkeMTlF/kKhFmFFD",
  "s7bkQxA6dyzg/TER",
  "/aHk93f4guh11RW0",
  "E1aT6E07UvREbQwK",
  "z6ell16aPvzwQz/U",
  "XOx12267LT355JMj",
  "+txK5GjK5CKeFJms",
  "xADFe0i+pEwoYnTi",
  "tehDjup0tKXEY9kX",
  "r8dkZed2ynvZXhuR",
  "JBIvze23306//e1v",
  "2cspn2fnZcr3krC/",
  "THS2B7QwvK4oYZ0I",
  "xZixx7LkRuL8hkYs",
  "FmF2SFxUIIAYg/Ao",
  "2jlnLh59KRqS+4XH",
  "IVJJ4qW0CwFR5X7W",
  "WWexASzjWMZqvaSt",
  "qKHZ3IbmN7/5Tc7t",
  "d2B9Inq9GoYmgMbM",
  "xsV2QOEEuipITpnk",
  "2sj9oBNRT9TG5bzz",
  "zuOONcV44IEHaOed",
  "d67aMbiEziXsLN5J",
  "6dZjywhJGE88mTAg",
  "cdynnHIKi9XjefFC",
  "2vJFrhcS+QxMmDie",
  "BQsW0CWXXEIXX3wx",
  "Hwc+UzyXQ1W6F35f",
  "NTSVepgo8Xp4IrHA",
  "svMtAcYSFBXgnV9s",
  "scV4XMDoE61NMf5G",
  "6/sNdR/j/rrrrqMj",
  "jjjCnwNlITmUV9MW",
  "ea/E+Kz2GB/qGjMa",
  "n1sv1NpQdMUutJNx",
  "hsdI4XIImy8konE0",
  "Alw1if4dtMPvf/97",
  "f1Uknh4ZbHoyNjcX",
  "XXQRh7SKAaHlWoJz",
  "V3oZS/gOSBjf7oEM",
  "4xIGH/qMP/7441yY",
  "sNpqqw3ycEgRjhiE",
  "QZt8Dt7b9tJgUj31",
  "1FPpscce4zxoe0LV",
  "anKlUcB5jUWVGGUy",
  "/mQ8YKG13HLL8TmP",
  "cWrnP5ar2VsqhWkw",
  "2JBiA/1KhNBx7FKh",
  "Lt/NHreFC0WdH5Vq",
  "Y2u8FhqZOBcdjExw",
  "9Eg/P1qpKiMcqAwo",
  "e4ApyqxZs+hXvxpe",
  "dmvKlCkcEqslOFcx",
  "qdkC5xJ+k048El7D",
  "wISXFqEGFPtg4kPq",
  "CLBDZYWemWLY+oG2",
  "B0RC6+uttx4LRyM0",
  "L95XMBL5KEUJG5Lf",
  "bOdKitoCunIh51lC",
  "5pInLWHxMIwBHBfG",
  "JIxNqG1I5buE3O05",
  "Ub6jjOGw528q9U+0",
  "QH3Bbut8zTXXuL7N",
  "PSP9fNfQOdwrgRUa",
  "mHQnT548aFDJhBmU",
  "A6KrusYHbRLRzrGQ",
  "0047jTtMVZOg81wm",
  "N8mllBaT8FxKKAwT",
  "HRZU//znP1lMXjqF",
  "iFdFjEV7pSjeGReJ",
  "FRwDPCOYRO38Mkxa",
  "MGbxPP4dXhNU5+Pf",
  "RTh6uO9biWYKihJE",
  "uZ51OwdZcqVx7iOc",
  "jo5cGHcYizDO7Pxm",
  "WQSG4fvJGMUxIcoB",
  "cXc7tG9LqFU6fK6h",
  "89oS9shS3puHxHmC",
  "81TGkOOxQ+Zyk5F+",
  "vusIzbh4NW+++eZB",
  "bfPkQqAoOJl/8IMf",
  "cGW1DQylkehtVhq5",
  "YCK/EgMQVazSVk4m",
  "MkmYhpEpnhVbdN0+",
  "9yUv0xaRLgYmUfk8",
  "bJhsRTNQRKEl9xNV",
  "/Oeff75fEWtPAsWM",
  "TkUJKzhHMeawcJPi",
  "HzwHTVgYmRhX0v5R",
  "ws6FxTu1BMcjRUgY",
  "y/BsIh1IFq9iSItX",
  "Vq4P0hFMUaqJpITZ",
  "Sg7iQHHk9HI+v5Sl",
  "4D+CdrBbBkqitou0",
  "hNIczJw5k4466qhB",
  "z0EUfcaMGVRrZJUH",
  "MKEhp1SMTNxCLw/h",
  "/TXXXJP/TYTaZeKw",
  "vZB2dbrdq7zYJsVG",
  "dgs90cWUBZv0Ypbe",
  "5pB6EeNTUeoZ6bID",
  "I03OZ2jOHn300X5r",
  "SDEoZVxiTJQ4WVaM",
  "whxrjEkcu6Te4HgP",
  "PvhgOvfcc/kaIV5N",
  "mQuxj6Tj1EtlulK/",
  "RKwWy3Zq2NVXX101",
  "kfaRGppOHyShAvGi",
  "aJ9lxQZtERH6DUsR",
  "kGDnQ4pckWhSYrK7",
  "/PLLeT+c1+gIIh1E",
  "7MnR1u2T97JzlosB",
  "A1K8OdJVRIxV8eTg",
  "39DHHO+NMN0ZZ5zh",
  "V7kOt5hTb6ZSD8i4",
  "E28Lzn2c26gyL0xB",
  "kfEF/UzxfoYBpLVI",
  "yF/SaU466SReFNp6",
  "oHb+NQjL8SuNi0TL",
  "JHdYjM3vf//7Li9H",
  "Xlvxat4K5WgKc4lo",
  "sWI7oFgBMg92grOL",
  "R1O9no2NfZ7BUHvp",
  "pZd4kkCRy2gYQy6C",
  "7dK+DkaciDKjSxGk",
  "mSR8jYpyezLBZIdB",
  "i7CZGJz2uSzeyKDw",
  "ni2DJK/B+8mFQTqN",
  "YLJC3prcx+0hhxzC",
  "0mL2dy0Ud9c8TaWa",
  "lDuGbQMSYwjezBdf",
  "fNFvdmBXmUsfc0gh",
  "4VbypKvJcM0S7Mpx",
  "EXGXCl+RIsNxoyc7",
  "OghJKF0KhIbLfSwV",
  "zdGsLWFf0Oc8Tybm",
  "E8wl+LstueSSHGWs",
  "Rm/zcg1Np3aUq666",
  "6sAHFAhNjxQ9oeub",
  "wr8/+pmjCxD6Blfj",
  "/QsR7TBg50fZyfgy",
  "0Ul+JCrL4Y1wrRyv",
  "9vltd0gRw1FC7ugC",
  "9M477/ian5LfKfvq",
  "+FGqicv1XfYpNFoK",
  "xxcMMxQISsGbHe4r",
  "5/PLwfX9C/eTxxiL",
  "G220EX388cd+Eweb",
  "ao/Pen//WhN2Q9IF",
  "SWeUBVEJKRtjR9uj",
  "6dSOUnrB2u28yk3Y",
  "bvQTudEJw0Qg+VN2",
  "H3MpLJDzFR5LeDQh",
  "kn7MMceMqB1mtc7v",
  "oTwg4uFBv3QYm5I7",
  "CuS7ShheUcJqaNrP",
  "IVwObyZE2sVJUe+G",
  "Jq4vGKObbLKJ38oS",
  "jJbWdL2/f61pBEOz",
  "1cvvx/m3xBJLuHoz",
  "TySiC8r97FLjaU55",
  "miuttJI/edsSDopS",
  "ywsFjEj7Ai8GmYhC",
  "ixA7ch+Rl4n7LtJc",
  "lTq+Yhuwx5Q9SeMx",
  "vMMQtbYLikRyScPm",
  "StiRcxxOid13392f",
  "Q2wlh3oGC8DVV1+d",
  "Izi24awLQGU0kJxh",
  "WeSgCNeR/1Ti80ud",
  "gbqJ6LagnS699NJB",
  "PaMbfbWj1AfSUcTu",
  "8CPalRJqPvbYY+n0",
  "009nIxNINWmtGcpT",
  "YhugOPbDDz+c9t57",
  "70FdjEAYjl9RghC1",
  "BuT5d3V1+dXckgJS",
  "z+A6g3GISnQUOUme",
  "N8B1SFGqiehED6UQ",
  "VIReTz+zbEbi6gi0",
  "cNH3XAaRtqBUwgAM",
  "SduTIJp20u0HkwAk",
  "jCBHIiF0WQWGwSMv",
  "uWqFXk77MSZktPtc",
  "ZpllfG+trQChKGFF",
  "PPMbbrghe+dlvDaC",
  "NxPIfAgD+oorrhhU",
  "BCmFhopSTSTN6re/",
  "/a3rS0bccrIShqZT",
  "+Bw5AFKBG4aJWmlu",
  "YIRJZat4E6SnOW53",
  "2GEHvzAJRie8nNLF",
  "IwyhZ1sWZSjvpnQ0",
  "gpF58sknDxKCDkOL",
  "PkVxOb8PO+ywQTmb",
  "IAzjr1xEskzabN50",
  "002cM6dOGGU0EHkw",
  "cOKJSLsMBJ6Kv1bq",
  "80cygl8nooVBO8Ez",
  "BGRgKUotkUEm4QPp",
  "5IML/YorrkjXXnut",
  "n0+Mc1a6eUh4vdY5",
  "mjIh2Y9t/UwJ/0NM",
  "Hq3v4BWS1BUNnSth",
  "B+cxpIyw4AMyZ0A+",
  "rBHyGPH94M2U4sKV",
  "V16ZLrvsMv86pCjV",
  "RNRISogQVMybCUa6",
  "VDRWZBGQL2Z3RlGU",
  "WiKFBTgnoXkpnkwY",
  "krfffjtXuMpFX8Ll",
  "ot0XBo/KUPp19nHB",
  "mMQGjVJ8D3hnxZOp",
  "Hk0l7OB8/trXvsYG",
  "mLRtlVQXeP4aAXu8",
  "wuA88MADaZ999qnp",
  "MSnNQcJLGSsh3/mW",
  "Sn7+SGdQp/D5Kqus",
  "wgUYQd4aV2mMcjal",
  "uRFxcyn+EW8fWnCt",
  "v/76g3KJZXGEfV29",
  "DYUt6YbbCnE//9Gb",
  "FuEPfBZC6BBsh3cE",
  "38MUFOC4JUSy7rrr",
  "cvW8GNT259h9biWd",
  "QFGqiRiNdgGbnQ4C",
  "vv3tb/NEaLdpdB0f",
  "4q2RdBi7AYLkesri",
  "TFJnotE45SlKufyA",
  "1jNeL7ls8pw0R5Cx",
  "EovhuPAaonw2araY",
  "eS9e0EailIhEKRrJ",
  "U4SwYEWnsTjF4xBv",
  "h/g80niQr7mI/vjH",
  "P9Dyy5tFrp2TKlEL",
  "O5qhVI9q2w8565wa",
  "aiuX4eaYoaJijp2A",
  "FlAIDM1nvRh+US68",
  "8EI19JRQgEkCxphM",
  "MjgnjzvuONpvv/0a",
  "IjRnr1hxH60qf/KT",
  "n9A666zDz0tKgBQJ",
  "4TE8oFqsp4wGku9s",
  "t1uVrjkovBszZgxt",
  "vfXW/LxEFnAu2z3O",
  "i2HrNg9lHErXLsm9",
  "JorQ/AULfO8pjEM2",
  "EGNxmItEuTxFMG3l",
  "8hSP4noR8Y1LLPKw",
  "2IvF8pSjPurPdFEk",
  "n6VszrT5M0oVPf6x",
  "4/sipUXyplH8g++J",
  "53FNwjwp3k5pFSjH",
  "bTeVUJRiFC6UxOkg",
  "DogSJI2eoApTqmC7",
  "zS9gSwbtJB6hoM9R",
  "QdnGptqLjaD3x2CT",
  "wh7s+/Wvf50efvjh",
  "wJZqrueN635DFfK4",
  "HD88l8WAd0a+n7Sv",
  "xDE9/vjjtNdee/kt",
  "72xELFrzqJVyCTp/",
  "JSVFupNIhEEMLEQV",
  "0KJRDCsZT/Ia1xy0",
  "wueQNoLnu3v6/AYG",
  "be0tbFBK7jIX0vWb",
  "RRf24WPC53sGK4+n",
  "rHg80/yYiwtjZtxz",
  "oU9LBxuoMEpZmima",
  "Z2NSGib09PbybyAt",
  "APGcSDfB4Dz00O/T",
  "HXfc4escYhNjvBJV",
  "6Tq/Vnd+Cnp93qFz",
  "XSX/DrZXX55DdNmR",
  "dk/KsmKUk0D5gIuh",
  "iabtf/vb3xrCa6TU",
  "LzIJ4WIP78lf//rX",
  "QXmOQYZWsdBDJS6y",
  "wRe6oXst20hYEN8F",
  "eajz5s3jHsvYHnjg",
  "Ab81pXgxXb+7opSL",
  "nb4h4XIYbDDsMAHu",
  "vPPOvmEm3kzpFy5p",
  "HkHYbVlFF1dSZMaN",
  "G0uZbJoNQBiZLGsW",
  "NwusdF8/fTZ9Kk2Z",
  "MoVmz55NCxYs8EP4",
  "2MaPH0+rrb4aLbfc",
  "ctQxdiyHxLHFYnGK",
  "x5KUTAyMoX4s9mJE",
  "qZQpIsT7wFCU1n9A",
  "5NTEw4nX/u53v6P7",
  "7rvP/76i1iLyZPVu",
  "yCnVp7DTlpyTuP/J",
  "J5+U4kCsqJHJx1Sm",
  "JT+XiBYL/JBh8tMK",
  "96kmOlCb26MpXjtc",
  "uP/yl7/QAQcc4HtO",
  "Clvg2djV3oXPjeSc",
  "Gs6jGWzsDf73ws/O",
  "ZvM8geF9RMoJGybx",
  "999/n9tTymJPJm6Z",
  "1EXqSVFGikvESkLZ",
  "4kmX0B6eh+ddWqiK",
  "GgT+be7cudySMggJ",
  "Fcq5LIamPIfPhCcR",
  "pJJxyvan6dVXX6V/",
  "/uN6+uCDD2jaJ1No",
  "/vz5PCbsggkYfjCA",
  "28a0cMEg9C932W0P",
  "2uxrm9OYjrGUpTwv",
  "6nq7eikSN9E7O7dS",
  "vKjRAsFsiT5IyH/M",
  "mHF0+eWXczqPGJ9i",
  "kMrjctD5tbE9mhEr",
  "H9lWJZH0lBK84uNc",
  "VIVKPr4yf+DvEdH1",
  "QTstueSSNGfOnOIH",
  "ogOhoam1oSkeAjQT",
  "uO222/ywHSYVEYeW",
  "/QpfFwZDEzlh9v6F",
  "n40CBfFmAnwnyfdC",
  "aPA73/kOPfTQQ35I",
  "RVa8mEQxmYWhsl6p",
  "X4LGn5yXEgaXyRBG",
  "IYpG0dscBpsYn3J+",
  "YqEEJYUgpIMXzmf7",
  "/Pa9gVxMF6FILk+P",
  "PPwgXXrRxfTaK69Q",
  "1LsutCdbfKNUcklx",
  "rH4OM18r+iiTz1Ey",
  "laIVVl6eDj70UM7x",
  "XmyJCZTtz1EGofCs",
  "0cvEwo9D5akU9fb0",
  "sKFp583JIlc8u5GI",
  "MQa23XZbev311/3c",
  "UfHqlovOr41taBZi",
  "G5olnD8nEdH5VAXK",
  "/Xb/dtnpz3/+c5kf",
  "oyjlgUGHogPou+Ii",
  "bntXXFUPir13JY6v",
  "2GYX1Q1VFSmV5DJB",
  "AQk3YuL83ve+53tG",
  "pFBCvJparKeMBhIq",
  "l45cUriAfOmxY8f6",
  "RqEU62FzMTJtxDNv",
  "F70ZAzNLn37yMR3x",
  "/cPo0IP+jz6cNIla",
  "E3FqSyZoXHsLpWJR",
  "fpyIEG+xfI4ok+b7",
  "qCeP54jGtbZTPJun",
  "SDpLn3zwEf3qpJNp",
  "7z32pGuv+gNNnz6d",
  "cpBLS3iLVm/MZrGg",
  "9YaXfD+7Oh7P4feA",
  "MYBczbPPPntQlAHP",
  "6/gMP9WuWi/VuLS7",
  "35XAlVQlyjU0F7r0",
  "Pt91113L/BhFKZ8L",
  "LriAvvzlL/v6mQil",
  "iYevnrCPVy5k8j0w",
  "iaEYAULXMmnjggOP",
  "JjQKxaMEo1sMVNW6",
  "VaqNLIJk0pX8RNxu",
  "t912/Jycq7ZEkbw2",
  "CLwPxrSEo8WoxWvh",
  "YbzpxhvpOzvuRPfe",
  "fTeN6eigbH8ftcRj",
  "vCVRVY5JGVXuCFVH",
  "o5TEOMHYgqHKxQw5",
  "6u3pogQM0lSCIhmi",
  "1ngLTXr3fTr5hFNp",
  "rz32oP/85zFKeMcg",
  "FegwFOV7i+oDJn9U",
  "oUtVOZ7DeIRH81vf",
  "+hZra4p+qHwfRQlC",
  "POayuMI5dPfdd1Mt",
  "czMrFToHm3tyR0VB",
  "eAQJqRI2sJEQRdED",
  "LTPHs9qrinozVsJG",
  "Jf4+ku8kOV5yTuFi",
  "vvXWW7K8g+wjxtWA",
  "B9DksghSLSq90HMZ",
  "M0Hw66L4W5siA841",
  "iwwIwsv3kPPcf/+8",
  "0fHr6e3yJ8R0uo/f",
  "10yqA14MaO5hwgN4",
  "D67YhSelINfZNjCN",
  "pubA84Uh/3g8Sddf",
  "fz0deeSR/qQvq1+7",
  "ync49PxWiqV+uLxO",
  "mh9ISgc2GFPIlVxp",
  "pZWc3mconUAmGqFc",
  "FjcRiseitHDBIkq1",
  "xKi3v4dOOeWXdNuf",
  "/8zFPy3xBBcDxfMR",
  "TH7++6Rz/Y6fHaGc",
  "9zqE0bNshhL15WP8",
  "focd/n068ZcnU6Kj",
  "jXJkFnItyRbqQ2g9",
  "k6GOtnbqT5t8uZin",
  "Y9vd20PJhNfuNmJy",
  "SaGDO3v23C/kkI+U",
  "Zg+dl/v7VbtgMlLm",
  "7yffD3OV5D1L45Fa",
  "5mYKlVgqPYcxF7TT",
  "zTffPEgcWlZpom2o",
  "KOUghqWIPIsxKXlO",
  "55xzDk9qMuFJNapU",
  "YcObIKtBPC8eP3mu",
  "N9NPGcpRP4TS8xlK",
  "ZzPUn0lTPJmgJEJu",
  "0Qh7SboXLaQFc+fQ",
  "nFmf0eczZ9Dcz2fR",
  "ovnzKJrroXTfQmpN",
  "xiif6aPenkWUSiT5",
  "uE3RAsL5ad4kp6Yw",
  "L7RQrsK+OInnQ5K/",
  "ZcN3l22PPfbgdpvi",
  "xRRDVVGqjS1pZJ+3",
  "a621Fi2//PLlf0De",
  "LJj6+vuprz9N7e2t",
  "1NPZRYcccBD989q/",
  "0HhodaZS1IK8SSzE",
  "shnKZdKU97ZYnopu",
  "CH9Hc3mKwQtJETYv",
  "YxShGGTaIczO6TgZ",
  "uurKK2j/ffamz6dO",
  "pbaWFg7NR2IwFLOU",
  "SMQIpmmqpYUSySTn",
  "n+K6I9XvsvhF8ZP0",
  "o7arhxVlOGQuw1wi",
  "C7kjjjiCSsjNrJqR",
  "WSmPJjiSiP4YtJM9",
  "GUo4AZM/VnDlelTU",
  "o1nfVGLFiYGGDWFj",
  "IB47CJdfcMF5gyY5",
  "O59F8hThwQSQBcJF",
  "H+cmBi4bZq0pzr8S",
  "7Tvo5bW0JGnmjBlc",
  "MfvCE/+lyZMnc66W",
  "VNTiOLA/ihzW/+p6",
  "tMwyy9AGG36VNtlk",
  "E+oYO55a29somUix",
  "Z6S1rcPXOWMD2dJB",
  "4wtH1HhECz2V4r3E",
  "RCYUGqCgv98IR591",
  "1lmcp1pY9BBUda7n",
  "t1KuRxOI51+u/5C/",
  "Q3cu1/cZ1qNJCFeb",
  "nLR4IsqLuyMOOYT+",
  "++hjtOS4cZTMeM4M",
  "jBcRY8f7xSQiEOAr",
  "iaANkNk3h9oibBTh",
  "qnO8Fd69F0V1Cehz",
  "pmnixIl08SWX0Dd2",
  "3ol6slB3SLK3UowA",
  "kXLCmEePd3g5cR9y",
  "SbJoRkvODz74yH9N",
  "OahHs7E9mu3t7b5K",
  "gsw/JXzn8ZXuBFQt",
  "QxPtR94M2gl6fo8+",
  "+qhfEWvnrbhUDZfz",
  "72poNn7Vn+2plPzD",
  "VVddlV544QXq6Bjw",
  "ULLAcirlh4wljIxz",
  "UdpTQmsTeVR4L7tb",
  "CUJz8Fy+/NJLXOT2",
  "8AMPsixKxpNSEY+i",
  "fI50JMr0mlyttjEd",
  "tNRSS9CYcWNp5112",
  "oQMP2p9WWGEFolQ7",
  "9fX28j4yJlBJLuH3",
  "eCL1BUPTlrIYTv5I",
  "bk1Hkxx99tlnLI6N",
  "zkFi1LKgdICYr57f",
  "SjmGpowLWxcS5yPG",
  "0KGHHhqYOmWngwy1",
  "2Er3ZSjV2kJ96V7K",
  "9PfRMUcfRXfdeAst",
  "v/gEyvX2UlsqzgYm",
  "57F54uuS8kIwNhF3",
  "LwIMSvvzuXUlGgix",
  "rnuesvEE9aXhTc0Y",
  "4zOf54rzS668nPY6",
  "YH/q6es3eaPJFKUz",
  "RkKpvW2MP+5yeXNN",
  "6us18yJ+q9tvv50O",
  "PvjQirSJVUOzsQ3N",
  "mNUYB/MF6hGOP/54",
  "l5eeTETnUZWplKFZ",
  "kqamnasj+XTlelTU",
  "0KxvKvH3sScyCaFf",
  "c801tO+++3KfYfk3",
  "W5fO9q4AGIW2tIoU",
  "FEAeJRGN0dNPP02X",
  "XnIJPfbwI5x30oqu",
  "IukMUdwYl0MZgpxn",
  "GUEbkYH+xQiZoe/x",
  "aqutxj2ev3/UUTR2",
  "/DgO2ceicf53GKWm",
  "6jTChqZdIPHFisbB",
  "F8LCsDu+rng3Dz74",
  "YLrzzjsHpawEhdD1",
  "/FbKMTSlOEeKYySa",
  "8Oyzz9IGG2zg9LnF",
  "DM1ELEGdnYuovaOV",
  "zjrtVLr60ktpsVQL",
  "tWRyNL6tnTLZgdQY",
  "v2gCBqFnKCIIXvTz",
  "kY1pLfRyFOXXZr3X",
  "56MpyuSy1J/LU2dv",
  "H0dAOJczn6dfn30W",
  "fe+IH7D3tLc/Q+0d",
  "HdTLkZI8p8/gN4Gh",
  "yRX5/eZz5Bq2+eZf",
  "p3feeafshidqaDa2",
  "oRnxzk2xo0r4vlXN",
  "zRQqmaB1tMtO0NS0",
  "Lxxy0VGUcrAnHxl0",
  "3/zmN+mQQw7h53Gh",
  "lm4fUnktFZ9S/QqP",
  "AvaBgSmeRdlmzZjJ",
  "npdddt6ZHn/0UWpN",
  "pSjl6W/COGxFviU8",
  "JekM53BxHlc2x7dt",
  "yRTlIhmKxiOUj2B6",
  "I87Tam1tpw8/+IQu",
  "veQK2n/vvehfN99M",
  "lM5SoiVFsXiUiwW4",
  "L3Sr+ZxiFw+ZQAs3",
  "kXKyCwoQrrSLprTq",
  "XKk2ON/shR42CKCj",
  "CMj1/LMNS/s+yCF/",
  "MpKnf910E11z2WW0",
  "VEc7teQztOT4sdTf",
  "s4iyXHjXz7mZkC5K",
  "xCIsaZSMor95lHMt",
  "i26Q0mQ/JrYIfxbu",
  "473wGL3OCQWDPb00",
  "duw4DqtHozFenJ78",
  "8+Pppuuvp1g2yxJK",
  "KPTDtYONxyjC6Kbn",
  "OaIp0hEJ4x5j85e/",
  "/KXWMCiB2IWnp59+",
  "Ojly22gYmZX2aI51",
  "ifNPnTqVLy7252rV",
  "uVKJvw8u1lJIg0H3",
  "4IMPcp6TEXEemJxs",
  "DU1c3Lm/MQrD83k2",
  "MvE+IhQNzyIE3s88",
  "6RTq8iRJIH3CxQFR",
  "dBkxeaGLOjsH9SkG",
  "eF46EmX7evk4uvv6",
  "+f05Dys9YAy2tSdp",
  "wYJF9IMjj6Cjf/QT",
  "Wnr5ZSmTzVGqo81M",
  "0DkTBRDjsNCjKYLu",
  "w/2m6bTx2iIlQFpT",
  "vvHGG75Bqh5Npdoe",
  "TWAbTRAnv+eee0yv",
  "cYcWsMVyNDHuF86e",
  "Td/5xjaUnj+P2oio",
  "A8Zcbw93AZIIh/Fe",
  "EkVErcELc0P1oejn",
  "I0fTAo8gfIRlI+5n",
  "I1Fa1NlDybZ2Skdi",
  "7N3Mwmua6edORLhg",
  "XHDJxbTH3vtQGmMf",
  "xmo85XVt6afWFtPj",
  "feGCTlpiiSU4v9uM",
  "9wRtscUW9Morr1A5",
  "qEez8T2aOa+grISF",
  "Cbx+s2kUqKRHE5bx",
  "i0E7IR8Ngx6TMLCL",
  "EhRlpLBMSHf3IBHo",
  "jTbayPeiiAFq9w+G",
  "EWnr7eE8hCcB/44E",
  "feRonnLKKXTUUUdR",
  "36JOakcoLmqkUaC/",
  "Bw9Gb1c3dc5bQAnK",
  "85aKRqg9maCOVJK3",
  "MS0ps0XbKJYmGhtL",
  "UTJLFE9nqD0Jj0eO",
  "WhJ56u/uoXEd7XTz",
  "P2+gQw7+P3rnzbeo",
  "Y9w40/vYy9H6Yrh8",
  "gKH+zS6+kwp8vB88",
  "sKhIlOKosE8SSv0j",
  "Au1AzsW1116bz8VK",
  "TOKtrSk68fifU8/C",
  "BRTDAg2eespRaypJ",
  "Y8d0UCIeo1jUBMhj",
  "SN/iqnGIs0d4i+Wo",
  "6AYzmTekseQjXjV6",
  "jqI5qUbPUXtbihes",
  "7DHNRSiRzdKYjjaO",
  "SMQjETrx57+g+++9",
  "j1Jtbd5vYCJ6uB+P",
  "mWI/GN1ouykdjrBJ",
  "BbqiDIdEpq666ipy",
  "5DejZWRW2qPprKmJ",
  "ytczzzzT7xLhMtHV",
  "+2RY78dfbco9D0XJ",
  "ABqUOKekd3Jvn1E0",
  "QKArFolyNbcYnrio",
  "o1IVXgl4BKVPeHtb",
  "G82aNoMO3Hc/eu2l",
  "V0yYHDOKh3Ee5vyO",
  "H/wcjsF+jEID6/jw",
  "2cVII28M+pz5KPX0",
  "9dJyK61MZ5xzLu20",
  "527UDWklyKpID+hk",
  "ijLwjMaNBzcCAels",
  "rui5ZiYt04UEkyFr",
  "9a2zHreGhbfT1hBt",
  "RjTiMTpV57aO8nXX",
  "Xccdq4Ct5TpUDiZO",
  "73gswnmO/bkspaJJ",
  "NgDReSeXILrlL9fQ",
  "ycf8lJaEYRlPcEgc",
  "4zSWgAGXp75uU3Qz",
  "aIwWOXTkX9r0ecML",
  "xT/wz4iWpnlsvhde",
  "k8sbL6l4Oo3cO9Ei",
  "6GkiN7w/TXc+9CCt",
  "utGG1NuXobFtrZSm",
  "LKX7jWqGXUQoOr24",
  "rm255db0xhtv+b/f",
  "cLnZpZ6HYTkvax2R",
  "DCJoMQQjT6KyEnGS",
  "epSMFa0t1EQdKuXL",
  "HitAQuL2Mdj7y/Gh",
  "8lwUV8JQaW5TaRE9",
  "J03NM844w3f18kGo",
  "lp9SJlLpDWmRzTff",
  "nL0lEsrGbTZtLti4",
  "L8LruPULFNI5ikfi",
  "FI9E6ZH7H6Rv7/At",
  "eu/tt6kFIT9c+JFz",
  "RZ43Ay3p8Ji9JtgG",
  "JjA2BC0jkx97FavF",
  "NmhxThg3jlKJGLW1",
  "pmjurM/oJ0f/kG6/",
  "4QZqa0lRNBHzJ6J0",
  "Tze1JNFPOcsTWtTh",
  "oiyyTEB0Qvfcc082",
  "MsMy2SiNixT/ALlF",
  "pyqMw6GMTPs+ywGh",
  "aQLSTNIZrhpn3dsI",
  "ggr9tGjOHLrusito",
  "sRZoZbZSWyJFyWic",
  "erq6KdPXy6Fzo3np",
  "SZp5Y5kNAet5e5N/",
  "9x8jR5NbU2KsI/fa",
  "6GeKziYWepFMjqvX",
  "2QMKzydF/JaWbV4+",
  "OIC25/yZM6k9Zb6T",
  "fJb9O4jGJq5r8G6e",
  "fPLJntaujtcwUHi+",
  "SM67/W9yHY5zL/vB",
  "iyd7H1ErKbSDbFUR",
  "KVi197fHx4QJE7ho",
  "zJHLR9PIBNWw8JyK",
  "gg4//PBB1riilAPO",
  "IRiNqC4/4YQTeHWH",
  "Adna0soDPZnC1JCl",
  "zs6FlEqZ/TCuu3s6",
  "OYdq0bz5POG899bb",
  "dMSh36epH35E7RA1",
  "7++jca0pDocNbAPh",
  "NBOCGwjFmR5DnlHK",
  "BikKCSIFr//ihsmo",
  "t7sLFiEbtgj9oXjg",
  "rFNOpntvuIkLirq7",
  "u7j7CD67v7uLMrk0",
  "Fxh5v0DRTcaaeETw",
  "m+y33378G4gBW2xT",
  "lHKR6zxukYcIxQWp",
  "BA8Ci0K0VgVoGcmS",
  "XOkeSiXi9Perr6a5",
  "k6fQuFiCIuij3tdH",
  "/b291JJMssEHgXY7",
  "DM751WIgWtEIexOP",
  "p2wJFmrHNjDOTTjd",
  "GJ0Yv0mKUoqinEIT",
  "xxjD52XyvLXh+Yjp",
  "ADZ/1mz6xdE/okxf",
  "N/WnoXuYpQg+lAsF",
  "s1yBjlt5jHaWO+20",
  "E2244Ya+gSG3WDCW",
  "2g9eKZ0gRwHmHpyT",
  "kqYkqYHSnSdSoG2M",
  "FC00Kth00035Onzq",
  "qaeyniy6JxYWcooH",
  "297k36DagM+YPXu2",
  "kclzw7laKKyhcwDl",
  "UCf/rT1oGl0wut6P",
  "vx6StTHQt912a7rv",
  "vvvYe8cDPGoGJlab",
  "eK67s4vGjx/P5xs8",
  "BostvjjrV/b39HM7",
  "yH323IPefPVVNv7G",
  "omgHPsPMgPwRh9u4",
  "4tQz3AoOW8RP+D7O",
  "bZm0CncsoKcvzdqa",
  "OMZkooUvHn2ZNEXQ",
  "u7yvl3595RW07yH/",
  "Z0Jzff2U6c9QLJWk",
  "dMZ876E8HfZjSKnk",
  "cqZbEiYu/B7dXb1c",
  "aPD++x9Ss5+/Gjof",
  "ndC5hBiRQ41iPSkS",
  "sj00Q70WXsyFnYs4",
  "5xHC51FOkemj2ZM/",
  "pv2+9R2KL5jH+rZJ",
  "VLTHI5Tu7ePJHOOY",
  "slx6U/z4IsUf+0Yy",
  "Hw/UI7znvVsT18C/",
  "e5JmEkL3vncaBmnH",
  "WJo3dyGNaW+n2V3z",
  "6ZhfnkjHnXoa9WSy",
  "fkoPkII/lj3yrmv4",
  "zkg1OOqoH7GxKtc0",
  "dBvCvkE6uIKGzkfG",
  "UJ9vH7MYk4UpInhu",
  "mWWWYWPykksuoRAw",
  "KrqZo+HR7PJcs4Hs",
  "v//+vvWuKOUgF2YU",
  "uUjISXJbZCGD+/B0",
  "Su7T+DFjaf7sOYQE",
  "sHy6n446/Af0zttv",
  "mbZxuSzl0NEDoere",
  "Xj90xpsXbmOJE28T",
  "72Uc+nfs1Ygab6a3",
  "xfnx8Nv4MeMp24sQ",
  "fYzlT+D9gFRKKhLj",
  "AqTzf3kavfTUM2ws",
  "ZtACM91P/b09/Nk4",
  "TlDofbQfS/jGDvXA",
  "E/Ld735Xx59Sdex8",
  "MtyikYJo1krIcLgi",
  "Nzl/0YmLQ4lRVHOn",
  "qSURo5v/+hfKzJ9L",
  "qXyOJoztoDGtSUrG",
  "vCYLWRQDtbDXHuPJ",
  "dUM+tT028VikjRCd",
  "iEfJlzvyQ+/5nLeh",
  "65B3TYCXM4re6xFO",
  "f0l3dtJYbo7QRW1t",
  "rfSHSy+ll/77X8rl",
  "s36xohgooowhKUHw",
  "bqKF7HLLLcfGpYBr",
  "HTalugwV3bGfsz2b",
  "Y8eOpd/97ne+t376",
  "9OlhMTLBRbX40Gol",
  "R57mstMNN9zwhSRX",
  "RRkJGODwkuywww7s",
  "FZSKarlwZ/rTyNRn",
  "sWRc1KW3OXK/kGt5",
  "8gkn0IP33sNhNnhA",
  "YCQi5yrd18/hKXgy",
  "v2Bgwo+BSQmTk/wb",
  "Qie+0Wn9O/K5imzU",
  "n6F8Okt9nd2cY4ZJ",
  "csLY8SaMDm9HVzcd",
  "8/0f0vTJn/Ck2z6m",
  "jXskY8JLJo0uaDGv",
  "kz3O8FuJAb7bbrv5",
  "uqKKUi3sXDOcd/Dy",
  "2KlThbqY9n2ZsCEr",
  "JmMaxtZHb75F9950",
  "M42LR6k1FqF2aM9C",
  "NL2zm8duNJdl3UxU",
  "hftj0trs8SfPRYfY",
  "h29h9GHz94HBaca6",
  "GKEoOsL+JsfTLEpR",
  "hogQfc+iTk6H6WhL",
  "UQtyM/vTlMpH6bQT",
  "TqRYxmhnyu9h51OL",
  "KguuWROWWIwOOeR7",
  "vh6wVKyHxSvZLAy1",
  "IMI5uc022/DfZsGC",
  "BZy+FUJ+6lJDU0+G",
  "5kJPDDQQFG0oSrnA",
  "sIQMEbwk0IpEjgwM",
  "RA5FJ5MUi8GrmaF0",
  "pp+6erpo7NgxtKgT",
  "p2mefnPOOXTz3/9J",
  "y0xYjLI9Pex1iGXN",
  "ZNLR2uaLsEs+pUic",
  "2B5LGJn+ZMXFAwOP",
  "eb98wIae6Ik4tSWi",
  "FMtnaGwrqmrTPIG2",
  "xTGxJmjOp9PoZ4cd",
  "TrmeHg6VQTYlmzXt",
  "7LgXc5EN+0ghlB3C",
  "XGONNfxcOc3RVKqJ",
  "HV6ElrJtWAWdf9l8",
  "htNQuBgu20/p3h66",
  "6S9/pt7Zn1M800+x",
  "dB+lFy6iVC5P4xJJ",
  "GhOLU1skTvmefmpB",
  "AZ+Voyl5lxCSsPMw",
  "uUrd2jA5yn1/zHtF",
  "PuZ6YLYENoizEzyf",
  "ZtxHoUqRy/OGtlwd",
  "yTb2bqZixAZnNIPi",
  "wxi9/crrdOMfr6F4",
  "HBEY5PNlKZVKUksL",
  "5I3wfZFPDYF700kP",
  "IViTkzmQnqOOmupT",
  "LD8TiwHkVj722GMU",
  "Ym4koitr9eHVLPd2",
  "KgqCaLRWnSvlMnHi",
  "RPrGN75hKsy9RGl4",
  "AUSoGUYVjFAYXIsv",
  "vjifcwijP3jf/XTZ",
  "ZZfRhHFt1NfZSWNa",
  "W7hKtT3ZQi2xBHSH",
  "KBWD0ZczhQQo9OEi",
  "AmNw+gUEnicDcKGB",
  "3RvZJZk8n6UoDMw2",
  "k1eGalm8B1pfwkuC",
  "PsqLtY+hd198lc45",
  "7QxqH9tOnT3dJrSG",
  "woECCo1D0a7F7wMj",
  "VWRTFltsMdp6661H",
  "6a+kNCtyjccYBFjc",
  "iHeyMHRuL27kVooi",
  "OJc/EqF5c+bSQ3fd",
  "Q2Nb2igRjVBLNEat",
  "sTjF0zlqiyepLZqg",
  "FopxhAKpJ7L4K6w0",
  "t9Uk5P5Qm2+AekoT",
  "vlGK94PxnMvwWGXj",
  "0jNMEW2IR2NcvBTP",
  "5DklZtGC+RSDNzMa",
  "5+vMUhMm0JXnX8jh",
  "VUnvwTVKPL/y3aW4",
  "ZN1116XttttukLdT",
  "O3tVn2JGJv4GKOwJ",
  "uZF5dC0PoJoWHsRA",
  "H3TZ8Stf+QpfUDBg",
  "7FL/ocIp9Yp6jIoT",
  "ZIhFoywixIn4Jhl+",
  "ICyODSGlxRcfz/dR",
  "nSqeTHTf6etNU3tL",
  "GzcmTvdlqbunn7r7",
  "+mjSu5Po5GN/QYvl",
  "E9QBTwjE23NZriSN",
  "oWQ0Co3XNEUpTTHO",
  "8yJKenlXqC4d8Fai",
  "ajw/sEUHQuaQTIpF",
  "4pTKU9EtnoAUSp4i",
  "8J5mYdSaDeLuif4c",
  "TUwkWctz4tgOuuOP",
  "19B/7rybJ+0MvDI4",
  "YqsFGeer9qYpEU+x",
  "Lie646X7e1mHc0xH",
  "B7W1tPMxYeLF5Lf/",
  "fvv4465Q403C7vV+",
  "/tb78df774fzU4wo",
  "5E8idG7nJQ4YnAWK",
  "CZEMb1kuwstSWyrJ",
  "4uuP3Px3ii6aR6lU",
  "nJZC6kfOVOKyBZhL",
  "E2W7KUZ9xOmM+SzF",
  "o3neEtaWigxsyUie",
  "Wmj4TWSKYO7hSpQk",
  "jPkc32JrhQQTb6Zp",
  "A/IzTS4nivTQocFE",
  "LGD0RqI5ao3nqTWa",
  "p2x3F+V6e+nKk35J",
  "He0JyvZ1c9pMZ2cX",
  "5fsRGk/gq5nHORMu",
  "P+aYn3jpBEgPgic0",
  "WD7NzpEt5e9aqfFR",
  "6nGFDVkUSRcru8A0",
  "5PyTiI4cbTmjQqrt",
  "SvyRy06vvvqqn3MC",
  "/G4J8fiQ+lJK8yF5",
  "WmJI4ZQQSSN08Tno",
  "oIP4IiBtH7EfPHfY",
  "ZNXZ3ddNcW9SSkUS",
  "dOzRP6EutKtLJQfJ",
  "miDsBS+FH1LjPC0J",
  "meVZ6kTC5QMdRga2",
  "WASSJ9J1xBifhfIp",
  "Q23Gy+JNGl5OKHTe",
  "saX7einf28/C00uM",
  "G0+/PvlX1DV7jslX",
  "YzH4gep6lthIDFTx",
  "5qwJA55MiLXbF0ss",
  "9JZeeml+LB5gvwhB",
  "5ceUCiDXd5xfa621",
  "Fq2yyiolTdTQrsym",
  "IWzeRwtnzaTHH3qE",
  "qL+fOqIRNsxMKFvy",
  "qE36CmvderqXpoPP",
  "8FthCL1w8zsCIfyO",
  "x3743ArFe12HUBTE",
  "edxeBATP8XWBPaf5",
  "QYVHMEjxb/fffz89",
  "c/8j/Jsk41HqQMck",
  "FBxlTX4o0oCgFoHf",
  "EB3PsNmFVEp1EdtE",
  "Cq/qxMj8MRH9H/oF",
  "1PpAqn2WfkRE0112",
  "XHbZZQcNHPxhh9Kg",
  "UpoTES7mbj5eTpdU",
  "ln/729+m1b60Ci3q",
  "XEDpTB/nJEI3EzIn",
  "LWj/loiy4Dm6iuB1",
  "6Pzzx8uvpDdffInG",
  "tbRQPGq6Fkv1OBt4",
  "PHF4xQJ548mA1wPG",
  "I4zIFMGraTT6Uuzl",
  "MK8Vrwe8HfZz8YDN",
  "VK97kxde509M3gQX",
  "i1Eb8sAiRKlYlOZ/",
  "Mo0uPPNsam1rYd1A",
  "syDDZG7GDFi4aD5r",
  "8uE3QPGEkTeCAQqD",
  "EwVTyNvso3HjxnD4",
  "fHDHkeH7Syul0+we",
  "VVu+CGkuyDOU673L",
  "+cXjJJbk8/fVp56m",
  "N55/iXMwW/vQlGFw",
  "PrSdQ20WhgX/7rDZ",
  "BUL8HhjTsqhkQ3Pg",
  "1ng5B/KtzWssHV3x",
  "iEaRyxkx74UmDVGz",
  "IbcTofeLzjibsn29",
  "7OmFcdrb38etaVH0",
  "h3a4WETDuEEOOnI1",
  "QbOcP2Gg8NoYspSF",
  "TiK6n4h+RUTbeKfq",
  "HygkjMZyyCk3AMKj",
  "dt6D3c5JPSqKVJKb",
  "PuVmdSnnxU9+8iPj",
  "CfC8mUByEKUDEHIc",
  "Jc/p5Wf+R1decCGN",
  "T7VQApIkWYTHLS+i",
  "1/nDLwIwwWmeIIwR",
  "aSa+GGV9Q1EKA8SL",
  "MWii8d6v2GY8HEY2",
  "xS8usoxQzvdCT+d4",
  "knJ9vbT04uPpwVvv",
  "oP/eey8l242XVmSb",
  "YHCioEB6usOrK0ak",
  "3fdcfg/8PgceeKDv",
  "dbLbUUKLUKvSlUqA",
  "cw9GFCIQOOckIuEE",
  "xkACYkM5evjOf1Ey",
  "nacOiGSn+9mjOahz",
  "l7VgxCvs5gmu24BA",
  "+4BsGRaePPb9oj+z",
  "IDReyYgfVufe6fmB",
  "fbEYlUUsHxv2t/qs",
  "Y7/WRJzef/V1uuum",
  "W1nGCUBwPtFq8sol",
  "SoPFM37DAw7Yj5Zc",
  "csmSW1AqI0OcGqKe",
  "INfWqVOnjuTt8kTU",
  "TUSfEtGLRHQrEZ1D",
  "RD9EfbSk9Ttum3u3",
  "Y4joO14P8yfDdmKM",
  "hqF5FxFNDtoJqvYo",
  "0rDbUiJcYIfwlObF",
  "pFHAKOqzOmPk6Stf",
  "WYc22GADv92kdL0R",
  "w0oWLhw+R3/wWJQu",
  "Of88Snd2sScSeVUt",
  "qOocJHUiHhGrA4jX",
  "J10mG94PRltUcrEi",
  "/oZBJUarPwHyvsNv",
  "/qRECOt7xqx4QzAe",
  "YjHqQ36bF67vS/fS",
  "2GicrvndRZTp7aEE",
  "LNF8lj2zaNOH3wOy",
  "R22t6IxkpI0khQC/",
  "C4xMU41vfhtIc6y4",
  "4oqDQuwAt3UQIlJC",
  "jt3lBGFznHvizXRx",
  "JEA7FnPnx++9S688",
  "/TSNb0lRK3IrOTUk",
  "441H4lxqf9zJeHbQ",
  "0URXH9mGem6g3azJ",
  "v2atTYk2oNUkxhR7",
  "PfMD9zkSgn+TjmEm",
  "csISSFxMKB5P4k5C",
  "EKO/8MxzaNb0GdTV",
  "tYir1dPxCKXiCdbd",
  "ZKUJKGLEYqynucsu",
  "u/Bvg+ugUl3kHBXv",
  "Oze+yGbp4Ycfdn2L",
  "W4noZ0S0kWd3obEN",
  "WvlsQkT7et16riWi",
  "d0s8tP9RHTBaCR5O",
  "Xs2XX37ZNxbwB5Xi",
  "IA3dKbacB1b3CBHj",
  "PnIzsSAB3LPcq9aU",
  "0AaeY0MpR9QST9C9",
  "/7qTnnjoQRqTQsAq",
  "y2LKEH/2q0uRg+Xp",
  "30kOlvQ3Z/08NjZN",
  "/3LxmsjrbAkUzt+0",
  "PJlB8kYyOWEzkinG",
  "wDTPGf2+9rYW6mht",
  "oaTXGx1FCJ+88Rbd",
  "fdNNFIvFidKoXDeG",
  "LybZVCJJmaxZeYu3",
  "E5tcJI3300z28DJt",
  "tdVWg7pbiIdTF3pK",
  "uQwU9UW5x/nAYtGt",
  "GIQXmvEIPfXQw9Q1",
  "ex61II0lm+eFExaM",
  "MPw4oiDeSxiXnjez",
  "VI/mF7yZVjGQEWkf",
  "eE68p/IajFl+jbdQ",
  "5HHM1xOk5nhta73r",
  "Ahubkm7DHSfz1P35",
  "HLr+6j9Re6vRyc1H",
  "zHfEhp+Jb6N5vqYd",
  "eKBpeKLjs/rIfIIF",
  "kn3Onnjiia5vcb7X",
  "yOZlakJGy9BE7kCv",
  "i1cTzeGFQpkHpXlh",
  "LT2rSAzXVoR1IWlk",
  "nkcRTMTTm4MHDsYV",
  "LgishkL5TJ66Fyyi",
  "P1x0MY31jMxMtpfz",
  "oFDRzsU4XuefAW+k",
  "mVSMF8KTOBGD0p/A",
  "PGPUD3l7HUEipn+5",
  "TFSmInX4zTZSeeL0",
  "87w8vU5ILqWSHDrD",
  "JS6RQXW8qYC/+Yqr",
  "afbM6ZRKxCiLvMu+",
  "fjZ0KZehvt5ubq1p",
  "Jql+1uWTXE7cN1Ec",
  "M3HtuOOOfjGQoGkr",
  "laHZczQHhNYT7I2T",
  "/HvJtQ4iEU1Qz7zZ",
  "9L9Hn6D2aIJa8zmC",
  "UBIqro330DYmZdxI",
  "P3KM2YDN2pc9jAX/",
  "LoYkIg6DDEtvkSjR",
  "B//zLWOVn+exbULw",
  "eH9/Uem9R2s0Rv1d",
  "XSzX9K/r/k6zP/2U",
  "FvV0U74vTTFYzUS+",
  "LrB4gb/61a9yYZU6",
  "YqqPcW5kPYM/4usR",
  "Q5zdkS2piRlNC87J",
  "q4nm8BLOk0qvZrgQ",
  "K8WR0C/OCVRNJ5Nx",
  "rrxcZ511fDkjqUi3",
  "w8S4z7IUyVa65193",
  "00fvTqIEwtWxvCd8",
  "bBADk++zEZnz9DI9",
  "IWd4Or08TjxnC0Bj",
  "f7vqHPmc0YKqc6gl",
  "Fds4tBaxQjNiYHqV",
  "s+2pFsp099I8XNhy",
  "WWqF3FMOIfA4zX7v",
  "Y3rgvvv5u7YmU2yM",
  "Qr+PNfYiUa8YKMG6",
  "oZL/LLma+N3EANh8",
  "8805NAfsyUsnMqVc",
  "JFIFjxCKgWRBIxO2",
  "y+s/+WASffLOu6yT",
  "iTHGhloiRdFM1leK",
  "8CvPRc/SMzqDFB8G",
  "VZkPkQgXLVhk+kLu",
  "8v7SGcwrIuTrh+R9",
  "834DzRxsYXfch1cU",
  "mr1jW9v4df1zF9A/",
  "/3o9dYxtZ08nfjeM",
  "U7mWiYE+fvx42m+/",
  "fdQRMwpIpzlJ/xAb",
  "Bc8///zzLm+xJTUx",
  "kVE24nBVCSzVWmKJ",
  "JWjOnDk86WGAuYTP",
  "az0Z1vrzG51sFt5K",
  "/Ma42GLgZ+maa/5E",
  "hx12GBua0AKBiHlP",
  "1yLjyWyJUaYnzYLr",
  "6WSUWrr76FvbbkVz",
  "p8+kFk8SKNffxxXc",
  "CIfjPiYB1q+UMB8m",
  "Ck+YXUSazfNe8YH3",
  "GP8Avx9ex8dnSa2Y",
  "x5isintt7FEIWZNC",
  "0tBuh5MSMoE49nyE",
  "90vj4heJUvvE8XTV",
  "o4/Q2PHjuAd6JJWi",
  "SF8/RVBtnhrLvdvN",
  "Bw1oZZpz1hOZj5uL",
  "5je/sQM988xz/uSF",
  "/C9U+nd391b1/C/3",
  "9eVexyRdR1IHZMEi",
  "HZTsydzO17K1Ru22",
  "pxKJGSjQGqxRKtjv",
  "Eebrz1BqBCM55jXX",
  "XJNefPFFvrbb3zvf",
  "30PxRIpy0Qj1dHdT",
  "IpmkbCxCkUyaUpEY",
  "ZVIRuvm0s+imyy+h",
  "JVItNCaNKEGWW03m",
  "0GISyZlGadcfv7xA",
  "9FaPeS932jzwbr2f",
  "DH/N6BDfJ2t/Z+7S",
  "g8fRQf+eZ2kxjEPj",
  "mcaY5Ft0svU+iKMx",
  "MSTjRCmD/XK4qkRY",
  "GxSvy1Ge+ihHvdEE",
  "zU+naUwiSfHx4+mm",
  "556mZNtYXjR29/bQ",
  "YuNNtK+7p5O9m8jZ",
  "nDljFq2//kbU3d05",
  "qDlF4bkpTptqUe3z",
  "L+j9g87Fcl9f7N8h",
  "DTdjxgxyYHyt9Sxr",
  "xWgvhZy9mpofpthE",
  "IyafS1b0HR3t3CVD",
  "LqDIW0pn0hRLmp7B",
  "MA75HGLPR5TuuPFG",
  "mj1zFsuHsNGYy3OI",
  "GfMQQs5cOOAZmWxE",
  "sl6eMTKN58LkfXH1",
  "ORf9oPAAtzlfvmSg",
  "FSWeN54K4/GIBnpT",
  "WOvP22JDbCkWh0b+",
  "pfGQSmgeeZopytHn",
  "U6fTE/++y3QRaW3n",
  "FnexVIJiqVYuDgq6",
  "6EruJryaQIwtW9+2",
  "0ZF+8JKPJRO1nHO2",
  "YSTecymssp+T+7Zx",
  "Kffld7Xfpxl+Xxm7",
  "KPiUSnPxDrFCRCRK",
  "aU7lIEomWoyRnsuz",
  "IRmJRql//nya9Mqr",
  "FIcqArQlMSaicc6x",
  "RoRiIJowUH3Oig2c",
  "L22KeeQ/ydv0H3Nu",
  "DexO6z+r649o6ko6",
  "jXgqzbXC63Eu1wpE",
  "OCQU73UOk6gEZNck",
  "amJu8/7iFRJNyVyM",
  "UwDwm3TPnUv33XIb",
  "FwglYlGWNOrsWujn",
  "q0LPEecQjByE0PF7",
  "wTtsL4oqsYBRgpk1",
  "a5brrttQkzLahua1",
  "rk3d0TlCBH4VBYYk",
  "MB1w8rTGml9i7cy+",
  "/h5zjrBRhGaMecrk",
  "spTrz1Jbaxtl8znq",
  "mj2Xbvrr3ynan6Yx",
  "KdPiMZ9JszByKhFn",
  "gzPpeS25b/EgLTwv",
  "xOXpZ4pBKdp5LMDs",
  "FewgDJbKo0rV5HRJ",
  "xapdTOBPiNbG2ph4",
  "LRWrgDV5nyiCaIFx",
  "GSHTtSSK57OUysXo",
  "0Rtv5e/Sn06zFAz6",
  "rsCRGcl+ccgVGpuS",
  "M7ftttv6E5SpFM5R",
  "X19zVJ1Lm1JM1CIp",
  "Y6tgwCgUA9ReBKPg",
  "TAxUCRGDQqMV7ycG",
  "p9zCOHDJUax35LeC",
  "XjJ+B7s4CLBSRBSG",
  "vtk3Ay+lJ46ejebp",
  "83ffo0/efIPGJVLU",
  "DuOS867h7YxQayrB",
  "Rp0/XjzNy4Hxg/cp",
  "opk5xL8hjSWKv5O3",
  "FV4PuODOE2P3+57b",
  "up2eaoVsvvSS9ELn",
  "inhRqTD/huNtpRgl",
  "Uin+7vfdcBP19y6i",
  "bLZvUHcu47E0vyvS",
  "Yfbaa89Bv6WcW/Ib",
  "a551dSnBmL+KmpRa",
  "JHc4eTXR+9X2FijN",
  "jXiUTDgzSnvuuSfn",
  "KPEgj3huBA5nZSmd",
  "zRiJH0xEqRQ9cte9",
  "9OG777CXL5rLUksy",
  "wV5M6O/19XQTcdW5",
  "MQYl9Cb3RQB6oDBo",
  "IFfT6OJJmNzT3iwQ",
  "a5YJSqSTjDD7gJTS",
  "gKSSV2xQMOn5RQdi",
  "6PKEJIZsjm+RVTm+",
  "rY2mvvEmPfvwo5SP",
  "xSnT38feFYTZYUwX",
  "GpaFjyXZff3116el",
  "llpqkJB2MzhFxNiD",
  "p8gOk9sTtlT/4jkx",
  "SO0wsl0BXFgNLK+z",
  "i2IKe3w3OvhNUPBp",
  "G5myqEE4GekbiDmb",
  "trIZliqSSuv3n32O",
  "8osWUhtlTdtWeJ8z",
  "/ZSIw1s5oBRhjEbJ",
  "bx4YfyIZJjq49n35",
  "twGvqCn6sR+bdrID",
  "RX/wlMoi0VwnBuSP",
  "TFGh1zksIvmYXgMG",
  "zyNquorhvsnNRieg",
  "aCzHLXBxjUI6z+Q3",
  "36YXn3qGogksfPpM",
  "s4rcgP4oUoSwINp5",
  "5539VARJ25Cxi9do",
  "Wlf1+eEPD3fZbVki",
  "+gY1IbUwNJ29mgcc",
  "cADn36mxqUgfZPYQ",
  "JaK00047+RdcDhfF",
  "IpTJpv1uDdi/p6ub",
  "uubNo+v/dA21QJoi",
  "FmUxdgghQ/qHDTDK",
  "UVtryujvceGOl8zv",
  "iTBzWznkfOYR/jab",
  "JPjjOdmkOp218hAC",
  "57CYeEOM1ApC6Ilh",
  "tjgmpqjZ/AIi6c/M",
  "Yf2BQgRj5BoDlw1R",
  "GD7xGLXHonT7H69l",
  "w7kLvc3RGxke4D6T",
  "X1nYW7jwPn5b5Ecj",
  "JWHgd2+eSUpEsTFp",
  "y4QtBjgQTVbcwiCF",
  "NwlVv0g32GuvvXjx",
  "g1t0qvryl7/MxWa2",
  "R0m8xpK3CcTb2Qzg",
  "e8PQBLaBziH0fiMx",
  "xmkqMMRjcYrAWIeU",
  "T/cimvT8/6idi+ry",
  "FOeWk3lKJmKUYuM0",
  "M1hKTDyGVsMEO0ow",
  "3GaiGSbVJW49Nv9u",
  "xprIFYkByuPb72tu",
  "GiuIlxLXFNHxjBcY",
  "m1Lox6/hVJ4MZaMZ",
  "SsaJF8R5pORkMnT/",
  "TbdSLGaiepJmgPnQ",
  "Lo6ELul6663ne9nl",
  "txU5s2ZIzag11157",
  "neuuV1ETUqseSvBq",
  "XhO003nnnUe33367",
  "DhTFu4DC8xRhHT5M",
  "WAhZQuLI1s6MIfwG",
  "j142y902Hrj3AZr8",
  "3nvUkYxRIh6jSD5L",
  "XYsWUQ4XbBQC5RNs",
  "mEnrR/aOSIGMl2PJ",
  "npKc54Xx8siidrGB",
  "1xkIT7CHgqtUjWA0",
  "vxNLmhRf0+Ez+H+2",
  "zcGFP3IXnhG8lZE3",
  "YpkiTGfmAfX1LKJE",
  "PEHTXn+TnnvwYdpm",
  "j50p152mRBTGuS02",
  "HPmCF429dV4BAfQ4",
  "t9hiC3r00Ue9fzPC",
  "+I2OeIFwHkHVQLyP",
  "IqmFCRthXygdbLbZ",
  "ZtwfHucgurPA4BRt",
  "UvHSYf/PPvuMnnzy",
  "SRZ1fuSRR7h4Q65l",
  "9jVtuEKhRgPfc/nl",
  "l/9Ca1P2JiMXkxdm",
  "OOeN5FYm20+ReIS6",
  "Z86i2R9+QIu3tFCi",
  "JUGxNFGmr59SyaSX",
  "Q+0twEyipVkccj6l",
  "wdw34wb/J6ezjDU8",
  "zPExDYwNHnfWAgAF",
  "O/IYRTwY21ycx+8T",
  "pTSqf7wcTuyLGqGs",
  "NZhhWHrLFS4Eyvjy",
  "aea4+/p6KZqCQkSK",
  "ulHACKWJ1iS9+Njj",
  "NG/6DBq7zHJe5Xme",
  "EglzrplFtgmrY4Hz",
  "3HPPDUrREHlAyT1W",
  "QsEaRLSx1xGoaaiV",
  "LsK1XgumouBCfuut",
  "t6qhqXDbSZwH8Dbh",
  "ojpmTAdrZg7o8BlP",
  "UT6doUQsQZlcjroW",
  "LKQ7/v5PFj+2c7WQ",
  "s4gqbM6LwnNZ43W0",
  "eyQb74WZwJI5E0oX",
  "bwleI3mXA7fk5WfC",
  "E2L2kRwx9oiwtubw",
  "23AdSiR3U7oVxeyO",
  "I17oHN6WDnyPdI4W",
  "j8fpgWv/Qr1zFlIG",
  "kx20+BxUGyQHEbfI",
  "04QBL0ZTM3g1JW8Q",
  "4PeSCl6EHjfddFO6",
  "8cYb6emnn6ZbbrmF",
  "fvzjH9N2221Hq6++",
  "OhdpoAJYfj+8Duco",
  "Xgej6nvf+x794x//",
  "oHfeeYdOPvlk1pDk",
  "gi2vK1OzCG6LAYTi",
  "lULtUO4SxN2rMpSL",
  "5LgLUCyLVIMs5fJp",
  "+ujFV6hv1hzu6tWe",
  "bKGWaNykq8B26s9Q",
  "SyTmeRnNuEH3LB4f",
  "3mMJUw+bo1m4sQey",
  "sAXlgM6tpLFIjmbU",
  "Snnxw+qW53JANN6E",
  "zI1wu5d2A0OTF3gJ",
  "aslHOfUnkkUutmmF",
  "2/X55/T8I/8Z1DrW",
  "KLEMtENEtzSMWXjQ",
  "TdqB2YB40JXqgyiG",
  "I1dRk1HLrvDwav4t",
  "aKfddtvNlztSmhe/",
  "Z3lfH4u0S04dhMfZ",
  "H8g5cmmKQPYk2UL5",
  "1hZ64bn/0Zsvvkxj",
  "Wlook+6jRCTBeZnw",
  "IHDImnsOwziFBmfc",
  "C0t7+nfeJICwm2k/",
  "B1ABi308CRX0SWcv",
  "J8LrJqSOfXCfq08R",
  "juaOIKaKvSjIMy2A",
  "nSTy/SOQmDce03zU",
  "80h6XhjOG83nKJeP",
  "U6a7h6a8+Dq9/OwL",
  "tNlu3yJa0EvxVNLz",
  "plgfV1CRKpXVaOeH",
  "cDAMTfzW6bQRe/e1",
  "YBoUnFsymcu5Bq/l",
  "GWecwaFw/Fbz58+n",
  "Dz/8kK9F06ZNY3WM",
  "efPm8fOff/45GwAi",
  "qI3fD0YoqqzHjh3L",
  "ns9vfetb3Dbwjjvu",
  "oGuvvZZfA6MUv3Oj",
  "GwPSVQV51XbIXIql",
  "+nNpSuSTFImhk6ox",
  "1iLxKHX1d9G7z79E",
  "/Z2dlEKf73yOEmkY",
  "ljEzpuAJjZvxDKej",
  "6OFiVHCnIL/M20Qg",
  "gPzSMuJMU4fIINki",
  "z//pH6v/Pvw5xrvJ",
  "kkcmvsDySeyR9Vym",
  "WeSceuMG1wm4OOE1",
  "jfA499JV/Ep0I4CW",
  "60lTbwSGYZLGtbbT",
  "3M65fE167J67acdD",
  "DvEWM3Hq7u7yDsEo",
  "ReSyOU53WWONNejV",
  "V18dVHXeTKoRtea9",
  "995x3XVjIlqFiD6m",
  "JqGWhub1RHQgEe0Y",
  "tCMuyKJJJzIOUoWn",
  "K7bGIMirA4MHf+aV",
  "V1mRL6i9vX0sg5JD",
  "jAv5TIk44XqeznRS",
  "OpemZPsYuunKP1A+",
  "00OUTdKY9qSZ1KIx",
  "I2MEAxKGKZL8U0lK",
  "wAD1qoGliwe8nwhP",
  "c51R3LR35NC6l6NJ",
  "IubMFarmYm7C5l6I",
  "zmsnaYxRk5NnInJG",
  "X5Mfy9f2dADt38M2",
  "PTNRSB8ZHT/2CHkh",
  "Qj/8F0uwFl8ylaJk",
  "S4yeuPaP9LVvbUGR",
  "RBth9oYxHUd1fj7H",
  "ob9YFFW+xsCSiZI/",
  "J52mCRMWow02WI8e",
  "e+xx79MjZf/9gsZo",
  "uV69cl8vAsy4nsAb",
  "efzxx9OPfvQjeu21",
  "1+icc87hWxiXKFJc",
  "tGgRG4f293IJTcKo",
  "hPdz7bXXZj1J/PZ4",
  "r3oInZejnQnwXZdc",
  "cgItvfSSRJEMAuPs",
  "30NHL7xnqiVKkUyE",
  "cvF+ShAMzzzlMp20",
  "cP50mvvfR2gM/j4Y",
  "lznkPuY4JYTHGgQn",
  "CIVDXv6xp3uLc1p0",
  "bUE0+8XzT8aw/R3x",
  "0oSlm5mzB7lXGJdD",
  "3i17K3k48eN0NOPp",
  "3BrvKa4l0MyMcEoN",
  "dsyYFBsscz1jk2WB",
  "4eGMZimH755KUCt/",
  "Tp56ehZRLJai1kSU",
  "Jj39JH0+dTJ1LDme",
  "Ui0dHN/oaGml+d1z",
  "jRczk+dx/53v7MRt",
  "nPE7yPmJn4AN3Dpf",
  "KNb6+hCEWbgTRz8c",
  "BdyvIqKdqEmopaEJ",
  "jncxNAEuzB988AFf",
  "sBCCkY4SzZJIr5hJ",
  "HV4heLilHRi63mDB",
  "zsGpmAnJxRMtNPm9",
  "SfTCc89RIhEb0Kvj",
  "cBuMzIHkfLvlJOd3",
  "eRXk7InwPJGQVuIQ",
  "F3QzPU+lycUcOPdM",
  "8Y93PnraeQNGpfFI",
  "DiATjTcJsd9zcO4k",
  "a/n5ItHGuDWeW+i/",
  "IMTmiYrDk4kIYjpL",
  "yShRa0s7ZfMxmvL6",
  "6/TKf56kdbfZjvLd",
  "3RRr7RjwgFohYvFW",
  "wgiFN47DmNEobbPN",
  "NvTEE09QBGLZXv5m",
  "o4Pvj0XM3nvvzXma",
  "X//61+mjjz7yvb9i",
  "TNpyRLb8UZBhhn1h",
  "WP7vf//zi9bE49To",
  "vy9+H3gzYWj7qRww",
  "2jh1AOdxns9f5DVm",
  "+nuJ123RPM16613q",
  "mjufQ+CsMeuFn2HA",
  "ceENF9+ZMDW0cW3N",
  "S45ASB5ogC+Cvfmo",
  "cpfHbJxZaZreio5z",
  "Nb1QeDYHw9SLMsDE",
  "he6nZ1zCyDSqD6ai",
  "3hi9eBvjfcU1hY1Q",
  "vm5EKRePUjqLfc25",
  "AAH6XD5G/bks9ff0",
  "0xvP/o+23P3b3E62",
  "o72N8rkMxSOQZstQ",
  "LIerT5Zzq3EaDsyN",
  "ZsgnkhGkoStVRCTL",
  "0IzAkR2JaFyzCLjX",
  "2hX4JhGd4LIjcpxE",
  "VFo8mjIpaqJz44ML",
  "OAbzDjvsQMmkqQ42",
  "F2T0Pc9TLptlSR8s",
  "QhKpJD3z0EO0cOFC",
  "aku1UyIG76TXMhJG",
  "oicnxNp3XhtJ9oRw",
  "aNqEv4zwsydHxPmQ",
  "xtOJvC/W25S8SWyQ",
  "MIH8EudneZXnCKF7",
  "Fa+8iYxRdCAfUwqH",
  "zHsN7rdsipIGtjhk",
  "nXJG3gjvl4DBzDqD",
  "plIXE/H4VIyymKS7",
  "eygzew699sDDFEug",
  "CMrIxdjjhEsWJPRv",
  "CZJLxACTFibOSnkC",
  "ivX5DoM3T4p+4LG8",
  "4IIL6MILL2QPpt12",
  "DgznfbRz42xJJInC",
  "SIGQ0SbNssdJ7ovR",
  "2ehMmDCBvbq2YS23",
  "aTQViGR5ESVFdvgd",
  "P33mBaKefpOLjJxZ",
  "PtehO2v0bgsrxhPI",
  "p/b2RW41b/yafNEN",
  "41D6mnMeNF8nBlrD",
  "mrEsWruFuprS29yS",
  "OIJKhAjH4/1l3Hvt",
  "J/k4vRxs5GhD0ggb",
  "jGVWv4jGKBk1++D3",
  "ePGhR428UTrHaUAL",
  "F86nSM4IvaNpBH7T",
  "r33ta5yiMdAU4Itp",
  "AUp1kLQbsP3227u+",
  "7FJqEmptaIILXeWO",
  "LrroIl9bU6ruUPGp",
  "A6nxwd8YE9VXv7oB",
  "P+bUCfT9YWOJWMoo",
  "C129RIoy/Wl6+PY7",
  "KJWMs3A7cjGRJ2Wk",
  "REQX09O7syRG/OR+",
  "Ni49/UtvshDjFO8j",
  "t6Lbh1C8eY2RNTJa",
  "nCJ3NGC8+h1FfAPS",
  "886I+LtVYCTFBXbH",
  "E5FqMdXxA49xCxHr",
  "lliK0r19lM/1U0dL",
  "C73z6BM0+6MPOL7o",
  "F50UtDy087ikuhr7",
  "rrrqqjR27BinQqJG",
  "QELnMLKlJ7cYhlLI",
  "YqfrCPZvY3uKbaPT",
  "bm9p60fi/cNgZI8G",
  "+J7IV5X0BNN4YUCO",
  "x/xWeEwsXwbtyN6F",
  "8+nT51+iRBRFezmv",
  "CA9SRaaIb6hmCOYx",
  "CoAQRTDFc9yZi8f1",
  "8Fvhewx6X27KIJ2+",
  "TLEQL0a9oiGzIBUP",
  "q2cosu6mJ9qOxQZH",
  "Esy/Sc9zY5ya6xHk",
  "19oTCRqTTLJUGbeM",
  "zaAwCNJtRC8+9ijN",
  "njrdjF3InrWkqCWF",
  "gr0YFz7itxw7toN1",
  "cG0NXBRMZtLqiKk2",
  "dsOG//znP64vO5SI",
  "BrTkGpgwGJrOIu4/",
  "//nP/ao7AC07eK2a",
  "YSJsdjCQ11lnHS6s",
  "WLBgAf/tcXGVc0E8",
  "nHhu8vvv07uvv87e",
  "AXgFknET6pbiHYSt",
  "oLvJBT+ex9KuQBW9",
  "SqN753kYJfSOD7Pb",
  "R3opXKblJLaIP7HI",
  "Y1NdOmBgymtEvJ3F",
  "4sUjY+v4wevhbeJt",
  "SXjHxXqfnofUtMPM",
  "08LOXn59SyLKnqPe",
  "Tz+lNx55iKKJFv6N",
  "MJGjA7PokQIZO+LV",
  "lIprVAejqrpZ2tiJ",
  "8QNPI347MTrtvPDC",
  "lriFXrmhfieZ9OW3",
  "lcp+0euEQdsMvy8W",
  "Mej2Jr+ndLgRIzMW",
  "iVM+6qUo5E0e4/xP",
  "P6W5H32M6iD2MLK2",
  "rF+xLdqVYhjiMfI3",
  "varw6GDDUSIXw222",
  "gel37bIMzgF9TPFy",
  "yoLS08/0jEcjxi46",
  "np4hy9cPrxWpJ40k",
  "zR8AL0gzGY5SIHLB",
  "ns08PJw5SsXi1JJM",
  "Uaazk1597llKtZgu",
  "aLzowe8Fo7YNkb0I",
  "9af7uVDS9sCLLJxS",
  "XUQPV8b6gQei/MSJ",
  "q6gJCIuhCbkjp4ah",
  "yNOUylDTJcbcKo0N",
  "crt23W1nWmz8Yjw5",
  "Q0vOyMqYwd3SCnka",
  "pFPE6flH/0OUxeQV",
  "pRaIsrNGn8mCRGib",
  "u3xISCsyeFJgCSEJ",
  "mflyJKbIgI1SLhAy",
  "HkzxLhrpFBNuE6NS",
  "JFC4y5DX61g8mNJx",
  "SHJFpROR3ZXE9m7a",
  "k5/pq47j9z7Da4EX",
  "TcHQjtESLSlqRYvN",
  "/gx1ROP0/L/upt5O",
  "oyQ2XJgalea20S7y",
  "O6hkbZaxJe0ggXQA",
  "klxMKTgcKg/T/k3l",
  "36VVJRBDXbyhQELm",
  "0m2oWaqCJ06cOEj4",
  "3ja6cW7n4brLmd8m",
  "l8vS3PcmUay712v3",
  "mv2CRJF04fIXdVZ3",
  "L98DGfUMUr8bz9Ab",
  "L/K8lBYJf9sNFfwu",
  "YeLNFIkjHpdRVprw",
  "x6x3LLwQ5AWhhNe9",
  "YkGrc5Fce+DFRDpA",
  "DOeFnxaTp3w2Q7lM",
  "mrq7O+mxO/5N2b5u",
  "U6iUJU6LAVns652r",
  "kDmC1rD8zmjXC6+n",
  "Ul2kVbZcJyCH5siW",
  "Xq5mQxMWQ9PZqwlt",
  "zUMOOcTvjNAshQrN",
  "DrxxG2+8MWVzSJmI",
  "fcGjlMn0o/SaJ6n/",
  "PvDAQEgu08eC7GzU",
  "eZ5KNhbFkyl5mjzh",
  "eB4SDpUbb6fp/gG9",
  "ShQjDExAkpvJoXVP",
  "o5MnGjItIf0OIl7n",
  "EM7F8nLHuKd5QbcR",
  "W8fPn9SszbxeckQ9",
  "w9PS9evq7KHWeJbG",
  "UJbi/b2UgIckFqFZ",
  "r0+iGS+9PIQWJhJT",
  "zSYdR8RQgsGJx6ig",
  "bJYcTQmPD9VRRbwU",
  "YnTbotgA92FEQTcT",
  "OpnYkCsH7zvy6qSD",
  "i+15l+8txlajg++P",
  "38j+3vIbym/CQmXc",
  "XCBK2b5+mvnqq6wZ",
  "2wYvpTd2EvYijRdq",
  "sllalzKWPDF3bCb0",
  "PvxmuhIZo5WNVCvf",
  "0jcUJZXGbw0r2pre",
  "MXjXB7meDERGjNat",
  "33vdU6Lg6wY+C39+",
  "VOugFS7mNbwmhkhM",
  "nJJRtHqIUEtLkj55",
  "+TWa9/nnXDyUjHLN",
  "OyViKCKS/OEIrbX2",
  "GnwemsfmvGqWhUwt",
  "MVJwZtEov/svfvEL",
  "15fvRg1OmLLQ70Aa",
  "Jv4+QTtCg+5vf/vb",
  "IE9CGCYrpXqgGhht",
  "1ozmoOmQE0dI3Pvb",
  "Z7NpniI++eQTeveV",
  "V1k8pS0eM20c4VPM",
  "pSmKCQVhaO5PjsnI",
  "XNSBFIBLNazxgIpk",
  "itdLmYsUBryTgKtb",
  "vef5fbyKV6OM51We",
  "szFq9h4An2099rr3",
  "cCDNqjb3F1GYoL3+",
  "QDm/Y5B5zJWllKAO",
  "eG8yPewdicWT1I1f",
  "ZFE/vXD3XbT8ll+z",
  "xMEHv7fkELa1tlFv",
  "b78/+W+yySZ+i89G",
  "R9pCSstJkTnC74Vz",
  "Dr8HesCj3R9SCnCL",
  "TkGY1HELIwqhcDHY",
  "kdoBfU28FpXmM2bM",
  "YN3Njz/+mCZNmkRv",
  "vPEGTZkyxf/tmwHk",
  "WEt6SzKR5DFnzq0o",
  "h4GzXmFLJBel3u4e",
  "mvHGe9yaEuc29DVN",
  "61djZPqLTC+lBQYd",
  "P+aiPntsSrpMwG/s",
  "teDiTj+8EJUqcfOZ",
  "3PGHldSM4Wk0baVy",
  "XFQhTNcg7haGSnpk",
  "kXsdxYzOhHkeSktS",
  "mY4AuulMZNIHEP7P",
  "x4kyeX53o9kL4zUW",
  "o3nTPqWPJr1HX1l8",
  "GWqLt1B/Ns0GtaQh",
  "gLFjxnJ+9eTJU/1K",
  "6GaJStQSu1hQ/haX",
  "Xnop15U4AJvn79TA",
  "hMnQBKe5GJoAF2x4",
  "N21BWsmHKnysRmj4",
  "Gah6tturmRwuPL/Z",
  "5htTS0vK97iZooIs",
  "t11c1LmIWiMpyre3",
  "0QdPv0B9ixZREjlw",
  "8DZxGNgYV1x0g4kA",
  "leEcApPwtSkYYE8m",
  "X/CNoWhyNI23w1ST",
  "GwPVb3HHUiZinHrG",
  "JT/nhfRk0rHysXz9",
  "TbZsEdL3noeOEgs3",
  "e8aneUtv6jPal2Ya",
  "zZksS55ETNgbE2AC",
  "oTRKUqpljBGg98Ts",
  "c215mvzQvTT/p0fR",
  "ksuvQflIH0VyMcrH",
  "2ijW10c96Pue7OBu",
  "LJgYjaZhgjLZPlrt",
  "SyvTyistRx9++Mkg",
  "8fihKNdYqsQYtce6",
  "nRuJzTYgQWFYvLBg",
  "ByBPFWLtW2+9NXt3",
  "8RjFh/Yx255Km3Hj",
  "xrFhKgwsiEzRFQxR",
  "pAE9++yzLHf0+OOP",
  "szEqRq2E8aVAyz6u",
  "wu/rYkgE/X2Cfv9y",
  "/z5trSlaeaUVuK0k",
  "foueHpP/itgzfroo",
  "rKv+Xl4gZfNdNPvD",
  "d2neW69RSzZJ1Bah",
  "ZAYLMxOWNqLnUohj",
  "xpTkWssIEmkiv6Wl",
  "v5AbGuRVmy9qjF6/",
  "DSXrYJqWkcgbzbGV",
  "63nic0QZnGNe4Q+r",
  "3PJ1wFxX8IkwntkQ",
  "jMHjasL5UBHFWYPr",
  "BAvLQ93BM6ahZxvN",
  "mKgLzFn8PuZaFOP+",
  "7+898Sitt+UO1EPd",
  "FOtJUrSjnaJ9i4xR",
  "ib7vFKWttt6Wnnzq",
  "aW5OgTB6xOXvW+c6",
  "m2EAC00Zt3K9QYOG",
  "e+65J+il6xPRGCJa",
  "RA1K2AzNHtT8ENHF",
  "QTvCk4DOHW+99dag",
  "yliZOOyLcLN4DOoZ",
  "mYjt1mn2xI/+0vCC",
  "IOFdPG7Yp7un2891",
  "o75eeum55ykLr1IM",
  "l+YIJVtaKJJP82TB",
  "xTsoBOLQGDwF4sk0",
  "1eAmD9ObKEwTcf43",
  "2IDS25zzy7x94ZVg",
  "49gzRsFAAcDg5xOW",
  "OrTnK/F7iEf9edD7",
  "7p6xafwgOZ7rcDxm",
  "sjDHxYYoPzR3IOMS",
  "SWcpzTJPGconIpSE",
  "iHsmRwvnL6KPnn2R",
  "Ft9/dYrlTJgSYTr2",
  "orB+aITyOeMVsccK",
  "8jS/9KUvsaEZdgqr",
  "4+2wfKFEETyVMPSA",
  "dOYRgw4dfXbeeWc6",
  "+OCDWS4GgtgSRpfC",
  "HQmPiaSJfd7aBuFQ",
  "hqH0RMf5i5Z1yIM9",
  "7LDD+DMg4QaDE52D",
  "IPqMc1ok3cSgFS+V",
  "PJYihLB7raB9C+Pb",
  "r4iG9xJj2Fvc5Hhc",
  "51hHEg0X5k6aRPkM",
  "tGHjXGUuckGenWdU",
  "HTxPppESM4Ybjz3R",
  "0bQ6d9l9y4cEUkE8",
  "lLCUM6Yqj0sx0KIo",
  "UvI6ArHb03QIQqQC",
  "3k2zEMR9NFaA0WmO",
  "jzV50RWIIxXm3+Bc",
  "hcg7Xm8MYemRLsdu",
  "riMS8GYB9p5e6s+k",
  "adpHk03xkCdgz5/j",
  "/f1NhyGiNdZcw1wD",
  "8bl+ipmGz6uJPUZt",
  "RYl77723lFzN+6lB",
  "CWNy0CVEdJvLjq+/",
  "/rqfdC95ETKxyAVY",
  "vZn1gQmFJymVSlA8",
  "jokUK0N4omI0fvxY",
  "lu2AkSnyO2I89PT0",
  "mMVFPE593d306nPP",
  "UTIWpVQsSq0wXr0c",
  "RM6zYs8DpE68JHzv",
  "MYemxDvpXeh5MvO0",
  "NyWf08ghedXeBb2M",
  "pa+yqRj3inkkJws5",
  "fcgT8zTzpPDH9DGX",
  "x2Zf5GsNklqRPC6/",
  "+GFAEkUq4vH+HdDL",
  "JCOHwvldUfMbwFOT",
  "7c3Qa/+6i/r6uxGT",
  "g/uNuyLBiYTpVwpT",
  "xBiTCyaMsM03/1rg",
  "HB0WCr1+MvbhXcD3",
  "QvEY8ibFsLSvD6ut",
  "thpdeeWVHCm5/vrr",
  "aauttuLvzw0A4nE2",
  "TvEeMFDxetHBFMMT",
  "Au+dnZ3ckhKdzD77",
  "7DOaNWsWh8vnzp3L",
  "G/YZKk8T1y58FppS",
  "HHXUUSyUj9D6cccd",
  "x7mednU2kP0lTFcP",
  "OXioOIehaXuUpWgK",
  "v0VvTydRNkMxMuP+",
  "81depWgWepJZSmQz",
  "Ro0BnmnJafZyMU2+",
  "pKg1eDnLIg0mSg4y",
  "Fots9v6SSz1YCcKM",
  "eXM/NpC/yePSFCoN",
  "ysuUfE2pQvdTamyZ",
  "M29R6ilYSH42R0qs",
  "+/g3pMMkWlvos48m",
  "UzbTR4mYqTQXOS4Z",
  "t9iwgEn4iyKd/0YD",
  "exzK4hPnN643d999",
  "t6uh2bCEzaNpFwbt",
  "7bKjhJpsz2W57dKU",
  "0acwX1A2hMkxqaLA",
  "QjxJ9mICXiaesGJx",
  "mjNlCs2ZPoPaUi1s",
  "EbZFY9TZs4ja4H3i",
  "EJKpGucCAVSri8eS",
  "u3jg1nhG4HUwobj8",
  "QMtJDrVHPd1K7zXw",
  "aHr5jpjcgPF2ejIo",
  "kvfJBUfi5YJ1N9B+",
  "kh9zhM7k9UTZK2Fa",
  "4ZlXe/t6/8fGCYxD",
  "rzIeHYLYYZMk6oNe",
  "XixKyXiC+lDgg1ac",
  "CAunEvTJSy/RvI8+",
  "pMTqa1ESoTro9MGQ",
  "ghGOz/UKVtiDxuFJ",
  "83fZbLNNjQct5B4z",
  "OV/s0Llt1EihD64X",
  "rFLQ0sKGIQzMU089",
  "lfbdd1//XBOPoWhe",
  "SqoGfhvkW0LUfebM",
  "mWxUTp48mY1T3OLf",
  "xQi10z7kfRZbbDGO",
  "xCD8js9HaB25nTAm",
  "0ZYS3lQYw/g85H1C",
  "NB7G5g033MD38Ty+",
  "E44b4D1EginsebRI",
  "O8D36+03Rj7nHHse",
  "QjaUKEtRnMQoPO/p",
  "pwVvvkPxaIIXXrGM",
  "UXVgo81LYcG4w6KO",
  "tWw5pG4+x3g1OZPZ",
  "y7PEGDf+wmJIz3Ly",
  "PJTwWHJ/c3nfnJdz",
  "aS0Q8BjXCoTVTejc",
  "XDMydlsedpYaL6YZ",
  "w8ZwlDxO85leNyNP",
  "11dywDHGYbBCNzOB",
  "OS4Zo88/+ZRmTZ1M",
  "E1delWJIJ2jpoERL",
  "K58beDucB5I3PHXy",
  "lNAU2zU60q0Q8xH+",
  "FqKsAPnFyy67jHbd",
  "ddegt9iKGpiwGpqz",
  "iehEIjrfZefzzjuP",
  "TjjBNBiSyaaZxJAb",
  "BeMhGggReqn5tNpq",
  "q7DYs50KIX9bEe/H",
  "44/efIMyPZ3UmoTU",
  "ETTootSZyVI0hc49",
  "XkU351CZQh82KDEh",
  "GfvMVJ1aRqZ4LTCF",
  "GNF3EyozeZq4P1B0",
  "EPeSGEUEnsWZuSWd",
  "yZGKIgHLA5/j4+do",
  "ej2z/ebnAj5pQIeP",
  "Mz65QMAYm/i18Hb9",
  "uQxPVnxkOaJsGplg",
  "UUqkoLuXou6uTvrg",
  "4cdoqXXW4YreXDZN",
  "+WycorEkf8fMoMIB",
  "k5qA3xVFWAgfz5u/",
  "kMJOYSW7vXgRb4+c",
  "PzDurrrqKtpjjz38",
  "8LkYleKZgFGJEDb6",
  "R6MVJYp4sME7Kb2k",
  "xTgvjJ7Y4XM7bxyf",
  "JcdjSyDB6BwzZgx7",
  "/tD+c7vttuOiIxgM",
  "6Lt+6KGH0s9+9jO6",
  "66672EsCTz68q3Ls",
  "YU8PwveIWELigMct",
  "2jR6hVccfswT9Xw+",
  "lzo/nUFxhM1jpgtQ",
  "FLmd4kH0IwuIOEBA",
  "Xbp4DeRomnxp+SQX",
  "LUkvJ9N7bV4MTo94",
  "HleKLI9DDvdznqXn",
  "QUcLyQheb5rX8oj1",
  "QuJcSOiNW1wicMvp",
  "KhyWN49xrUibRMoB",
  "r6e3wM1y9ISotz/D",
  "F5feri76fMrHtOyq",
  "a1Bv73xqaWmlaMw0",
  "E8CLIM6OBQ0WUNM/",
  "RWcrnQNHAxiZMs7t",
  "3HCMz0ceecTlLf5L",
  "DUxYDU1wAREdi2tU",
  "0I64EJ977rm8erCT",
  "7kHYL8CKwTYMzK3I",
  "oERpw4028IX6MaAl",
  "3CZhdPZEtXbQB6+9",
  "7hftoOsGRTLUnkqy",
  "jiXCW5KHyZ5JliDx",
  "Ovdw3uXAJCbhcPYw",
  "SIiLc6EGvJ0mdIdj",
  "RYs4eDlNL2ZjVErF",
  "uXQwR3h7wONupqNh",
  "zk02QnPepDeQ2SLT",
  "BYoF2KPpGZ+cI4bz",
  "HTMVeh/DyMxlKJrJ",
  "UTJuAnu5TJ6S8Ri9",
  "ce99tPkRh1Es3kbR",
  "OFbgOUrGYCgR5bxi",
  "FsmhE5Fy5NatvPLK",
  "NO/V1yns2FW2tval",
  "7e2DN/Gss86iY489",
  "lg1JO5cbOZJPPfUU",
  "vfLKK2xYQsEA4W9Z",
  "yEjBjyT8A3mu2KJ2",
  "oCVg3nievH1lcsJj",
  "FAJhgyGLziK4nsH4",
  "RCvQb33rW5wvCs8m",
  "jOPTTjvN957Ie9ST",
  "hibnDXr6tzDaWAs5",
  "a4phMhSlzg8/oWwP",
  "wsNIMTAFcxJWlkI9",
  "HrOsa2nGH7dq9HKs",
  "2avJew2MOc6vLIan",
  "f2t6mRvDMeq/zkib",
  "sWEZRZ4lPJvGSmRT",
  "0nutGKHsu2R9TCBa",
  "EcabCQ8lG7WecLup",
  "TjfXHPaMejoVbIB7",
  "35WN0zgkRs33m/3R",
  "JxTbIU59yE9PtVO0",
  "zWhJ41qEc70lEuU0",
  "jCcff8J895BHIxoF",
  "RCPEwSXXHeRbO/IU",
  "NTBhNjQlhP5vlx0h",
  "JSIeAgmlg3q4CCuD",
  "JarwN5NJFCFbrM7F",
  "G2VrD8pgllDnh6++",
  "ziHyGFSVMygqIGqN",
  "JSiPkLuvhedVqvKk",
  "hVC4qRBlg9PLyxSp",
  "I6loHTBGYVB6RQl+",
  "a0kzScHjiaR/9mB5",
  "BidEnOX8Q54Yh+r5",
  "sTEmuerUPz/FqJSC",
  "IP+XMY8jA11pTMDf",
  "BAONbAqxFmAOM3U2",
  "xy378ghBRuF1yXIf",
  "+L5onGa/9w7NfPl1",
  "WmmbbYl6+jADcV5c",
  "PySfPEFxUy0JsXLz",
  "uQh3fvWrX6VX6sDQ",
  "xHkgagVigIlnFt8N",
  "XVOuueYaWnHFFamr",
  "q4vPp2eeeYbuu+8+",
  "rgxFCBz7iqyRXUgE",
  "CkPhoLAH+lBpO4V5",
  "o3YXJtvQlcINgM+E",
  "kXvLLbfQbbfdxh5Y",
  "eAX32msv/g54DpNY",
  "vURtWEPT8ybzYjFl",
  "9EVhUXFxDIpV4jin",
  "4zTvnUmcsx3F7x41",
  "Kg4Yh9Kpixd+GLee",
  "MScC6EhRGSjCM4tC",
  "41PEwqx4OQJ7HD0J",
  "M44LsA3oGYhsGGZ4",
  "KJrlI4haI9F4Hb06",
  "cU/QzLvl9/E8qrKQ",
  "lusJPxgoCsIiF/v6",
  "RU2e2Dtey0oEfWlu",
  "Nzlt0nvU248Cxzx3",
  "P2NPJlJeokaXFMez",
  "4YYbeuemqY7XabC6",
  "SJqXzEUwOs8880yX",
  "kLmghmYNuQuRcSI6",
  "yWVnSIVsvvnmfj5V",
  "vVyEFbuNn7QDNFf9",
  "9rYWLgQSA1PCjnZv",
  "WYQnuhZ20dQPPvDf",
  "L9vfR9loimK5OHsB",
  "jYGAcLbnffByNE2+",
  "l4lr+ft43YHMvw3I",
  "G4nGJhV4PP2cUvZw",
  "5DlEKLqaJvA9oLtp",
  "0jMHFkGmGN2TNuJ/",
  "tHIy+XnvXfDPnlHC",
  "4TAvv01uWyhPacrw",
  "f4lYgieo/nQfpSlP",
  "rahaRdlRups+ef5F",
  "WmkbtKnzjORsjiKe",
  "4W7CuQnq7erm34er",
  "++NxDp+HHTEqsUAR",
  "Y0aqwnH/mGOOod/+",
  "9rccbn744YfZuHzw",
  "wQc53xLnEvYRIxCh",
  "6KG6/cjCxs/Rsxaz",
  "xXLhCp8fygi1uwYV",
  "KmjgFvmgWEzD24rn",
  "4LHC9t577/k5YWEG",
  "qS/ANtz97xgzXkmc",
  "7Ph9F06dTP0oeMll",
  "KR5Nme4+OatQz8u/",
  "lMdSGCdtZo1XU2TH",
  "JH2l+O8DcS8sMkXm",
  "B2FzvJc3SigPTyZr",
  "2HrKuGzo5vh18HLG",
  "8mahZsTM8Dpj8PL7",
  "ecdiPJ7efXgs+QfB",
  "dxFtCkvfdqDYnXJR",
  "nBdp/g2SLSnqWbCA",
  "Yok4p7REClqlopgS",
  "5zgUWcx1Kfz51Y2A",
  "jF8pNEQEQtL5HHit",
  "kaWN6sHQBCejKNSl",
  "OAj5Tb/73e/opJOM",
  "XSqSJS6C7mG/UNc7",
  "wUa/yVnLZAZ6S+Pq",
  "i7y1lVZa0TcyoSUn",
  "hR2o4oV3Ef2AJ3/6",
  "PvXMmUctrei80UfI",
  "oOOeGpBMyfRQLBr3",
  "CnUk6d7o2pkuIgMd",
  "PaRtJHs5pVKdu4sg",
  "fIZcTU8w2nv/eCRL",
  "ceRncZjOvN73enkh",
  "P0x6yUjCFPxETS4l",
  "DGAU/nAuGB+Uyalk",
  "f6nX9YR/t0iOstEM",
  "dxaB1iCKEjjpP5Km",
  "LLycHGOPUhpizzmi",
  "FIfPIpRmj0jcvA88",
  "H9E0UbSFpj96Hy38",
  "8eGUSCQp2ZOlflSz",
  "onCoH4FBY0jFY0nK",
  "ZU1JU3d3L2219dZ+",
  "svtQyDir5fiSPuVG",
  "o7GHPbEwaOC5RA43",
  "ch5PPvlkNi6nTZvG",
  "YXOcU+PHj+f9RO5o",
  "QM81Oqh1pC+hZVHo",
  "xRyKof5NDFo7r9Ou",
  "kLd1gCWXUzy0+F4A",
  "YX7X0P1oIdX9hdqe",
  "eH7piUuZ3xHpHtwS",
  "MUe5fJbiyMnGuZeM",
  "U7YvTdF8N3VOep/G",
  "xmFgQ5UhTWMjKYol",
  "E1yFDlUF9gDmvCiF",
  "dOuKYYGI1ZjnhWTD",
  "0GRMYpWGz81Fszye",
  "OAcZ1hvC1lEU3kEO",
  "DdEAjHGEzU1xnin2",
  "wWfFWfAdAutxGJ9s",
  "UIpoO4QcMIhzlI2w",
  "Erv3rQf0c7lwj8Pt",
  "YmSafFCTy21yN1kN",
  "w/Nspr3rC16d8XI3",
  "E5Skvo405fsjtPDT",
  "adTd2UVtOaK+bI7a",
  "oDvc1kF9vb0UjWaQ",
  "oE0rLT+RVlpxefr4",
  "Y8gheUVHRQhK5bTT",
  "yof89zqfX12Pf6ii",
  "Y5zfqC3I5cw1cJll",
  "lnLNyyypK2I9E0Z5",
  "o7LaU2IVgc4I4tUU",
  "KREl3BSGFzkfMkpc",
  "bY6OIhL+lMlXKtDl",
  "dZ/PmEmZ/rQvWWTn",
  "e/Jqv8CjwY9Z127g",
  "Isti657nwZhvAx5P",
  "eDf8kBxLI5kQuVSJ",
  "Sis8Ni6R/8k5oBLy",
  "wzsYAzOag+Ea82RL",
  "TA92hOQ9e5AlXLCZ",
  "6lovSIfOP3gNvD48",
  "WeKCZnLXIHwEbwof",
  "v+f1HNikep0ols1T",
  "ri9Nkz/8mOa8/R5X",
  "z8ZboFDoXTiRCweN",
  "0RhRPBFl0XZsLa1J",
  "Wn6FZdkgE++e3Nq6",
  "kGEARpgYmdIO7kc/",
  "+hFHOtDlCPJFH374",
  "oV+1jUkDXkLkdssE",
  "Ip5LMZiGq+q2zy+R",
  "P0L+Jwp14L3DhqIM",
  "PIbnSdotigYsEM+8",
  "pPwA+TzxstqGpxi7",
  "MKjhxZdCOPv1YZqs",
  "5TeSqn/73Cws3OIi",
  "u1iM+ubNp0Vz5hnd",
  "0micx05ffw+HiI3A",
  "uRdFsFo+StoLf6Z4",
  "or0RLCFp7knO7SJj",
  "pjGDpyiBXG48B8OT",
  "F4k8xtFrHPua14ue",
  "rcgRydjyc679TkWD",
  "PYd+MwbvGiPXESkQ",
  "lH8b/B5ffL05x9Cd",
  "wYRn58+bx6lBqdY2",
  "X3bL/r3xW0thWSzK",
  "vYMq+FduXuR3Fskz",
  "cXiYFr6ItOVp6aWX",
  "pKlTp5XqSHuWGpxw",
  "XKHcqtBhbF7lsjM6",
  "bohGm8o71AdSwDUw",
  "qSL53SweZEIF8GhK",
  "mJMnbG/wfzLpA1MU",
  "1JIaaCeJ8DAKYrjl",
  "hlX043X34UnDC3Gx",
  "4YjwnPe8VLhKjqYp",
  "EhLj0Rh5RlvPPI9T",
  "TIxS9mL6XULM47hX",
  "1MBeF5lUIl7mGIfs",
  "oM5nwncAbeoAT365",
  "GMxJzrfMxFAyYPaL",
  "5vC9jMfNCDzLeW6q",
  "6iUAj89qQ8gy0UKd",
  "i3rp8+efp2U22pjy",
  "/RmK5yLcJg+/k513",
  "aEcBUBCEDfqQ8u/C",
  "QFvL2iKeTIx5kSlC",
  "1faNN97IhiSMPSkm",
  "s3Mt7bxguS/fj4tU",
  "vFQN8dbhXIQRid8D",
  "CyCkFUC3cK211uJ/",
  "Q24WJn/j5TChYLwe",
  "n4vjg2ELjU20n5Qq",
  "dhQB4Tm8Xvqsi9Eg",
  "xUM4Fsk3xS2+n3xX",
  "qYCvNfa11v598b3E",
  "Eyv/ZtJExODMQJeH",
  "vYvzJ0+lrtnzeL9k",
  "NEIpNBjA+Yx+3hg3",
  "qDL3GijweOZxCY+l",
  "acYw0BHIy5fmA8N4",
  "QDGgGXv8lJ+mEuVD",
  "YaEkRA+4LaQp/hHd",
  "i1zUSJplePFnvJkA",
  "1wEjmG4SPP3rhudA",
  "5EYLeB2n00izhYHf",
  "AOFzeDmx6EQHIckL",
  "x7WIz0BPX9PUHZnF",
  "LRbS8+bMpe75C2hs",
  "cnF+AacaeQtx0wgi",
  "4kvCEb04mqdAw1J4",
  "TZCiVIx1gHG55JIT",
  "aObMWSNJDWx46sXQ",
  "BH8koq8T0fdcdoau",
  "HZL+7fCXEl4KvWVo",
  "nZbNZbkQyA5V2MUT",
  "spDAc1MmvU/ZTNqE",
  "ufwiHpnUBjqKmPwu",
  "E15DOEtyMI22pifQ",
  "ToU6mqbYwAime1p3",
  "XjhMNDXN9GRez54x",
  "v0rdTDaRaMaEy3Pw",
  "TGLygmdS+jfDUekZ",
  "hijq8SZMNopzKFiS",
  "Hbn5pKl+zcEQjLFn",
  "Eh1HTHcUE7ozITkv",
  "39QLeyVgSMJ478/T",
  "Bw89TOt+/4fs5kE+",
  "aV9vH6WSRgDaVPNj",
  "3h/wrCUScVpnnXU4",
  "H9AufrE71NQaaTMp",
  "Cxap5EaIHCCELtcA",
  "8VraoXCphhYPJXvX",
  "+vrYQEVhxXe+8x02",
  "XNEpCUYmDCf5DUTL",
  "tZhXT/7Nzv3EfRF6",
  "f/XVV7niHRvC4sjJ",
  "tLtlifEpqUAwWoEY",
  "wLWmME/V/t44Rhjg",
  "6LpjohK25847n/D7",
  "x/LU/elUyvb1cjFa",
  "JJPm7kAo8EPnKl4I",
  "Sncu3kxBn3TSMXnW",
  "4vlEUwLP+MRx8fMR",
  "LzRucidhWJpe4kbw",
  "DOPKjCXPW+n1I8dj",
  "GIKQMzN+bTPeTY60",
  "GbasbsYLXNP9x8vO",
  "NH2+WJnCFC+ZK85A",
  "a0xTKy9dwkQuaSDS",
  "Lc8jRxP5mC3JCPV0",
  "dlHvwvmUXXKCKRZC",
  "7aO/OBlQXEEBVhjO",
  "jUZg4FpopKSQaiPn",
  "OK4T48aNoVmz4A9z",
  "5o1mCJnXo6FJnq7m",
  "QS4hf1zQ9ttvP67M",
  "DLuYsWJWhDJxy4QF",
  "m3OVVVYZFDIHg0Jw",
  "npHx+ZQpxpOJ4hYu",
  "dDF9ycWrYTwdXu4l",
  "h69MjiVXsXrhNc7F",
  "jAwYmdypx+sMJHJG",
  "cU+yiFveeUYmwm/w",
  "JvJ7GbvWdPcQsXcR",
  "YpYORN5rudkdT05R",
  "Qjt21sUUTUD//b3f",
  "J4ruI8ZLGveMTBQm",
  "5GGwck2tCKGYKlgW",
  "l/aq4/HpbakkdSIc",
  "GY/TtDffpPkffkSx",
  "VVeiRBbGS447FPHv",
  "jInbu6iK8YTfDaHn",
  "O++8c1CL0MIq6loi",
  "hiOMPruABiFE6F7a",
  "IVzsN1QvcTGS8Br0",
  "N99zzz1ZXgipOGLk",
  "SStKMUzF2ylFPPZm",
  "Y2vAysIX7wfPEzZ8",
  "3je/+U1+LxjITz/9",
  "NP373/+mxx57jKvP",
  "cZwyPuyK9TAWPRbm",
  "ncLrA8NcfgM79cIM",
  "WnPOwihbOGWa16M7",
  "QUkep9wx3GhOehEI",
  "E7bGuPJC43bHHb8R",
  "wsB9fn/uAOat17gF",
  "pPd+EmVgw9EYpCK6",
  "iZ7nLPUezVM8O5Dl",
  "yAap2cO8xutICaOX",
  "8zC9hg2enKanEOG1",
  "i5QCQb940By/0co1",
  "8koD4X/vvZA6g4U3",
  "57hCKCJD1NtLLe0d",
  "nC+Tzw2WJpPfHY4W",
  "KYQKEqxXiiNjV6Im",
  "Mh/ht8a5Pb90neEj",
  "iWg6NQn1Zmi+6a0C",
  "rg7aESHX888/n95+",
  "+21u56aEGzs/zoQa",
  "EQaPs4ajnQ83lMcE",
  "K8pFM2fzxdgk78NI",
  "NKEpSYM3uZeeUDsb",
  "YAAh6IFJig1KX/rI",
  "exz1vJ1WgRB7KHmC",
  "M0U7EipjiRVPTBqf",
  "I15NY8ya3sRG2kTC",
  "ajErDwvhe+912I+f",
  "h4cFRqbXfZmbMSPA",
  "bnwqKBQwhqTxtvqB",
  "c09/b8CDkqO+3h7+",
  "PZjefvr8tddpqTXX",
  "5GmzvR1eMXOsZhKH",
  "MQPDCRMXPCQ57skt",
  "BoJICNkyP7VGwtq4",
  "RW4kjg9eBxiZYpxh",
  "gpCKcslrxLkGQw+G",
  "50477UQ/+MEPuHAI",
  "7RIlbC39xu38Sts7",
  "akdNxJiyw/PyOfI6",
  "e8EkuZvincd9eKJ2",
  "2WUX2meffdjIRGEB",
  "JI2Qa4r3Ri4ovkdY",
  "0haGwj4ueDPb2oyh",
  "ybnASGWRrj2iC5vP",
  "UKYvTfMnf8KhboyN",
  "9mSCEjG0UEVO7cBY",
  "kSIa09nLeCqRl83v",
  "51d4e7mQntYlCxHx",
  "GxhJIjZDvQRm48E0",
  "t9KtB6krXEDkp9EY",
  "6SIz1s1CEAYnf1cx",
  "J72cbelp7gfZBx03",
  "rk8Dwu3sG2VPq5E2",
  "8j2ZnArgHV/UnF+x",
  "DJcesRTUojmzCcKj",
  "2SwC87CCpWjNGJzY",
  "XwxNRIa0TKE8Csc3",
  "wO+MRRRSc0rk+GbI",
  "y6xnQxP8iYi2IKKD",
  "XYzN1157jS/MEmpS",
  "wonkvQzoHpriDrSu",
  "k+dlsBdOsAg/ds2f",
  "z/mbJmE/x5WtfoEA",
  "51/C6ycyROJZGMjF",
  "ZCPPM0xNpxHPaPVC",
  "8P7k5uVySlgexhiy",
  "JznP0/M+mF7FJvQt",
  "XotEDq0nMUEZjyhC",
  "39JmzigYmdck2MNi",
  "JlXOI/PmwygnbZrZ",
  "AlNLJmp8JFwt7xmf",
  "JmPMCNFzMN3LAYOR",
  "nMhkqSMWo84spKOI",
  "pv33KVp9z709Dy+8",
  "YsYbIqt0O2cW9zFp",
  "IYwMw028z2EJmwPp",
  "6oNbufCL11K8aJKf",
  "aYe/kMe22267cUU6",
  "DDzWK/TON2nxKN5L",
  "MTjltfgtkGs5depU",
  "1uDEvjgGKULE+Svt",
  "JlFMhfswYPE74hYG",
  "Me6LpJLkf4keKO7j",
  "/Idn9aCDDuKQOgTb",
  "r732Wv4MvB9SAsKA",
  "GNaF4LvgO0oumzyX",
  "ZUNQUgMyvIDL9PXS",
  "wk8/pUQ0xlGJBM5o",
  "vG8Gv0l8YLHHKSqm",
  "M5fkW5vmCJw8YnIb",
  "fe+hcTeyVBEfHs5Z",
  "M4b5n7zn4N00URBP",
  "BYI9maiKNwNQogT4",
  "PAmKi0eT8zu5Ta1c",
  "T8T7aiIarCfByhTm",
  "+gDj1a7P4fxNVnnw",
  "rj9eiN6Xc2Inq6f7",
  "i3MxnaXZM6Zzeot4",
  "0rMZU1iWSAx0IMO5",
  "Y9rHhmec1it2upZ4",
  "jjFGR2BkXk1EF1GT",
  "UY+GJjgEGu1EdIzL",
  "zrgYa55m+BED0ky0",
  "MZ6IIVQtfzsOq3vC",
  "1jaYpHu7e7ycRM+7",
  "AbWB7Be9n8bTYbwf",
  "fsWneCgltxIGInte",
  "vJ7f3sdxsj8bZIM9",
  "FUCMTPZqegLJvtwR",
  "Blo2TnlIj8Db4Xkt",
  "WVAaUkUsseK1woTs",
  "ilcMZCZBVKHjuE2F",
  "IwqC8EFcec6pApBJ",
  "grKC6XwUxb/51bGe",
  "hxKPMzkam2yjhbEs",
  "ze/to6mvvkbz5y+g",
  "JZZYjJKxOPX0m7C5",
  "FJuYriWmdjcej9FK",
  "K63EguEIR0uukhia",
  "sjioJdL3HmNdDEW7",
  "5aOEuaVyG0YlWjrC",
  "g4lQOQxEkTiCB85+",
  "DbyKMCRff/11jpCg",
  "YxAeo4AHn4fXDoXt",
  "0RRDTML3ML5QFYyc",
  "Txi7MOSRjwzNWBQb",
  "iaGK18FIw7FBN/Py",
  "yy+nU045hf70pz/R",
  "1VdfzYssfEc59loj",
  "UQe5le/B3uZsmu08",
  "yUPk67F4iTJZ6l/U",
  "SV2z55i8yxwWP5AD",
  "i1FrIg5rzQ8zc5MF",
  "1sH1Gizkcc56nnXp",
  "Uc7pJ16xD4/LmGl6",
  "4HXjMW0gsUeONTJ5",
  "1HAI2hQOyvhicli6",
  "eSLuHI4wPc+N7qZ0",
  "E/pixbjpTSQGuHm9",
  "Ca3ToMYOvuandUGR",
  "OIH/PCIehPGZ5lSX",
  "roUL2PjmfWNRyqQH",
  "IgyyUMQ5JrJwkJJS",
  "Ro4ta4TrB4x4pLiU",
  "yN+bKS+zEQxNcA0R",
  "/cRVogmTA8Kwdv6Q",
  "3W6v1jqAjU6h9pjg",
  "h8JjUe7ggyIgM//k",
  "aZlllqTFF+swwV9M",
  "XMi9hKcSBRicW5el",
  "lniCej6bQZlFnZSK",
  "QLoE3kzkMSJ3Ebp7",
  "eWpJJCmfMZXZUeh1",
  "soam9DL3jEQWTpbe",
  "5gMi0DAIE9wHPWvy",
  "NL1QOAzWgapz86Tx",
  "tpiJzbyn95mY6Lj4",
  "B7p9KS98aFrWIbwN",
  "ExI2cZ7doTk2ShO5",
  "CGVi0NA0mn/w0KRj",
  "WcpCDxPGBzykWSMJ",
  "nY1kWN+Tdfc4POlN",
  "gKwDbzwxralWWpRZ",
  "SK2Uou5ElHo/+4Di",
  "M6dQbuIEysNT53UZ",
  "SWJS93quZzN9/HdB",
  "V5TW1hRtuumm7FWT",
  "Kmrzd+W9yz4/XBeB",
  "YrzZiw0JFcJAlIWH",
  "eB9tT4T0bj/jjDNo",
  "33339feFkSYpAHgd",
  "jMnnnnuOnnzySXrh",
  "hRd4QoFBKYZqYb5w",
  "sRxV+3pjF/bgc2fP",
  "nk1vvvnmoA5XMA5Q",
  "bATDE0VICONvtNFG",
  "bHyK8QZvKNru/vzn",
  "P+ccdBieMIZhwCJy",
  "Y2ueFspPVW2xjbxH",
  "XqThmipdpuJE+Swt",
  "v9xEPkWgzRqLQkEC",
  "YxTl0llKZ5B7HKNM",
  "Swv1TZtG6c/nUgKp",
  "L7E89aX7qbUvRTjV",
  "U3GzkIKR6GvYipfP",
  "726e88YcRmjCeDsj",
  "WJilTQ51LkbxTIrf",
  "B5XkyHuO+oak0dBE",
  "mgrUILjILxPj8Ydx",
  "aOTKTAU59k17UQOT",
  "MhOhfm7rg+uUVwbE",
  "FqTnSfW8rqYtpfFb",
  "5nhfr0gR5x28jjwf",
  "DYT25TqDy0Iyn6C+",
  "fD+1trZTd2+aIv29",
  "XsoB/j1JUTL97tlT",
  "zNfPHC05YTyNHddB",
  "c+bOp7xx0w7/5wvs",
  "iV7e/FdrnU2Xz7fH",
  "sHSmkwiI5FNj3GIs",
  "jsDIvMkzMpvS21XP",
  "hqZzvqacSMhxgmK/",
  "nfOnhULhwORvmfsm",
  "fGj6UvsTJZr9FmCv",
  "MO3X2//u62QWhquG",
  "eOwVeHqTgcn34sKg",
  "ILViz9gUAxUGq+ju",
  "GeMWM4bJ6ILwu4Te",
  "jVSKKaVNeAn9xhdj",
  "vCko+sGG98pgwmR9",
  "P29i9SpbxdthjlFy",
  "MjHhmBC+8UkaPUjo",
  "YmYicWrN5am/t48+",
  "ffUVWmeDr3ilTV+s",
  "lrYvzni82mqmMMv2",
  "YMLbCYN/NLALYYAt",
  "ISSpMfAu2j3AxSsJ",
  "ww3NHBCGxr+hGh2e",
  "THgLYfBdf/31XICD",
  "6m94KvF+hcU3Q+VY",
  "uqYQ2L+Z/R6F3hJ4",
  "KBGOR6/1xx9/nP7w",
  "hz+wAb3eeuvxse+8",
  "8860/PLL+2kOaEm5",
  "++6705FHHsltKSX0",
  "L95TWyqpmqCDFP9G",
  "uQEvrvGMR9kw/gIF",
  "Ez/GwqLP51K6r59a",
  "43FqjSIn0aRCYKFI",
  "EOMvGOISVh78jC9K",
  "5K2CPDkxT3fWyBoB",
  "PDbND8wrc4NyI/0F",
  "MHsdB2rIh0f0MUVi",
  "aSAUwgEUiYpwwU/p",
  "GCPI/M2RT22fS7YH",
  "eeDnNeNUxohSHPuc",
  "BXa3Lfl9YWQiCoFu",
  "YiVyq2erhCPPpQaE",
  "I5O/vHzN6112xMUZ",
  "oT+0oZNcqLAVNDQ7",
  "LGliyaRg5Sh/q+Ek",
  "VHC/UEdQ/BtCobaf",
  "YD/28zmHObaBfE4x",
  "Hgfuy2MxMv3nvPtc",
  "IMCeEiPEbvqte4UR",
  "nig8XpvMJYwwO/Iv",
  "Y0a7L8FC0rB+M/xe",
  "iVyC4tzBRCrbzUWQ",
  "O4v4nylSTNI5JUrp",
  "bIYSqJBEaBifmonR",
  "p088QfmeXvZo2hfV",
  "obzO2CBxVGi0mN+Q",
  "RoXCKIStbWdrrYqx",
  "if1we8kll7C2Lop9",
  "YMghlIsJBVX0yM+E",
  "EQqvIB4j31LeR64R",
  "dnGPfa0YrsLcPt7C",
  "AgJ5nU2hjJFclyTM",
  "D6P4iSee4FD/Bhts",
  "wJ7lU089ld566y1+",
  "PY73hhtuoMsuu8wP",
  "y0tnJPHqVhtpjCEp",
  "B2LgY8Px2L+J9WDg",
  "N6AIdU6fyVGNeCxC",
  "LYgosG2FROMBI5D3",
  "9STB/p+96wCTpKq6",
  "t6uq04SNsLCw5Jxz",
  "kpyDBEEUFJCcFMnp",
  "BwEFBAEVRFGUHJUg",
  "AopEkSQSJeecNoeJ",
  "nbvq/86971a9bmZ3",
  "enbCzu7Wna++nu6u",
  "qq7wXr3zzr333Brj",
  "zHFjHLuCe6hgHhWy",
  "PDk+1+cFLnDtm1HS",
  "EEwLMwjAtH93TqbJ",
  "hPJMkP4nWp862au1",
  "nlrM187J+rxe3aE+",
  "JtYOWVADA2cLusc2",
  "e7O1g1VZQsM+9HOo",
  "T/QDZLbRQmwLAsI6",
  "uC9g83vf+15YO9uO",
  "4Ypt3ls94Icgtn7W",
  "U7KBvuf4NJO4ozXF",
  "YSzCXvcA1jivmrhN",
  "sx5LD4UZ2zJIiBxK",
  "lMXNA5KJ5xTBd5N4",
  "YFf4sGoxS6yWsJvi",
  "jscghPJ5xn3O5fCg",
  "Z4nfrlKC3XlRFnzC",
  "rbCEkST9CHMqxy8M",
  "p8griZtNgKUAzCT2",
  "y6Xr8B6uSI/jNBNw",
  "h6P6TJCiGa+/RsWZ",
  "06g0m5G0JmvaL9My",
  "y0I8P3LF2iVeB9ts",
  "AXlbXkRlmDRzHK/4",
  "HGBzp5124kpAxxxz",
  "DANMAJ4vv/ySXc4Q",
  "Wd93333pgQceCD0b",
  "eh4ac2lXFbMT0Wz3",
  "vcoL1UsbKSi1Jbv0",
  "POrbcU/PHzshy3aD",
  "g2kFwESsJrwz3/jG",
  "N5j1nDx5Mh166KH0",
  "7rvv0re+9a1QtN6u",
  "nT6YZoNpPXZcm3K5",
  "IjW553AMknlepfYv",
  "vjKsKDJoyhwqAwd1",
  "tYT2OmcwRlwti9eo",
  "BZ0GLGrsM7u2mdUU",
  "2S+RNopCQJQl1bht",
  "YTp7R5pgLaW6jyWp",
  "phnwVoWvmm1sj4oe",
  "vVWpzP5ZW/GAlSxM",
  "27SXekYTYAlAM2Y0",
  "GzfkBcB0gqn9D7Xj",
  "MVnto91hZIxm0EJu",
  "CwLQVLCJGIheDQ/r",
  "v/71r9ygMPsHSIk7",
  "4vAyHfA1Ls0e8Hu6",
  "V6Hr3HKBh7JB7F1W",
  "XbqvM5rKNqhweyj0",
  "blgTkTLyLR1M4043",
  "rIUyl7ITk/sdsoLC",
  "YsLARHJ9H85BkJ0L",
  "k4la44jSDKjqAlBU",
  "OT4zxVkDZfKdIgNT",
  "r5qgJDOmAkZRwx0L",
  "78OwnlxSTxkVZk7F",
  "lSexqFVOuEg5qLgi",
  "buf81Mk068P3kGYf",
  "XpOerq+AiCq7jQD+",
  "az8fmjrbet9scKcA",
  "DqBKs69V1w71ze+/",
  "//6wIs2DDz5I2223",
  "HQ8YV1xxBTMTCvr0",
  "GVAP+OrLntoyRDa7",
  "qr9bv9jsnlr9+0bO",
  "WfevYT4KpPE9Yjx/",
  "9rOfsc7pj3/8YwbU",
  "N9xwA58/zmuoXKd6",
  "rfS4FWDDkHFvW71X",
  "AuaXKzTz8y/5fwZm",
  "geyPy6gi8efrkTF1",
  "ptcUv4nYUFG6rf1u",
  "9qai7z15PBoZJKOE",
  "HivZR0N3NCxndr/d",
  "w3f2xNU2ndTUlx3t",
  "KdQF917LJMY2Z9P7",
  "hSxyLcKAfop+hnhu",
  "qNf00f5sQOaswTni",
  "+csWFKAJ+3kjTxRk",
  "m4LZhDQJZnvqYott",
  "3pvNGME0tqun7F37",
  "f36FDmQdS2C7rEIw",
  "KDsMtTB1//w+dLsb",
  "NlFZTWU7DdMRCkNr",
  "PXTDfOhgKGyIBPHz",
  "/gFYWRvQ7CtkJw1A",
  "JJfd2pJcBBe6y4u8",
  "lyXFIFPAawgwwWSa",
  "Y2I2huPJzPEwqyJg",
  "00GWuBn8M+kkpZKQ",
  "fqpw7fMpL75UA+Sj",
  "S/T1QXfUqJG0/ArL",
  "hu+FLQN4Gnwg05M7",
  "346d1AxtZIP+85//",
  "5GQZDBqIWwR7CU8G",
  "3M8KDhXwabC/zZyr",
  "1bORNtizs3vr2cue",
  "nic9fWaDsXpWuKf9",
  "AFzYbV8BN+4Dnmdw",
  "n8Nbc8ghh3CYAEIB",
  "YEMBNux+WwuQSVzn",
  "cwgx4LZUKVH75EnC",
  "TgMkOciWRn5NhRPS",
  "eF/273HykSgzWDC+",
  "5wCaBJLolFVEgo5K",
  "hUnyjmwZiiHVHmeA",
  "ErYNQU3TTjgoppZd",
  "xMGGa9Vt1YeuY7f3",
  "VEpUH+pBu33sKgkX",
  "W+9ms5fABJik4dr9",
  "8pe/pL/8pSEOqx5k",
  "wl3ePjhHO//Z/JwM",
  "1K/kIBgGoth1Pjws",
  "YqwMmEooExLN1O2B",
  "ncECYresATl8rXNJ",
  "AYRyuUVTJUMZzuj9",
  "bI7JJAcp+LSTe1T0",
  "nav+cEaB/bl5D+bB",
  "SBg5xrXHdZSRzhOy",
  "rogXK1Ey8LjMnhyX",
  "VCgRkXdNEhKmkmsj",
  "c/4QIKXLJfLAhHJJ",
  "yvDIRe+Py9OZ5IO0",
  "51KJYzPhbk4QlXLk",
  "ey5NefZ5quTAfKWt",
  "a92TMgAq4KRptdVW",
  "o2eefrYGXHJe1RA4",
  "BWxAZ8ftAkghThfZ",
  "2XfccQfHUiHxByLn",
  "cJWri732fKRijcb4",
  "2gOyLYVmAyh8hucF",
  "knPwimQixK2ut956",
  "LFGk7k0AQhyTam6q",
  "hieOA8wrQKHWO1eZ",
  "JDCsKhBvH2O9m97+",
  "rJ7V1etz11130W23",
  "3UZHHXUUSyCdf/75",
  "rLox2Cb3xY7xjcKT",
  "bJPEvVrIVWgXaaOM",
  "47GWLCZqiDvGBIy3",
  "MVVY6y3K08NQhnvM",
  "CpTmM/OKxDnOHBfN",
  "WU0KYm+6SRBHA2Y8",
  "yPXQjexSQwAzOlOf",
  "dUG57mWkvYniB3X8",
  "x+xoDX3e9PS5tgG9",
  "x72NW+q+j61vprHe",
  "/ZAMu8HgkNrEgYXc",
  "FiSg2ScxdzWtZwyX",
  "U2zz3mRciVgqmCQ1",
  "JEPtvRo3uqUdp26q",
  "3lBP/Trq2oJbW4cW",
  "jfUMK3T0sk8FoJGs",
  "kcgccXIOxgO4xbkK",
  "UVIyxzHKJapc35k1",
  "+ypZ/gwMS4Vd6Igr",
  "w6AIRqVKVT4QMJkY",
  "yLFvFMNEdaAyfx8d",
  "uWSn6vVhmWjoY1ZK",
  "VEXmPmR/kmlqTjlU",
  "TDXR9LffoelffUXj",
  "l12+x+sUQoZAMtdR",
  "BEEvRVRxB+xTdUgG",
  "ARiOA78NVQLtw8jG",
  "BrAEmwkXOUAcgKCd",
  "eW1niOuxK3izB3Gb",
  "WUcJVCTgbLrppgyy",
  "oXWpAuRon3i1JYTm",
  "xNop+6gZwTDV70QS",
  "0gcffMBySi+99BLL",
  "SCHuUsGkMp7Katnx",
  "qton9Lpgnzg2aG0C",
  "dCKWc9CBZtgvo8kg",
  "Z9oHfig7Vbt6bR53",
  "56yZlG9vp2aItZch",
  "IhRQ0vMo7VRFsque",
  "yawBmXiDakPCXpIV",
  "jR2qWXJspssTMiOl",
  "afqNriuxnUjak3mj",
  "UqV2v5rT6UdImO8P",
  "9x0DPHuAfBqiM6e9",
  "2sCT77+JR1bXbj2L",
  "+bXkICuJLbY5m04K",
  "8SzRkLq5sBhkLiRA",
  "E3YZER3Yl7AABNBD",
  "Y7NegLo+Rqo/Ntid",
  "fbDjsHo7/t5+v7fv",
  "wWC4CY/rCsOgM9fS",
  "2kSlSolLrfmlCnmZ",
  "DJWKBfIgUAyXMjLS",
  "kSgAcKBFwSvQqxSq",
  "0YFeH4M7JMMIEJQy",
  "cyLKLjqZkqhju81Y",
  "SB3Z2lx6UstMSkIO",
  "nJDiwhYdTNXidKDx",
  "6Zpa1sgcD7C+OLUd",
  "DyXtkvJbPtb3RPKE",
  "4yxRPs4TV2ECcYAl",
  "SiNZCAMngBGYDJTs",
  "owK51RS5DDBLVHLy",
  "ppa5S04lSaUwOUl1",
  "+GQgR5lKDMtdEKyG",
  "Cz2boEVch3KUIqeQ",
  "p7b2Is187gkau9TK",
  "5AY5cjNwc5YpKJap",
  "kkiS41YpKAeUMFmY",
  "a6+1rqnoIjqd7MJ3",
  "kgSlwv7c/95M46Xs",
  "GEktJ4k64VtuuSUz",
  "ixCUVwN7iPVV/84O",
  "tdD9wdR1jnWgYbnF",
  "FlvQjjvuSNtssw0D",
  "S9W5VKbTlibqKYxD",
  "P+/p+WG7yBVwAojB",
  "vb/SSitxIg8+B+v5",
  "2Wefsbv/4Ycfpmee",
  "eSYUorcHQk0UUle7",
  "AmckDeF9W1sbx6fW",
  "H+9A3x8pkoP2jzYs",
  "TDq7IdMOZZvTrDEp",
  "mpJo2wLkGIIx4nKo",
  "+/OPqVpNUJbXq1K1",
  "UIT4LJXFG83MoCbV",
  "sF6sokXm9UU3kplL",
  "9CNCpSH0jipRFWlx",
  "KfYaYOuSWzHObRyv",
  "sthoyBWiANqbAg45",
  "CQlam6ZgOYAku+kh",
  "um7E3tXgZWAvgfFm",
  "VMNJnsZ+Is4UerRS",
  "0hJfcGlZvGctTIlD",
  "VVDK7ZPd+nyU7OFw",
  "0f4SHuttom82LboU",
  "PytYhRfx3th/uSKT",
  "ckdKU6Imusav9xTj",
  "XhuSRPO19QaobQ+A",
  "ZpgrcylhNNC2rVAm",
  "k6Z8PgaZA20LItDs",
  "swsdMZt4QIMBweBU",
  "LyURzwiHxuyEAozH",
  "9qzdPO9rMy3BHsJd",
  "mUxSwgzg8p0MZGYo",
  "i4LyNdFnLo7NDsr/",
  "Wvao/b81AMH9rQ4s",
  "KfUYZb7qcYRZr07F",
  "RH8Cygr09cPqPlVT",
  "HxkDlS8Dm9HSDCVZ",
  "UIZS3YA1xyPjcdJJ",
  "UDVfosBzyc2irF/A",
  "jEfKSVDnZ5+Z5CTm",
  "YQk4ih/cAOu4rp4w",
  "m/gfMZBg9Lq6unjw",
  "xe8VikNTlUZZVU2E",
  "UFD11VdfcUymuqwV",
  "hOG+g3FUkGkPRpok",
  "gz4OMLnffvuxODrY",
  "S2yjMajYl7KWuk+b",
  "9bSTe5QF1ZhXO5lI",
  "BzZ7MqvgUNexXfk4",
  "PjCoAM8nnngiT4bB",
  "csLF/uc//5mZT92P",
  "Zsiq6UTZjjsdioSg",
  "Wrd/xLLxcdi/jzAW",
  "XMNqxLh1z2rnsI6E",
  "F8lX8TXlSWFQp5Np",
  "fk9wJlsVQJG9CCjT",
  "iHvFEJHBIuCi3e/R",
  "T3qSRxIgiHsGtzru",
  "iwGiHPbSGG8h7vfZ",
  "v5+daZnz0OmvyU+m",
  "T/MEx6/yBDzTlKFk",
  "VtqoampGJXG/Hmq0",
  "MNickkVhodfL9EPt",
  "pxo2g++WW25Z+vjj",
  "T+bm568yxWNiW4iA",
  "5ly50GHQq7MbJEwB",
  "Zww2B99QMUQftShB",
  "qRpm8rDuYSbOANSh",
  "VCZNCQ+1oSF0jkEe",
  "98swe+Gsvf8PXPvh",
  "ry+zlVupM3afG/cc",
  "5IrAjICNlOx2n3yv",
  "RIQKHxBu57iuqAQl",
  "6/pVweJKRRN2oXO2",
  "t8ag9ZqSS45fpeZk",
  "luMysV0SbmNH2LH8",
  "e+9TqdhFTjbFlZUw",
  "4Ff4+IiqlYAHYK5F",
  "Xakw4weXNeKbUQFG",
  "JwFDYdoX7VKlWAA0",
  "kTiGSSKYPI2TBChF",
  "SIwtWq66rFA0gIbm",
  "wQcfzG5x7Afb2lm9",
  "mkSjAzb2ZScP2e54",
  "PDsAvrFoZSIt1Wm7",
  "2NXlrpMo3bediIDv",
  "9ffxG9gfPC5gPMHA",
  "INkHIQIQmH/kkUc4",
  "zhPnbItMK9CFaRnC",
  "oTANv7Td55zBK8XA",
  "OcQiiWtnjkkwaYLa",
  "J09jVQQnyRy9mRza",
  "UcfRqz2h4v+50g+Y",
  "Sk9CUkyyD6pboVAC",
  "EWKgoxhmAFcONlH2",
  "GVM4CeaW3whjNRHe",
  "IuymCh3h9ziE07Cb",
  "mucjLn0FdnboijCb",
  "fbHaiasBm+i36ItV",
  "n5qam6hpZCtPRzEZ",
  "5THLTCzEZS7XXhNc",
  "FlazQaf2RS3mgP4J",
  "T4Lq1l533fV0yCGH",
  "zs3PXEhE5wzkcS+I",
  "tqACTZU8+g8R/aEv",
  "bnRbuFVnOvVl3GIb",
  "3PhMHeiVWTJ48mvu",
  "ER6o0IjTKWZIUOIu",
  "jP/nsm9gOMDM1TIh",
  "Nb9pBozerCbBKByx",
  "LEFlk3UusukycIF9",
  "FIbTRGMpewnXXui6",
  "M9+hzGVVC88hTkxi",
  "MpmRNdV+zBEb+SaJ",
  "8wRjg99iJ71JaLDj",
  "v6D3yS46uOX8gLpQ",
  "E95FSAAc+w75lTJN",
  "eeVV6pw+kTJLLSsD",
  "PH7REcd/GSxcQJR1",
  "UwxkWkc009ixo7kE",
  "G9ygPsILHFdK6A2i",
  "aZtQAGeDQXyGY0Mf",
  "BUjD9xhMFJTquvgM",
  "YO2kk05iBhPtC+BS",
  "WUsMQtrfYbpPfIdX",
  "/M6MGTM43hHxlGAX",
  "EQsKFzcWAF0sCjRh",
  "6rrHvvGK48MyYsQI",
  "9qSATYUSBpjLVVdd",
  "lb9Tt7fGceIcNYNY",
  "XaHIKocOKH4LpTKv",
  "uuoqeuyxx3h9Ped6",
  "Nym3nkFiuGY3Edfz",
  "ZxZT6lQy11g/8HRO",
  "nkZJDlPBpCbS49R9",
  "V8PKO6aelnG9OxZl",
  "GCZw8WMADwydYCCc",
  "xvkac6h9RTPXxT0e",
  "ua8VKOIBxDGds5nQ",
  "yVNIzIia8TkiTtP+",
  "PJwX9nCdZN91MZ1m",
  "EscOeTNZZvYynaRs",
  "S7Mwc64RoreSJMF8",
  "ovgF2q+GhywMVpPp",
  "34NKBfqght+Ixqs8",
  "I+ZybEfMCpDpbf0/",
  "8gXfFmSgqczms0R0",
  "HhHt2+hGGDyQ9ADT",
  "uKeY0Rzah0S9NEeN",
  "/AsyWx08dEUYPQmJ",
  "KsRtlsCQgIWQTHPx",
  "2AGYoU7y1+9fo26t",
  "Rgz7YuU+HsVwrOWw",
  "bB0GsyoEnYEnwRTW",
  "kUv4TrLLhUGR0cV2",
  "13nkJyoSCCAJ9Lya",
  "SlRzGWNN0MEgGQrJ",
  "qwORyE0lEXrJLKSX",
  "9ri+cwGgBbFiM9uo",
  "6+OPabGlljNZ9kjw",
  "SRmXJ2dNkIv68Z7H",
  "4SWopPPWW+8Y9oSG",
  "xOy4aWUSdZBQwAjT",
  "V81GV/YTGelnnnkm",
  "s5jKXNogEO9tPU1s",
  "i32//PLL9Pjjj3M5",
  "SJSFRE1xlRUK71/d",
  "QGW7MJUxUTZUv8fx",
  "w/1tJxFBoxQ6n4gR",
  "3XzzzdmNDwYZYFIz",
  "13GuqkupAvVbb701",
  "x5QC/P7qV7+iO++8",
  "swZo2/Hmgw041ZRx",
  "tieMNa5wZaVZ2qhC",
  "+anTuSIQV8HyMW0T",
  "5QTWeOVNBWBKexa0",
  "KD0F7R2lWo06A/oJ",
  "n5po1nItcYSVmJEu",
  "fIoYtBkVpISnQOpp",
  "VTGBArRlFOpQwF4G",
  "BaQmrtK86lXkz3t4",
  "noheg8Vy2t/Vrc/3",
  "JAy9kZhPM4ulCpQT",
  "FDC1NFGqtZnKpRKD",
  "znBb48HRe4v2jGVh",
  "GLsaac+4FvZYjr4D",
  "T8Rc2BMmPO/dudl4",
  "YbQFHWjaMZt4Eu3d",
  "F7AJtkHdWRp0H9vg",
  "WE/PiSjGyMqs1C+N",
  "jAqYjWQ6ReREkjT8",
  "qFbgVSd11ONvNwA4",
  "7XXqMWvtACMsKieV",
  "Aw3qNnwAymp6IZPC",
  "MismwxzMozj4AGS0",
  "vrJ8UuVtRYRaXIXq",
  "dtRfNWdoEgxCDXlT",
  "xajI42hAzWB/k0lq",
  "6+rmBA0vmaJqsUCz",
  "3niDEpttTVUqUqVc",
  "pMBDDCOqEiFqFCyJ",
  "JF0hEQiZ2HZ4CcBr",
  "VO5vcMyuRqSxjwqk",
  "1H2tExRlOfEegO30",
  "00+n3XbbjcEeFs2C",
  "Vje4shzYD4Dk//73",
  "P3ZN//vf/2b2sj75",
  "px4w1pv9vb6vn0jZ",
  "We4wvG9vb6enn36a",
  "QS0+B6CEEDuSnXbf",
  "fXeO2dSJL9bHcwkM",
  "KkIZ8HyCrNPVV1/N",
  "5TTPOOMM1tHEelo1",
  "yf79esazvza7pBKt",
  "2mT9eHj+zMKjbeby",
  "1DV9BifEgafkyQEk",
  "voycl7ZxJOFAU1Zj",
  "NdH3uW9g0hB43L/Q",
  "ZoWBRAdzWcNS5MKM",
  "i91iMcN8ImUcfXFF",
  "K3MZMprkMPjs6QHC",
  "52I5TMRDYjwauuMe",
  "LASooSse0zv5NV1C",
  "gJ0QEIv9pTyPxi21",
  "JDWPbK2RiFIFDvu9",
  "hpIsjFYPrnXSpx4D",
  "eBDQz+fC7jd4os+1",
  "KBdmWxiAJmy6aRxo",
  "fd/qbWW4tGAa82TH",
  "PsU2OFY/QOmDQWLL",
  "DFtRV+pPRaAxkDmI",
  "YcJDmfUytcAc1gGb",
  "MzDNnIGh/V6ZTAZ8",
  "SMjxaiRJ5PvwSIxL",
  "HAOjJwMd12SWxCAk",
  "BUkMmFQXkvcYZKyy",
  "eYb1xHkCeLLkEYu/",
  "RzJEsz12P0FOCu75",
  "KpUqPrvMW5uaKB/4",
  "VKj4NON/r1OZq+uk",
  "KEDCTRPclWBRRTC+",
  "iqx6dn8SrbTyCnI2",
  "kFtxhuYRYt97fa+f",
  "2QwlgA2YCwwkF198",
  "MTN9+B6sIACmMpxY",
  "R2saT5s2jWWAsIBl",
  "tDPTlUmF2WLvoWvV",
  "/K6d/FN/fNhO2VW7",
  "brp9LspYKpOPto/M",
  "80cffZQeeughOuec",
  "c5jphLscoBlgH3Gy",
  "yuZiAMX+cF6o4IRz",
  "AWhFmMArr7wyR+Hu",
  "wWQ3Ffjbv8GpbvA4",
  "mI7KgGhWuzWBFOUI",
  "4/Tmh4OWdJ1djCar",
  "Z5rP+UzRfwytj31I",
  "vXBzvuF7K0ZTJ4ym",
  "jyLLXXYOd7aATAaF",
  "FjjU93ocETCN7qu4",
  "0HufyKrnIfwNw9jK",
  "pNEw+GVh45ZeYTly",
  "06kwblvbqx0Nivdo",
  "FwsL0Kyf3NVLPWk8",
  "N/oVtHWhezsXdq/B",
  "EZMH6rgXFluQKgP1",
  "ZlNMI5FyGQMkmxDb",
  "wFlNQQ5Lc5DfW9/p",
  "g4MHa9fhGE242Goy",
  "0nkd/d+KVZvLY7OK",
  "ewi7YB9rA9sLsxoN",
  "YlUwGGZg4wFUmR4M",
  "Ofx/QghQHkDKRlRa",
  "glVZGoVjM6NqJ/za",
  "42+aJCYATWTU+iVO",
  "uvDg1nSIysUSJZJp",
  "6vj8C6oitpAlasRN",
  "Lg9nUw2Jr3dAnusx",
  "068yQ6xFWVUn/uCZ",
  "xk1rTXKNo9TPAbjw",
  "3aKLLko33ngj/fe/",
  "/2VgBpYQIAyTER18",
  "VaoIQGz//fdnF/Up",
  "p5zC22jslt4PBX98",
  "p4wb2n4m1DOVPYFI",
  "nazWA1Ztx5q8hGPs",
  "KXFHQSySf1BmEvXN",
  "99lnHwagGl+qGppQ",
  "BMD5AryBDX322WcZ",
  "cNvHVP88G8znm54f",
  "A2xO9rN+zwDNUqFI",
  "xa4cJ/OFIN4oR7BL",
  "Wmuo9/QD5sFQcSEh",
  "ZjQw5Zc5fhiMaLWH",
  "jL2a/qsqDny/cI98",
  "U81HvSSNxYd8LZG9",
  "oa16X1/l2jQMY8zY",
  "sVSqRrW4Z8eiL2wx",
  "mrD6uOT6fobrMZcg",
  "8y4iOiIGmXNnCxPQ",
  "JNNIADbv68tG4YPO",
  "6sw9ucPmtPRmvW3f",
  "3/3314bi9yH8DWDD",
  "7sGyuJmFCULyjEik",
  "ACBprWfOXq1WqKV1",
  "NCdXdJRLDL6QTJA0",
  "rFDSI0qCCYR+n1+l",
  "BDJbTPYPH7ufwFfC",
  "NuG1KgMbhKarKNOI",
  "hb9zCQJEWPyqQw5G",
  "MHUBMuHhUtn1qeJV",
  "qcKuNBFbZ5Agopnk",
  "U8poQBfJDcrMSAR+",
  "kioJZHkX+LeCikt+",
  "NUkVjsEsMjBMVCRO",
  "jGPbAlQBCsitJinp",
  "Zyjhe6Z2usSU+S6q",
  "oBgmBsGcLlHVw4Bf",
  "JMpVyC0RJcmnNAbR",
  "QpWyXoo8l6j82Yc0",
  "4903yE0CuI+kaqXA",
  "OpnlYoEKjoBKsJpV",
  "36dVV1md2UFm9yB6",
  "PcBV7noCRcou4j1Y",
  "SE22sV3lRxxxBL3x",
  "xht0wAEHcPvAMaJd",
  "qBtRK/Ncf/31tMkm",
  "m7DIO+qhQ3uzvg3b",
  "k5yekgVs5tIGp3qs",
  "9TGcCso1qUi3UTCA",
  "BawrXu3njQ1s9ZxR",
  "SQiufYBklJwESMZn",
  "2A7AU9lbDTPA97gu",
  "G2+8MW+vdZx76r82",
  "89kXAMqhHhy6IBqa",
  "HFfsE2UzrZRKpclN",
  "gNUkcjNN3I54koRw",
  "l1SGKtOnU6m7k0aC",
  "GWY5ogT5ZWjABlRC",
  "BnXF4f4gDCES0AKq",
  "Bgg1QbUrojKqY4F1",
  "9x0qBR4VyCOozEI5",
  "swjAVXU4cwOud0gW",
  "4WlSThAV3QqVHCS7",
  "uVQOHCqRvMc+4S2o",
  "BOD0q7x/HIsw0sR9",
  "oIz9El6Jl4q+knlO",
  "8IRQ1Gz9QH4f38k6",
  "8BaIRq5MOE3xAN6P",
  "T2Vfwj7Q3xUE50sB",
  "edkkZRyHxq2xNjlQ",
  "JHCx4P75hE7MccXl",
  "MiWCCusGT5/ZLvXE",
  "ZvOctsNP5ofxRY/X",
  "nszpZ2o6AbYlxtBv",
  "1lxzde77c2lXEtF3",
  "iWjGwJztwmcLG9BU",
  "sPmThqepxtCY8fDW",
  "RqxlwNR1Flv/LHJL",
  "ynt1b/b0sNH19T1X",
  "iWltMQ8guTdqLKvS",
  "oNksx2zXqcswtc7A",
  "xO0DdYocSg0jwqwj",
  "XOUYRKpSrs50P8gb",
  "aVaski9SyUT3Kws7",
  "/ZyobrMesLjn5xCD",
  "GroIa80Kf6Puzm5q",
  "++gTkTOqFJl94t+E",
  "5ExVrqs+1NEPRo8e",
  "TQNtPQEbG3RKhSgv",
  "THBQhgLansi4vvLK",
  "K3mwRRymVpXSRBoE",
  "/Z933nm0/vrr07HH",
  "Hkuvv/56TXzkcDe9",
  "hyp5hPPG8SOW/De/",
  "+Q2tueaadPbZZzPg",
  "xOcYVLEe2E6c3zLL",
  "LMPi74jfVOCs8k0q",
  "+2Kra6gLv9Fa2RLJ",
  "Yk8OlL2sE7I30jsw",
  "BTm5js4w5ronUBMl",
  "7AjTL8y/JgaJ3ixr",
  "YHL8siEBmMnUPYgb",
  "HOuXUY3LTDARXIn9",
  "YaIn/Y3T4BjYsVKD",
  "6YMCUNnhHoqo635D",
  "eSPDPEqSkCxynJiM",
  "1d5D/t+Es8h+sUvr",
  "OuPZYCUqKaPpg81P",
  "JWnskoubimUB91O7",
  "5rnNxCMkZH5o242Y",
  "gksde+3PdOzQ8Rjt",
  "2ZYwQh956aW5isdE",
  "9Yf9ieiEgT6fhc0W",
  "RqBpJwj1yfDwVjcV",
  "HswYxDCgNSKPMK9n",
  "jMPd6kEGHhKIQdOH",
  "ST3Q1AFQgf6Yxcd9",
  "7cGDh7MOTL1ZD0np",
  "odlaeeH61rFIfFYk",
  "PyTi8CbGiyPAUGVE",
  "vgV401rLLCZtwCSq",
  "A4luH0dgGle5agJG",
  "MWEs1s5xaY6RdpEB",
  "WCqamKpHHOBl2lY4",
  "zoTpQnJ8OuiDHYWO",
  "Z5Vo+ttvUwC2iUqU",
  "dlKUSaYp7XqUckXq",
  "iI+7WqURra00Ycnx",
  "Eo8a1mzvn80usUZj",
  "qzSpBH1OYyIxiB56",
  "6KH01ltvcUymVvrQ",
  "wgtafAGuY9Qkv+yy",
  "y1iSSGXLbHZkuJvt",
  "vrfbs14zhATg/Dbc",
  "cEOWOsIAi/PHs0qB",
  "JNb52c9+Rg888ACt",
  "vPLKYca+rmMDUGWC",
  "VO+zUavvawIM0AcB",
  "5iL2SScugIvt02aA",
  "JhTgWRPXaNzADLow",
  "OUNzgwKDlGrVyRhc",
  "5KpjyQCPgWeiZl8M",
  "Ik3IiUzUpM+ht1VM",
  "GEqVtwHYFJkj/CZv",
  "b3oks5pgsLn0q/zP",
  "v4ntVGLJvhZW7KYe",
  "Y5S5bo5Lgmdq1jeP",
  "DalyBNY34VCuWqZy",
  "qUpNY8ZS86JjqVIo",
  "Usmw6KrAYDPqWD75",
  "5FOWNWt0sjDcrSY+",
  "3zIJpZEllRIZM9jK",
  "K6/IfUAVZOYi6WdN",
  "IrpjYI5+4baFFWiq",
  "9NHNfd0ID2u448Cq",
  "qB5XbP0321Up7wOO",
  "r9MHaf0AZoMSPEgX",
  "MQ+TCmtrGtbDgLi+",
  "zOl1MAjfm1fV0VPm",
  "IgS+thvVjBDKavBn",
  "Rt0Z2dwyuIFFAmtS",
  "psDBA9GnhJ9i/za7",
  "YRNFGd5CxoVz0EPW",
  "UgEtD4RwlZvkAzkG",
  "A0r5HOzr9XWwHVZK",
  "Mq+um6aZb79NpfYu",
  "Kpfz4gcML0gUMoIB",
  "LJNJscSRXdu5v2YP",
  "HvWTLwWh6hJGn0Ms",
  "4i233ELXXnttGEeJ",
  "7Gv0TzCa6KOIT4SL",
  "/Nxzzw1BJ8yOlVQX",
  "23A3lShSoGbLO+Fz",
  "lT5CyUlknKPKEbQ9",
  "cc64HlgX1wxM6Pbb",
  "b8/sJl7xHNOYTmwP",
  "w/XVTHFcz54mAPVW",
  "n3whBlWD6BqLHNbX",
  "wX3n1GnkQ75HQ1bg",
  "njYlH4VFFL1YAY/o",
  "SwLs0E+4b2BdaMqC",
  "sQQJL6q2LHWG7dn1",
  "DXCIvgkGk0NggvBz",
  "bM/fYVu44RN4xfsI",
  "YMp6knSk4JLDaJgR",
  "Na57PS4uU2nYTMNY",
  "cndml795PoTsp7yH",
  "Gx0AWUrOmhhuA2hZ",
  "Gi0JmagmWm2jDWjk",
  "EotTJpXluHRVXIDZ",
  "QBNj0yeffFzz+fxu",
  "syNl9DM8H/J5VPpJ",
  "0D/+8Q96770P5van",
  "7iaiw+PM8oGzhRlo",
  "qqj70X11owMAoRKJ",
  "CkM38iCOrXerVCA0",
  "HIEi1TirZ3Fg9UBk",
  "yeWX4/jOEmeqRw2b",
  "AZvWQe+nBbMBpAI8",
  "DeMCzT3jJkfNZRhi",
  "wJTJULdeyHQYZoMZ",
  "SYtxQb1114j/sYam",
  "g6EuwZWDEsg2ZUbU",
  "BPoHXjSA6cM4TC6K",
  "ypewq07/D5McAHRF",
  "LLrz88+pMG0KuxLL",
  "FSQG4bXCi80sg6Fa",
  "ZplIZ3YgBrI5DSIa",
  "BwnwA9AEdhKu8u9/",
  "//vcF+HGBwBFrCWA",
  "FFhLCLLvvPPO9N57",
  "74XsnC0ppKBtftHI",
  "rU90s4XocT5gHjWT",
  "Hstzzz3HFY9+//vf",
  "8/VTEXkFpQg3ALN5",
  "5JFHhiU7VQoJ+8T6",
  "tjuyUROVHUnA0+ND",
  "n9YOibkJ3NLq9kaf",
  "zc2YyW1QYjANC2kS",
  "4rhuuClYUFXgxQwh",
  "wFkE8BB7KUCzwn2D",
  "J2RV1D0HaJR4TQGx",
  "jsRyJnwqJUr8inhK",
  "xG3K/rE94jMd87mC",
  "VAGwAjoNyDTHqgBY",
  "GdQIeFoAVNlPA1jt",
  "9fU7AchRWADAJsdx",
  "EuTJKpQrFmjptdai",
  "wEmRk5CJaZrjbWsT",
  "0LRtwGWs7xcE04lV",
  "fWloGYfl88UWG0eV",
  "is/x13NpqPTzHaNU",
  "E9sAWYyQhNlcx1Dl",
  "DRuYA2TfapxTbP0z",
  "ASzRewxAs2bO+loy",
  "hj3Q2uBnwkrLMzPD",
  "SRYcUygPchE9bgxI",
  "1DOZNZnmVvY7Bjo1",
  "HTAFSKoLTwbFEFwm",
  "HGEtAT6dMtILjKs8",
  "yaC0alhMGXRcw9rY",
  "Aw/2a87XZJ2zq09l",
  "j1gGSaWQDNtpZd/K",
  "Mcr3AopFbyVyo8v6",
  "xc5uKn/+BSXTWXFV",
  "wo1uKo/otXeAo4OA",
  "llpKJMCGKlFA9S63",
  "2247+te//kVrr702",
  "V+mBmxxtAf0RgBN1",
  "wOFGR5IPGE60BztD",
  "ne+laTvYp+0yHs5m",
  "x6fVx6jZiUL6PMJ7",
  "nBdqwCM5CiBcheMV",
  "SOL9H//4R5ZA0uug",
  "k2f9PQ0TauT45Jjs",
  "TxHW4nKb4Z4QVEXa",
  "CC3O6GjitWv6LCvU",
  "IwJlSIzB/2VOzDPM",
  "H9cIithJ7ivMdoIl",
  "1Exz/VwSb9CPSgYg",
  "qmSQMpq8nUnQwWfM",
  "ZhqQx7/BLKMBmGa/",
  "IcgMGdaorwuItMCl",
  "AZS6vq5rM57MbrL7",
  "3cSM8vOj1lWPe+Ck",
  "MzR+jZU5vIWvAeJd",
  "cHxlUWKwJyCYpKN6",
  "14JiNfGtPWTYw23+",
  "2Wdf0OTJEJeZK/uM",
  "iHaLy0kOjsVAM4rZ",
  "PKav2ejvv/8+iyk3",
  "4j6PYzTnbD3JxUyc",
  "NLEmA1fd6DC7ogr+",
  "X2KZZQVYmMGSXUoG",
  "BNryRrOzOc35a5lL",
  "C1han7P7juO6ZOBg",
  "lx/HZ0p8Gf8Gx1ia",
  "kpEcq2mAj4Ns9SLv",
  "yKmmKeGnOXO94pZ5",
  "kEuAxQQbGcoaSYym",
  "VDvCeQqbqrFi1Rr2",
  "JEpiqI9DrWl71SoV",
  "8yWa9e47kllbLokm",
  "YTLFNJQOpA7iGxM+",
  "LTFhSSNB0w/NqLpj",
  "UVMwVe9O32uvveiO",
  "O+4IXcCopKOi1Pge",
  "Wecnn3wys3AATmDq",
  "NKEFbUPPV+tvDxQb",
  "OxRmC9PbbV+BIc5D",
  "zwnnqswlrsPf//53",
  "dpMjlADvp0+fztcI",
  "rnJcx1/+8pd00EEH",
  "hfGaNouJ/QCUNmL1",
  "SgF4UXDMbKUVL2gn",
  "rXTPmhXG+epkiF3n",
  "JpkG4CuKccSkCAvc",
  "4qJHi/5VccW1jhAU",
  "9BW8R+IP+kLCB/jG",
  "hE6yybFwIh48AeyW",
  "lyx4FjZidzv6kakO",
  "FAJCAaIM/pTJtBhN",
  "DdWpl0HjVwMyw3NT",
  "FtOsr/022l/0qqAT",
  "92DsYovRIistRykn",
  "SeQlw/AGCN2rqXQX",
  "tCJRfGBBs56kw+Dh",
  "AKOp+tdz6SrfkIge",
  "HMhjjS2yGGhGNmlu",
  "pI/gwsPgFlv/TB4a",
  "0XuAHc2atEMTbHeJ",
  "Pmjw/6ixY6gpI8wL",
  "BqYorjPKDJ2r4+qp",
  "GkgPWaQyIGLAqkQs",
  "o++K8DNzFZJ5zskM",
  "LLouen2yIwx2kGZC",
  "hSCwnB4PfhWvRIGj",
  "QvByDSRD1oAATRQy",
  "bKd9fAqEwwSEHgCy",
  "bVwKsOpT++efUrmE",
  "QRthDC4lvYjNUqCP",
  "BexhJIfYf9WFr2ug",
  "RlqTsAMPPJBuvvnm",
  "EGwBMIG1AehEMhA0",
  "I6GfiWMGe4eBWUEZ",
  "9qEKBhovqJJC85Nb",
  "Uasd2QOtPRFTzUSc",
  "uwI8ZScRr4nym4hr",
  "RdhPV1cXr4v1wHL+",
  "6U9/oi233DLMarfj",
  "QPtu0Tb1IvcEoMnJ",
  "QRoSUaJ8ZyQ7o+5z",
  "SRSSPiyTtyjxLQxB",
  "MQwhXxvXyF35AJqe",
  "SIa5KLEK+TEkHsm2",
  "/DmAJs8ANQZaeVLt",
  "nwYUIgY6jBPVAzST",
  "Np286TGKBn10vFZY",
  "jPZBTc6rSRiql8jT",
  "OGvNptdqQVWfllp+",
  "WRo9fnGWViv51fD+",
  "KovN164q/Xby5Mmc",
  "TKn3eEEwfS7oeKDx",
  "2m++CZ5ori12lQ+B",
  "xUDz62ATMZt/68tG",
  "qC8MVx06vA2K4tjN",
  "xs1hrUY8RAS0wL33",
  "2acTKZ8rUsKpkueG",
  "YfOhdiLkPfBJwktQ",
  "ctQYWnrllamS9KiA",
  "wcNJ0KKpLBURswUR",
  "PmYwTB10yA5ZVT7g",
  "P+bBkKXRAbJqmUqO",
  "7wLr6MONR/y+Emaj",
  "EhXw3oUWpkdBJU1B",
  "BTp6WMfEgwVl6mY9",
  "v4AqZdR29igfEOUS",
  "VSpCf6+SIrcs8kxF",
  "J0clp5u1QxNlj/UE",
  "yz6c7R4VAT5xzhjQ",
  "fGhyJqkM4IWsVKdE",
  "eSpTGaUsq2kq+0kq",
  "sTagTwUqCkuprsVq",
  "NdQB5MQBLr5UpkQy",
  "RZ+/+jr5M6ZQqdhJ",
  "1WqJPNSarppwBM/l",
  "WLFiuUCjRo2gpqa0",
  "PED8/ifEgY0DE2eX",
  "mNT/jzrqKJYu0hhC",
  "rAcwCbc5XOXwKqAO",
  "uepIKlOmhs+U9dN4",
  "TbX5CWjWJ33U/w8D",
  "2FSdznoQiu1/9KMf",
  "sQySAkw8oxByguuD",
  "akKbbbYZb4PrbGuV",
  "9mpGj1aqr6CPJTlB",
  "Z0S2hZIouep4lHLS",
  "VC3nKShKyU/fc6jS",
  "WaB822RKA3RWq5Tj",
  "Ppagsptg1zZc3ihX",
  "UPYDKlQTVPAdKvoO",
  "v0e/qFQDKkK6qALN",
  "zSqVElCflTrvSChE",
  "f0YsJtavQAqIY0R9",
  "1sosJqCzKX2hgHS9",
  "oEolfk9UCPBe+nvF",
  "R7xmpJWJZCV+ZXe7",
  "PGuKhL6L403wAiaS",
  "nw9ctUs0NbFeBdeH",
  "3eK1MZ2IpKmkXa7a",
  "VXGSVHST5LhpdAzq",
  "LiPDvEor7LYzBdUM",
  "K0SUunJUJejHBtRZ",
  "yMk9ghfEL/Oz6YX/",
  "vULliqQK4jr013rz",
  "yNVPFOe09LRPW5ta",
  "SQJ9BmCpry6F8qoK",
  "pOfSYlf5EFqMhL5u",
  "CPI4ytDpDRtqEYNJ",
  "0M6gg2RPGdOxzX4Q",
  "1cENAwVcP2A1G7FM",
  "Ok1LrboKUbnKVTOA",
  "LROIL7TWUV3KOd4P",
  "sIsW5xBp4EVSK/y5",
  "cZGrrIpo8Kl7TmIe",
  "UQOZ5YhMnGSYBW9K",
  "RmoNY3wGNx8LTwMP",
  "c7wlEn4ALDGQ+RZ7",
  "IuLv8hvCnEg3VlF4",
  "iWNT9zk/uJWFMS46",
  "ic6U92F8WaXKALJj",
  "VhtRpUzpbIYck9Ch",
  "8yUGLlWfH/pIJkGp",
  "w4EyZRiVgVNwhKSe",
  "Sy+9tAZ8IgEIvw/w",
  "ecghh8QTugZM4zjx",
  "+tvf/paOPvpoBuCI",
  "bdWkH7zeeeedtNZa",
  "a4XsqJ3V3Nv+v55U",
  "Ja5ye/IQhp5YmcI8",
  "OeiBadc2y/GOygqa",
  "ISsMS+ESkVEMcjlR",
  "loQ6XlnincFihpne",
  "zGzK/nr+TP407lnc",
  "2nLs6v7m3zTbKVMp",
  "LGykrxm6/y1XeZjo",
  "ZFbT2Gn+zgegT1K6",
  "OUsVVyoaFQCEqxVK",
  "eWlqXmQsrbb2mjzJ",
  "SjU3hbJ6uLYqwI8F",
  "/2M/qOOt3WI4MJqN",
  "hInV6+VqCAeeCVjw",
  "2S9+8Qv+H+NtP+yP",
  "RLRc7CofOouf0D3b",
  "dFNu6s993RCdBElC",
  "GkSvrrvY5mxaD1oN",
  "D03UeoYLqJFkBNdx",
  "aeUN12XmBA/uQqlI",
  "eZRpqwbMBnD8lrHQ",
  "jRW6lSWrVaWL+Pct",
  "sCmB+ibJxzAZ7Ay3",
  "Y8f8yDUHiZUyGFB2",
  "8+G38KC3ZFTM4BOE",
  "2bOOVBViYGpYEq5O",
  "AgaEpHqJSX5gd7wB",
  "jGVUA9KSlXADmqQh",
  "To6wwSkLQ+vAHIlK",
  "C9iV83fBsDgJam+b",
  "SbmZM8hLJSkol7g6",
  "UuiC9oURxL1ZZJFF",
  "uN72QM2hVHzdjs9E",
  "1jhAkU4+4PIFyIR0",
  "0RVXXEEnnnhimPQS",
  "25xNwbsCPgBKJAmB",
  "FUI/UxAwZswY+tvf",
  "/haWGW30+WVXarFN",
  "suIjtydXqzKlYfEe",
  "5SdLeWGhpY9JP9HY",
  "x1AA3frfFkXXxBso",
  "O0RZ4ZJUJ5nowozK",
  "RC5hLbWfcRKenfFN",
  "+pmRLzJyQ3Y2ub5X",
  "9tKOrayN4YyeJ6oK",
  "UX+d8LwqlEtEmQwl",
  "silyUmmqelJRyK0G",
  "tOw6a9OyK69MATwM",
  "jhHuNwlUktmvAvsS",
  "UvLiiy+FE7ChmIj1",
  "JQehp88QRw/ZtHQ6",
  "yYk9SBwrlRD+Ai1X",
  "jzbffHMeU5Hc1g/L",
  "GaUZ5GPE7M8QWgw0",
  "Z2/tRHQkEd3S1w0/",
  "+OADZmEwcAIkxYxm",
  "76aiu2oaR/fJJ581",
  "1Ez9UpkWX3UVamoZ",
  "QV5KSr2VE+IWVADL",
  "WZ81IuuW9lxNnGWU",
  "kKBxVpx5qoObJboc",
  "DohGwkhkSWTxQykW",
  "STJQTTwBeXCrCWui",
  "7jOVTpFBy+j9mW1U",
  "GxD7wnayrsSbyeAl",
  "YFKALD43FVAYOXsi",
  "/RIeixmcTZIDhx+Y",
  "uvH5XI6mfvwxH3cx",
  "L/qLCLQHEFUQWEIM",
  "oOPSsssuWxNXOxDx",
  "h+o6W2mllTgjWkEn",
  "wCUAEWqZ33rrrRwX",
  "rZOTGGj2bhrPCTCi",
  "npaHH36YqyThPWI2",
  "EfOGPjdu3Di+xlgf",
  "/aeR51dUEcuAWpMA",
  "hxBlOzzBZjTdhENF",
  "1J4vFMNs8AigIfxE",
  "WESedBlAp8kx8t5M",
  "yDjZxg8nY2AHeSJm",
  "staVyefPAT5NGUn5",
  "DOuY5wU/M6LfkXKR",
  "kUD71387yg6PwKlm",
  "i4s0k4LMUAZJ5Zj4",
  "uyjGNJPMUKVYoXwF",
  "hTCljCwAsHSwBG2w",
  "3bZECAUqlqkrl+OM",
  "cwlTwERLqo2xxJrv",
  "c7lRTNCja10d9oym",
  "SnShfSLUicuUBojD",
  "lHN+5pln+nsIGMfH",
  "zY12dmz9txhoztkQ",
  "pf4DhGH2dUPUIYZO",
  "HR7ccYnK3g0PQx0A",
  "o0SEgAO91Y03J8Ns",
  "t3X8YjRqyfHsOkeN",
  "5FJVEhCErbTLv4me",
  "pEoY1QisG4CpbjH9",
  "Poy1YoAYsZpaNSQE",
  "mSbrW0GqgkqpgQwA",
  "WjYDE5hJZWHK5jcN",
  "8Ovhfxn89HcjAWr9",
  "XNfVrHM9FpGBUfeg",
  "DL6ID5N6zMyHhowm",
  "s03k0IyPPyEPSUB+",
  "ldLJFIONlJdkYIBX",
  "ZbiWXHKClB4ciLRz",
  "YzrYoLoN2DX8DyYT",
  "LA3A5iOPPEKHH354",
  "TXhK7DHo3XSAr5cw",
  "glTUBRdcwG5KsMUA",
  "m8ji32CDDeicc87h",
  "/xtxvdrahj1V+VLX",
  "rjKqehy5rm6qlhBJ",
  "afqLtnWAQY6ZljhJ",
  "lQsK27RVGAHtn0tL",
  "IlyGCx9ov0IIiWjb",
  "Rn1lNothNhlgYn9G",
  "nB2f2QBSNTVrhdyj",
  "CZsNMiPdT/meZZos",
  "xlNNYSBy/MoVSXRL",
  "pbNU9n0qlcqUammi",
  "tbbckiqOsJaliqkI",
  "FCaFVfl+6ljzxBNP",
  "csyqnTg23Kw+bhPt",
  "AQoSuZxouoLFvOaa",
  "a6irCyRkv+1YM47P",
  "dbHz2PpnMdBszE7t",
  "a71TlL3abbfdhnVn",
  "H26mLlJbkw8ZxY3I",
  "R6UzSRq1yDhacaN1",
  "KQgkU50ZPriX7LhK",
  "jU+0skDVFCDWrG/e",
  "aCqSX8dq2uLMYWY3",
  "l4PUOE79TuK2VKtP",
  "mA5x7XEyAMeaWaLU",
  "cKObuK9wEDa1mmVw",
  "k/VZA5DXF5YzdJMz",
  "kDbAUyVZdF8qFs2D",
  "n2zP8WhVhLV5NOOz",
  "L6Q8H0Bk1edBz47x",
  "w2CH0nbjxy8e1qYf",
  "KMP+UY8bWeRahQsD",
  "EH4TCT8HH3wwr6Nh",
  "KWA456eEnnlpKvOk",
  "oSho/wA1APX33Xcf",
  "/w9XOpKsEDd5yimn",
  "cKnKvl1f6DjWFlNQ",
  "bUdNwEtopSCw5rk8",
  "xweHMcVGp1a1MhXE",
  "hZV9QkbRsIk17yVh",
  "B+AS0FUr/1Qhd2Qq",
  "B+nkirflz6IJF2tr",
  "hpqawm7CCxAxm8qg",
  "Rr+p+pu1uprR5DRi",
  "MetCAHQiqomHfoJc",
  "T9p0texzUhKAIyZ3",
  "a2yxGY1bbgXKNLVQ",
  "Jp0R+S5XNHhdlGNF",
  "hSDkAplJ+n//+99Q",
  "lxhi+Y3Iuw22zS4h",
  "KNJFxvNf2slPf3oe",
  "s5iQK+unXWLE167u",
  "745i65/FQLNxu5KI",
  "tod8Zl83xAPjhBP6",
  "hFMXUsYFDKQOauKm",
  "fe+992na1Bm9bg/g",
  "hGoZq2+6IT942Q0H",
  "1sQ89LUiDkzd52GM",
  "pmEEI0HzCJzKvqMY",
  "TfnfYjVNjCYPSo6A",
  "QJVJ4XVMLBhiJ+2B",
  "SXUptZYyA0QjmQLX",
  "NxaNsWRZJNbUAxuJ",
  "xQBiHSh5EY5EE540",
  "y55jzCwdv5oYTT1f",
  "sEdyQpzJO/WjT2nW",
  "rDYegItdyNiuqHYL",
  "L1z3nIgmLLGkuEYH",
  "KNwJAAixgccddxz3",
  "GYBIzZaG/eAHP6Ap",
  "U6aE8ZyjRo1ipnM4",
  "JDvMH6EpUjJXwxQA",
  "LAHmAVzOO+88riSD",
  "a41rqxqN559/fsNF",
  "KeqLLsA4btCrla1S",
  "dQ5WWUBVogoku2T9",
  "sCxjyORHIDLUiLUY",
  "+4gFJc5QRzyzur/Z",
  "JQ4dUMe3YjPl+9p4",
  "zaiUJYe4sIyYFY9p",
  "PguF17WYgr5H+IsN",
  "KOtjNK34zZ4M585e",
  "GCMpxuEpBpw3tbTS",
  "hrtuS25TExGUOXy5",
  "b7xdGAYkJSexYDL2",
  "1pvvhPfClsAaLjGa",
  "PWWiAyifceZpfKzn",
  "nffT/h4OSkfuRURn",
  "9ndHsQ2MxUCzb/Y4",
  "EW3b1ypCsMsvvzyc",
  "2cf2dVO9TAlLihIL",
  "PvvsS4557c1kG5dW",
  "XGcNdv+Jew61KEX6",
  "pD6jVTNVe7Kornkd",
  "u6k1i9XNruuDMUGm",
  "KwM3da+bpB+wmBYO",
  "wvecNc5gVgYAcb1b",
  "7ndT9SeKBeUzlEGL",
  "BdtNli2SjAxTGpW4",
  "FDYTotX1On783lwI",
  "zYRnXT9zzZOeR4Hj",
  "Uuf0mZTrzFESAuBc",
  "nlJOQMsdCkvvcCyf",
  "Ziv317SWOcTDYRhM",
  "4Q7UuuYAny+++GJY",
  "IUjrekOiJ46BbswY",
  "wBjtUFsCCtcdAAVx",
  "5RrDqdqj++yzDydi",
  "9GbSBOziCjI5QNw1",
  "9mkDDp0Y4HdUeorf",
  "m4Q13odO+FTtgbUq",
  "dTKoWpNRIQItQVly",
  "peJPAG+BTrRctHxT",
  "d9yq4GN/prnmfjhB",
  "kz+ZvBm5HV7PKEbY",
  "meWh1qeZj3HhLXOM",
  "2g/N8yac5NVJVKUy",
  "ad7eMbq1eWTik8PP",
  "shXWR9lJlwpdOeru",
  "6GRNTQ59sCpA6QTi",
  "lVdeoUmTpoTXDRNI",
  "hCANB7PZTPv/ww47",
  "jJ/Rv7j40oH4meMR",
  "1TM3Y3Rsg2cx0Jy7",
  "2RK0Nu+dm43xUEAF",
  "Azz09QFc3wF1MNe4",
  "pkYG8t5mlPN66c0k",
  "jhVxmQI6WUcS14Uc",
  "+tdjzzDW8QmxjBUq",
  "lPKsF4dBwkMyCLtd",
  "8NRtp/GrrkHLbrCp",
  "aF+CaayaKsYaf2Vc",
  "a8LiVWVbSggTgl8I",
  "KhxLxdIicF/5cOvJ",
  "wMychDkffKaB/QxI",
  "fbBEun9wj1XzXpjI",
  "ahXxhC5VfOhyBpR3",
  "fMrhFVqXVWh/VikP",
  "HUAf7IYnn1XxvU/5",
  "oEpdYGtwnFXoCIoG",
  "Z86psP4ftAQ1mQE1",
  "XBDRit9A2IAylXru",
  "iF2FpiADXQa4Aoqh",
  "ylcISkQpj/xCjkqT",
  "PiYn3UpOFqOvxOkV",
  "SxVJqEh4HD+22Pjx",
  "7GZtNPW8nhlTlk3u",
  "v0Pf/OautO222/J9",
  "B7iU+MAk/elP17IY",
  "uw2OtK43bH5wnWsi",
  "1bxacI20ghLMfpXr",
  "WqWbbrqFnnjiCV5P",
  "NUtLpQJdeOH5DGo4",
  "Ptf4IpNGOgyuXf4s",
  "lBnCO50aJai5JS0T",
  "yKokImW8NMt+OX6S",
  "3EwLdU2eQk4SFXrK",
  "zC4WyKUcmf6HeuQO",
  "+kqJCsxKoi8Juygh",
  "JmAi0a9do22LRB+J",
  "sUTfxfboHyXoTkJP",
  "E1qXPsnCOpmogR5Q",
  "ibVysQ+woj6hICx/",
  "xp9j3TLlHIdylKA8",
  "OZQ3n0vYiSxIPKya",
  "amBhiAreIwQGyhEs",
  "SQZPh8s11ItIhEKf",
  "cpCIBNd3gtKZBCXd",
  "BE+eCmXEtVZos523",
  "pabl16dKqUqJlEtO",
  "2qFk2iMvhd+qUAWV",
  "nJwsJShFyWSG7r/v",
  "AW1xxiuEjPTeVTvm",
  "xmxGUicPOhmV73U9",
  "Lfagz3ZZdtxxBwbH",
  "KBYwAHYGmigR/XYg",
  "dhbbwFoMNOfOJhPR",
  "3sad3meDiwrl4BRk",
  "agKEgkoFmnb5tgXd",
  "cO4ai2ln6uMVTJZU",
  "Moncbiz4bLbhgdIv",
  "Gy06h9bZbityyMVo",
  "SHlcU08pxVrGJNS3",
  "VEBsaUuGTCEDR8OE",
  "WFnoof6eqQrC1XwM",
  "IwMGRbX7hHl0+DMu",
  "H6mueONyZykiLn8X",
  "6XJKqcnaReomyzqS",
  "mW5peZrzkIFMZJGE",
  "TTUhBLo/vDcSSqLv",
  "GcWXIRkIDCYYGATg",
  "T/74M47t8hICVLSt",
  "KjBht15TkwDNBk1B",
  "ot5DjftUHcCf/vSn",
  "HI+Je4tX2CeffEJn",
  "nIExJLbBNL0fl1xy",
  "Cd8LhCQAaALMo678",
  "HrvvQeUKWLRI69AB",
  "yMTzyjCMczJ2eZv+",
  "itrcXDSgXKZ8V3co",
  "pg8L+5epXa6udOLk",
  "OfwvVXwk/lE+0xKs",
  "YUyz1g+3JoKh5u3s",
  "Fu1f4XPAcs9btcjD",
  "OFJTrYgJAZ54aulX",
  "E2Jj9U8NfVHvgcZO",
  "K+uIdVAIwfEBCh2q",
  "plPc7zLNWVpvh50o",
  "lYRbPMnMcyqV4X1p",
  "SEka63pCWCCsRLOz",
  "7f46EKEldrUuNb1n",
  "+juanCfjmG5HYawo",
  "YjDRpp5//nle58EH",
  "H+r3cRHRRVA+IyLQ",
  "oVGVhtiGlcVAs392",
  "wtzKJWy88cb80IU4",
  "sl0eTzULVRcNHVNd",
  "Xf1hDOcXs3UU9eH2",
  "6quv0gcffBReB8R8",
  "6Tp67mA42VUWVGjd",
  "nbellpFjxfnF4Etk",
  "jUIdPBP/hSElLNVo",
  "u8AMYynVfWTwCMGm",
  "Djoqxhwm7eBhb2Wm",
  "mlJ3vB5YFgeMTZQ8",
  "JIBR1i1yTXMnXLSy",
  "iP2ZLe2CfdgyLLJP",
  "63NNgjAyS7pdlK2u",
  "sWUGhOI3KtL+cFzI",
  "av3q3Q8gTsq+QNXr",
  "A/AUl6hMiJBYAj3N",
  "vhi203gyO+P029/+",
  "Nq222mp8jwFEuWKR",
  "59HPfvazsFxibINn",
  "OslFIgky+7USi07q",
  "IPCe9JLcZyBtpdqN",
  "Vb/BrH+EifiRl0Y9",
  "NnAFl5GVLr3RUmwQ",
  "8KfJPmEsspmUMZMY",
  "SgwZnVlLjUETeGQf",
  "Im00xyVMNNJto0Sk",
  "sO+YvivxmpHKhADH",
  "aKlPZhIdTvRneBkA",
  "kiNNTbbA4d/EhDRf",
  "KtIMSD6VyrTSWqvT",
  "hA03pEo5z/0ToUDi",
  "JpcwI5TvLJeLlMt1",
  "8b148omnaeLEyeFE",
  "rr6Ma39N24KWcVXm",
  "Uj0KOumXsQseO8jL",
  "ZdhbgVAXECwY6zD2",
  "DYC9b3ImziaifpUI",
  "im3wLQaa/beDjSt9",
  "rvx3r732Gv39738P",
  "6zfrrFAXuLEWFtMH",
  "ozJfCqLBrjzx5NP8",
  "QC4VTeWgQJgwmOs4",
  "lM6mKO3Bz0s0Zpml",
  "aPlNN6RSocTVQfJc",
  "lk2TcKLscS0Rp0yl",
  "ZKzKYBEmB4Ssn8lK",
  "V1kTA+DENU5fkxNi",
  "vT4jaaK/rcDTXuzB",
  "URcd3OxFQaTujyVY",
  "zMCnJTFl8NOkhUgW",
  "SZMo5DhNtrzl4sO+",
  "IWcE5heDGcIFprz7",
  "PpWLeUbWyIZVjUS7",
  "9rxWCGrUMGnS+syc",
  "3GTYTHx2/PHHM0sD",
  "KSOwmWj7L7/8Mt19",
  "d58KdMU2l6aAH/f0",
  "N7/5TSg3A9Yartyt",
  "t96aVl99dWY060MV",
  "Gpnoavw1/5+IWLbu",
  "9k5OBIkmQJoAJBnf",
  "InEk4I/bPCZtaDuq",
  "e4n2n4D7GWEvEjYS",
  "ZqJb72uTf76+SPY5",
  "USnMOje/aRYOReHj",
  "kYz2CMxqH9djFj1P",
  "fKcTO2h1akY7A0qL",
  "acX2iCNHKUkwtd2V",
  "KqW9NLWOaKZvHXII",
  "eaNGMruJvpH0EA8d",
  "1Y5X7Ul9D+1TDZPQ",
  "+wn3O8Bof01i3mVM",
  "6knKimO8k0kDQpW9",
  "9nkyirAMeEmgxDKA",
  "BM8qJmcitvnAYqA5",
  "MIYgk3WI6J652Rjl",
  "tNCBUbNZS23ZJSwX",
  "pgQifaDZcar4/1//",
  "+jeVKngUi0tIXHcJ",
  "jlHy3BSlEIeEwPsq",
  "UcpN0mb77sW1u4MK",
  "mEVxeUmSjgwWzGqa",
  "cozCcpra55YMECfy",
  "2IMfYrHMQCJgT/8P",
  "qJCoMBPJ7Ai79AA0",
  "RYeP663bMkg6yBjA",
  "GkomGeFpuwIJs6oY",
  "RDEghrp+ETjFEKJZ",
  "uRoGEMWJRQxumIFq",
  "ibuHsaeoFW2CClwn",
  "SeQlqe2riZSbNZMH",
  "QT4pyyTMA7A1oCUn",
  "jO+TaLsOimr4f7vt",
  "tmP3rB2zDO1OJAbF",
  "5SWHxhT4Afy/9NJL",
  "9MILL/B7uGvBRmHZ",
  "a6+9wnsIN7qGujTk",
  "UUEcHwMTI/cGthzA",
  "qr1NwFGN9JDIFNkT",
  "OUwVNR6TYzLDyj4a",
  "YxnJD3EspukXHH/N",
  "8ZR+rwv2yRM6jgYX",
  "dYeyI+/rgae91Aq8",
  "2xNRfK86n/K8KCek",
  "L9fIIiUc8twMTxhn",
  "FatUKeRp2ZVXpLW3",
  "2UFE9g37b8fc4tpr",
  "Ih6u//vvfUhPP/10",
  "KOum1qhiQO+3z7j5",
  "Le+afUwCQlFjXvRM",
  "v/Od7zKDOXUqiuwN",
  "mCGGJj23IWuxzTuL",
  "n+IDZ28S0beN5uZc",
  "GSp1qMSIHbu2MAy2",
  "ANb68NL6yHZs0Wuv",
  "vs6ZsTqDV+YTAyEM",
  "j0G4fMHIIT1h7e23",
  "4kpB+Xw3lSomQ1UT",
  "dyx3uQJJlTmCi0sq",
  "Aen6cFsb8MYuNqOF",
  "GbKQGFR04MBAaaoA",
  "+eZ748bW0pE88GFw",
  "VJe2qSpkD1qcUOTX",
  "DmLKikiFH7OeyYu1",
  "S/HVgExlTdVlHzK0",
  "xr1nMZ6FMmJcxW2d",
  "SHnU2TaLZn72Bfme",
  "w9fVBhSIydPBBolt",
  "jRrul7ZnezKBrFO9",
  "p8rgo1YzxMR1YItt",
  "cK3ei3DLLbcwuASb",
  "iX6I73fZZZfQ68Lr",
  "9kHWSu8j2q/Dm0m8",
  "ZvfMNolrNhM+nrwZ",
  "lzk0aG1JMDCKAG7M",
  "UgIcIsHHYi2jBB0B",
  "qwCcrIPJAK8XVpNZ",
  "0oi1tLcXwBiBTZlc",
  "Ru/5dw0wZXCqzKeZ",
  "NOrnnKhn+m4Y0oL4",
  "VidBftKlLvRxJ0mJ",
  "pEN7HPR98rPNlHY8",
  "ynXn5ZojbKAayVNp",
  "RSc8E8FmQntSxw3p",
  "X+JFGAgdTSU8bLCr",
  "45ROHJdccinOcMfn",
  "f/nLHQPJYF5pxWH2",
  "Xr0jtmFnCz6CGXpD",
  "FaEliOi+udkYnVMT",
  "I7TSg1bTWNBjNO2g",
  "cnX/KHs5dcZMeujB",
  "h8N1JJFE3EkVUznI",
  "QVamK+7Z5tGjaKN9",
  "9qQg6VAV2dJh7KWJ",
  "0dQylAo6GYiKaYIA",
  "J8ywTqRhQo3gtDKi",
  "kswjwFOShrRUXtUC",
  "mFKfXJkYjqFELfSQ",
  "4RT2s54hYdYD/5sl",
  "FHmuqf8ssWKi+xe5",
  "HPW3VbA6FJnW2E0+",
  "l0gUnmPO2H2H/xzO",
  "KG9vn0VfvvMh+SYe",
  "L3STsQtdrhSuM9pr",
  "o4ymDRo1Vm+ZZZah",
  "TTfdNAQ6GCSRYIRM",
  "VICchWWiNRxMkw9h",
  "Dz74IH344Yc1rBnE",
  "27HA4EJHrGajBiWG",
  "8DllwAjueWdbu5V4",
  "I4k8yvKHAu1ci1z7",
  "oD3Jc6Xda7s2bT+a",
  "oEXMooLI2S412p3G",
  "9W2HuVhMpR2SEsZc",
  "h+VdLb1PK6469ESE",
  "8dV2LHWCCm6Vun3E",
  "NiZo7a03pXV33I66",
  "8gD5Ka7YhUudTEo/",
  "FOZQXNOQdJs8eSr9",
  "+c9/romZ1L4mskf9",
  "V2VQl7mCSt3/Kqus",
  "Ql988QX/D5JkgO1G",
  "AzDhKo/jMOdji5/g",
  "g2OTTFb6H+d2BxBQ",
  "RueFaPXC4DrXknha",
  "Hg8gWz9jMF3xWXpF",
  "WTG9JogBKhXL5Jer",
  "5KRRHtGlqoM63QXa",
  "cu89aMySi1I23WTp",
  "SUo2OYxzwE1GKEy/",
  "gztdzGjhWULuzH5q",
  "qUoD/kT0HIOuqUEO",
  "UXVHs1NhrkgJGQYj",
  "rFhiWFOOGwVgVZdb",
  "OJhGCwu3G/c4H5MZ",
  "OyQRyVQbUibWvEqy",
  "koYGmHVUcoUTjIyb",
  "0sSzFcslAfGc5FGh",
  "6V9+JdmwCvQSct1R",
  "FYjfUoLLRDZqdnKC",
  "xnRtscUWrMep8WYY",
  "GL/66isu34p7u7CF",
  "jswrQ6iCehBwzaFR",
  "+u9//zusJa96pqjY",
  "pP1S20WjzGbIrCUM",
  "u1mtsmA74q3DsA7D",
  "lodZ5CYjnMNRuL1K",
  "/XIBpSKEDj1au+KV",
  "TAajDHTRvexlCTVr",
  "pT9FWevRIsoUJgxH",
  "j49/F9urV6Q285yf",
  "LThe0791osj91GiD",
  "Yl/5oESB10RNGY/2",
  "O+wHVEYSDbLRC0bG",
  "zdNJgMhE4X8wzslk",
  "iu699z766qtJfF/Q",
  "h/T+YMFzdCDmaXYf",
  "RH9F5jjO7d133+2T",
  "V6NBQ2D2okR0aAww",
  "FwyLgebgGZ5fx5hk",
  "obku2Dpx4kTu1GHi",
  "i8n4s80evJXtU1e0",
  "vh/uZh+7vtrxfI7j",
  "0osvvkyvvvq6ydbE",
  "AzdBlVKOHDcgB5mW",
  "gU8esmLzMoiNXnxR",
  "2uhb+1JbpUIdZYcD",
  "7X0HUj7CHLaXy5Sr",
  "lIgQMG9YP3Vbh246",
  "3zAkAGcusscNINQY",
  "MLi4UaHHAEh1UyOD",
  "1K8iTtNlTcAup0Sl",
  "RJVp0kRFSq4h7qzo",
  "VKkAd6EfMSJS9i5y",
  "4cmxwDUIllNeoctX",
  "BPvIGasOa/NhwXpF",
  "/h4uRsSvGW1Nx7gI",
  "HU1iwuAs3CQqKbHm",
  "npulhOtTAbWis000",
  "5e03ORGhAiAAmaYq",
  "LlWVX6HPB/C6+GJL",
  "UJJrpPdinMAeTRCk",
  "bVZpxx22ocCXSjRw",
  "m6OdP/nk0zRzZhsV",
  "CpBwQThE/1VLeqpG",
  "UlOZZJB1LIe/gTlG",
  "CEsq1LNFkiJCUzj7",
  "PAkJsQJts93WYSwx",
  "19y2Chdo38W2+pzS",
  "OM50kCI/AR1WuLxd",
  "SmeaKD9jJuU7ZlIF",
  "2pqm7RaCgApow0j6",
  "qSaoWPUpj2pF0L1E",
  "m0dmNnRUA4CzKhX8",
  "gApV2a7IC/QvRStW",
  "9GQdecV7aNj6rlmC",
  "us/QJ6z+VJUSleFS",
  "DUSvFmLpVZ8qyAAH",
  "M2uy1vk4fPw+fs+R",
  "eGruv+LJaK5m+Hza",
  "ggpVkLzoJqlYDajg",
  "QFvXoRkFl/K5Ttpk",
  "9z1ppS22J9dr4j4G",
  "3Xc3AWkjEdrHvUG/",
  "wIS6o6OTuruKdMXl",
  "vw/vIoAl+hWWSLOy",
  "sRaA5ynMbq+qiwkm",
  "+9xzz2X2ctKkSQOV",
  "OV5vLxHRRkT0HSIa",
  "0ODO2OatzQ9PwPnd",
  "IH/UTEQXzu0OwCLg",
  "ga8uCiwqYQHTsm4q",
  "LaGs4EBKW8xrwzmh",
  "DvO9994ryVKuR8lk",
  "mgcynHOhXKSk4xFl",
  "UuSnkjQi2UTZphba",
  "+fv7UdPo0bhI5LsJ",
  "6uzOM9Aq+BW+ZtlM",
  "hpNfQjMVPfQ1qg5k",
  "6Zly5RDDcFglKxmM",
  "GXqEWRdUCzLrIS5N",
  "Ys9Mcg5YTl/ZG7+O",
  "vaxlU1gKCQDWZLdK",
  "nJhks0bSLwagMmiV",
  "+DXJypVt1D0Zsj1g",
  "UcPjl/AEVBzhuEkn",
  "QYVSiaZOmkwds2aG",
  "lZBqWCzTDpuyWc5M",
  "bsQQjqZMFgx1zJdf",
  "fnm+txggNd4LSQ26",
  "XlxecmgMzxdIz2jl",
  "INzrt99+m7788kv+",
  "XuMyUSIU900/w+MF",
  "8jWwnkJ4wvCe8DEU",
  "yZJpFrNov5oqWuZ/",
  "ZvU5LCWS+hKZICPH",
  "pW5rUxtdmHmjDRtE",
  "kmDy6hrgZyX/GCCo",
  "n9lZ4+F7OzFI65yD",
  "lTThMqH0Eto1exDg",
  "TTD91nhPNFQlX8lT",
  "Kp2mEckWKpd8mlEu",
  "ku95VChW6Mtymdyk",
  "R6nFF6G9Dz+EqgmX",
  "ChBoT6bI8VzyUnLt",
  "VYeUa6C7Lk/K7rjj",
  "Dpo4Ue5Rf01VIfRe",
  "KzuKewTmElJjg8Be",
  "wk4yLvKNDNiMbQGz",
  "GGgOnZ1DRLsR0Wdz",
  "uwN0cnR6ZITaGenq",
  "ItEsbfoawzD8K6f0",
  "ajiPakD33HMPs10J",
  "DB7VCmVbmiUzE4AE",
  "wCSZ4oECFWWK1Sot",
  "vvqqtOUee1LFr1K+",
  "IgxfZy7P27opl3J4",
  "aCfTodtc4zP5J8PB",
  "UeMiIykkSbQxjKOJ",
  "u5LPksJimKxY6Gli",
  "G4xOGIfxHqyiZJcj",
  "flPc4BJrJosmN9hL",
  "TaJQKGmk30UDaBS/",
  "Zmqcc4KFagoaTcAQ",
  "ZIKtMclNGLwBAHGQ",
  "+NxxaNrUqZSfOUtk",
  "VfCxSdSyM1ABOhoS",
  "bTel0uVWykRp8cUX",
  "p6WXXppBjk6WwGo+",
  "9dRT4WDXE4Mf28Bb",
  "fQUy3CN4Uz766KOa",
  "+437NXbs2HCd3qxe",
  "1BumkxWAJmhohm3W",
  "0qmUJQol4T5j6caC",
  "ndcYyJClt6WKTAUh",
  "yfi2ssSpEk3C7M/Y",
  "eyHVg8LFei/10Y3u",
  "bRhHHenZRvJmUbxn",
  "qGWLPpfE9QsoiwSf",
  "TAs5mQzlEz77hcvZ",
  "JuqqVOgHp51ELcsu",
  "zdXCWkYuwhWM4BqH",
  "woPoUooqAyZ2YDi7",
  "OnP029/+ZsBi9HM5",
  "0XMGuESYhCZaDqIB",
  "YEKT7orYRb5gWww0",
  "h9YeJKJliej0/uwE",
  "DCdAwcUXX8zvlflR",
  "TULbEMezIBgyneGh",
  "/ezTzzkbuVSuMBjB",
  "YMUSHuxig+JeQA5c",
  "WmBFqgHluou066EH",
  "UtmDJApK3DmUaRlB",
  "4xcdL0yKm6BSHkOP",
  "JPgAZIGZ1KB/KdFo",
  "QJoBmZrZyoMegzsp",
  "ccci0pxxLsk+XAZT",
  "9QEraXbJlQhAsySC",
  "7gB5vkcVStbIp4Ty",
  "SdZgKHFpJu6TGVFz",
  "jIZRETkmI+yuep0q",
  "1M6ud82WV2ZVjh2g",
  "E/GZ5IEJD3jgx/aU",
  "SlKus4ty02eGlVLU",
  "ADuQ2Y9XDEhYer1/",
  "cMsFtZMfgBboZnKm",
  "uwGxYO0/++yzMHxC",
  "k4RiG1zTcBub0cL9",
  "eP/998NCEjDca42p",
  "lecOpHaiGud2aIQN",
  "LuUz45o13oFStUJl",
  "UwZWJYTsyZRmfIdh",
  "KpqQY9pypPQgbVhC",
  "X4wsUZglLhMwjedU",
  "hQYJU8FnRu/SSJHV",
  "aGha76W/SGIeWEvt",
  "N7JEwDWSFdNytAYY",
  "w9Wfz1OuWCAvnaKk",
  "41Iu8KmYcanQnaP1",
  "tt+Odvn+/kTpDGUz",
  "zdSZ66aikZDyUpKQ",
  "BVMPDp711113HX36",
  "6acDNhFT17lqFw+S",
  "4XYeaQHMwmD9UGzD",
  "x2KgOW/ssoFwE5x6",
  "6qk8YB9++OE1khfK",
  "DuE7uMN04JifzRYJ",
  "/v3vr2ZGspCHCymJ",
  "XBuu6e05SXKqUuYM",
  "rvFsKktNqSwtttpK",
  "tOuB+7F4O9I3uzEI",
  "sao6gKRLLhIeQpF2",
  "yQLVcnMCziw9P5Un",
  "MXIqAK/ymZEuYVYF",
  "MVpujc5lBCBVdFoH",
  "T3F5w92tJfDYJW6k",
  "UTQejt1yfEwR86oJ",
  "RpyQYNdZ1uz0sNSl",
  "gGU5B5MxW1/73UFV",
  "INRRlzi6qutSvpSn",
  "aV99ReWKgAyWpalG",
  "agBYcK1bmpsbuH8B",
  "u871PqKdLrEExBnE",
  "tAoNMp3BpCiwmT/i",
  "G+d/qy0fGIlwowSo",
  "XYIU6wFo1ofl1IOd",
  "MMHOYjTt9zBo3rI7",
  "mIGgyofZEl3ymXwX",
  "9T/9LuyLysrzfiR0",
  "RMCfuL01+Uaq89iv",
  "VuUtS89W3ehaVlLF",
  "3+uF3O33YVWgUFxe",
  "koLY3c59P0lOKk35",
  "SoFrp3cXinxOAJ+j",
  "RrfSsef8hHwvRelk",
  "hpOj/KBKo0eOoFKh",
  "SPmiVADSGvRdXd00",
  "Y/osFtbnWO8BmIyh",
  "m+EZaodkDbA9TESb",
  "QdqTiK6NAebCZfFT",
  "fN6ZBj73u5DzNddc",
  "ww/wffbZJ6wMgUFD",
  "AaYG5M/PhmEKD1Uk",
  "ojz37Ev00ENSJk8r",
  "ZUAA2i0DnCRY+xGk",
  "nBckKNmU5RjMPY48",
  "lEYssggD065SiSa3",
  "zyIvkaJCd4G6IQat",
  "ZRxDIWV1wal8irAn",
  "cMOFEipmG2Y+jIsP",
  "LCa70xjUuQJIEz6V",
  "3DIV2WXnkR+kqBJ4",
  "xi1XFZazTt5IqgpZ",
  "701GKw+InPktrv1Q",
  "XsUSmteydxJTJuuG",
  "A6NhMUNW0zC0JWQW",
  "m/eoEFQB8KSApk+a",
  "SE5CXHYwG4go+6Ux",
  "e3O0QNh1m3lfcskl",
  "Q0FpBTEqkYLvsa6d",
  "DR3b4Fm9dI2Kgs+a",
  "NatGRg2fI1Qiks+R",
  "JCIYR130ADhDtQb+",
  "SllO6bcC5iI2U9lK",
  "CfeIQkOi16/3P9WU",
  "japw+cZlLeEpUjhB",
  "3O1lJOrwZDByh+Oz",
  "qAKXSp+ZyZiZSNru",
  "9kjXU6XHTD+Ciz50",
  "/aNvR30YiUqouIU2",
  "jklwJ4Tb/YB1Mg8/",
  "43haYtVVuYKOB3e5",
  "61FLSxNVyyUu4Qgf",
  "C/qJliNG2ddrr72W",
  "pk+fydd4IAhN3Pox",
  "Y0aFIVgDZHCHH4Bw",
  "bCLahYieG6gdxzZ/",
  "WQw0571dagKh51oK",
  "Se0vf/kLA8wjjzwy",
  "BJgLSnwb3Do4Fcy6",
  "oSl30UUXUWdnN3V1",
  "o86vx27oXDdijITN",
  "SHpwYZepUClQxnNp",
  "0RWWp51+sD8FKTju",
  "EtRd8WlmoUDJTJZd",
  "7qEGpmE2NIlGgvtV",
  "o9IAQI0fYxAZlaus",
  "cqWSqhmEoI/piXAz",
  "xJodxKIhHhPHahIU",
  "eKBTWSOqWexKJ1IZ",
  "RQbZyE1vCbeHLnfV",
  "54sSF+zYTQGnqkdo",
  "BOlNLBkY4rxf4X0j",
  "MYEZVCdBUydPkjhJ",
  "i6EKS+AlHNZSbFTi",
  "SCdAmvQDwKKAUpl4",
  "TT6xmU9lN2MbPKvX",
  "rVWDR0S9JeoKrwf+",
  "PeGSekbTfg1VJaB2",
  "APkt4+5WXUn1GEQx",
  "mgIMQ01Kq/9prKRO",
  "+LCUjDehJsaZYzM1",
  "AcgAR+szyVCX7HTE",
  "YOqiqhO8f1QJMvGg",
  "HEeNiaUjC/qxFmKI",
  "hOAjMOukXU7+cbPN",
  "VPB9KnIWPdEWe+5O",
  "uxxwEHXnuyiA3i+u",
  "PWLvVQkKihpQqzDy",
  "UIjNfOvNd+h3v/sd",
  "A/xUCjobA2Nwlw+A",
  "92sqEf3IVMrDuHY7",
  "dj0wRxjb/Gox0Bwe",
  "1jEQUkhqELvGAwta",
  "nAsKG4QHIASLYcAg",
  "r7z8BicGQWux4mMg",
  "JAo8yZiGJA/AEg8E",
  "pRIV8nnO4Nz3mENp",
  "3HJLUYuTpMB1qMtk",
  "cuOBzVnkzP4pM2ji",
  "Hk1cVxC6wrRsnOjf",
  "iatdXIsM4BxxeXOW",
  "OlzzRgiekxS49KNh",
  "BnmoUymk5Nd0+5St",
  "tMvZaeZ5TcylyTwX",
  "Fgig0tRb1uxZM0hy",
  "2UmToStueFtv0IhQ",
  "Y3Dl0pIS3wrwOW3a",
  "tBrXnOgCiug2WyLR",
  "WDIQMptZlkUy18HQ",
  "4N7ZiWr4fMaMGaHr",
  "TisFLSgFCYazqXvc",
  "nkiwmgN0Lq262nq/",
  "FJBg8qdM5uwYzdmZ",
  "AFuoL2gJVVMAwWST",
  "S4iH0Yc1scgKPsX9",
  "HSWyIfmHw0dMqAh7",
  "EzRm2WI8owQgv+6z",
  "KNtcChv4ETNqjofF",
  "5DVpyfx2lHCkChFw",
  "xVvHCWbYcahcLfL1",
  "y0EGCSxw4FDrYuPo",
  "+AvOIz/bStmMxxNo",
  "yDh1FwqUYGWLKnWX",
  "uslJSAAPnmXjx4+n",
  "Cy64gCsC4Z6UStC5",
  "Hbj4TCEn+tzf4AY/",
  "3NQfX4yIoLf0ev+P",
  "KrYFxWKgOTylkE4Z",
  "iJ2huhBAAtwsmrEI",
  "szU365kGWzLJLgs5",
  "2Da7CkehniY/aAUE",
  "gZMEm3bppZexaw+D",
  "YSaVICcDuRA4wjB4",
  "uZRKZpgZwIM/5SUo",
  "vdiidNwFF1C5KUnJ",
  "NFE661FXtURByqWy",
  "U6F8gmhGsUoFD5V6",
  "qlT2i4a1TLLOHrJA",
  "WdOStffgCos0L/OU",
  "pBLc4b7RAqQq5RI+",
  "5YzGHgAemI+8G3C2",
  "KbQ1tQwl/g9rOJPL",
  "WnxYoM9ZQgIRtAOr",
  "DnX7DuUCl3K+w1qD",
  "ed5W9DSLPFDqAGni",
  "MFnk2mTnOlgHmpoO",
  "5Y0+KAY1MDFFhyif",
  "cCiVBtebJjeZZUDR",
  "mXCF0fS74NxnGaZk",
  "Jk2VaonjyCos8O7T",
  "IouP79O95mzjcrnG",
  "JYuj9KtlKhRQDcgI",
  "1XCqe9+eUrPTx4xt",
  "zqahNuI5EJ1afQYI",
  "CPUZ9CdTCH3w+P7J",
  "88HUr4dgga/XHYLi",
  "UsWmbVZOWNBEhVl+",
  "F4oDHuLHKzxZqSSS",
  "rCXJ7T1wWBMT/Uw0",
  "KX3KM7MITU2ANcfo",
  "XiZEM9PHd2AHfSo5",
  "DhUch/IuUSGBPoY+",
  "aOKoOX7aizQ19b31",
  "Gda1lyL6lb3wc8Bo",
  "1bJ+rWS2cx+DVi4U",
  "LRB7XaqSl/Co6iap",
  "UCVKeJjUelRKtFCH",
  "H1BnsZsBY6q1lc7/",
  "01WUWnwxngg6iSz5",
  "bpoy2RZKJ1PM9HKh",
  "iRL6i0fVsse6vP96",
  "7N90zz33spu9igcb",
  "jgEIvJ/GE9FAtYv7",
  "VBpyI5PYcz0Rvd/v",
  "A4ltgbT4CTw87dcG",
  "cPYrO10NyUI6sK+9",
  "9tphPJbGx2nZRxWF",
  "16Qi1eIcDlYv0YRj",
  "RMblVVf9XmL/8GDO",
  "pCmZdHlAxPEDgOqA",
  "l8t1cfzTqptvQt/6",
  "4THkOikqdHVRNeNQ",
  "rlChYiVBhUqZnHSC",
  "ugrdVIG2ZrqZustl",
  "KlfLYXUgTcZBZgtX",
  "/jDsIGeRG4FmPwHQ",
  "KDFirLFHnpSxZDaU",
  "PxWmkwcp+UyYFFNZ",
  "hCv3yKuynoj9ihJ6",
  "jGvct0voWYsymbYr",
  "UjPotfoJ/5aZbHDC",
  "AgS4IUaNTGCpp1yu",
  "+NTe0Un5ru6a8nZ6",
  "/XUSkslgnJmzcUln",
  "NwJ/mPjYi11L2c6w",
  "hTmNCMLH1i/DPehp",
  "YonPS+WCBUTdGr1e",
  "vV8cs4k2WlcuNxQN",
  "Rxg1XMimug22wfMm",
  "4SbZC8FariHbrx6F",
  "KEHHZjqFsTTMovk8",
  "ZCRDdQWb4Vf9zTks",
  "TrRo+Vl7wW8KW6ke",
  "jai6FpczwHVIJ6ma",
  "8agDEzgvQU42RUVU",
  "KfOEme0qFMlFXLjj",
  "0t4/PIbWXH8TKneX",
  "OJkJbmsRuC+HskIQ",
  "Zse1a25qDsMZTj75",
  "1DCeGc+6gVIVmctS",
  "xigNGetextarxUBz",
  "+FrOZKePGCjACXv1",
  "1Vf5gaKZ6vbAj5k2",
  "Hmj68EoYt6hmIs7T",
  "yicBBjQREY7i9xAm",
  "cA19+cVEyhfzzOVV",
  "fYBjuMwL1JRtoubm",
  "FslIh7g0NmjJ0t7H",
  "/5DW3m4bPk+IlJfg",
  "bnPT5DU1kZP1qJJ0",
  "qJxMUSdGrVSSfFfB",
  "mibdRO50FWDnspNa",
  "fznU+qvKgMeuNIBK",
  "fC510Ln6SJh8YMd1",
  "RXFliP+UGFB1/0WS",
  "Krb8irjRzSDLADNK",
  "8lG3oGbTyvFrffQI",
  "iLK8E7LljZSTyja1",
  "t3dSR4e46SJN1tr7",
  "PWJE765z1gu1NF3t",
  "13rt15qqUFCcGiaT",
  "nQXdvq7DG9DoMQiz",
  "izRUVfrGrtplb1Pv",
  "IcFk1S4ewbJIJh40",
  "nc0g80VKs4aFB7Sf",
  "6f92Ml6UaFObvGfi",
  "Nk17FsbSyA6ZzxRw",
  "zm7RZCCJAa2TNQp/",
  "2+w/1LzVsJaoTrqb",
  "zkiMcxBQOpWhbFMz",
  "uekUdYLFTGYo7ydo",
  "l4MOpF0PPohmFYvU",
  "2jqWqlVJrMKESoF7",
  "d3e37COdpc6uTg4z",
  "ufLK39Fbb70VVlwS",
  "ianygIRH2ZOD2GIb",
  "aIuB5vC3TgtwXjRQ",
  "O4U7HXqFxx9/PD+o",
  "bHFeDAwAl/gc7KBW",
  "C5nTMhQGvIFj0SpI",
  "Sc+jKVOm0k9/ej5l",
  "0hl+OANQlitFYWcw",
  "wJRlfRwiYJ+PgPum",
  "LB17yUU0doXlKBVI",
  "LFc3ElSQaY2ScI5D",
  "7fk8zcrluQSeMCi1",
  "A5udtOBryUmT6aoD",
  "IgAbM5Dscof4MxiW",
  "iI0JM1k5szViK0PW",
  "UiueaEUhi9kJ5VfC",
  "usuIWbMHZBVsN4Mn",
  "i1ZjkDQZuJrgYAbM",
  "qgu2U7NqTQyq41J7",
  "Zzd1tLVLLJ3lXnMS",
  "mjTgsxZmo6aAtfa+",
  "1mov2vqL+Cp2fQ++",
  "aZKWghZN0EJMoNbZ",
  "Rj/Hc2Lq1Kkh2xyV",
  "iO35HmF99Fm+pwyo",
  "tBpQlRzEXEOH08iC",
  "8RLKiZmJW12blgQ3",
  "KxHIJMRpH7D1NjVG",
  "82t6mL0sDHKtAgph",
  "5rn+lgGmGv+MY0ok",
  "UX1I5ORyXTnKdeZC",
  "FhLPJUzwOnJ52uib",
  "O9Epl/6C0iPGUHPr",
  "aK5o1pRMU1NTlq+p",
  "MsJa3S2Xy7FSxltv",
  "vUOXXHIJX2d9Pmui",
  "loSaxBbb8LX4CT5/",
  "Ac6ziWhRIrp7IHa4",
  "1FJL0RVXXBFKWmy/",
  "/fahELMmgNTH0s1u",
  "GWzDrL++Mg0/lP0E",
  "3X77n+nhhx+lpqxU",
  "sxjZOpKKpTw/iBkw",
  "O0lUV6bOrhwl/DJ5",
  "rkvJCUvQjy65mFJj",
  "FmFtSLiSoVdXKgdU",
  "LPmccb3oYosKWKxE",
  "yTihTIpmwoYDIeqa",
  "A9ShEhDAN7K7UQ9c",
  "WExIG3G1Eo65FMYw",
  "zEr1tdpQlMTDAy30",
  "AFlT03Ih8v+i9yQJ",
  "SrqoxiZ+VwZLds8b",
  "mZZI3qhW/08/g/RK",
  "nvUHofMpTCwG0e58",
  "gWbNmCmJTXUxvipV",
  "05C8kclOtmOFkdBg",
  "u1jxCjkjtRhgDp0p",
  "Q6nhM5wgFwS0xBLj",
  "uZ/p9+hfSNjqbfKg",
  "+1RGk9cxk1KePvkB",
  "ufAyJFNUrEjYiUy2",
  "EEccsfUhoFNhdehq",
  "MthDsQOj9oBJZFhQ",
  "QUJEQve6nQw0h0XC",
  "TaKFk+ysRWOdVS4J",
  "v81KFWYBi1pGJaFK",
  "lV3oTlOaw24wUSsX",
  "yuxqX3PLTekHp55A",
  "XYkKT5LLZnIvue+S",
  "YKVlICGMr89gfH7K",
  "Kafw88yu0qQu9JiF",
  "jG24W/wkn/9sOhF9",
  "h4iWNyK4A2aPPfZY",
  "OKNG9SHNPFVwN6dl",
  "sA3PUpAnSEQAMNb6",
  "ynCLAwSddtr/UVdX",
  "jsslwuAW1zjNZCpF",
  "nocAe5e6czkKikVy",
  "KUHLfWMzOvri88lt",
  "aaZSpcgDBmoMYzBJ",
  "eUnKZFLMupSqkS5l",
  "5HJWt7dhW0x1E8lM",
  "VXea6PExIDQMJpIP",
  "xC2ucWWRi9sGfzXA",
  "ULTlTa1y2ZfGp9ll",
  "72wWM5I5svQy1dVu",
  "smSFSRXRecSDcmY8",
  "x3EilED+RwWm9plt",
  "5iYgWUiyhO1YzUYq",
  "A9Ub2gykjDCY2vuC",
  "RmC9yHdsg289XfPm",
  "5oypRS9VuDQLHROE",
  "+nKVVWQDWaZuctxf",
  "YTRdEfwXSVXeBqA2",
  "1dLEk5yaylVh/4ok",
  "vDQUxP5fQkQiCTAJ",
  "/7AZfQNWuZ8k+rSo",
  "677WgxF5GyQUxWI6",
  "wUaWK+w2h24vJysF",
  "qP5VJc9xab0tt6Az",
  "rriUxq20PLlIFvJR",
  "WCJJadfhalxIgrMn",
  "0AD0KDUJ4Pnzn19E",
  "//rXv0OXOT7D801F",
  "9vVZGFtsw9XiFjr/",
  "2idGBHfAGE7btJ76",
  "2WefTaNGjZrnjKYy",
  "Yfm8sAB42AJkiosv",
  "oPfe/ZDOO/d8/r9Q",
  "LDEogtufWZVSiQeK",
  "puaRlMw2U767izLI",
  "avUD2nTvvWj3Hx3N",
  "NdORDFTiusIuu6wm",
  "TZrEcj9FZimj2uFa",
  "DlJfVedPXNGSPR5q",
  "ABpNTTCZLDMknKdh",
  "IA07YwFXLDVi0bqE",
  "9ZVrS1NG8WSGjQzL",
  "Y1rr1LghTe1lEwun",
  "uqFhhROEDnBMp8O1",
  "4svVgNra2lgY304I",
  "UrdgPQs5J4M8lSob",
  "4BqjCpAy5hqbhmpB",
  "mmSir7ENjdnXHAvY",
  "zJVWWiF8j/sEtzna",
  "g2099X99LgAwsUSS",
  "uce4nQlGnAFlmrLU",
  "Omp0WBdc+piZoGnY",
  "h5VwE0kP1YavhC71",
  "msmbaF5iYbYxmPNi",
  "A1au8lX3PowN1b7C",
  "kzKVPhK1By+TpaaW",
  "1lBVoVASJYvFV1uJ",
  "jjz/PFps1VXJS6Z4",
  "EsvlO9Mp9qQAcON5",
  "hrACXC/0A3299977",
  "6MILLzagXfocvsOz",
  "Db+ByTC8PQNlOkGI",
  "LbaBtLhFLTgM50gj",
  "NzGg9rOf/YymT5/O",
  "g8a5557LbtJ5wWiq",
  "q0ifgXjY2kyr66Tp",
  "6quvpcce+xfHNOFh",
  "CYDM27gQPiZy01lK",
  "Z1q45GSx0E1Nrkvt",
  "xQJ968Rjad31N2S5",
  "H8iqQ0haGJwkBwki",
  "qUcytVGjQ1lKyUSV",
  "IRPZ59Dcg9taGEvf",
  "QcZ5pTaWk4FplWWC",
  "JMsWlYPAUJoyeSY7",
  "1s5Aj7JqzYDI5fZq",
  "XYphFRXDdPIga2JF",
  "dSDkrPlQQ5OshCZT",
  "OcivsnZmpSqxYcg8",
  "L1Xw6ocMFsCmLaKu",
  "MZWqXtCbaXKI7gMx",
  "wgAh6lrFvsBowuwM",
  "6JjZHHxTbVPJKJc+",
  "g1KTiy++OP+v3gFM",
  "vtraOkJwqXqakBtL",
  "4M+6X7qNaqGC2bND",
  "X8DKNaF8KbLOLb1M",
  "0Xs1yUlGk5a9B6qj",
  "GSozSF/jV/QLrZlu",
  "9wMtWGBllfe41GWZ",
  "8/E40aL7lJKV0n9U",
  "w5OZf8+lXKJKnaUC",
  "9xsk8eSqJVpkpWXo",
  "Z9f+lsautTqrW3gO",
  "qvuIi4Z1flEVFyEA",
  "CFvhGFnpT7gf7777",
  "Hp144omUSkmGPwzM",
  "cKQU4lAuV+B68wNh",
  "cT+LbbAsBpoLluj7",
  "CQZwDliWej3obG9v",
  "54fefffdxy5Tu2KI",
  "XTVE/x+oh1cUyyfv",
  "UfpOExi4FB6YsapP",
  "xx7zI/riiy+Z+WQA",
  "4wHQgZ8g8ksFyiTT",
  "1DxyLFW8LFUTHqXc",
  "DHluMx1x1e9oiRVW",
  "oXLZp5wjCsQYoCjt",
  "kJdJU9H1KB9AY8+j",
  "XDWgXBk6lAkq+MjU",
  "dllTs8QVPxBx5Yum",
  "nu8xCGRJlCoGQmhj",
  "whXvsk4g4rwqVcSA",
  "Im6zKrqWEEq3K5L4",
  "IoMUsjOOsCngdeHy",
  "hsYfdP0KBN3AgPX+",
  "pNReVE2Fs9e5Aook",
  "WaDqUJG1BqHFie2I",
  "IFFaTDrUBVeel6ag",
  "5FPGSzFY7uiYRX5Q",
  "5sX1Eqyl6CUdcj2A",
  "7AqNGJllRmaO9a9x",
  "LmWZLGjb+OijT+jj",
  "T77gawBA0Z3P0Vpr",
  "rcXhEcim5ftcQjxf",
  "721odoC0Uca9kTjk",
  "wVwG2+bUDyW5K6By",
  "qUDVSpGF9Qu5Mi23",
  "zHJce9tLpCnwHUqn",
  "svTaq2/xsJFMpjkc",
  "RZOCEFscmAkNORCQ",
  "9MnxEKpSoY6uArkJ",
  "yBhVyOe5G7L68szp",
  "L77KClRCpayyTL6Y",
  "LUfSmZemspOhIrwB",
  "CSg2QC8ScdBYACxF",
  "U1Niio2GpWrPItSl",
  "ithNZLjjNUF536Gi",
  "WUrQ46xG7/GdVOuB",
  "u9tkrZvJmwJJ9MVC",
  "GSECYPod8pNZKjno",
  "H9CqhbciTd1dZZrZ",
  "nadiOk2dAdHaW25D",
  "5/zhWkosuyoFHUUW",
  "YUfoTqGUlyy3ckCO",
  "71JnocLx4ZgEQiu4",
  "VCrTjOlt9P3vHUgz",
  "p7dTuSC+D556ogav",
  "aa94BsL0tT+myV+6",
  "79hiG0iLgeaCCTg1",
  "S303IvpsMH5kjz32",
  "oI6ODq5NjQz21Vdf",
  "PYwhskvZ2RnEg2kK",
  "XiZOnEwnnHACjWgd",
  "wWyLxzqaAn7TKbjb",
  "BZjCXQWmDmwLQNKY",
  "CUvSOTf8kcYtuyx5",
  "uTJ1VUrUjeMvyLmA",
  "2eNEHWyfcLkeOOtN",
  "+mA7o6o+miDDFU3w",
  "Hi5zU5WnfomGD+Aw",
  "sKOiyak6l6K1KS56",
  "/T6sHMTbyG+JNqZm",
  "zBpG1LFqpIe/JcfF",
  "23DiktkPMsk96H9G",
  "CTh4DwM709UZVZCz",
  "E3e+nhw0Z1OmBoZJ",
  "ANrKiy++GE5OsF/U",
  "P19ttdXIN5WHtOZ2",
  "bINrWvkHWCOZRNId",
  "0ZZbbhkyzhoi8dpr",
  "r/H6Osn7mpsV7cLg",
  "HqyD/gVXO5eZrFSo",
  "mC9wCVgvleLkoDXW",
  "WpN8D31E+hDYzXyp",
  "TB1d3RK2Uqnyew1b",
  "UQUGlexiNp9Lq6ru",
  "pul/hrWXalwmgUYP",
  "0fQ/+zMGi5xMJK+6",
  "KFOKWVWqpYVZSy64",
  "gIlhpcyxzH7Kpe5y",
  "kSiTJCeTYdC44z57",
  "0RV/vpmWXXN1anaS",
  "NGJkCzPDHJeazITM",
  "cWdXh0zgXAjhp6hc",
  "Frf7UUcdRR988AGL",
  "2w+FK9suQdogORCj",
  "0dgathhoLrgGdPAg",
  "ES1LREsPRhynZq7v",
  "vPPO9NBDDzHovOaa",
  "a2jNNdesySYeCpeM",
  "6oDitx55+DH645/+",
  "xGLtSGZB1rm6nlAX",
  "3Y4L1HgqcokWXWU1",
  "uuDGm6l1iSUoCwaj",
  "UqGpkHcqVNid3t6N",
  "WE8iL5MyyQioB46q",
  "JhLDGcZsmgqNKo6u",
  "MZA1EioM9KLFLjcp",
  "MWv2Z1HSjyQbSbm+",
  "0I0YSHIRuwtNLeaa",
  "farcCx+TcTmCZTVZ",
  "uuKKdFnKqQxwUETZ",
  "QfaHMljHeg4y950k",
  "yxohIUgXvEcCVm+M",
  "mb7a7QFA5PHHHw9j",
  "YjXJ4Tvf+Q5rCmIg",
  "rtd2jG3wTPoEQAdA",
  "ZYa23nprvi+aeDJz",
  "5kz63//+16Ow++zu",
  "P+4xdDeBS5LpNLuU",
  "OSSF463ztMY6a1M5",
  "iUpXPuVKZe5XEHBv",
  "GtFKrSNGUq5YpFQ2",
  "Iwx/WDZSAKfIhRnZ",
  "sDAJyMR2mjhPmVTV",
  "qjbYS/g5x0o7tX3S",
  "JOJh6cZ36RRLnyVQ",
  "YMBzKTOihZzmDOXB",
  "7KNaEioWJRw69pyf",
  "0KmXXkIVCNJDmD6J",
  "9g0t2ogtxDXEdWlu",
  "ztKIES0y2QoS3AcO",
  "O+wwevzxJxjYl1FE",
  "YgiAprjipbhFg7qc",
  "MdCMrWGLgebCYV+Y",
  "OM61iOjGgd45EjgA",
  "OPGgOuKII5j10IFo",
  "qOQ3kBmrJfMQNH/6",
  "6afTCy+8yOyB1AN2",
  "KJfPsa6mJjdABHns",
  "WAgmV8mplCjrJmn8",
  "+uvRubfdTIsvuzw1",
  "eS6VXJ+qKAWXTFK2",
  "Fe52ommz2hlcguHA",
  "qyTomIQb1afk91GJ",
  "yq8lG3At82ipyTwP",
  "k3kigeqStR+VHrKz",
  "cEPgCgamJltXpJRY",
  "k1NjPbkWkZXJCxc8",
  "smZRCo8HTOFmEdgK",
  "0Ivv+G0PwEIHJh0M",
  "Z3evNQO5HqCgraCM",
  "qLYTgI/9998/zGSv",
  "Xz+2wTGNEQSmxy08",
  "4IADaNFFFw1jMPH9",
  "m2++SRMnTgzvsT2R",
  "lPtkhE9NJSj1agCg",
  "irlcEraYyzPDiTT0",
  "xSYsQYssvRSHdZRM",
  "e0ymMtTV1c3bIf5Q",
  "YzVrFBkSJvTETOoE",
  "cGoynEncY/1aSbiz",
  "k4lqk3giEXaEo8xu",
  "yVcrNKO9nRyAx6RH",
  "M7s6mdUsgHHtylOx",
  "WKZFlppAv7rhBvr+",
  "KSdROZWhtJeloFim",
  "SrFECUdKqiIRChNi",
  "PKNEg1hy2D1XEoRO",
  "Oukkuu/eByibTVN3",
  "d571gdWlPVQJlw0C",
  "26E5qNgWCIuB5sJl",
  "bxLRoUSEUi67DnT5",
  "MCQO2KYu1qEwnZGr",
  "m7WzI09HHH4UffjR",
  "R+Qgjsr3mSGQwTQq",
  "nYfEIBZ0r1TJSVSp",
  "u6udltpgPTrj5uuo",
  "adw4anVRMq5KXbkC",
  "1yz2kTGdTlETkqKS",
  "SWY4wAZKXKTRAFTQ",
  "qPp6HCMpzAuzL2Yp",
  "WgtLDumikkc9VAuq",
  "kVSxRaXxndETZNCJ",
  "AZZjPI3ou4nVDNlO",
  "k+XO2bOo7Yz75arr",
  "UNySMJShRJKQaiBq",
  "xrm+6iBYPzj1BA5t",
  "GSO9/shi/s9//hN+",
  "B0YHE5cdd9wxzoAd",
  "QgOLiSWT8ailpYmO",
  "OeYY7hfoMxoC88QT",
  "T4TVw+z70lMIhciR",
  "oe1UGJxyUAeDUemL",
  "btLhez1i5Ej6zoHf",
  "436UamqmPEAY68dC",
  "aqxCTc0t1JnLh14A",
  "XbSda6Y6QCpPzhD+",
  "qXGWpm9wvKXdT2qE",
  "3VWmjL62hOVdcdzl",
  "gJqbW6kt10UdiC/N",
  "pCifK1B3V44B77bf",
  "3J3+dM9dtPLmG1N3",
  "oUwpyKx5LgUuAgWk",
  "XetkCy5y/A+mGK9I",
  "bEScJcpLXvOnG1j2",
  "CJ8BbBbgXdBYhEG0",
  "+kIJDVjMaMbWsMVP",
  "8YXTuojoISLayCQP",
  "fZOI3h+MHxoqRkrZ",
  "TPv33nvvA/rBQYdR",
  "Z0c3pZKpMDYznZYH",
  "vAeZEZMFXUmkqbvY",
  "RaNaU5TwA1py3Q3o",
  "p7fdRmNXXpH8Qony",
  "ZROP6biUaspSd7FI",
  "k6ZNtcrYSdk7gDZ1",
  "5WnCTsh2WqUk63X7",
  "1MVdy2raoNLIKGmG",
  "uepl6u8aBlWZUGZs",
  "zL5CV7zRKpT9RnWh",
  "sX7gSIUkxMVB44+T",
  "Dcw6LSOiMoR6rW2g",
  "oe/ndJ/rGW5dAFzu",
  "vffe8HOp4uTTkUce",
  "yYPvUMX4LuyGWyeV",
  "tyq03Xbb0XLLLceT",
  "MvEQCBP9t7/9LVwf",
  "91tlrULlB0uMX2GI",
  "53qsWoHJHuhSmdyl",
  "KQDAq0KGLKD9v/99",
  "ah07lrpKJdagRHIP",
  "9Cix5EtFSqXTIaCU",
  "KkJomxG7KYoN5n/D",
  "6GtlLfEEoMysNfEz",
  "fYoBaNhvg68tHNts",
  "FpwvktXgxmew6Igu",
  "b3bRRemsKy6nC67/",
  "PY2asATLq3FVn2qF",
  "CqUiFcoVjkFFMhV0",
  "frkMZT5vEt4Q5+pz",
  "YhUSLa+79noGl7ju",
  "2g+QzT8099/Uq2+c",
  "0YzjWWJr2GKgGRuS",
  "h/5JRKsQ0WJGImnA",
  "3CKLLYZdDr4BQAoL",
  "kK0pafjcc8/RwQcf",
  "SpOnTK6prIHseSQG",
  "aWY8ahR7mRbqzBco",
  "hezZfIHGr7s2/ey2",
  "W2jZtdblAbdYrvJS",
  "KJUpXwQPCRYxoGK5",
  "JLGRkCuxdP50wLKZ",
  "E9bJnF35uzAm0xKC",
  "t+IwwwHQqgmt9cvZ",
  "pR66A7VWuonbDEtW",
  "RjGZIg0jMjKq+cnJ",
  "RQAH6ZQkBwUJ8pJJ",
  "mrD00kZHE/I0Xrjg",
  "PT7H/73FaNospoIR",
  "ZTGffvppmjJlCoNO",
  "LLiXKBiw3nrrDFmM",
  "78Juyh5DMufYY4+t",
  "SfoCsHrqqafo9dff",
  "DIs44FVlrWqSSKxJ",
  "gU4spk2bRkGpLBXH",
  "SiJijgUqEdVCicaN",
  "W4wOP+6HVHaRVFOm",
  "LlTUQXlFSlCuWKIc",
  "2oXxAkhcpmrIGu8A",
  "2FiNxzTFE2rkvlgy",
  "bM6LVuWqScazFq8J",
  "k1OirJOkpkyWj2PH",
  "b+9NN/79Xtr5+/tR",
  "DmylH9Do0WOkfTsB",
  "62WOaG5loAsZIgy3",
  "eEYhZAftHAYX+rHH",
  "/oiuuPw3lG1qYrUM",
  "JC8hlhVZ/VB4GArj",
  "azcb78RsLAaasTVs",
  "MdCMzbapRiIJgVnf",
  "GAhdTmQQD4Xpgxts",
  "QWQARx79858P0XHH",
  "HcfriKuqYATfRQuS",
  "GTmWdyEKSknKd3eT",
  "41Yp6Tk0csKydNHd",
  "d9M2228XzvrZ5eUl",
  "ObmBRanBWpgqQJrZ",
  "zQkGnFygIM7obbL+",
  "Zg+LDnCcXS7l/0Lt",
  "Tta+NMlVodC7JjIY",
  "8XWjBaiVUAAesR8V",
  "Z+c65SapyM7M1UGV",
  "NQh9yaSF6aCDOuar",
  "ry7JXT2ZMpP1YLCn",
  "9zqY2S5WXHu4z++6",
  "6y6+JwAmmBBgwvDj",
  "H/84ZjOHyMDE4Z5s",
  "s802tMkmm3AS0Ogx",
  "Y/gV2rm33nors3B2",
  "yARUG2AaUhFqPTpM",
  "9rGhz8F13tk+iwEq",
  "l4QF85lAdrtLKYSf",
  "VCr0wxOPp9XWXpMS",
  "phoXaqCDXUc8JLdn",
  "4znwlWkP45BN/0K1",
  "Kp5IRZqxkjmu7dzK",
  "JA8nalHZSl0vWt+o",
  "QaDdYiIKxrdcpWqx",
  "RGtvsiH97q7b6Mwr",
  "L6PW8YuRX6xS1vco",
  "Be3MfA6HzlWQGLz6",
  "VcoEHsubeW6Knx2I",
  "0xSwTnTiCSfRzTfd",
  "xuU481zXXEA86qMD",
  "lMpEa2iBJpQfGrCJ",
  "g31MsS04FgPN2GZn",
  "/zWgE0PG1qbOep/N",
  "zkAfTJ1Bu1pNlKUc",
  "Bdvf+7cH6ZSTz6Su",
  "7jyzdBXUPE+5BOGU",
  "qgMXlUfpZJLSzSkK",
  "3BRVqg5rSXqOTy1j",
  "W+jkW2+jb/3oOK5m",
  "ks93M6NRcVFqjiiZ",
  "TVHFS1HOJ+oqB9RV",
  "9qmjVKWuSkC5wJEF",
  "zCczMQnW3yz6xAvX",
  "PU+4rP2Hxdbyk7rP",
  "Jo4z4bDuZTEBLU/o",
  "fCYoZ16xlKAhaBbR",
  "D4zeFwOHunBsVaJC",
  "IsXahKUgSZXAo1yZ",
  "WK/QHdFEFQdgw6Gg",
  "7FClIAPPhKWWoDXX",
  "Xo0SDkY7+QwJVZBk",
  "gZYmBkGAeyT02Nnh",
  "dnyu/q9gxM4k13t2",
  "3bU3UUd7jjo7u9mV",
  "iOSKfff9Lq2//vph",
  "/e16tkWzZGG9tZ/e",
  "SqjO+xKrQ6uzqQlc",
  "GrKADH+wa6eeeipn",
  "/Le2tlCVpcoc+uij",
  "j+nWW28PK9MoDJO+",
  "FRluKVy9kHrUr5Lp",
  "FH366WT6fPI01moF",
  "yAr8CqWcZirkoX+J",
  "eE+Pk15u+fOfacKK",
  "K1B34JODsrCJBHWg",
  "hCXKV7pJSjgea2iS",
  "A/c6tDZTVEkkRSPT",
  "SfBSSXjc5lHqtYoK",
  "XQF0ayWhSAEoFCSw",
  "oJoP+nHVhTs/JYUa",
  "WFGiSvlyUZhXdmOn",
  "KeFlae2ttqaf33IT",
  "/faev9EqG29GpYpL",
  "rU2jKZMdQW1dnUZj",
  "mCiXL/LxlosVZigB",
  "altGtHIoANznmNB+",
  "9ulE2n67negvt9/F",
  "8ZwoX8las6Y/qHxU",
  "CUlDQzTX0mzz/fbb",
  "r5HVnx7s44ltwbEY",
  "aMbWm2FUeYqILsK4",
  "QURH98VtAjZkOCR0",
  "ANjcfPPNdPLJp4RV",
  "hdQNCDaES8K5LrNq",
  "iMNSEBPGIyYqdOSF",
  "59Dp119DydGLkpsr",
  "UzHfTlUPzKUMUqxD",
  "yJVIiJKtTSx/4qWQ",
  "aFThwQuDahEVd7i2",
  "s0izRJqAhkEx1X50",
  "QR4AFmEukagTMZHo",
  "vqFMi0mMCNlQZklF",
  "V5OZTAA8iL37qE0k",
  "Ops4DoJ8EZKAGPgl",
  "qKV5BIM8sEZwmY5b",
  "fDHKZpsZdOj1Clkp",
  "HLcvlYP6ax9//DH9",
  "85//5H3j2oP5wXLB",
  "BReE2qz2YGhnug+H",
  "9jXczQbbME3osjVK",
  "d9ppOxbMB6OMewBW",
  "Df3hmmuua/g36t+j",
  "DWEiAtc7/y6LXjpc",
  "FhastXogOjo7aPyS",
  "S9ADDz5I2++0I3V2",
  "d0mJRgBgxFi6SPQJ",
  "qILbn3QZtHEpU8gH",
  "IdnPJL5xIpHJREdY",
  "C/oQ6NVSMkV+Jktu",
  "6wgWg0eZWDeZpWQy",
  "S0HVoW6/SJWkR0Vo",
  "zbtZSmdHUqESkJvN",
  "0urrrUkX33o9/faO",
  "22iz7XdgwJ2Crqbj",
  "sf+hq7vdSBcBJPos",
  "V4SCA/isWMxzOAKu",
  "JY63qamZnnzySdpq",
  "q61YcQH3AazxcGoj",
  "559/fiOrPzP4RxTb",
  "gmLxEzq2vhie4X8i",
  "onWI6NhGAOfSS0PC",
  "c96b1kS/7rob6bTT",
  "zuBKJKVihZlMJCxg",
  "cFW3LtbVShkYKNl9",
  "1ZTkwP51tt6WfvPg",
  "/bTBd/akTKaJmp00",
  "M5Dd+SLHd6I6kJeC",
  "m1jEqjFYVdwMJTDQ",
  "AdSBQQJIMwuYS1Ql",
  "mVP5Oy6Bp7GYdbWe",
  "OS7TigfV2uv295wg",
  "hBhSgEuU2TQxm4hv",
  "CyCWjd8zrr7uQp7y",
  "5RK5XooThA479Ah+",
  "TNgZ5wAnusBmzJgx",
  "IPfnyiuv5OuP5BGA",
  "duwfiSkHH3wwv9fE",
  "FAzYdjxZ7F7v3RSM",
  "91SXvrW1lT//yU9+",
  "wkBfCy/A7f3ll1/S",
  "ddc1BjTrM9BhnBhT",
  "LnPNbg2fwAQCn+H9",
  "6NGj+X/IWWFdhGrc",
  "fPttdMc9d9MKq67M",
  "k7WqB7a1HBYYQKJN",
  "EeVRwUYmSHRsjdg7",
  "/NYJnCsKDiRlwaQq",
  "cBzq6s5RW2cndebz",
  "lKtWqKtcopmFHBWh",
  "15poou6uEnW0d1G5",
  "WqLRi42lvQ49iC77",
  "8y102d/+SiuuvwFV",
  "U1kK3CTHTybAqBIm",
  "jBWilMPHj9hLVb/A",
  "uaDNYqKt54qa5L/9",
  "7W9p77335smZSkfp",
  "67y2PqqExEAztoYt",
  "Bpqxza1M0tVGn7NX",
  "Gw5AIJTW8Ylrop96",
  "6uksHI1BoKu7K5Tz",
  "wQDLsZssGC7sGcdK",
  "FQPKJj0QgDR+5eXp",
  "7Kv/QIdfdDF548aR",
  "A4AKpOYkKJFKsrZe",
  "MV/i8on5QoGK7KCX",
  "rFgIT3P5SSujXMtN",
  "2tmw9nuVTMLnkiQk",
  "WexaJk/qmkfSRxy3",
  "Zml3ghGSWDOJ7+Ts",
  "XLCRzAJJAgQDa7j1",
  "y1XKNLfwgP3d/fZn",
  "dgkC1RrDheslcixS",
  "YQkABok8/TXs++23",
  "36a///3vDHzwGwA6",
  "uG+XXHIJSx5p8okm",
  "oqjUTpws1LuxW9uw",
  "8xqKgP8BkLq7O+no",
  "o4+kddddl/L5HI0a",
  "NYqvP0DS5Zf/pi7u",
  "efZW7+bX5C8skLB6",
  "6aWX+Z5hHdQ41zhE",
  "Fuw39dAhhI5l2x13",
  "oCeefYZuuPVm2v3b",
  "e9Oo0WM5frkzVxQF",
  "CPSHqs99DbHQndUK",
  "u9xzFFCHX6FOv0Lo",
  "1Z1BldqrZeoo5JgV",
  "rbgoG1uReGasny9z",
  "H6BsisYuuzRtvuce",
  "dNaVV9INDz5I511x",
  "BW2y7TZUrlSoOZml",
  "Yq4ok52qT12FPE/G",
  "0OfB+qI0K4AnMukL",
  "RYm1xPlhAgtgDZCJ",
  "QgRnnnkmZdJNVCgg",
  "8a3M3w0X66Oc2KCo",
  "lMS2YFpiOICA2OZL",
  "W7q38paTJ0/makFI",
  "Lhgulsmk+CEPcbvv",
  "fGdv+t3vfssDqj5k",
  "wUQ6htGBQdOuWipT",
  "qQrXcYWaUynySwFV",
  "3SQDulmffkTXXngR",
  "/eefD/Dg57lpdq15",
  "CYdSnkvFUhdnjkot",
  "ZwyVRJ4jbl9OLgJT",
  "iYHOqudd3yM1WUj7",
  "qjA7EZMn5fdqZYDY",
  "la7b8auWtwQjZOSL",
  "jBh8MpPlARHsJpjX",
  "zq4crbPOunT33+6h",
  "UaNHs2sdSEX0R8H0",
  "IthOGBvPS9MvLv4F",
  "nXX2ef26L7geGLBX",
  "WWUVevbZZznmc/z4",
  "8QxyMGgjMx3amvhN",
  "2+Vrx2j2tv/+2Lx+",
  "TvYXTHOZUQP60I5V",
  "YB2nteGG69N9997D",
  "4B2xmTCAoI8++og2",
  "33xLbruIV5zTdai/",
  "D7Yuo1bH2WKLLeiR",
  "Rx/mkqaoPATwhd9E",
  "HGPCSzDo9JIugy+0",
  "R9G+TfPr9ClT6eUX",
  "XqTHHnmU3njtNfrq",
  "iy/JL5apZCY9rPVq",
  "fkuUK+sYOheBo2AP",
  "xVuBPj9mzBhaapml",
  "acONNqItd9iBlllh",
  "RY7B9pIpqTRWLHDm",
  "uAs1iQrc8D4DzaQp",
  "n4l+gL4M932hmg81",
  "MrPpZmrvaDdZ9ymu",
  "pvSjHx5Pb7zxFidU",
  "QULKTtBH2xwO1a80",
  "HEUTK+dgXJRzaI4q",
  "tgXBYqAZ26ABzUmT",
  "JtEXX3wxLIAmkh3g",
  "OsfAxiUn+QFPtOlm",
  "G9Mdd/yFQY2IuVck",
  "1tIwdqkU1oe8SoVK",
  "HV2UCBzKtI7g2K+s",
  "61K+s4MB0jP3P0DX",
  "/PoK+vDV16kVWnsY",
  "zN0ED1SohlL15TdR",
  "uFFiywT4hLIsHEz2",
  "9VgErcks8V8CFmGI",
  "ywRa1veyXVSdh8Gl",
  "AZu6jwqAopekEocD",
  "SBwnyk4iJhUMZq5U",
  "5JjNRRcbT3/5yx20",
  "2uqrU65YkN1C25AH",
  "Ipez9v1AtP6gC3js",
  "sT+k666/eYDuk0u/",
  "/OUvudYz/tdSfWDe",
  "fvrTn9IvfvGLMJkI",
  "rkqwRrhfvQ2OCzvQ",
  "hFC6hhugQo3IegVc",
  "GevOO++k1VdbOdTG",
  "RFILJIe+//3v0913",
  "3yNtswFAb0tX1euf",
  "cntP+HTNNX+kgw46",
  "SD5zXaPRidCPKicE",
  "zWybwccBICihGiKL",
  "xAk5EF8vFDmmceb0",
  "GRxi8e7bb9P7779P",
  "Uz6fyO1E2VdlVwFa",
  "wZADVI5dZBFafrnl",
  "aMmlJtASS02gcYst",
  "Rl42LTHHXor8apkK",
  "uRylPNOPEi4lMxnq",
  "yhfIcwJKgsHkOFMJ",
  "G9F4ZQDkTEay5wEs",
  "8R6f4/TBxl922WWc",
  "da66mQCb2BYAW2PF",
  "53X7guG6oxoUCIIG",
  "1EmGRrcutgXCYqAZ",
  "26ADzU033XRYPEjB",
  "TuqAK7F+jNVolVVW",
  "optuupGlmMCgcEyV",
  "GawEoFbJA6DzkjSz",
  "vYMHxjGjRlK1UiK/",
  "DJYjxSXkpn36Kf3z",
  "L3+hh+6+g6ZM/IIT",
  "H8Da4PtKpSgMB5hM",
  "cyl0gAEQtIHEnLgN",
  "AZYixwLT7e0oGABM",
  "GNcz188kJZiZUGZE",
  "4ZZHkhJiUlNgspo5",
  "xnSZlVaiP/zxT7TG",
  "WmsxeEPWcK6Qp2RC",
  "QB9+O5frZtDAsWhV",
  "ol133Y2efua5ft8f",
  "jYuF9uq///1vrjQF",
  "4IPfURf5TjvtxNqo",
  "GuLQKBu0sANN34fb",
  "XP6HrBDAEO75aaed",
  "RieedDxLesFlDqCG",
  "33r++Rdo112/yYk7",
  "vL2Z0DQKNL/2HX4X",
  "upIjW+iZZ55iCR3c",
  "b3XjQ1YL7ysVYQVx",
  "fJjAwa0P0JZ0Uxyb",
  "qQAPDCs+136Q9lJh",
  "nLVm0uu6LGVGCQ5v",
  "AdvpV6shY4p1maVM",
  "iwRYuVhmUCvJbpA4",
  "gzeiyIUaVHNXE3uQ",
  "6FcwSU2QPeLzKJX5",
  "9+756710ySWX0Qcf",
  "fBDGFOs5hXHfiQRX",
  "YcI1BwCd14bzg0rI",
  "q6++2tuqbxDR2kNz",
  "VLEtCBbHaMY26DZc",
  "soIxCMDg2haAggEw",
  "Qe+++wHtsssu9Mgj",
  "j7A8Cw8+pjSlskBu",
  "qhmJ57RIy0hqTacp",
  "190dDg7wLCOCcuTS",
  "i9GBp59Mf/j73+nQ",
  "E0+jRceNpzSkUygf",
  "JuFowo8k6sj/PmRZ",
  "LPFozUCv/SyoAZni",
  "DhedTXabaxKRAZmm",
  "Wnmos8kC2QRR6aok",
  "TbAofMBMJvaDgXeX",
  "3Xal++67j5ZfaXlq",
  "a2vjiiwYBHXQxgCp",
  "g6Qm4YBFAqM0EKZg",
  "EgLfYC81K9n+/Vtu",
  "uYXZLhjuD7PDJpY2",
  "ttmby+L6iZCBw7Vd",
  "ffXVudQkGEwAIGUB",
  "cU01MYgnEw2WQOwJ",
  "DNsxobi/aC+77767",
  "iLhb7Cd+B8eh7lt+",
  "NaAQbmq0X+2LzKQb",
  "TwG0LaGbhHrjQSUg",
  "B/JepSpPAF14d6uo",
  "rV6kQncXlctFKleK",
  "VA3APDrkeCj5gz6E",
  "WE2JHU0im52q5GM9",
  "VPcpl6i7VCEH0mC5",
  "AhUKRTk+jtf2KIOk",
  "o0qZq4wlkyl6/70P",
  "adddvkkHHXQwvf/+",
  "hzyxhVi7tm2NUdVn",
  "IuS8hgPIhKFfQ0e1",
  "AYuljWLrk8WMZmyD",
  "xmhCqBkPWMTdNZpQ",
  "MK8MY2Qmm2SG57TT",
  "TjWDo88sTMWXpJSU",
  "B1alTG5CskptWZIR",
  "Ta1ULhUZUKYR/F+p",
  "0LTPv6Jn//UEPfC3",
  "++jt/z7BAycGdAz0",
  "MC8l/4MZcUzMIZJ2",
  "OEPVVCqJJGlqmTtb",
  "H5IZk6TEsqlbGS5H",
  "2VaSeAJCOb8SOWBq",
  "nARnsifcJNdibmpq",
  "orMuuphdpYgQBRjA",
  "+QPYQapFsmbBNiGc",
  "QJgkSCChlOftt/2F",
  "jjrqmBqZnLkxBa6a",
  "9Y/X22+/nScA2Hfr",
  "iCxn6sIV+q/H/s2Z",
  "u9BfRLtSlkhCHYQN",
  "q993b6xnb5Oh3p6T",
  "vTGOg/2c1WICeq4A",
  "QXpPOBSkLOxeuYL7",
  "BnmqBMtJrbHG6ryu",
  "i5CSkrCBN9xwE51w",
  "wgmcXc1FDhIeVf3a",
  "a9pX00xsPb4VVliB",
  "brrpJq4AxVqVKbl+",
  "uHe2koDEaaLeOjQp",
  "82FCk1aQwn6ZhSWP",
  "32PCpOBWQmGkDyGT",
  "XNzZ0h/sCYruz64q",
  "pgoL2B8fvysxxPL7",
  "Iokmx+rxdm+++SZL",
  "caFMp7j6pYRkvRj6",
  "cLcG2ykeFH8e/KOJ",
  "bUGxGGjG1h/7lIiW",
  "mdMKkEdBNutASOAM",
  "trGb3A9oxx23pSuu",
  "uJxWXXVVKhRykpxQ",
  "FFDJ8VxIiLFYCAw4",
  "6EfMCplBCsNV2kOW",
  "dIJybe306D/vp/89",
  "9wL97z/P0pcffMw8",
  "YyqTpgJc6imPmhIS",
  "Z6iskwLGUPdQAy3r",
  "Ei20uhEycZUNUqFp",
  "gEq4uIsMFMFMBeSm",
  "4EYMqFAt0+hFF6F9",
  "v7sf/fC44yg7cpQw",
  "RakkA2gAN6nUIwMs",
  "WGB8zokdqZQZnMu0",
  "xeZb0bvvvh/GvfbX",
  "ba5ABO+XX355+te/",
  "/sXxmdkmyTiHKxfr",
  "XHPNNXTccceHgEqv",
  "h0rFqAtV5ZAWBqAZ",
  "JbRFCgq4V5KEBkWB",
  "ErtqIRr+k5+cSWec",
  "cXp4rR1cx7LPyXub",
  "bbY5dXR0MIjCvUcs",
  "ofDq/TNNgMO9QXwt",
  "7g9ibo8++mgGsgoM",
  "0d9UXgnvW1taqVCU",
  "pDC7CpUqE0hlL+kz",
  "dt/Q/sCA0JPftY9F",
  "gaoNSDXRzL6f+D+b",
  "SfP/HNuMdco+x36C",
  "tURMMSon4Vj0GBXk",
  "K6Dtb/8YbLPZ7gZJ",
  "hoYUR2KLDebCRRVb",
  "bHNpG/YWqwMWCmLN",
  "n302R/JznpuAEbi0",
  "iL784iu68647WdNv",
  "4403oXK5wswmQCML",
  "qqNGM4CcA4AEV7tL",
  "XjLDyQKQSpFsWoi4",
  "e1LWznNoqVVXpp33",
  "2ot22HN3al10LM3s",
  "6qKOrk4egDw41ysl",
  "1gYEwwhAyCwiBmYz",
  "oCbA0Pg+x7KVMAjD",
  "lVzFUuUlwSC5yklI",
  "XKIPgyL0AQG8kh5X",
  "OoIMTKqpiVZZc006",
  "/Jij6bwLL6Tdv7UP",
  "NbGovrCItltPMsGF",
  "2QSoxMCsWd/ZTBP9",
  "5OxzubznQA2idpYy",
  "DK5ULGAvAfibsi0h",
  "sNh4442pu7uLnn/+",
  "ORbBtsuJKmjAZ2Br",
  "VYqpt9+ekw33Cbme",
  "uya76XXANUDsZbkM",
  "EXaJO9x++23poot+",
  "HmVJN6Wpra2LwR+S",
  "sN54403DCFY5TAGS",
  "R1/XQuibaZvS5Bcs",
  "YAvBqr7wwgu0zjpr",
  "cWISgK2ykkgO0nXB",
  "WqYwSeL4aklEwy0R",
  "YJplRhbVq9KZNDmu",
  "lHHF/wiBges/ZVzd",
  "KHmpBQegLgFDX0bC",
  "Hxbuy4jrxGQKMZTN",
  "zfw/DNcD7Q9AeOJX",
  "kzi84Ec/Oo5efPFF",
  "U0FINEhxXTWRTcHt",
  "cG8/MBzzOeec09tq",
  "OKFTh+aIYltQLGY0",
  "Y+uPHUNEf+htpfvv",
  "v5/22msvGs6mTIQy",
  "X0gKQjzWHnvsTpde",
  "9gtafsVlqWLAFgYT",
  "AOgkV7DJUrGYo3Ra",
  "ZGFQ8aRYFmZSGRx2",
  "TULgmV3WAJFFKhXz",
  "NOnjT+ipRx+j1176",
  "H33x7js0ecoUCTFA",
  "OT1TcxyxYJytWtLq",
  "PXKsCqQUYLhulSpl",
  "sDgSY4q4Lwx+cG8D",
  "bI1dfkXaZttt6Vt7",
  "f4tWWX01FrQGUMVv",
  "cWZ3u7jLASzVRGxb",
  "MmMDZNtnMpwIASbn",
  "D7//Ix1//PE12cz9",
  "MVsex2Y1MXhff/31",
  "tNdee/D3OEZ1l+M7",
  "uPvvuONOBvbM4lou",
  "eJvp7M16YzR7Y0Tn",
  "tZantjPNvrdjHTmk",
  "wi8RSLYVV1yW++P4",
  "8YvzegCXuXwXpdOt",
  "dOONN9IPf/hDZjJx",
  "7VCQQF3HfSgGNlvD",
  "8WmfUP1MDQFBfzvk",
  "kEPoyCOPpLXXXjv0",
  "EihDidhK2x3O4QAm",
  "qY+reDmSAGTL89gx",
  "1klP2oVOOpSVj9pu",
  "lKWOcw6ZV0xifADM",
  "MjP6L7/8Ml137Q30",
  "8MMPm4pYUaiC9nkN",
  "YWHvgmE2h4N80ZwM",
  "x7/ccss1Em/9IRGt",
  "NDRHFduCYjHQjK0/",
  "tiYRvdZIUtm8Hoh7",
  "Mx0gbPkjGOKvEKd4",
  "+pmn8UDY1JzleLem",
  "bBMP0Dy4IK7QSdYA",
  "FmVidNDxkLWNknqo",
  "DV6UWDPInAC84vfa",
  "p0+haZOn0KSvJtJH",
  "731Ar73yCn320cfU",
  "NnMWuzEL3ZJQoKwV",
  "9lssAtAimz5NTVmI",
  "Rns0csxoGjd+CRq7",
  "+DiasNzytNa669CK",
  "K69EE5ZejgEnu9uN",
  "HqDUUhb2C3cHgy+S",
  "IWDCYsIVKDGTpZJk",
  "zY4dsyhXijnyyKND",
  "xmYgnyEKRmD4fbxf",
  "ZJFF6PHHHwszlXEe",
  "mvmL67/bbrvRE088",
  "Fca/2vGJuFaNMJrz",
  "O9BUYKPgJtLMlLjI",
  "bEYkdRAXueuuu/D1",
  "Q1UegCV8/867H9B2",
  "2+3AYErLsUasKAeE",
  "9Ov4dIKmrmncQ+wf",
  "8cDSppEJjwpBTTwp",
  "BbO65ZZb8rYcM5yW",
  "49f+ZE+0mN1EcIDZ",
  "v7qwYRoCgvrqdliB",
  "glz93zfhJmg7miyF",
  "a/Ppp59yFvZjj/6b",
  "XnrpJXr//Q94MifM",
  "Za0r3k6SsydM84Ph",
  "fM4991xOwuvFbiCi",
  "w4bmqGJbUCwGmrEN",
  "CauJgWW4JwRplrO6",
  "hzEAacwSYrzWW289",
  "DvjfaacdqMJMihsO",
  "KB25bkbbLU3NnMlO",
  "qCVtssWZkfSrrCWo",
  "/Q1ZqjAdiLzAMKD4",
  "zZJoBeI7ZH9/9dVX",
  "NG3SJEsHUdxzygqB",
  "hRk3YUl2c44bN44B",
  "RCIc+AS4qbRK2N9R",
  "e93EhOqACKZSXIii",
  "XYjPm5uzJuNbWKTL",
  "f/0b+r//O5uZTHvQ",
  "F53P/l9/LdenYF8B",
  "E1zlDz74QJhpjnOB",
  "Wx3bFEt5+sZmW9F7",
  "773Hn+PasfyMidHU",
  "1wUdaMK4So3rhqyc",
  "sneu49Pvfvc7OuCA",
  "A7hNwZ2Oe6wJNjvt",
  "/E0GVMoGog2pWD5A",
  "6UCYTuJ0QgA2Vd3i",
  "dlwlDO833HBDnkSg",
  "32240bo84UB8NGJN",
  "FVRrNjeSc3S7sJCB",
  "8VCwBwAlWA3jrRWm",
  "dB1O3vGSXG8dfQ2A",
  "EiLrWD755BOaOHEy",
  "Rw6EXcfIijqoBJYQ",
  "7wLMbmN2v6plToev",
  "NYgFDiei6wf/aGJb",
  "kCwGmrENCau5zjrr",
  "0Ouvv07D2TTGCoMt",
  "BkGNtRKA7FC2OcMD",
  "5N5770XnnXMurbLq",
  "SrwObwcwA6Fmo+EH",
  "gXNUECEDNh0/oMCv",
  "Ujc08wA6vdo4sUwq",
  "y+yiygdhG2X3OKlD",
  "17cSKpS5UpCkbkV7",
  "gFWXHtzkOC/sP5NK",
  "hwyRulu1JjUGTwA1",
  "qRLTyrGRMlASXfTz",
  "X7A2oIpnayaxuDn7",
  "N5AqwNAB2WZLFZig",
  "hN8NN1wX6iQy2OyY",
  "xffq/fc+pm9+85uc",
  "zGIznvUZ2IMFNHuz",
  "wQaiyqThOCFOrgld",
  "Onn62U9/QieddJIJ",
  "p8D9T9OM6dP5Hh9+",
  "+OF02+13hiyfnbzS",
  "yLVrxLSd2gk9Gkur",
  "YE90NKOKT7Z+6oQJ",
  "S9DKK69MyyyzDOur",
  "Imsdrl7EdXKt9kxG",
  "YjBNgo/NKqpaAhhK",
  "LidrwO3MmTNZnBxS",
  "S6+++jq7jd955x2u",
  "SmXXfGfNVnP7tU2x",
  "F6GpidrbZ4WhKto3",
  "NUZWlRvmB5AJaxAL",
  "rBKXn4ytrxYDzdgG",
  "wq4ioh/Oz3GaggOs",
  "MpBmwNXsZdW6k+SC",
  "CotdH/KDg+mEE39M",
  "yy6zLFEFiThVEXGu",
  "yOCScpHmk2Am03c5",
  "Cozd7nBUY/8lIzCN",
  "onkYuGBcycfEkjEb",
  "YkrmlcpRRrV+p7Ga",
  "nBARxoP5HAsqdcDF",
  "FYhBsztXDLN58VvN",
  "LdnQjQkWM58vCABJ",
  "CaBVthQuTQzIRxxx",
  "ND380KMsas3lDEni",
  "J5GNL4PvwGQl22yU",
  "Xn+bDTv99NPpwgvP",
  "l2sVCEMF0JDNtNJr",
  "r71Gu+66K1eM0fAC",
  "fN9IstL8DjS19rsC",
  "QwXuADs/+tGP6Cdn",
  "n2GAZzODT7CaYMAv",
  "/PnP6YLzL6AElxY1",
  "mrHmuofSWQMAlBRg",
  "KtgMPQUW6wezJZDs",
  "xC4tnoCEPTQR/I/P",
  "0WZxHkjYAeCUOGOZ",
  "COn+we5iwoTzBwBH",
  "e8G1wv/oMloO0i4L",
  "qbcrZDEtiSY9JrlO",
  "CoyjuGI7RtNuz8PZ",
  "GmH9zWUYHqLIsc1X",
  "FgPN2AbCvkdEt/e2",
  "kj6E7YEscr0OTDus",
  "H9AHcr92cgIGMwxW",
  "YFf22GMPZouWWmop",
  "jrtkRsNV15qwj+WS",
  "1gYXsAhToIf9wv2r",
  "A1lUIjMafD039TXt",
  "TD0/jaPD79jXFcZZ",
  "xdAh9KUsnmoN6j3g",
  "GufMDgobBFcgJzol",
  "U1xe8u677+bs2k8+",
  "mbeqAeoKxzW/8MIL",
  "6ZRTTmGworF/uVwX",
  "Aw6ATUxopkyZYirL",
  "5BksY6JgM2Z2/NxA",
  "tr/BtNryqXL/NZTA",
  "r1SlPVVFDUGTZY47",
  "7od0/vnnU75QYhUF",
  "4ASwvbgWEL+HRJSA",
  "VE36iW04mD3JGghW",
  "2d5fT7b99tvTY489",
  "1ttuniKirft1ILEt",
  "lBYDzdgGwpYios8b",
  "BZr1MjoDETA/2PI0",
  "esz2YK+xnFgQ8wbX",
  "7lFHHcExZahtDhCX",
  "zWa42hBiHNVFqJp1",
  "elx2jFe9LIq6+WxZ",
  "GJgyQwp8FdDaGenq",
  "egc4S2eSYZUdPQ89",
  "FhGiltJ6AkbT9MQT",
  "T7Cr/PHHnwjdr/Pa",
  "1J2K64Qa0hDXByiG",
  "CxMC9Xye6TSX/YMb",
  "/bPPPmeGFkkmuC6S",
  "1BTVRB8ot/BQmbpm",
  "bQZQ7yGoN8QZQpAd",
  "k51SqcJt8corr+Br",
  "hKxs6JG2tXVwW4Ww",
  "ODL2XVeY0IHIKo9t",
  "7k37t06c7HY6EM/H",
  "ObGr+O6zzz6lCRPw",
  "GJ+jXUREZ/f7YGJb",
  "6CwGmrENmXg7KgR9",
  "/PHHNa4nZfUG8kFa",
  "bzo4D4TVs4X4P4zT",
  "TCTYpY4ybsccK1mz",
  "6l7DJho7pqwuTMs6",
  "Jr2oKoktOq3nBte0",
  "gkRdxz4e7FsTQGyg",
  "qq5PuJn1mtvXRMGW",
  "JlegDOcVl1/J2qdw",
  "kdtuznltes7KTF5+",
  "+eX04x//mLPyoQyg",
  "cjdYD3F2SCSBJiTA",
  "PkTHbUCujCaSXrDu",
  "cDccN2JRAfjrvQH4",
  "vLurixNloIkJQfY9",
  "9vgm3XDDDQw6ud53",
  "pomLJjQ3t7II/r77",
  "7mtiYLVvxEBzOJid",
  "ta5s9WD0P1tOzH7t",
  "xXYjogcH/GBiW+At",
  "jreIbaDs8d5W2G+/",
  "/cL4JZuhs93J/TV9",
  "SNugbCBMg/ttSRSw",
  "DgrS1OUNIAC2aJed",
  "d6MddtiBK9h8/vkX",
  "NeeroA6mjCKApHyE",
  "dSR5ARqWxSJYyEhC",
  "yB4glCFlSZdyARGg",
  "zKRiQZYt3qMyUFNT",
  "JtSkVKCl54BB7Isv",
  "vqBf/+oKrvKz5x57",
  "09NPP02eJ5qFwwVk",
  "2uesMlQIV/j973/P",
  "TJ3GJ+q1RZLI448/",
  "TjvssD3Hn7KWqRW6",
  "oRMPgNT5wXDeYJxt",
  "CR2WLdJMfRNfDJC5",
  "664701VXXcUxuao5",
  "iphMXCck5CHzXKV8",
  "7ElPbPPWdPKnEwhV",
  "lhio+F6daKnpM2Wr",
  "rbZqdBfPDMiBxLbQ",
  "WcxoxjZQBm216+a0",
  "AgANskY1McBm3Ya7",
  "3lwU/C8u6fpMWZuJ",
  "gCFZQZlFDPTbbb8N",
  "bbfddrTjjjtytqwm",
  "LGAw4UHfFWCnjCSs",
  "p4QMu7/aovAAlcpy",
  "KWNcM6j4sl98D2br",
  "3XffpSeeeJKeeeZp",
  "ev755yWz1mK47H0M",
  "h3tTH7qA9yquffXV",
  "V9ORRx7O/yvY1GPG",
  "eR144IF0zz33hqEO",
  "9SUr5wf3uV3xR4Gy",
  "PWFQ6Sy0M+icQgoI",
  "4LultZmmTplCI0aO",
  "Zm/CzjvvShMnTrSy",
  "vgVkDoQ8VWz9M1WI",
  "0BK0Ax2uohMKBbNq",
  "DWKAl4hoowE9oNgW",
  "GouBZmwDZSsT0Xu9",
  "raTMmjJldpbmcE4G",
  "sq2nRJIo7hQPcvt4",
  "omxWLNClXGmllVgj",
  "EDXgV199dU4iWnLC",
  "eC65B4NOoAJOBY4A",
  "ivVJQHZykIJWbKNJ",
  "MwBeyKyF4PV7731A",
  "b7/9NoNK6CUiWUZj",
  "PxVo2fcirJvegNj5",
  "UBnYO5hKKsEQn4lz",
  "vOOOP7M7GOet8lT2",
  "BOCYY35It912W1gq",
  "UEW5VUdyOIDpOZkt",
  "a2XHNuvkpymTpa23",
  "3ppu//OtIdsOmaMZ",
  "M6ZxeMD7H3xEu+++",
  "O3388acMuDUGEMy1",
  "TDBioDlc4o/t+uj1",
  "sl8DZXZyUIP9e3ci",
  "emBADyK2hcZioBnb",
  "QNoUIho3pxUwGP7n",
  "P/8J2TpllAaiHdoM",
  "HywEaQPUxu2Hs7po",
  "lf1DCT3R6IvApz1g",
  "1B+jDeiQDbzyyiuG",
  "uoBwcS699NI0YcIE",
  "lmwBaFIhbnV/KzOn",
  "UkcQlgZ4hCYgGEvo",
  "SX755Zcs9QP3cEdH",
  "V03Mpoq3a6Y6XPYK",
  "bPVcNf5UKsP0fu3n",
  "ZANxD+yYNWUz1TKZ",
  "FN133320+eab83tJ",
  "EBKtUQArRAkhWx0Z",
  "2MhOR1ymzUzPD2ZL",
  "BKmEEdoGrsce39yd",
  "S3UmUzJxaG5poSmT",
  "J3PYBKrb7P+9A5jR",
  "RPa9MvGSkCbyP3GM",
  "5rw13A9MPNF/0XfR",
  "vlWaaSDltaKCEfLc",
  "3WijjbjWfAM2ioja",
  "+3UQsS20FgPN2AbS",
  "fkNEx/fmPgegsoHM",
  "QJmCv57KQA7Evuck",
  "h+N5AHAAj7Xi43Zi",
  "jr1NT0xuffk6/S1x",
  "B8vgUFsaUGI3sRQK",
  "pRBYz+YMas5Ds971",
  "f9Rrx1ikm8P1rwDT",
  "/n9eAU372ijgt2MV",
  "kfSChJ+77rqD5Y30",
  "OigwR+wiAPxdd91F",
  "hx12WOhm1xi44f4c",
  "rHf1g43FOQBQoz74",
  "xT+/iCcYTc0ZGjV6",
  "NH315ZdcsvPll1+k",
  "PffckyZNns5gHBn4",
  "NnOWSilgj4HmvDS0",
  "Uyg9AGCec8459Mor",
  "rzBTj+pXvUkTNWo9",
  "xWc22O5RkGPdfh9A",
  "bAutxUAztoG0XYno",
  "n72tpBI8OnDaUh5z",
  "aw65HKcIQHbmmacx",
  "c3rggQexnAtqEqOV",
  "19cx5+3qRKHV5gfw",
  "MZxsOJRgVJYOskdn",
  "n3126DrnZKuqVITB",
  "cX7wwUf0ve99j955",
  "+71Q0N0nScpSoXo7",
  "M13Ob85tYTBY854m",
  "JBqaAXUD2FlnnUVn",
  "nHEGTzSEWc/S1KlT",
  "WcIIygGoGY6yivND",
  "HOqCbDopQjuSEAhz",
  "P12ZpC4yZiw9+eST",
  "NH784rwuJKmee+55",
  "uvfe+7m0KioTgXnU",
  "8Ak7/KEvzyqd4GvY",
  "RYPt4ttEdE9/zj+2",
  "hdzqs3TjJV76sbQG",
  "Ddj++++Pp2KQTCZ5",
  "8TyP3/dncRNekPKS",
  "KIoTHP/jY4O2WVOD",
  "n573E37vJpzAcZwg",
  "kUiEv+W6Li/6GV6x",
  "1O8Xn2Hb/h7fgr7o",
  "9ZtXix5HJpPh93vu",
  "uWcwY8aMoKOjI+ju",
  "7g4q5Xzg+6WgbdaM",
  "oFwqBJ0dbcFBB34/",
  "QPhsNp0JPC8VEDlB",
  "MpnmJZtt5vdY8F2j",
  "v9+vNuy6/KrtTfeb",
  "TqeDdDoZuG6CX4FZ",
  "FllkTHD33XcG1Wo5",
  "mDJlUlAul3mZOHFi",
  "4Pt+cMsttwQtLS28",
  "/UD0r3jp/9Lc3Bx4",
  "nsP3EfcQ/zsuBemM",
  "G1zyiwuD7q62oFjo",
  "DHy/GEyd8mVQLuUC",
  "368ExUIu+M9//hO0",
  "trbyfvDM1NdG2x7a",
  "FPqGtq1UKhX8/e9/",
  "Dxq0ccNgbImXYP5d",
  "5vkBxMsCt7zayJNL",
  "H34DNQgmyGHQkE55",
  "wSorLx/MnDEl+Pyz",
  "j4LFxi0SJN0IXOqD",
  "OZvNhgM7XhWI1uwz",
  "keDv9MEeL8MXaOL+",
  "6f3FKwbSVVZZJXju",
  "ueeCUqnEIDPX3RkO",
  "3NVKKeju6gj+ePXv",
  "g5GtI7g9ANDpfvC/",
  "DsiNnF9/r1/9pAu/",
  "29TUFL6CtBoxoiVo",
  "bs4G66+/bvDii88H",
  "5XIxmD59alCplIKu",
  "rq5g+vTpQaVSCS6+",
  "+GI+H2wbt93hsSjA",
  "A8DEvQTIZNDpUTBy",
  "VHPw6ScfBNVKIfCr",
  "haCjfXrQ3TWL2yza",
  "6Hvvvh2MHz++5n7a",
  "7a6R9od1MPHQPoKl",
  "QXtvGIwp8RLM30us",
  "oxnbQNv/NbISYssG",
  "OtsXgtWoiPLhhx/T",
  "W2+9RePHj+f4NYiV",
  "w6Wq8XpY4CKdXdyT",
  "JuxonGTsdhz+prGr",
  "Kg0Dt+JHH0mmNeIy",
  "EVzBJTetTHusg3jN",
  "F196njbYYIOoyg4R",
  "tw8k2mCdodCZ1KQl",
  "hJJIpSaHY/Q0gzyT",
  "SXNC1y677EIPPfQQ",
  "rbrqqpzkhcxyZM7D",
  "/Y/tTjzxRI7xU33V",
  "WCdzeJjKFskr4myR",
  "zIdKQEmu0AQ5KsTK",
  "4l5KUqFPue5uvvco",
  "SjBz5sya5DY74RCv",
  "vRnWsZ951157baOH",
  "fkL/zjy22EwDjJd4",
  "GWr3+cknnzygbAvc",
  "43Cdg9XMpJPBCcf/",
  "MCgVu4M333gtaGnO",
  "1jCWEbsQucTr2bGe",
  "GM54mTNjMi8XMIE2",
  "Q65tS5nJIw4/NJg+",
  "bUrQ0Q6mqMKMJlzo",
  "YDnzua4gl8sFP//5",
  "z5n1wTZgEe02MtiM",
  "JpgmO6zDPgewk2C/",
  "zjrrzKBQyAW5XJec",
  "Q7UcTJ06Ocjnu4Np",
  "06YF22+/Pa+Pc8CC",
  "7eZ1u4iX6P7ifgIX",
  "Kqs5cmQrs5lvvPlK",
  "UMh3sqscLnNhM4tB",
  "e9v04MYbruPwH20b",
  "aI9gNpW517bZ2+8r",
  "U6/v+2Atw2BMiZdg",
  "/l7m+QHEywK5PNTI",
  "E2wggabnuLy4DvEy",
  "YcnxwbSpk4NioSs4",
  "6/9O43Uw+Na7m+yH",
  "rz6M45jM+Q9o4r6N",
  "GDEinCjY8WjaPlZe",
  "aYXgySceZ5BZKRc5",
  "TlNAZ4FjORHb+M47",
  "7wTbbbcdb6Ngs5Hf",
  "7+/1s8M4sD/0DQBe",
  "vK622mrBU089wYBy",
  "5szp7DIvFvNBV1cH",
  "f4bvlltuua+FEGj/",
  "itvzvF96ugfJpBvs",
  "9a1vBrPapgVts6Yx",
  "sOzsmBHkutvYfZ7P",
  "dQTbbbs1T5RxP9V1",
  "rmE/eo8bbX8al/7L",
  "X/6yUZB56jAYS+Il",
  "mP+XeX4A8bJALjs2",
  "8hQDAzNQrGFzU4YX",
  "MJtgNbFccP5Pg472",
  "GcGXX3wSrLjiijXJ",
  "PwAi+vDVwVlZJR3k",
  "8WC34zrjZfgCTRvw",
  "4R4CJCL5Iry3CSdo",
  "zjZxGznt1JMZZILN",
  "xODuV4sSu1nMM1vY",
  "3d0ZXHfdNcGSS45n",
  "MJDNpgcdaCobi3aJ",
  "48YycuTI4Kyzzgo6",
  "Ozv5mNoNGztrFpKc",
  "2vizc8/9CTNkGlda",
  "z1zZscjxMm8X3NNU",
  "yuOELrzPZFLB4/9+",
  "JASaAJYAmu1t04Jp",
  "U78Kbrrx2iCbSQWp",
  "pMQca1ur97Y0en91",
  "mz7YiGEwlsRLMP8v",
  "8/wA4mWBXLKNPsnw",
  "4BsIZlPBJcAEQEXS",
  "czgR6KMP3w06O2YG",
  "d955Z5hdru4nDOqa",
  "hIFBGt8rSBk9ejQv",
  "CkjjpfdBbDgATHFP",
  "ykCsjA+/N+1DwytW",
  "WXnF4I6/3MoJF12d",
  "sxhkKtgEgIOL+quv",
  "vmAgt9hiiw460MQ+",
  "0O4UGO63337Bhx9+",
  "GOTz+WDmzJmc8FMq",
  "FZjRxP+vvPJysNZa",
  "a/C2AMLatvU66MQp",
  "BprDY9HnDMAl3re2",
  "NgcHHPC9IF/oDDq7",
  "ZrHrHJNigEywmbNm",
  "Tgm23WaLYPHFFuUE",
  "R2xj30dNUms0xEfX",
  "Q7tq0E4fBuNIvAQL",
  "xjLPDyBeFtjlhkal",
  "jmY3EPZlgNSB1R64",
  "8XDdY489gvb29qBS",
  "yAdnnHJy4BIFo0eO",
  "YlcqvtcBAOADLvcx",
  "o1uDTNoNjjzikOCD",
  "998ONlh/Xf7O3mdf",
  "j00f9HZs6NzuZ7gu",
  "8xpoNrLUtw+87rzz",
  "zsHDDz/M4A0gE6wh",
  "AB3ewy2NZeLEL4Of",
  "nHV2sNwyy3JbaG1u",
  "CQEr2hGD2LrBvj7+",
  "F6yjfG63Wfkcr60j",
  "sgElKNh0sw2DRx97",
  "ULLk8x1BsdQdVKoF",
  "ZjVnzZrFy+mnnx6y",
  "7fa5xMvgt3HPTYTq",
  "FmuvtUaw1Tc2CVoy",
  "ySDlZQPPSYaLQ7Ie",
  "4ivrJzlg1UeNbA3e",
  "feeNINfdzuE9uNcz",
  "Zk4JunPtDDz/es8d",
  "wYQJSzCr3sixaQyn",
  "3d51kiWfy3p9sJHD",
  "YAyJl2DBWOb5AcTL",
  "ArvgQdWQaSKHuqmV",
  "cazXE5zToutqXJ0+",
  "ZMFInn322UE5nwty",
  "He3Bvt/em9nOpmya",
  "gaWs67J7Cm6qRRcZ",
  "FVx80fnB9GmTmGH4",
  "xcU/58FFj0MlaHB8",
  "GkPXF7Co29QmCMz/",
  "MXTzGkQ2wjja98nW",
  "ToVL87vf3Td47rln",
  "OcEGsY/KcAJwguHE",
  "/1OnTAqu+dPVwTc2",
  "24TBAtoF2hDaTf21",
  "6Om9JoLo/2h3clyJ",
  "YLvttwr+csetQVv7",
  "9BBgAmyC8cJniB+9",
  "5557glVXXZW3sZn2",
  "GGgOzcJtxoBMvN7z",
  "17sCv5QP7rz9lmDC",
  "EksFLU2tQTqZCoEl",
  "9FkBLluamoOmTFYm",
  "KS1N3G4uvuhCTkJD",
  "6AbA5vQZk9mF3tE5",
  "MyiVc8HBBx/EyULa",
  "VvoSfmG3d322IJns",
  "l7+8rNFH8s+HwfgR",
  "L8GCs8zzA4iXBXp5",
  "sZGn2u67717D8s1N",
  "Yo6CBnWf4jN1iWM/",
  "Z51+WlAq5oOZM6YF",
  "e+y+G4MDLMhQhzbh",
  "yBEtwe7f3DV45ul/",
  "M8DEApHv5597ltdT",
  "15N9nAoyGzlG3c4G",
  "ptgfXJsLAlCYH4Cm",
  "ggVbu1XbGdzPTU2Z",
  "4MADv89uaQBOxGuC",
  "0QTI9AFAO9u5DSGZ",
  "6LVX/xf8/MLzg+23",
  "2yYYv/i4MAkNExZM",
  "ZBRs6Oc62ANUgl2C",
  "4PoWW3wjOO+8c0NN",
  "TIBaMKr4H6wq/u/s",
  "bA8++eSjYNdddw2P",
  "W7PJ688tXgZvQTvB",
  "hAT3Fvd4qQlLBB9+",
  "8F4wc+qkIN/Zxu3h",
  "m7vtwoByREtryF7i",
  "VUHmiNZmBpl77vFN",
  "1seECkIh381AEywm",
  "JheYWLz3/lvBuuuu",
  "zbqpOhnpbdFJazSp",
  "kXatCURY+mAxmxkv",
  "NJBLXIIytsG0DYno",
  "xUZWhB6cXUpNXxvV",
  "2lR9TK3draX8oC+I",
  "/9Mpj0466SQ677zz",
  "qFjM00033UTXXXcd",
  "1xbefPPNaa+99qbV",
  "VluFtQg9z6V0JkOV",
  "smgt7r33t+npZ56t",
  "qS+O/+1jbuQYoX+n",
  "29slGweijvHCXoKy",
  "N6t/zkGvUtuLlnhE",
  "TXfcU9RE32233Vi/",
  "cKONNuI2kCBZB9+r",
  "DifuJz5DicdXX32d",
  "Xn31VXrjjTfoiy++",
  "4Lrj2C/2hbKQKCS/",
  "+uqr0zrrrMOva6yx",
  "Bi266KJ876WWvWi2",
  "dnd38//Y5u2336ar",
  "rrqKbr31Vq5RjuPl",
  "YzHHDcMxoI3Hz/HB",
  "NS1fi1aOpr7vvt/m",
  "Z0jac1mfckbbLNZd",
  "veuuv9Jll13GWr4t",
  "zS3cVlBOEjXocW+3",
  "2moruv3226m1tVWe",
  "S5kMzZwxg1KZJG8P",
  "Lc2bb7qVTj75ZCoU",
  "ivxbUT362RvakLZl",
  "HKs+q/T52Yf28TAR",
  "7TIQ1yy22NRioBnb",
  "YNtDRLRzbyvtueee",
  "9MADD9QANgUvjbRR",
  "HQgAFGF4+ANE6gM3",
  "lRTR6gMOOIB+97vf",
  "UalU4O87OzspnU5R",
  "Po9a0RV+2ENEWUCF",
  "AIo777ybjj/hJBbQ",
  "xr4wQGjNdBXD7g0s",
  "2rXdtb42PlNAPJDC",
  "9fPChjvQVBF+G+jX",
  "D8IAmjDUocb/aEsb",
  "b7wxHXzwwbTH7rtx",
  "28B2AHeMALRdJhJU",
  "LlW4ragoN8AF7jHa",
  "GNavmPrU+E2sh/tu",
  "15pWYXgcz3PPPUdX",
  "X301/fOf/6Tu7jxl",
  "s6jBXg5rtevxapuP",
  "bfBN24rrCKgD+N9p",
  "p52oKZ3ie40vcK9x",
  "Tz799FP685/voH/8",
  "4x/01ZeTqK2tjUaP",
  "GUnf+c536IQTTuB2",
  "9eijj3INc0wo1ltv",
  "Pdp8y29QPlekESNG",
  "cHu77777uKY92gSe",
  "ZY3WMIfh+LQOOtrc",
  "4osvTp999lmjp7o8",
  "EX3Sn2sVW2z1FgPN",
  "2AbbliOijxtZEQ9q",
  "tEdlbfCQxP+NtFFl",
  "hDAY4EGrlL0O7Fzp",
  "hxKUziTp//7v/+iU",
  "U07iz/E76aRHuUKR",
  "fyubFXCKh/uoUaOo",
  "UqnyQLHHnvswY4Vt",
  "sH8FLDAFi70ZBiGs",
  "h1fsRyvRLAh9cLgD",
  "TVhP11mvP4ClAky8",
  "wlIpVHCp8IC/wvLL",
  "0rbbbkv77rsvbbrp",
  "pgw6FTAqw4hXZZbA",
  "QKHNMShFuzSst7La",
  "ylBpJSCAkzvvvJNZ",
  "9k8++YT3pe0Dx4P1",
  "FJTa/zc60YltYIBm",
  "NpOipZdemp588kl+",
  "PhRz3fysaBk5gu8V",
  "gL/eZ1T5mTZtBn+/",
  "5JLjyfNSfN+/973v",
  "0dNPP833ddSoEVSt",
  "BLTY+HF0yCGH0Hbb",
  "bUf77LMPzZzZFlan",
  "wja93V8FmjqJ0opQ",
  "aCu9saGWXUREZw/E",
  "9YotNttioBnbUNjT",
  "RLRFbytdeumldNZZ",
  "Z4Wu6EYB3OwGBX0V",
  "96IAw1EjWrl03/1/",
  "v5e23npLas42UREP",
  "cgoMCBBQCvYS7zOZ",
  "LD+or7/hFrrkkkto",
  "+vTpNSUpleHsrUyl",
  "lrTUknEKzATIzN9s",
  "5vwANG22R9uXgkAM",
  "6MmkMIz4Ttlme8Bu",
  "ymbZ9QlbaqmlONwC",
  "oACu9WWXXZYySY+/",
  "43AKM+kB46n3ulAR",
  "F7myTQAhkyZNoscf",
  "f5yeeOIJeuyxx3n/",
  "CnyV/QRbBiCKY0Cb",
  "hCkbq6C1EcYrtv6Z",
  "hkz41TK7tTFZ5UmC",
  "Jyy0m0qGLKJOKHGP",
  "HSfB4RPt7R00cuRI",
  "+sEPDqE77riTw3P4",
  "Xvtyv4OEMOtwqWNd",
  "15XJhHo9GimDq21W",
  "/8cxfPe736U77rij",
  "0dNsJiJpZLHFNoAW",
  "A83YhsLWJKI3Gq2B",
  "jjapjFCjIMxmBhWg",
  "qitSGMggZH+Qm7He",
  "+uvQYw8/wm5JbNdd",
  "yPOAEATyexg0sI/m",
  "5hYBBoUKx+29++67",
  "1N7ezscGAIDBBNbb",
  "ccJ9BaBw1FFHcd1i",
  "xHcp8FQWZH624Q40",
  "YQr67BjHKOYS4DJq",
  "R5IoGW3LsXn4M+ep",
  "bQvtdYUVVqAN11+b",
  "Yy/xPz4DO4/2BMN9",
  "/2LSZJ6kAFy+8847",
  "HH/55ZdfUlcXYjLh",
  "ps8ymLBZUg2psJ/R",
  "enx6HjaDFdvgGcAj",
  "lhGtzfTMM8/QYost",
  "xp8nfLlHCU/iwguF",
  "HK+HHB3cG7jGi6U8",
  "uU6KfvWrX9F55/2M",
  "v+eYW0pwW+nOdXNG",
  "D9h03EvUQIep5wTW",
  "2zitkw99ZupzT8Fv",
  "A3YKEf26f1cptth6",
  "thhoxjasWE0kUoAh",
  "gtmxdI3M6Odk2Ic8",
  "eCv8cMeD+IEH/k4b",
  "b4x8JSLHr/JDHd/p",
  "w12D6dUtjwFm7733",
  "prIPIFxk9BEgn1QO",
  "lpJeksqVMrlOBBTw",
  "ioFlheWWoZ/97Ge0",
  "2x67cWwW3LAYbPKF",
  "Up8A9ZzOrz6GTwH3",
  "7GJd60HX/AxU5/Vz",
  "DG0Lpu1Vr62d5BNb",
  "/6ynfqKAH/1v//2/",
  "S6NHj+ZkG0xU1W2M",
  "SWZA0o81vlrDciKv",
  "iTDcGraD9oR+i1cO",
  "l3GShFt57LHH0iWX",
  "Xmzc5MIkY3JaLvs0",
  "YkQLH0s+n+Ntmpqa",
  "zUSjQO9/8BFtv/32",
  "Nay0hkAMVB/EuejE",
  "FefSB6a7g4hG9vsA",
  "YottNmZ8PLHFNuh2",
  "bCMrwS0J5lAf8grY",
  "+mu6H+wXD3s8jG+8",
  "8Ub+TjPT7dhOBQqI",
  "z4Thu6233pp++OPj",
  "GGQ2NWWZ5YLLFYoi",
  "mbTEdgJsgumsVGUA",
  "0Qx4ZLwDpMIVutxy",
  "y9Gqq65KxaKwDQMx",
  "yNhZyDqIKdDB/+qK",
  "1fNUF/5wAJkLgikb",
  "ibaEAR4ABwvamoKL",
  "2ObetE33lNyF10UX",
  "HUs//elP6XdXXcWu",
  "4iWWWIKvv05AtF9o",
  "gpaCTJ0U4HthIoW9",
  "1ImPhuDo5wceeCBv",
  "q/cUE1Pt5zrRU0Ya",
  "zwGsh9eLL76Yt9Pf",
  "g2ki10D0Qe3Pej0O",
  "PfTQAX82xxbb3FoM",
  "NGMbKnvTBJv3an//",
  "+9/DWbmddNMfw0Cg",
  "AwYMg9aDDz5Is2bN",
  "YvdWwnM5TrOju4uc",
  "pEeVAO4wl1pHjaR8",
  "CexlggrlEp1xxhm0",
  "007bm/grh7ONeeBI",
  "+JRMucxo5vIYXJLk",
  "uERjxo6iSy/7BSeR",
  "YOBDAsGIkSNpr732",
  "oqSJ67MH0P4YBjhl",
  "ZAB6WFbHDGTKbtiS",
  "Pgo8Yxscs3XkYuuf",
  "2a5he8KEBe0cclHj",
  "xo2j6dOmcezszTff",
  "TKuttpoBkXBBl3nJ",
  "ZJCQkzNJX2VmKcFm",
  "or/Y4Tp49mjMLoAi",
  "tjvyyCN5kgjwqJM1",
  "kRNK8auG2+BV+xz2",
  "gcQhKGoowNR9wwaq",
  "bdjeDLxCtaAPcka3",
  "D8hBxBbbbCwGmrEN",
  "pf2s0RXvvffeUKtw",
  "IAyDiMa7KdgCuwj2",
  "Aw/9fB5ySE2cNHTj",
  "jTfTYYcdwfqZRxxx",
  "FN1xx13U2d0VSiaB",
  "CV133bXZZdbUlCIn",
  "IZnyhYImA8ggCHAL",
  "JuOII47g38w2NfH3",
  "nR0d/BniNjFIDgSj",
  "oYOunh+yVSHdpOwK",
  "TH/Hjk+0wXdsc2/K",
  "gNsJGbENnGmSlrZd",
  "BYEAdujHAJdIpFlk",
  "0UW5r2+wwQYM7uCF",
  "QLiMSoupaoT2E1uX",
  "VMNMNEwH+1G2EjGZ",
  "SAIC6ERSjzKsqmFq",
  "J5nh2BCfqzqq1157",
  "bY3OpQLCge576vnp",
  "Y5hRzGbGNvg2rxXj",
  "42WhW45vtDzFaqut",
  "NmCVPbQ0JZETZLPN",
  "QSbTxBVWVlttlWD6",
  "9Klc5eP4H/8oWGbp",
  "CVzBAxWDUAFk9KgR",
  "XK94tVVXDP7x979x",
  "XWJUDZoxfXKw1Zab",
  "cV30JcYvGrQ0S/UP",
  "1DBGJaEVll82eOjB",
  "B4Jcd2fQ3jYz8Mul",
  "oH3GdC45h/KWXZ2z",
  "gjPPOI2Pwa5UM7eL",
  "VgSxS1pqBaKWlpaa",
  "ykZa8tMu+zmvK68M",
  "VOWfeFlwF22v+j9e",
  "tdLWY48+zH0NS2dH",
  "G5d3xILqO4cdejC3",
  "Ee0H2E73g76A6mHa",
  "N7S6jlSLyobPjeuv",
  "vS5om4WKYbO4OtSs",
  "mdO5ug/6dndXG1f4",
  "wbOhXMoFxUJnkM+1",
  "B22zpgUvPP8fLleK",
  "Cj3azvV87HKRA3V9",
  "jjzyyL5UADp5GIwH",
  "8RIs+Ms8P4B4WSiX",
  "pxt9Eg5kHXABWanA",
  "dVGiUuqp4/Mf/ODA",
  "YJ211g48x+VycXgF",
  "aHQoEb5mm5LB6FEt",
  "wV133s6lKTGYdHbM",
  "CH5+4XnBMhMWY0AK",
  "gIkSdT846IBg0sQv",
  "efDB4MSDXmcHLwCZ",
  "1UqBl3fefjMYO3Zs",
  "CPb6A5rsuvAKIrGM",
  "GjUqrNGu+7dBZww0",
  "h+a85vVxzO8LriEA",
  "oZbf1PaOmu8rr7wy",
  "l5YtFnIMMgEAy6UC",
  "lwwtmBKi5557btgP",
  "dGKHtq+TLnvyhfW0",
  "dCNeDz30UN7XjOlT",
  "eV/YPwAnfguv+A18",
  "Vip2B/lcR1DIdwS5",
  "7rbArxaDn5x9Bk9a",
  "9ZgV4NoTwoFYcPy4",
  "Nn2wu4fBOBAvwcKx",
  "xFnnsQ1ruaMXXniB",
  "vvGNbwxIQhBXaKmI",
  "60rcVnBzSZJQNtvM",
  "bjQViFeXPb6DKyqT",
  "9aRkXDLFVT1WXXUV",
  "ieXyPJas+d+rr9HU",
  "qVNpww03pLXXXpvd",
  "brw+EoMgzl6qsts9",
  "X8pbEjlJ+vHxJ7Ir",
  "3j4/7ZN97ZuakIAF",
  "bkTYKaecwpn8t912",
  "G7v5bLfacBL7nt+z",
  "zmMbXNMYa43Zlr4s",
  "bRmJL1dc/qswgxzf",
  "S5/Ohgk4+UKBnn/+",
  "eda/hLQUXNpI9IMn",
  "vqkpw5np2Ebd5+i3",
  "eN1ll1043hMlSDWU",
  "x/4NuMS10AQqiyF2",
  "23VF43TmzFmc2PjW",
  "W++QH4irXWOl+5gV",
  "Phh9YBwRTRvQA4gt",
  "ttmYi0y92GIbYptq",
  "hIF37G3FJZdckgcE",
  "lOXrj6lEkj6LMXCV",
  "SkXy/arRw5TqG3a8",
  "lqxTElmkMratcoWX",
  "zz79lMvJZRlElqil",
  "pZnWXGttLiWH2EjN",
  "+MZ+w8GLBNAWSoVQ",
  "k69UKtNaa6/DFWGQ",
  "YGCLLffVFBhrBi2W",
  "c889lxMYkCiBWFSN",
  "cdNBW67H8ABo83tc",
  "oy1p1NMyXK7z/Gp6",
  "/exCDlgwobroooto",
  "6aUm1KhHqGkVLtyf",
  "VVZZhb797W9zHfr/",
  "/e9/pq+IuDoqgOEV",
  "AFTjNKESgQkaAKGT",
  "kH3hWfTBBx/Q66+/",
  "zhnni48fz8eGdfRZ",
  "or/50EMP0R//+CdK",
  "JByq+iKThHXtErba",
  "bvrbPvq4/RlE9Gi/",
  "fjC22Ppi85pSjZeF",
  "evmqUT/P+PHj2aVl",
  "xzOqS1LdUUPhvnPd",
  "RJDJpII77/wLx3Z2",
  "59qDru42jrks5BEj",
  "1s7uM43jxOf4rFjq",
  "5qWza1ZQKHYFpXKO",
  "/88XOoPfXXVF6Pqy",
  "z0dde7bL23Z7qyuw",
  "ubmZXfyem+AY0XTK",
  "C0479WR2JX715efs",
  "2jv1tBMDSsh5pFJy",
  "DTG+wR05kC68ub62",
  "cCe6Hr+6jpwLFn6f",
  "iEIB7GU4uc4dzw2Q",
  "FeZ4CbnOCQq8lBv+",
  "P6+Pr5HrzyEi6UzQ",
  "0tQcHH7oYcGmm2zE",
  "9wCxyrqe7Xa22+Vg",
  "H5+6tqXNIgwE7mc3",
  "2GTTDbg/oY0jRAUx",
  "mnjFeyx4D7c3XuFO",
  "h/sbLu+XX3ohOPSQ",
  "HwTjFx/H8dW4R8kU",
  "4jibgmWWWSq4/PJf",
  "BYVCLpgxY0ZQKBSC",
  "t956KzjssMOCxRZb",
  "LGhtbeXznzBhQnDq",
  "qacGTz31FLvO8bvo",
  "c/gdLDvvtANfO4TT",
  "9Pv+GHe+fe31mrzw",
  "wgt9cZnfOwye+/ES",
  "LFxL7DqPbV7ankR0",
  "X6Mrgy1QVxjarerg",
  "KbsxmKYZq/C44/8d",
  "d9yRmUgvaSr7VKqh",
  "W01daraUkJuU92A7",
  "UBlIWUWpElKkPXbf",
  "m8MEVMAZZmezYp/K",
  "kGqJRGV3mOGpSPYs",
  "9ons2LPPPptLbY5f",
  "YgmqlMv0xVefcwhC",
  "V2eOurvzLL8EwXjN",
  "3p3Xepqe41LVxzWU",
  "6xRpDMr3YVlHc13q",
  "bZ4/xxLEqgKlEphx",
  "CIjjmIlMoSmBC8PY",
  "0OKchKgQ7LHHHnT3",
  "PXfR5599xm7pp556",
  "ihKOtH9lE9EW0W5V",
  "5HywTfs6wl2gDAF3",
  "N7wQl152MR133HGU",
  "CHrO4NaBDscIV7pW",
  "W9K+grAXlAB96X8v",
  "cl/casttuJ+MHTuW",
  "mc9Ro8bw+aM+Ofqt",
  "eiq0D2r/3fwbm7L0",
  "GbLcYW+++WZYlIGP",
  "o5/nr1JKYHDV+4H7",
  "AZk0KHT0wZYgokn9",
  "PJzYYuuTxUAztnlt",
  "vzCunF4NJfuWXnrp",
  "mtq/6qYa7HZsA77R",
  "o0fyYPO3v/2NVl9j",
  "VamFncmGunk6sNnC",
  "6cAeMAwWeuxYD646",
  "VI356KNP2B2PgU+F",
  "v/EbHF9mylxiUFfg",
  "quUP8R0+90ys6THH",
  "HMOSSiptpG5En6p0",
  "5pln0rXXXM/xaMCy",
  "KH2YSDRWR3kogI7n",
  "epRtSvN57b///lyJ",
  "6bVX35DrHiK2yIbT",
  "sysqeyqTBFxjAc3u",
  "fFGL3KEExxoCjF15",
  "5ZW0w47b8X1AfC8A",
  "05Sp08O+Zley0TjI",
  "oYjzRX9AHCSAJq7z",
  "+PHj6ZFHH+LwmnQy",
  "UyMtpQBTDf1JSzpq",
  "EQVxd5tCB5mkyBmR",
  "TNxUyuzll19hDVz0",
  "M3yvzxp7oqsTQuxr",
  "k002YVc+dCxRoajR",
  "OuWNmGp36iRspZVW",
  "4ipjfbATiOjKATmY",
  "2GLrg8UierHNazsT",
  "spmNrvzZZ5/xQIFB",
  "QB+6QwE4MIioADoY",
  "BcRqoV45YjJVKN3W",
  "xlNdSwwyCjoxeGll",
  "EixaPg/brLXWWjxA",
  "YTDF7wA86nnhe7ua",
  "ie4fx6F6fXiP8ni/",
  "vvxy3g77gDi8sqb4",
  "fTA/+Fzqu0eD8bzU",
  "0gxLNSYcrqaEc/nu",
  "d/eliy/+OTNECefr",
  "bGs9iBgOJhVZBGA2",
  "NWdo8y02oUUWWSSc",
  "fAx3k4SZPI0ZM4a2",
  "3mbLsF77iiuuSFdf",
  "/Xtm0jQOWKtNwXB+",
  "QwEy7TYKLIl2jCo9",
  "qC2vVcRm10Z0wqfM",
  "PfoR+qVO2LSv2eAZ",
  "59Te3s6TM/R1AE0F",
  "mSoYr94Fjr02VYgQ",
  "S77TTjvRfffdF/7m",
  "QGgBayyn7hP9vo8g",
  "844YZMY2rywGmrEN",
  "B4NocK91+iZMmMAP",
  "29/+9rf8YB9Kw2AB",
  "YWgMcspmINj/ww8/",
  "rCklp9/BNDvVLgep",
  "AyY+12ok+B4MJJIP",
  "wEYq64LfxACnIFXZ",
  "I3yn7nktL3nSSSfT",
  "+eefT12dnSLY3tLC",
  "xwtTFx8YkIMOOsjU",
  "YUam7PAAbTYThYx9",
  "ZAbD7b/pppsyazU/",
  "iMoLo0U0btwiDM4A",
  "NAD8W1pEpH+4G8IW",
  "cJl33mVHBpsMpjyH",
  "pk+fSnt961tc3lHD",
  "VpTZ1PCPoUjkQhuV",
  "iRqyyTMMhAE0NYFP",
  "gWJ9e9b3WoJV2Uzb",
  "q6ATQU2imzZtGve1",
  "a665hv773//WiLhr",
  "KVst9ajATxN8FHhi",
  "n/r/QDCamqCI5wHO",
  "Af2jD3Z/LMwe27y0",
  "OOs8tuFgQERfEtHe",
  "va04YsQIWnTRRXlG",
  "//TTT4cMw2CDJXUX",
  "ohQd4q7KZbjwSjSi",
  "tZV22GEHCnwBlMpy",
  "YDCws8+RtY7jBAOq",
  "7kdlSMAcpdMZZk62",
  "3HJL3u5f//pXjatM",
  "/9eYVB0wMeBecskl",
  "dPLJJ1F3VxfvC/vN",
  "53LhgIrPkimp47zK",
  "yqvSLbfcQp2d3fx+",
  "oEvh9fWaKkhBzOjo",
  "0aPotttu4fAI3F9k",
  "8yPe8dHHkCAbgZnh",
  "WH1H4v4A2Ep0+umn",
  "M0O9zDJL01133U3d",
  "3V3k+8OLga23VNJl",
  "BvzCCy/kKjhQWCjk",
  "8zR69Gg+tw023Jjd",
  "6Mi2VrCljPxQtB0N",
  "SUE4AuyAAw7gpVI1",
  "ccb+15UL6l3pysIq",
  "8FSQyP3WF/AIJQh8",
  "9vnnn9OPf/xj8jxx",
  "qddPJO3f0kmknT2u",
  "kyP0z4GKYdXniU5s",
  "G7RniOiQWMootnlp",
  "MdCMbbjYa0Q0nog2",
  "7G1FlIBD3NiLL77I",
  "UiND5TqHIa4RizKS",
  "kyZP5NjKdCoVMo8Y",
  "ZDRZQkHnpElT6eGH",
  "H6Enn3yKXngBiQcp",
  "Wnzx8QxAAFoxIGEb",
  "uOu22GILrqn8n//8",
  "h/eJ7xDHqZJEGHDg",
  "ygTgvvzyy2mfffah",
  "aqXMn0k5TanDjv1h",
  "XWZiHSm3N2rUaNb7",
  "fPPNNxgYQdZlXpkt",
  "/5NOJVmOacstt+AB",
  "H+AYLO9yyy1Pf/vb",
  "PdTZ2fU1gDmcSj66",
  "DsCQQ2usvhr94uKL",
  "yXUcWmqppch1XHrk",
  "kUc49XxO8kfz2tAO",
  "N910Ezr++ONNG5aJ",
  "kLqVkQy0/fbb01tv",
  "vcUuW00CUuA12H1Q",
  "J5NgM8Fy//GPf+R+",
  "kUqLdizkw2pCMeqY",
  "Vp2waZiJDQjxeaEo",
  "smMArPjsvPPOo1de",
  "eSWUPdNztOut2/tH",
  "yISXxDPAoYQDdrNC",
  "yRSS9/DcwPOif8l2",
  "Clz7yI7iR3cnoo/6",
  "9eOxxdZfm9dp7/ES",
  "L9ayZhAE1b5oday9",
  "9tpDIm8EKRGRGkKF",
  "D5EGymbTQeuIbPDr",
  "yy/lqiCQN0I1ELxC",
  "2gifvfbqS8HRRx0W",
  "LL744mF1E61osv/+",
  "+wcfffRRUCqVgnw+",
  "H/i+z1IqU6dODcrl",
  "cvD2228He++9dyhr",
  "AhkjrWLy3e9+N/jg",
  "gw94u0mTJrGcCqRV",
  "IOmCEnkqs4L/uTRf",
  "voOlmDo724Mvv/w8",
  "WGWVlYLRo0fyvnT/",
  "Qy6pY1Un2mP33QK/",
  "WuZyfpCHwrWTkn6F",
  "4JJfXDTs5Y0gKQVp",
  "qZtvuo7PAe0Axw8p",
  "nc2/semwr2yE6jUo",
  "4yglFqXCDaS58Iqy",
  "it3d3dxOv/zyy2Dz",
  "zTdnWS3IcdltejCX",
  "SNLHCS677JJg1qwZ",
  "QXv7rKBSLXC71rYv",
  "pSCl7VfKxbAvQHoI",
  "9wKlI3GOkDyCJBHW",
  "x/9t7dNZcqyrqyN4",
  "9tlngjFjRrHUEfoH",
  "ylBqm7OPx37vuMnA",
  "9VJBKg2JMifIZJv5",
  "NVr6f/5zYUcNg2d6",
  "vATxEmedxzbc7Cgi",
  "+mNfNlh11VWZ2YRp",
  "ck3EQEb/98dURkiT",
  "j5RhAAsCN+9rr77E",
  "rkeEPeM7sD2X//o3",
  "HE8KRhFZ03bSBBg7",
  "HBdcrOeccw4ztIVC",
  "TpKLvAQzmE3ZFt7X",
  "k08+Tf/616P8u/iN",
  "bbbZhlZffXV+D1aH",
  "WcxcjhlMvQZ6jHrM",
  "wDKS8S6SSDfccAOd",
  "cMKJoYA96E11tSsj",
  "pDIwjVw/20U5u+/V",
  "9adJTbgeY8aM4ni4",
  "J594nBZffHH+X+NS",
  "IRcEaSZmebfcmiZO",
  "nMguf02ywrWH9dGV",
  "OFeWqLs+OE9l9fB5",
  "NuPR+uuvT//4xz/4",
  "M7icNU4PlWi23W4H",
  "qRBVlQcvDOeBmEOx",
  "oZGXchy5BzgGva+4",
  "jrvsvC395S9/MeEh",
  "En4Bw3rMoqckLAQh",
  "HnAr77/f9+n1198M",
  "VRT0ftrXq09jC5oP",
  "2pAf8PHoPVXOMCCH",
  "41033mR9uv/++6lc",
  "CqTqVrXAISwtTSPC",
  "OEaYurd1oKOEvEe3",
  "VQF3DkFJSf/wXGFu",
  "gyBBJ510EjOm6NeF",
  "QskkDQ2+MkN9H7IZ",
  "07l4hl1FRMcN5PHF",
  "FtvcWgw0YxuOdhMR",
  "/aAvG6y88sr00Ucf",
  "SRWfUslIoQioGqjE",
  "Ic0e1QokUSlJl045",
  "+UT6yU9+wp9Bd++n",
  "551PL730UuiqK5RE",
  "4kYHZHXx4T00+/76",
  "17/S+uuvK245V37D",
  "SYhkCsBrMilAFyBM",
  "Y800dpPLXTpS9i4c",
  "WK3YMYkRk+MA0MF+",
  "AGR33313ev65FwVU",
  "MnESDWh9BXG9AU0F",
  "vdgvsuVhAA7YP859",
  "qy03D4Gyhh9otjyu",
  "1z1/u48OOOAgBgo4",
  "RJyLytAMhem1VN1E",
  "nqUbIANgBr32u+66",
  "izUYcR9wXno/cJw/",
  "Oec8nnTk83K8EZiS",
  "hBJUmBrs48exoNSq",
  "liK1Yxbvvus22nnn",
  "nfm9gkdMYvAd2lcy",
  "LTHHs2a18WTni8+/",
  "4vKMH374cTgpwSTB",
  "zrLXSYVODOZojkxE",
  "3ITD+9Bt8N4PfEoZ",
  "7UqUf91kk41kfwmZ",
  "UHFSUkL6gZ0Nj/aE",
  "e4C27nkpEy9Z4M+w",
  "LrvNK7Je4EvYCyYy",
  "0MKElJok/OixD/5E",
  "QOO69f5oSdm5SCZC",
  "hvn+g3OUscXWdxv+",
  "6ZyxLYx2MBHd3JcN",
  "3n//fU7KwcAGJkLj",
  "Iwcq/k0HME3gsaVN",
  "8Bs33Xwz3Xf//XTM",
  "scfS3vvsQ8+/+AIl",
  "0ymq+Cg7GYEhZVxs",
  "QAhhaAisI9sVcYl2",
  "TJgmDQGcgRm1mQ09",
  "DgycmhxkZ7drHJnu",
  "y9ZARDwnEj84UcgM",
  "aDDdRjNoFVz1ZqGL",
  "ZDYxiMqO6jkDZOL/",
  "//u/s5ih1WQnZT5D",
  "/VGTtYs41F133ZmP",
  "W1kpZdKGIitdz09/",
  "U8GUSuvst99+DFAU",
  "TGsWsxoShKD3CLAD",
  "07bZEAgbEEN7EEAL",
  "dg6XDFqUmMB85zvf",
  "pu222y6MydTz0qxq",
  "PUdhoMfw/1CAQA3w",
  "5ZZbJjwP9D3cZ52I",
  "aZtqlBHXSZEqLnC7",
  "DHwa0TqCXC9g5n+t",
  "tdZhlnHEiBbKZAR8",
  "FvI4H9Qcb6KmphZq",
  "bm7l/8Hel0oAbBKr",
  "DHCpGeHavgCOtR/j",
  "N5GEB+Zc+8xQSDfZ",
  "EzE8t7S/zWXbQAGM",
  "OMM8tmFlMaMZ2wLF",
  "bIKle/jhh2tA2kCY",
  "giAFTAqCYDoQK6hT",
  "hgcDG7v32GUq69ou",
  "aZgyXlgOOugAdtlV",
  "quKOzedEUBoDJVzf",
  "NpBT16cO8mA0ezpf",
  "dRF6ngq/CyhTeZYz",
  "zziL/nD1H9htqfvV",
  "Y7T/b9RmB+z1c+yr",
  "uVlkmg455BC66qqr",
  "qKOzjUaNGEkOZw/7",
  "PNjqeek1QujBu++8",
  "zwkp2EdbW0foZgdY",
  "Hmy5Kx309R6rBiNc",
  "5FBCeOLfj9K4ceNC",
  "xlOTuFR8H65fJAUd",
  "eOAPeHtMKKTSk9zT",
  "cnlwmVkweuoCVlYd",
  "YRoAV48//jitucYq",
  "YZEAXEsNC1Ah8oSb",
  "MFqRkkmN6z6idRRX",
  "s0Iy3JQpktSsYJXF",
  "z61Ep97aECpnVdF2",
  "zXCk26IQQneum/b8",
  "1jfp9tv+Ql1dOfYA",
  "dHbN4vPIpJvomWee",
  "ZcYbHg1sA23NNdZY",
  "gzbffHOub84AuCr6",
  "tWAmMWHDuXOYjSvt",
  "y6/K6y677MaeCJ1k",
  "IbRBxsjBZTTrQw/0",
  "PswlyIwr/8Q2rCzO",
  "Oo9tONu95qG5R6Mb",
  "QB4IAwMGwIFkI3RC",
  "puyZMjAwLWunblRl",
  "SBQAKTNULwVjs3Z4",
  "/fDDD2jNNddkWRxs",
  "l0mLcLTjgOmQ/dhC",
  "7jYLqJqdCiBt9lUA",
  "Bsr2NXEWrLob8Rtb",
  "bLElA/MZM2fWACg1",
  "O9atEZtdVrjsB3GN",
  "wtYAMP72d7/BVFdc",
  "roWilEE0DKpmBitw",
  "hpj76DGjyHET9Mgj",
  "Eq+KzF5xpUfs8GCZ",
  "zQ7DcJ3BrgLEgI3e",
  "aqstQjYTx6LlQnGN",
  "WQDcDzget6Ojkyse",
  "KSCFfI640Af7+LU4",
  "gMp0oZ0W6dhjj6ED",
  "Dvg++VWJW9RqOVGl",
  "Izlf1qF0MWnx2RUN",
  "9zna5GKLj6MNN9yA",
  "Hv/XEyHjrkUJbO3J",
  "3u4PsrK5zKhpmxpr",
  "DTWH9dZdj66++k+U",
  "THo0YmQzdXV1UHdX",
  "kbKZZrrwwgvotNNP",
  "pmee+S9r2n7yyScM",
  "FBHHCS3TKVOmMPu6",
  "+GLj+NxyOT12OU8u",
  "E8psNfF2l1xyaej+",
  "l/si/by/WeO9mfZJ",
  "HBMmLrjGfTTVyoxB",
  "ZmzDzmKgGdtwt5eB",
  "qYhoq0ZWhvQREmXA",
  "Lj322GP82UDKx9gu",
  "b9237N8xgBDl+DAo",
  "CUhSFrIevMGErRSX",
  "twDEBAtE77vvt5kp",
  "w/6EUQRwEfkkGwja",
  "A7idxKOuNwWfMM8T",
  "oIHjA/DBulJhKUvL",
  "Lrss/fWee8IBVgGw",
  "Ati+6JTqtf464MT1",
  "QBxfhlmmG2+6npkp",
  "ZWVLJi4PpkDd/s2M",
  "qday0UYb0dNPP0Uz",
  "Z85i4AHXqLC9NKim",
  "4EyPSxODAJgvuOAC",
  "TuQC8NRjVzeogi7X",
  "9Rg8bLLJpixbhRhA",
  "idOUWElMBAb3+JWd",
  "lvsJZnPdddflcpNI",
  "PgPO1UQ3jW/WEAAG",
  "j5UoTEQnJDgvCIev",
  "ttpqtOWWW9MDDzzA",
  "TC2ug8RFittdr9+c",
  "LOHKZCyVTIYx1vi9",
  "ZZdZlm699VZaYskl",
  "qFQGe502CVhJ2mef",
  "fenPf/4zh2FUygEz",
  "omh1YPexLY7ljddf",
  "p9tuvZX7z8Ybb8z9",
  "QONPAYylDCvY8yT9",
  "8pe/pBdflLhqLYqA",
  "Pij9bPAZTRwXNEwR",
  "StNHi0FmbMPa4hjN",
  "2OYHO4eIftXoytAv",
  "/N73vsfZsX3Ofm3A",
  "NP6xBvRxEo8rmbPG",
  "FYgEh3Q2w6+apQyz",
  "QZRWGcH7XK5An376",
  "Kf3pT3/igVq3sctO",
  "wpQZtePabIBnu+WV",
  "BdXrgAEc8Z7q4sT/",
  "SOo4/PDDObZVj8dm",
  "s/qTtW+7TkeObOWy",
  "jAgPQKwfTBNO7DhT",
  "3c6O1dTsboChP/zh",
  "D5yhLtdAkoMG2+za",
  "2FhwraDnCDYTxwgW",
  "SuMuNb4RryzWb0oe",
  "cnZ6Nst1sAHO9PyG",
  "Kg7QLrEIkI/yiqNG",
  "jwjBpLYPBZg6AZJy",
  "jQLmm5uaGKxlWee0",
  "g8aOGcMsIUDr3Xff",
  "zVWR7AQgTezqzTiB",
  "qlQKk3iwDSZA1157",
  "LbvCKVHhNtPV1c3u",
  "8rPOOpOeeeYpXm/q",
  "lJkM9BFzCqF5JPzg",
  "f0xAmP3s7mSNVoTV",
  "oIQtfmvWrFnGlS4G",
  "bVmENtgTK70WA6Fa",
  "0Zvhd9dZZx2aPHny",
  "3ILMiYNzZLHFNgA2",
  "r/WV4iVe+rD8cm7E",
  "5FpbWwdMz8/WPbS1",
  "9bxkNkilmwMvmWY9",
  "PdbUy6Sh0x0kXNG+",
  "1G10P/qZ7s91E6zN",
  "OWbsiOCDD98JOjra",
  "gnK5GHR1dQW5XFdQ",
  "rZaDSqUUFIv5IA/N",
  "wEIu/AxamdAKxALt",
  "QLy2t83kV9+vsLbj",
  "rJlTeT3oCkI7EBqC",
  "+e5c0N3ZFcyaNSvY",
  "eOONWd+zpaWFjwc6",
  "ifYx9+X6qD6mLsmk",
  "G4wdOzp4/PHHgq7u",
  "tmDGzClBuZIPCsWu",
  "oLNrFms1YoGGI16r",
  "lUKoTQo9ynyhk9eD",
  "3mFH58zg5luuZz1T",
  "6CoOhc4k7pWeE64R",
  "tCOvv/76oL29nbVP",
  "fb/Ex4pjx/84ZpwD",
  "v68W+f7hvnV0dATV",
  "ajV44oknWJ8RGoue",
  "N/g6pqlUhn8LfQHn",
  "cNppp/D1b++YEVT9",
  "Imtm4vhVPzPX3c7n",
  "gfdYoFeJaz9j5uSg",
  "VO4OiqWuoFzJBe0d",
  "0wPfLwZTpkxhHdiP",
  "P/44+MY3vsG/iXbU",
  "sEZoAmGsiWDEiBH8",
  "fsMNNwzefPPNoKuj",
  "M/j4w4+CXL5d7n9b",
  "W3DdddcFzS3pIJWW",
  "/ue6ySDpegF4WXtx",
  "Ew4vCasNr7ji8sHD",
  "Dz/IfQL9i8+x0Bnc",
  "dNMN3EZxvNAIVW3Z",
  "dBpaoYOv03vSSSfN",
  "zaPtviAIlhgGz+V4",
  "iRea0xInA8U2v9kv",
  "ieiUvm4EtgDl8zT2",
  "z66RXB+IPy9M2UoY",
  "jgcJFpDDwfFxfKBT",
  "ZRYN77XCCWLUoDEJ",
  "hgquRLAyDz74ICd3",
  "oFwgWFFUD0KN8299",
  "a0/aYIMNJPHIE9ZN",
  "qw0x6+lm6auvvqJd",
  "d92VkypsNhLxkY4r",
  "rkT9XJ8bEYMqTJ9K",
  "+4At0gpFWJZaakn6",
  "3VVX0hZbbB6ytCJN",
  "lGBpGs+tdbXbrzA9",
  "Z2XHsA9Ub/nVr37F",
  "n+XyRneRN5Hj601y",
  "yTZdV5OnbBexfgaD",
  "Wxgm9+c3YSyuX4U8",
  "Ea6FMLJEeq1wvMJ+",
  "aylCHA+Y3DvvvJPL",
  "HCKOM1+QKjv6vV7j",
  "+mttn099QlnUpq2K",
  "OAlkY2coX+ikbDbN",
  "62251ebsjkbogbiI",
  "y1QpSVymVrTCvjRD",
  "Xo9DM/2VhVXpLqzf",
  "OrLVJK+IBixCsq69",
  "5nreH+4dyraGx2TF",
  "Fet5SGUll48VzOOl",
  "l17KYTDahts7uph9",
  "R5nWzTbbjF32YJG5",
  "6pW1rzkZzhVJQDie",
  "0047jU499VQ+dpwn",
  "dGxfe+21MLlMQyBs",
  "Tc+BMvWG6HW89957",
  "ac899+zrbmImM7b5",
  "x+Y10o2XeJmL5fi5",
  "mf6fe+65IUMH1kKZ",
  "qqFgxBphAu33I0eO",
  "DJ577rkgl8txRRYw",
  "f2BelA0EI4MqJt3d",
  "ncHkyRODiy66KFh5",
  "5ZUNC5MO9wnWDK+o",
  "cnLwwQcFX3zxmWzb",
  "3cZs1pdffcr/d7Z3",
  "BOViKfjfSy8HS09Y",
  "KnAoEYxsHcFsUDqZ",
  "CpJgat0ksztg4MCQ",
  "CdPj8P+4nvhdLMoG",
  "gTkbPXo0nwtYJFQn",
  "amufFnR2zWQ2DK/F",
  "UjeztmAv57SAkW1v",
  "mx7MnDGF2TYs+H//",
  "/b4dIDJBWDuPWWFg",
  "LWVS+8LGalvQKjR6",
  "LXFu+A4ML/7fZptt",
  "+JqjMk2pVAja2mYG",
  "xYIwr2CTZcH/qGwk",
  "VWqUYQaTXK2AKezi",
  "Ski33nJT4PExC5OG",
  "39R7psyanodWp1Im",
  "Xc8R6+IVxwq2VY4d",
  "63h83/A/vsO1QUWo",
  "jz5+j9tQd66Nmcmu",
  "7hlBZ8fMsBoTmFiw",
  "m2A2uSpQVxsveI/1",
  "wHDiezDkuAf4Hywz",
  "KutMmz5J2OpyMfjr",
  "X+8K1lhjtfCa4vrZ",
  "jL7NELsOBeuvt05w",
  "91138PXCtUIVH632",
  "M336dGaOf/CDH/B+",
  "7IpWjd5jvW663e67",
  "7x58/vnnwSuvvGKu",
  "Ty1zaVegGog+rv1S",
  "7y/2PZd2bxAE44fB",
  "czhe4oUaWWJGM7aF",
  "RvoIBqYPsV8a8D9U",
  "lWX6WnkGC7QXzz//",
  "fGFrEhVmW7i+czJF",
  "U6dOZ8YH2fWItXv5",
  "5VfCeDKRMYokeYRB",
  "FEZq0003pWuuuYbG",
  "L7FY+NscH+mmOHkC",
  "bM4777xDJ5xwAr36",
  "6quRXqgvx6ismc1q",
  "4Xehx4gYU7xCEiad",
  "FgZ0+eWXZwkjsGjC",
  "EAlzC0YKv4tkJI6J",
  "4wQqMX0m9fRsAsOF",
  "8wazpawo2MWnnv4v",
  "M3Yq4J5KZUJh9UZN",
  "kqNkO71+LIbvunys",
  "uH5INLvppptYP1IE",
  "wIWldBK+xWjquQQh",
  "o6n3Va8bjgu/AzYa",
  "+zv1tDP4WuIewKQq",
  "jeg+avuolwqqqXyj",
  "cY5GlgfHoWwsfru1",
  "VZK+fve739GGG61v",
  "WGeR3sL2mVQm3L9K",
  "L2EBY8jMLgUhG6i/",
  "wQOIOS4cKxbE4Hqu",
  "R9OmzeBzw/aaAY74",
  "SMQg6za4dksvvTQr",
  "LRxy8MG0xRZbhAlE",
  "2tZUVxOVfZ5//nlm",
  "/vAd2E18rsffiKnc",
  "FPaJY8PrhhtuyMeM",
  "RCaNeR4M74beKxwD",
  "rhOYcbThuVTiAJPZ",
  "52DO2GKbVxYDzdgW",
  "OrAJwwPfriozFAH/",
  "czI7+UXBDVyGyJxH",
  "4kY6EwmEJz3JCkaW",
  "7K9//WvWYlSAaYu9",
  "6/paaQUACIMq5F7g",
  "tl1ywnheD4N2Opnh",
  "ko9TJk/mjP2ZM2fS",
  "EUccQfff/w9qaspQ",
  "Lh9Vw9EB3hZhh06n",
  "yuIAkEDncsstN+dk",
  "DuyvUi3S2DFjKV/I",
  "h9uo+5Dd6yYvsf55",
  "FJ5HKsVlNvEevw0g",
  "Ji7ZIgvdH3XM0fTf",
  "Z58PE5lwO2vdyY09",
  "5zSJx3YNaxY0kn9u",
  "vPFGrkyDz7SMaKlc",
  "oEwq3aPCgb53PTl+",
  "df1z6ENTE7XNmsXX",
  "6+lnnuWELEyEbMAY",
  "lmK09qvnUv+ZrT2q",
  "26nM0NJLT6BbbrmF",
  "3c6dXe18jqVSVIEn",
  "nZTEl5KVkKPXj93p",
  "nN0v1XNwvFgPmfOi",
  "i9rM/Und3DZQEw1X",
  "qeKENoUEPYBPbI/t",
  "ENaBbaCfqiBWJxC6",
  "rQJNiOKjP+D6oK1r",
  "4lgjWrl6LrbkmIJl",
  "LPp9T+1kIMZIVXPA",
  "+SE8BaVK58LuMCBz",
  "Vr8PKLbYhtBioBnb",
  "Qgs2UUkI8YzDoQ/Y",
  "VUBUwB0D+8knn8wV",
  "UbwkUblU5cEcgzvK",
  "Xd5+u9SmVlmaSPsv",
  "2p9d7zyqEe7SAQcc",
  "QJdddhn/RuuIZirm",
  "hZlraW2lmTNmcPwb",
  "BuSLLrqIs7xLZcnG",
  "BQDQ47RZLRwHAKmC",
  "zdNOO52OP/7HIRhz",
  "kYTvOCxLBIChYA66",
  "mswimZrfswNqGi+n",
  "4vf4XWXMsM7Mtllc",
  "o/qee+6n5iaRQkIF",
  "GWVYezPdtwIQzSDH",
  "seJzMLOIydxkk01C",
  "oNvVLawsx/0aRnZ2",
  "xw9pJ3mV48f1BVjC",
  "9lwNJpWht956i8/h",
  "2WefDQXTlbGrl5iq",
  "VxlQwKTXn8sqGgYN",
  "rOFVV/2W1t9gXT6v",
  "5qZmFkHHb8gEK6By",
  "sRxmyWt8orKh3J6S",
  "wn6icg5Khr733nsM",
  "HBV44vqgP33729/m",
  "iZEqISgo1RhbGM4d",
  "+9cYYXw3wkhDaYED",
  "ldpSVvbJp57hfevE",
  "QUtL9jW+GtuqEoDu",
  "W8/TnmDMrqJVX6yn",
  "iQAmXocddlif90VE",
  "NxiQOTQ1V2OLbSBt",
  "Xvvu4yVeBmA5KgiC",
  "6twEOz3//PNfi82a",
  "VzGaiMnDq2Z747gW",
  "WWSR4IMPPgimTJnE",
  "mebTp08NdtttF44z",
  "GzVqVJjljTg3xK1p",
  "nJ/GmOE7rGvHH2Id",
  "xOtdc80fORMa8XSI",
  "F+zsQCxeR1DhmMk8",
  "v0c856OPPBRssskm",
  "QXNzM+/TzuLHsUoc",
  "I2LtUsF3vvPt4K23",
  "3uAsa8QuIrMXx414",
  "QMSYTp8xOZg2bQpn",
  "zCPjF3GbiBNEbKAu",
  "Gpepmeiaua1Z6IgV",
  "7GifwQu+s7Oif/ij",
  "o4JsE+JFcS2j2M3e",
  "rr9eF41xxPXCeeEa",
  "r7feenxOnZ3tfD4c",
  "l9k+neMREd8qWeY5",
  "c+zR8eMayvF38zVF",
  "XOaM6VMl5nDaFP4e",
  "sYi49sj6Rzb6tGnT",
  "gqOPPpp/H9cbrxqD",
  "qbGCes81zk/jNvWe",
  "SzuiYMSIluDoo48M",
  "Jk78kq/PrLZp/Irj",
  "xnVH3C9nnHe3hXGv",
  "iM/UmMxKOc+f4Zq/",
  "//77wU477cS/YceO",
  "amwr3mNZd911g5NP",
  "Pjl46aUX+B6jvWrG",
  "PWKKce8R24rP8B7X",
  "dNasGcGUyRO57bXN",
  "mhGqJWhcK64Xflt/",
  "T+M69f9G+xfup30t",
  "Nf4Wy+ziMec2PrP+",
  "HiFOuR/2y2HwjI2X",
  "eKG5XWJGM7YFxdYk",
  "orOJaP+52VgZrXll",
  "6u7GEpWNlAx5ZCb/",
  "+te/pDfffJNOPPFE",
  "euKJJ7iWM1zeyn5i",
  "XX2vGcIa66ZxhLb7",
  "EFVhwEIhSx0i0Zl0",
  "kmPz1H2v4uTYJ5dI",
  "rPjMuD366KP01FNP",
  "sQuU2bHmZmaott5m",
  "S9p///1p1VVX5c/x",
  "m9A9fOONN5g1RkYv",
  "9j9l8jQaPXosqwBs",
  "u+3WtMGG6/H2doxm",
  "T4broKyYXXUGrCDY",
  "ReiW4lhxzGDcLv/1",
  "bzh7HtdSSicmGoqR",
  "1XPS2tcok4kMZWiA",
  "KnvJbJ8r7KS6Q92E",
  "hCZEJJbqgQr2CIKI",
  "/dXz0apBHHc6agyf",
  "i+ptQrgfmduvvPJK",
  "yK7art3wAW7iCvG/",
  "3nsc5ze+sSm3G7CM",
  "fJzJKE4U63HlqYzE",
  "ZXLJSQ9xtkmOG+Tr",
  "aa4FtnnxxRfpkMOP",
  "Yo1HbZsQR/esPoPT",
  "1IpOuA8QUQc7e9xx",
  "x4leZUKYQWU1lUnU",
  "6kkOSTtG7KK61rUO",
  "ORjeb+397ZD9VGZe",
  "y702Gqep10dd89rO",
  "8Xt2GVO77KodU9sX",
  "sxnnX/ziF9yG5sKm",
  "GhbznrnZOLbYhovF",
  "QDO2Bc0uIKKfzM2G",
  "EHVGlZd5bRonqGLX",
  "cH/uv/936emnn+Zk",
  "CsRkKsixReMVSNoS",
  "Tvagqa5SdT8ieWaP",
  "PfbguEPUgtbfhtlV",
  "jVjYPS/HojWYERcJ",
  "MABgAEAwYmQLAyat",
  "uvK///2Pq8489NBD",
  "1NWFJKY0lYqo+y1u",
  "TmyD41xzzdXppJNP",
  "oL33+lZ4/jaYsg3v",
  "FdhpHKPGpCbT4oLG",
  "MQEoofTgb6+8il2V",
  "ABK2vE5PpmLx+A2c",
  "AyoQnXLKKbTzzjsb",
  "oCHubj6mhF8Devla",
  "l/05Ak14d+vvC45V",
  "pX78QNyzGqOosZaY",
  "CNxwww08udC4WAWZ",
  "eh91HyitiISaww47",
  "hLbaais5NhKQpABL",
  "xdS13jruA1/bqqnQ",
  "VCqFIvqu59F/nnmG",
  "YyOnzWyX0AkDxgAy",
  "7ZKqgfFe29rsiNtF",
  "sg1CNJCABACqwvxo",
  "QzhXBacI3VBxfm33",
  "Gp+511570b+feCoM",
  "EbET5+r7wJz6lCau",
  "2X1A41LtYghy32rl",
  "l+YGaCKEAP1kLi2W",
  "L4ptgbEYaMa2INqu",
  "RPQHIlpmbjbWGC4d",
  "YGwtPTuBwB7wBtt6",
  "K6PZWz+2tQttcAL2",
  "DmX8ttliszAOEqbx",
  "anpugRdl4+LacDJH",
  "U2sIiru78/yKzGIw",
  "cWAVlZGqASTWceox",
  "4PXII4+k3/zmck5U",
  "aW0VvVC5tlVCdUZn",
  "Duev56WAQ+MbsV8k",
  "rCDx6a93/42PDesi",
  "413XK5aKnMWfaUoz",
  "wEIGNGLoUFpSz41r",
  "lVOZEr5LicBhoOkj",
  "yzwBl1CCP3PJ5c8C",
  "zj4ncv+/vTOPkaM8",
  "8/Brj2/A4ANwcAhR",
  "yNoriO0IghU5QICE",
  "LIRLHAZxBSIwqyzI",
  "JgQMhAUt/BGwOQJE",
  "RATILmsgLJsAARay",
  "OSDmWATCLBibLGBI",
  "wBjM4WN8zj21+n1V",
  "b883xXhm7O5y98w8",
  "j/Spe6qrq746pvvX",
  "79meCRdf1t7z9XNr",
  "XSy23NL+xpvLQlyk",
  "jsfjI0PLxmHDQkLN",
  "1KlfCRnxslK7ldJf",
  "F60tTZ3iOX2fvqwt",
  "6WxxlDhvbGgOiSva",
  "n8eYdncN8vVP48d5",
  "8+bZWWedVRKQetR1",
  "0I+CdKXUQmlJaoHW",
  "vHVNn3vueTv++ONL",
  "Vvlq4fGi+ftX+OeB",
  "8PtZNV4VW72NkFkO",
  "/QqEJvRX9sjE5lZX",
  "QhYqo6LMUHdlxwWW",
  "8y7Q7UElhGa+dItb",
  "B+XGXvin3wcL2HC5",
  "EBvSzHAXbCEb2NSn",
  "fGQ4D8FtOTh1ter1",
  "1NI4LGTAy3olASHr",
  "micubSnzOxa8msvZ",
  "Z3/Pbr7lpsxKKTdp",
  "VurJ6mxoVNC+q+N2",
  "q1RIrBk6tGSZ1Jz1",
  "fO2adbZs2bJgEXah",
  "5vvXXPedsk9w+yuM",
  "QFZat256Uk0QYJmI",
  "THecme6yZUGAlnF9",
  "POnE3f8SYG7VDiV5",
  "WlqCEPYfQXHSki+L",
  "r7PwDPxwrUNLxi0L",
  "TZUvksDz7HH9iJDb",
  "XSWF0gSm3t3n+SQl",
  "R8ei0Aq5kfXjxv+v",
  "3KqphgQhRKM5tcj7",
  "PXP88SeG0ItqV4VI",
  "+9XXhfAGvy+8fJY/",
  "1/EoXKQMK6a4JxOZ",
  "myo3e4DqgtCE/sy4",
  "TGzO3NYNeO09z7gW",
  "eXd0ta0tW2uRcauS",
  "z1tf6NdcfZVdcsnF",
  "JTGq4/J1JRwH1aVu",
  "c4mQ4DLfceeSlVfd",
  "iWbPvjC4yYXHRXoG",
  "swumrnChmbox64K7",
  "+vIfXxpe8zI7w4aO",
  "kCoqvSf+zIpFkx+T",
  "W0NjMdbenopO+bab",
  "M1HgsYpCpYY2bpKL",
  "vSkIz7rBdcHaKYJ1",
  "rVnastUSucKTQSqH",
  "HsSmrJWJRFJm2QzL",
  "JfIGe+xiuqwt6xS0",
  "JRSiGocD+I8AXTMJ",
  "39bW9Fp5HKfOi7t7",
  "82WQYgHv5yZpb+1W",
  "aDa1pFnYaUjFSLvx",
  "hp8Gq1xHfOi2xSg6",
  "Lh4POeSQEKrhXX/0",
  "GKzRidcjTY8ptWY+",
  "ZyeddHKnjPVq4dnq",
  "XhrJrej6fPAfU6rQ",
  "oG5DZSCBeXvlZg1Q",
  "GyA0YSBwoZn9dJvf",
  "fOGFdsstt3TpQu+u",
  "/l4tkbcoumvcRecu",
  "O+9kf/zj7+3LX/5y",
  "eC3U1sy+WBVj2Jq5",
  "moVc2W5JUyzkaaed",
  "EZJ9ZPVR3J+7pT35",
  "JO9yzM8rnpOE4oIF",
  "d9vpZ5zaqQ2gnNOi",
  "Kxd83jqWj39MxVJq",
  "eXIrnydMeRH8wUPS",
  "RBG3hPq2vA1je4ua",
  "1qdu8DDj0DspE5qy",
  "aobMylRUBrd6SD1u",
  "77XQTLI2lX4sXiPS",
  "wxSGDU9/APiPnNhy",
  "Ga5TQ3OXArN0r/Yg",
  "NOU699jJN/7vrVJs",
  "ahpmsLV322evr89N",
  "+1Dc5s9//nObMGFC",
  "eE3CfnPD+rDv0Tvt",
  "Uoo/VWzowoXPlIqs",
  "1wJxmSlPLtIPAd3/",
  "qk+7jTyXicyllZwr",
  "QK3Qvb8HoH9ws5kd",
  "YGaLtunNN98cvlxU",
  "7Fx4HJ2Lkr6ECwtP",
  "rPDuK6tWrbIbbkiL",
  "v0tcpLUMU5FZch+3",
  "WXCZ+7EvXrzEjjnm",
  "uJBZri9gxfXFHZd8",
  "+92J8Dje1YugX3HF",
  "lbbsrXesqTG1prk1",
  "tDux6vVE3dXs2cie",
  "Xax5SFS669Mzjj1e",
  "0Ndxi5UsbZ7dHmou",
  "ZhbKwYq9TLL4y2iZ",
  "x2S6JVN0tWxrr5UL",
  "ShfQbu10IS+roObs",
  "STWxkPSKBL1B69Wv",
  "XR+aAVx99dWh3qmK",
  "7m+LyPR7LF9H0hPM",
  "XnjhBTvppJPsr3/9",
  "aynb28W/jkfXUYXZ",
  "//znP4fXakFk+nVw",
  "ke/HowTC1atXlyMy",
  "5So/ApEJ/RksmjCQ",
  "2C1zpZ+wrRtQy8cZ",
  "M2aUvnC6s9bVErG7",
  "L1/kOggUS4/noYce",
  "soMPPjBbP7Uy6ou/",
  "obGlVHpGwnPRov+1",
  "8847L5QQcktYHAMa",
  "fyn3xu05bJjc3h2C",
  "TMk4v/3tQ+Hc7rDj",
  "yCyre8t4DK1w4eJx",
  "dakVM7VEa3teJD3O",
  "sh40OC3r5MJY75NV",
  "V89DMlDSaoOSIVks",
  "Zru1D86yzINlc4js",
  "kWUlAw2uS4Wkx2jq",
  "+dq1a+2TTz4JxzZm",
  "zM6hw1JcCSBfWDxu",
  "3ejn3l8brMSl7iya",
  "7WmlAbWLPPPMM62u",
  "Ls3OditzT9ewpxhi",
  "35+vq7HffvvZr371",
  "qxDXmFhaSWDTxoZw",
  "vk8++eQQH6p7wtug",
  "1gJeWmny5Mn2xhtv",
  "lLs5XOUwIEBowkDk",
  "R2Z2QzkbmD9/vl16",
  "6aW9FlK1QN71H3d+",
  "8XI0hx56qD3wn/eH",
  "19StpX5dfSjtM2rk",
  "zsEaqOUqXaSYTCXV",
  "CHczx2Ig3n7c9ag7",
  "vIuMMpzlFVZHotNP",
  "PzXUgFT5nFjM5DOb",
  "YzexZ8tLuOhvT25x",
  "i5knDelRVjW5PevX",
  "rwvCTu9VP/jp06eX",
  "6kymVsH2nAPIr3m6",
  "TEK9O3r6nJXGHjly",
  "hC1d+ro98MADoZyR",
  "whIkPDXXXcfvEgSY",
  "yi6pq5Pm56I4jeFs",
  "LVkM3YoZ96VX+apu",
  "k4GSQSHW9rDDDrO3",
  "3347lIPS9Yg7QZUj",
  "NF3E65y6VVkJT7IE",
  "LliwwCZN/lKa1NTU",
  "GmIzTz311Kx7UIPV",
  "Cl72SdekTO7ORGYa",
  "IAzQz0FowkBlfGbd",
  "PKmcjdRSG8vucEHs",
  "SUEuDHxZR81As4d/",
  "+xs76qijrL09bRM4",
  "YsQwa20ZFESCann+",
  "4AfnB0umhIG+dD1h",
  "Ko5/zLtNe86K11zC",
  "s1LZpXHjxthTT/3J",
  "9pg4IVgE8xnN8T5c",
  "dMVxs57UFIRlY9o7",
  "XMJJoQH33XdfsN5K",
  "aEpgKUYxLti+5557",
  "BtGtgu2KKWxtbrNk",
  "kJKBUsE5uF0CsyNG",
  "M1gMy0gGampJghv2",
  "9ttvD1n/w7JEKvUA",
  "10XxrHEXkQcddJBd",
  "dtll9s1vfjOLo23v",
  "1J88H0ssIdyTRVPC",
  "/vLLLw/F7ePyXttS",
  "R7K7+8/d/15zdeLE",
  "ifbE7x61MWPGhMQv",
  "VXzwbHfVXe3tD5Ui",
  "0RzU9lUJUmXQlgnM",
  "Oys3M4A+gH84MxgD",
  "dPwwKZPly5cn48aN",
  "69TOcEut67xl4La2",
  "titqDB2q9nx1YW5q",
  "N7ly5QfJ+g1rQovC",
  "DRvXhhaJS5YsSaZO",
  "nRraTqrdoI5Fbfwq",
  "sX9voahz56009fzE",
  "E09M1q1bF9pOqtVk",
  "2uZRrR0bk5am5jCa",
  "GhrDcrWkVLtEb1+p",
  "FoZaV20N1dpx/fr1",
  "ya233ppMnDgx7Mtb",
  "EMbz8JaGo0aNCo9q",
  "HThnzpy0jWNrc2ib",
  "qEe1UFT7RLXv1HO1",
  "pfShdos+fFlDQ1PS",
  "2tqabNq0Iazf1taS",
  "NDU1JRs3bk5efWVp",
  "OK86p96KNG5lGLcW",
  "1fx0/r216Lnnnpus",
  "Xr06zMtbOjY1bwrX",
  "TNdPrSY1Nm5MWz5q",
  "vmoFqqG5aH0tX7Fi",
  "RbLnnnuW9unnwdso",
  "Fn3/TZs2LbRanTt3",
  "bjgutZj081HJ+8uP",
  "Kf/cjzffhlJ/f/vb",
  "304qwH8kSTKmBj7v",
  "GAzb3qPqE2AwamB8",
  "JUmSZ8v9Jnn00UdL",
  "X1jeQzkWnt4/2/uO",
  "57/0qjkkMs06vmTn",
  "zbs2CCUJEvXHXrly",
  "ZXLQQQcFAeAiTL24",
  "ixACcd9xna8HH3ww",
  "CEf14Vbfa4lM9WHX",
  "Y+PmhmTTho4+6RKa",
  "qehcG/q0q1d2W2tz",
  "smrVquTII48M2x89",
  "enTpGkjM6NH7y8c/",
  "Alzs6vGAA/ZPFi9+",
  "JYgyCUcJRfVw14iF",
  "pfp750f6Wio0JTK1",
  "TOKwsbExefHFl5K9",
  "9/67kniMhU4sMP01",
  "P0d6TXOTEN5vv/2S",
  "V155OcxlzZpV4bgl",
  "MnXdJDL1uGnTpiAs",
  "NfS3xKgL5IaGhuTK",
  "K68s/Wjw8+737fa4",
  "R3VsEyZMKB2bH2va",
  "t70y+9fx+Hbj48of",
  "o+4JzWH8+PFJhZhd",
  "A59xDIZVa+A6B+hg",
  "tpndUu5GVOhd7j8v",
  "fyI3ZsmFGbkj43aP",
  "1cezatMyOOPHjw09",
  "zffYYw9ra2+xH154",
  "SYilczeml+DxQumV",
  "OAZ3q8aufG17//33",
  "tz89+d+hpJDc9166",
  "yNdrbGy2ESPTjGuv",
  "pSlXuc674gAVSzpr",
  "1j+GXu2KD/T5eotD",
  "re/7dXe8u+K9XaPW",
  "UTH3Rx55xPbdd9+w",
  "jtcHTbPyt+waj+NI",
  "FXOYur8HhwLy3/ve",
  "GSEm0rsC+Qezrx8/",
  "xoXo44QgnQu1E1XL",
  "z5kzZ6aF13cYEYrS",
  "yx0d7jdT96YN4fmO",
  "O41K5z1sZDiuTz9d",
  "bd/4xjdKhca9VFSc",
  "TFT094RfS++EpP15",
  "IfpK7DvuSBS3LvWY",
  "Xv+/9PqkulcUPlEm",
  "xGICUN4IoBO3qnZ3",
  "9gWxzRx99NHhy/mC",
  "Cy4ofWG70PQvbW+X",
  "WCuJRF5Cx8XW2rXr",
  "QryeYjRVvPuee+4p",
  "xWK6UPYkoEocQ15k",
  "SkS5mHr55Zftrjv/",
  "tbQvFwUd57AjCSYk",
  "aqglZF1dEJlLly4N",
  "Rb8XLVpUKhHksYF6",
  "j4TnlkScixAvr1Rf",
  "Xx/K8rz66qul9prC",
  "z0dcjigeQWgOSot6",
  "q1/6oEFph5nzzz8/",
  "xIhKJPr+46x337aX",
  "KdL+XNDH5bU0PyXv",
  "nHPOOaG3u0ozbd7U",
  "GES54k/T85YKayVW",
  "Sbx5P3V1dLrzzjtD",
  "eSs/3vi4ttf9GZdt",
  "El7zVHPwclnloOPQ",
  "cXtFgvhYPVtfy9Su",
  "VDG8ZYpM/eo4z8y+",
  "j8gEIBkIYEucnSUL",
  "jSh3Q7LIKbO5I+Gm",
  "w6rpFqPqk4ozF21u",
  "TVJbxr/97W9hDW+5",
  "F1sc3SpbiWMI2dFZ",
  "wogLc81BgnHChN1C",
  "0pWKe0tAeatJL2vU",
  "3NJoQ+pScepZzR9/",
  "/GnoW66SVC4sNHRs",
  "XndTxJYtx4/HLaex",
  "ZVr90O+9997QW9zL",
  "JMnzmk9Wij9b25PW",
  "IICVUT906PDQ9/vX",
  "v/51KOuk+fo5je8N",
  "7w7kc4wLtnd17pXE",
  "pGVXXHG5zZkzp1SK",
  "auiwulADNXQeas3q",
  "jA4aEkSXflB8/esz",
  "bMOGdaX2or4/zSNf",
  "maAoPNEpbiMaF9Yv",
  "Fz9vLij9PhPajxKx",
  "JNQrYMXUj9U5ZU8Y",
  "oD9Rbd89g1Hj4+JK",
  "BWop4cHj7jwWr1Zi",
  "NBWfWVc3NMQv+pw0",
  "R404MUSxhHGsoC+v",
  "xBwUGxgnGOm5Jx3p",
  "nF1yyY9CbKQnsCjG",
  "UIk5La0NSX39mmT9",
  "+voQg6iYyHfffTfE",
  "Lmo7iin1Y/DHLcX/",
  "xfGZfpxxTK3HRs6Y",
  "MSPEPSrOsrk5TQ5S",
  "jKTHZMaJQVqmJBzN",
  "Vev+7Gc/S0aO3CGc",
  "b+0vTkqK9xXPNY5Z",
  "9NjJ+H2ewON/33LL",
  "T8O+V636JMxj46b6",
  "kCCkR8Vt6lxpXtde",
  "e20yYsSo0r51DeJz",
  "4TGsRY98Uk6l7y0/",
  "f35P+/P58+dX6t97",
  "TZIkp9TA5xWDYbU2",
  "sGgC9MyOZna5mf24",
  "EhubNm1aiAGLLWjV",
  "piOOsMMa5zGDaVxb",
  "hwUxLk8jF7K7n8vB",
  "yy7p0S1YnWp9DjLb",
  "ZcyYUptMt0LW1aW1",
  "MYcPV8SDBbenXlMJ",
  "JsVTyqLXsHmzDR3S",
  "uc6nW9DiuMC47FPa",
  "ez11Z6cxoWl8n/72",
  "ec6aNSu0JpW7fujQ",
  "zq5zxz9o9X7Nc8WK",
  "5aG948qVH2dW1CRY",
  "aGVJjksK+Rz0PK5R",
  "6nGZ7uLPWz99/zov",
  "KoZ+xBFHhP16XKbW",
  "93qV6vzzne8cEWJF",
  "tb7v16+5WxR9WZHk",
  "C/y7RbWS1v7Qsz6z",
  "CquMlKzdFeKHWfcx",
  "AOgCYjQBemajmV1R",
  "ThvLGLnRJRRUGNxj",
  "I6tNhyBKxZSLrDj5",
  "J+5Eo2WxKCyX2E2q",
  "fbmgEu7ilKC79tp5",
  "pY4/w4enPdkVbzlq",
  "5KjwPrmPL7ro4pCQ",
  "pULmEpmqSSm8c5C2",
  "7XPXMXpikJ8HF71x",
  "qICLOxdAEmuKh1Tt",
  "ydGjd+xU2zPuOV6q",
  "U9mWhPhM1WJUPKSK",
  "47uIk+iLhaOI35/2",
  "au9IUopDMPLXzuc5",
  "bNiI0Lnp6aeftnHj",
  "xpVEmycv6Rw8/PDD",
  "ISzCRWX+PoyFbNH4",
  "tc5f80rh1/vKK68M",
  "gr5CItNjuhGZAN1R",
  "bZMqg9EHx7lJkrRU",
  "yud2+OGHd3JHx+7q",
  "7tyMXbka++vQccqN",
  "K9fwE0/8V3D7yn2+",
  "Zu0noVRPfX19cEvP",
  "mzcvuIC1rq9fqTm4",
  "29XLVGlOY8eOTRYv",
  "XhxKBLW0tJSGlzPS",
  "o9zrev2+++4rHYu7",
  "pfNhCOWcn/j+8G1/",
  "9atfTd5+++3gNv9w",
  "5fKkvb05WZvVFVUo",
  "h9zmcuFX+/r25tz3",
  "5FLvKgzCr9P111+f",
  "VJCXkiT5Wg18DjEY",
  "1hdG1SfAYPThcUkl",
  "v73OOeecz8ToxTU4",
  "fXkcN6cv0kqJlVoe",
  "LiJ07Iq9lFj66KMP",
  "Q9yjYiMlnJ555plk",
  "1113DefEC59XKsYv",
  "rmMZXwstO+yww5KW",
  "ttZkU8Pm8NjU0py0",
  "tbcnGzdvSlrb25J1",
  "G9aHovOTJ08u1SB1",
  "wdqViCr3HLnQ9OPX",
  "/N5b/k4Q5BLnEuk3",
  "3nhjp0L91b6+vRm6",
  "rn6/e6xs/LqWe6F3",
  "/a1z/ctf/rKS/6It",
  "2Y/Man/uMBjWl0bV",
  "J8Bg9PGxU5Ikcyv5",
  "bfbYY4+FQtyxJc8t",
  "M7GgyCet9OeRL7B9",
  "3XU/KSUDKQHovffe",
  "C9Y7FyNu1azkuXFh",
  "GItYt5jOu35+J7Ep",
  "cSmRKcGpcdlll5W6",
  "HsXbqqTFNd/NxwWX",
  "ls+e809BnK9e/Wny",
  "wQfvJ3vttVf2PiVa",
  "De8T1z8uph9fVyWo",
  "eZKU/tb/jrp1VZgL",
  "auCzhsGwvjiqPgEG",
  "o5+M0UmS/Fulv90O",
  "Pvjgz3SMyX/RDgSh",
  "GVsA9aiWn0uXLk02",
  "b94cXOYXXHBBeM2z",
  "pr0LjARIbIEsZ8Tb",
  "cauplknoDBkxPHll",
  "yWtJY2tLsrm5Kflk",
  "zeqkub0t+XTtmmTx",
  "60vDOt59KC8uK3H9",
  "Yguf3yt+7H5O5s+/",
  "LohytXnUuumPF/1Q",
  "qcz5qeaPDx2zWmgW",
  "IDBVdWJIDXy+MBjW",
  "V0fVJ8Bg9LNxZJIk",
  "71b6205u9bgsUleu",
  "w/48PITAj1nWOrmE",
  "P/roo+Tuu+8Oy90t",
  "7S0EYwtoufuP3eZ5",
  "y3K4LsOGJkcec3QQ",
  "mWvWrwuCc92mjUlT",
  "W2ty7AnHd9qW5ufH",
  "U0nR5eWNXBDH89Q+",
  "1VLxtNNOC61DNXS+",
  "VGapLwjNOAwib93+",
  "1re+lRTALVlr2mp/",
  "njAY1tcH5Y0AiuHI",
  "rOD7XpXe8KGHHmoL",
  "Fy78TFHw/o4yyj0z",
  "XW0nlSU9efJk+/jj",
  "j23z5o2hjI9nK3v2",
  "cqUK4sfld7zNZVw8",
  "fVBdWsDjF7/4hZ15",
  "5pml8kl/+MMf7IQT",
  "TrC2ljSr3MsUaRve",
  "CalSmfvavjLKtT3P",
  "kI+rBOi10J5y1KiQ",
  "rZ8WrrdsTrXQNKB7",
  "4vtdmfRPPPGETZ8+",
  "vdK7+YmZzVOlrEpv",
  "GGCggtAE6KOC87HH",
  "HguiRm0G+zv5up1q",
  "4+giLX0tFZ4u5CSi",
  "RKktZZnkO/LE4jX8",
  "bWl5oYkTJ9qTTz4Z",
  "hJDKMB1zzDH21JNP",
  "2uBBqRCNyxR1qhNa",
  "Jvn6m3F5pNSqMKjU",
  "2lMi2WtKtrV11Bat",
  "dXQtTz755FAftADm",
  "mtntZrahiI0DDGQQ",
  "mgDbh8MywTmp0ht+",
  "//33Q3/1JUuW9GsL",
  "p4umurqhHbU3LesJ",
  "bi2ReErrTLqoqgRx",
  "HVH/O+w3q0/pyyVw",
  "JS7vuOOO8EPg7LPP",
  "LonJooufuxjPP0/3",
  "27mQvOacnp/GilpV",
  "K43mPmXKFHv88cft",
  "85//fBG7UC3Mf0Zg",
  "AhQHQhNg+/K1THDq",
  "seKor7f6aL/55psl",
  "cRRiZDJXbiw+UtHW",
  "0YHFLWHCl0HvUD92",
  "9RaXFVNW1UmTJtkH",
  "H3wQnleic1JfwAvb",
  "+3eKF//3+y+2uPpz",
  "72+fX0+W67lz51aq",
  "//iWXOTXITABigeh",
  "CVAdxmeC86Qid+Lt",
  "LuNYPeGuVf3/u0vV",
  "xWW+NeZAiwXdFvLn",
  "KLamDoTzF7fLlOD0",
  "EAZv8xmv53979ynv",
  "fKRzJnF5zTXXFDnV",
  "ixVKm3X7AoDtAEIT",
  "oLqMNrMzzOy2ond0",
  "yimnhP7f3ubRk1Pi",
  "zwB370oguIUq7hEO",
  "XeO90b3fuIt3P3cD",
  "4XPWE7HyP1Ri62be",
  "Ta+/zz333NB7vGB+",
  "YGb/bmYDw7wMUEMg",
  "NAFqA/msD8xixqYV",
  "vbOLLrrIbrvttmB1",
  "k/vSLZtdCYZ88gt8",
  "Fu9ZLxGl8ybXr1Bv",
  "8YFy7lxMxln1buV0",
  "6673lP/ud78b7j+9",
  "XpBr3LLMcQnMQrKH",
  "AKB3IDQBBqhb3dln",
  "n31CTKcEUVweSJ8N",
  "LqAkmKB73EosgeWi",
  "0y3E/V1sxklScVyw",
  "JyXtvvvuNmvWrKLd",
  "4s6iTGDqEQCqTEca",
  "IgDUCqvMbKaZ7ZSV",
  "R1pc5M7+8pe/lKxP",
  "qgP5uc99rpQYpGWI",
  "zJ6JE18krjzu0MVn",
  "f8eP3ZPKJCyvu+66",
  "YMnUspUrVxYtMuWP",
  "n2VmO5vZAYhMgNoB",
  "iyZA30BfoKdmls7t",
  "gsom3XTTTbZgwQJb",
  "s2bN9tptn8TjWYUL",
  "zXxIQn9m9OjRocbl",
  "VVddVaQrvCt+b2b/",
  "YmYvbM+dAkDvQWgC",
  "9D0KKwLfHXfddVeo",
  "D7l48eKK1afsL3jx",
  "dU+w8izs/ppxrnqh",
  "Bx54oM2ePduOPfbY",
  "7b379zLX+HOUJwKo",
  "fRCaAH2XPbMEopvN",
  "bLftvXNZPC+++GJ7",
  "6qmnbNWqVV13zIm6",
  "6bh7VXi5pd7ELubF",
  "WueON50/v7wO6Jbm",
  "EW8jptY+B7sTqPHc",
  "fZ24jJC/N26bGZcc",
  "ircdr+OPHmcaXyMt",
  "Hzt2rJ1xxhkhkWw7",
  "Wy0dJcrdaWZLq7Fz",
  "ANg2EJoA/YNJmei8",
  "antbOvNWzxtvvNHe",
  "euut8LfH7PnwZfni",
  "3XHXmvz6efdz/vV4",
  "m/nPs/x24seuBFs1",
  "6alzUL6gfny8EpJe",
  "VsmFoq/rYtGL9ns9",
  "Vbe4dnWd5AqfOnVq",
  "EJYqP1RFJC4fyayX",
  "mNEB+iAITYD+a+k8",
  "y8z+oZoTeemll+z+",
  "+++3559/3t555x2r",
  "r6/fonWyKwHl63Qn",
  "vrqycOZFbV7E+d/+",
  "3lqIo/TapZpLLPxE",
  "/vw4cb90z3T32FB/",
  "f75Wav741drx8MMP",
  "t+OOOy60z6wBfpMV",
  "VZe4bKz2ZACgPBCa",
  "AP2bkZnoPM3Mzq7G",
  "BNSKMRZIykB+7bXX",
  "7MUXX7Rnn33Wli9f",
  "bo2NjZ0sm+7qzgvR",
  "vKCMXb0xceej2G2+",
  "pQzwWvgcjEWj01OM",
  "ZyxEXaR6mII/uutb",
  "SITuvffeNnPmTDvq",
  "qKNChQHf98SJE62K",
  "SFTelz3iGgfoRyA0",
  "AQaW6Dw8SyTaw2qU",
  "66+/3h588MHgflf/",
  "cC+95HRlEY1FVW9i",
  "G/PbqQW6EpX52NZ4",
  "3Vh0e9a7u8f1Plkq",
  "jz76aDv99NPtgANU",
  "8aemkKXyDjP7XSYu",
  "aQkJ0E9BaAIMTL6U",
  "WTo1TjezUdYHUAKS",
  "LKJPP/20LVu2LDxq",
  "mXfgybubu/p8q3XB",
  "2d0ctXzUqFH2hS98",
  "waZMmWL777+/HXLI",
  "ITZ9+nSrcSQsH89E",
  "pQZ1LgEGCAhNABD7",
  "RcJTxeJrlg8//LAk",
  "ImW16youdMmSJfb6",
  "66+HxxUrVtj69ett",
  "w4YNQZDGMYvVjNH0",
  "fcudPWLEiBBjqcxu",
  "icgvfvGLNmnSpPD3",
  "tGnTbMKECZ0yvXVM",
  "/v499qhZ4/SDCEsA",
  "QGgCQFe46Px+ltHe",
  "p5DV0+MPt8ZSunDh",
  "wuCul5j96KOPgjjt",
  "CQnErpB43GGHHWzy",
  "5MlBPCqL2+fU2/JA",
  "moOQ0OwDLIoyxDU6",
  "gk0BYMCC0ASAntgt",
  "Ep4HmdnXrB8iUVeU",
  "oJOIdbZGANc4EpL3",
  "Rkk8ZIgDwGdAaALA",
  "tnBgbqhFJvRv1If0",
  "2chiSdtHAOgRhCYA",
  "VLJgvNfv7KjADn0R",
  "1YT6n5ywpN0jAGw1",
  "CE0AqDRfyVk7q9ap",
  "CLa6h7iLSupZAkBF",
  "QGgCQNF8Ife3ghSn",
  "mNmMLOZTQrTrjBqo",
  "NGp4/k7OUulJO8ur",
  "PDcA6IcgNAGgVpOO",
  "/t7MdtDnVLUn1sdI",
  "MkG5MicoX6/2xABg",
  "4IHQBIC+7IqXu3d3",
  "Mxs+gARpey4GFpc3",
  "ANQsCE0A6Muu+OXd",
  "iNH3MjexKpqPNbMd",
  "Mxf9kBoSpUnmum4y",
  "s+Yss3t5Zo2MRXVe",
  "TK6PtoHLGwBqFoQm",
  "APRXMdqTANuSOH0u",
  "E36ylI7uxb7kpu6K",
  "BjPbbGbTu7DC9tYC",
  "6aIaMQkAfRKEJgBA",
  "78VpOdt2EI0AMGBA",
  "aAIAAABAIVBUGQAA",
  "AAAKAaEJAAAAAIWA",
  "0AQAAACAQkBoAgAA",
  "AEAhIDQBAAAAoBAQ",
  "mgAAAABQCAhNAAAA",
  "ACgEhCYAAAAAFAJC",
  "EwAAAAAKAaEJAAAA",
  "AIWA0AQAAACAQkBo",
  "AgAAAEAhIDQBAAAA",
  "oBAQmgAAAABQCAhN",
  "AAAAACgEhCYAAAAA",
  "FAJCEwAAAAAKAaEJ",
  "AAAAAIWA0AQAAACA",
  "QkBoAgAAAEAhIDQB",
  "AAAAoBAQmgAAAABQ",
  "CAhNAAAAACgEhCYA",
  "AAAAFAJCEwAAAAAK",
  "AaEJAAAAAIWA0AQA",
  "AACAQkBoAgAAAEAh",
  "IDQBAAAAoBAQmgAA",
  "AABQCAhNAAAAACgE",
  "hCYAAAAAFAJCEwAA",
  "AAAKAaEJAAAAAIWA",
  "0AQAAACAQkBoAgAA",
  "AEAhIDQBAAAAoBAQ",
  "mgAAAABQCAhNAAAA",
  "ACgEhCYAAAAAFAJC",
  "EwAAAAAKAaEJAAAA",
  "AIWA0AQAAACAQkBo",
  "AgAAAEAhIDQBAAAA",
  "oBAQmgAAAABQCAhN",
  "AAAAACgEhCYAAAAA",
  "FAJCEwAAAAAKAaEJ",
  "AAAAAIWA0AQAAACA",
  "QkBoAgAAAEAhIDQB",
  "AAAAoBAQmgAAAABQ",
  "CAhNAAAAACgEhCYA",
  "AAAAFAJCEwAAAAAK",
  "AaEJAAAAAIWA0AQA",
  "AACAQkBoAgAAAEAh",
  "IDQBAAAAoBAQmgAA",
  "AABQCAhNAAAAACgE",
  "hCYAAAAAFAJCEwAA",
  "AAAKAaEJAAAAAIWA",
  "0AQAAACAQkBoAgAA",
  "AEAhIDQBAAAAoBAQ",
  "mgAAAABQCAhNAAAA",
  "ACgEhCYAAAAAFAJC",
  "EwAAAAAKAaEJAAAA",
  "AFYE/w8RgHnqAmYT",
  "RgAAAABJRU5ErkJg",
  "gg=="
].join("");

export default {
  async fetch(request, env) {
    try {
      if (request.method === "OPTIONS") {
        return new Response(null, { status: 204, headers: corsHeaders() });
      }

      if (!env.DB) {
        return jsonError(500, "missing_d1_binding", "D1 binding DB is not configured");
      }

      const url = new URL(request.url);
      const path = url.pathname.replace(/\/+$/, "") || "/";

      if (request.method === "GET" && path === "/v1/health") {
        await cleanupExpiredRows(env.DB, nowMs());
        return jsonOk({ ok: true, service: "netstitch-cloud", retention_ms: RETENTION_MS });
      }

      if (request.method === "POST" && path === "/v1/auth/google/start") {
        return await startGoogleAuth(request, env);
      }

      if (request.method === "GET" && path === "/v1/auth/google/authorize") {
        return await renderGoogleAuthPage(request, env);
      }

      if (request.method === "POST" && path === "/v1/auth/google/authorize") {
        return await authorizeGoogleAuth(request, env);
      }

      if (request.method === "GET" && path === "/v1/auth/google/callback") {
        return await completeGoogleAuth(request, env);
      }

      if (request.method === "POST" && path === "/v1/auth/session/poll") {
        return await pollAuthSession(request, env);
      }

      if (request.method === "GET" && path === "/v1/apps") {
        return await listApps(url, env.DB);
      }

      if (request.method === "GET" && path === "/v1/tags") {
        return await listTags(request, url, env.DB);
      }

      if (request.method === "POST" && path === "/v1/tags/delete") {
        const actor = await requireUser(request, env.DB);
        return await deleteTag(request, env.DB, actor);
      }

      if (request.method === "GET" && path === "/v1/authors/nickname") {
        return await checkAuthorSignatureAvailability(request, url, env.DB);
      }

      if (request.method === "GET" && path === "/v1/users/me/apps") {
        const actor = await requireUser(request, env.DB);
        return await listUserApps(env.DB, actor.user_id, normalizeSearch(url.searchParams.get("tag") || "").toLowerCase());
      }

      if (request.method === "GET" && path === "/v1/client/quota") {
        return await getQuota(url, env.DB);
      }

      if (request.method === "GET" && path === "/v1/observations") {
        return await getObservations(request, url, env.DB);
      }

      if (request.method === "POST" && path === "/v1/observations/append") {
        const actor = await requireUser(request, env.DB);
        return await appendObservations(request, env.DB, actor);
      }

      return jsonError(404, "not_found", "Endpoint not found");
    } catch (error) {
      if (error instanceof HttpError) {
        return jsonError(error.status, error.code, error.message);
      }
      return jsonError(500, "internal_error", "Unexpected Worker error");
    }
  },
};

async function startGoogleAuth(request, env) {
  const db = env.DB;
  const body = await readJson(request);
  const clientIdentifier = requireString(body.client_identifier, "client_identifier");
  const publicKeyJwk = requireString(body.client_public_key_jwk, "client_public_key_jwk");
  const uiLanguage = normalizeOauthUiLanguage(optionalString(body.ui_language));

  await recordOauthStartOrThrow(db, await oauthStartBucket(request, clientIdentifier, "google"));
  await assertAuthFailuresAllowed(db, await authBucket(request, clientIdentifier, "google_start"));

  const timestamp = nowMs();
  const state = prefixedId("oauth");
  const pollSecret = randomToken(32);
  const pkceVerifier = randomToken(32);
  const origin = new URL(request.url).origin;
  const browserUrl = `${origin}/v1/auth/google/authorize?state=${encodeURIComponent(state)}`;

  await db
    .prepare(
      `INSERT INTO oauth_states
       (state, poll_secret_hash, provider, client_identifier, client_public_key_jwk,
        pkce_verifier, ui_language, status, created_at_ms, expires_at_ms)
       VALUES (?, ?, 'google', ?, ?, ?, ?, 'created', ?, ?)`
    )
    .bind(
      state,
      await sha256Base64Url(pollSecret),
      clientIdentifier,
      publicKeyJwk,
      pkceVerifier,
      uiLanguage,
      timestamp,
      timestamp + OAUTH_STATE_TTL_MS
    )
    .run();

  return jsonOk({
    state,
    poll_secret: pollSecret,
    browser_url: browserUrl,
    expires_at_ms: timestamp + OAUTH_STATE_TTL_MS,
  });
}

const OAUTH_TEXT = {
  ru: {
    missingState: "Сессия входа не найдена. Запустите вход заново из NetStitch.",
    expiredState: "Сессия входа истекла или уже использована. Запустите вход заново из NetStitch.",
    confirm: "Подтвердите вход через Google. После успешного входа вернитесь в приложение.",
    captchaLabel: "Введите цифры с картинки",
    captchaPlaceholder: "Код",
    captchaRefresh: "Обновить",
    captchaSubmit: "Отправить",
    captchaZoomLabel: "Увеличенная капча",
    captchaZoomHint: "Нажмите Esc или область вокруг картинки, чтобы закрыть.",
    signIn: "Войти через Google",
    successTitle: "Вход выполнен",
    successBody: "Можно вернуться в NetStitch.",
    googleFailed: "Вход через Google не завершен. Код ошибки: {code}.",
  },
  en: {
    missingState: "Sign-in session was not found. Start sign-in again from NetStitch.",
    expiredState: "Sign-in session expired or was already used. Start sign-in again from NetStitch.",
    confirm: "Confirm sign-in with Google. After successful sign-in, return to the application.",
    captchaLabel: "Enter the digits from the image",
    captchaPlaceholder: "Code",
    captchaRefresh: "Refresh",
    captchaSubmit: "Submit",
    captchaZoomLabel: "Zoomed captcha",
    captchaZoomHint: "Press Esc or the area around the image to close.",
    signIn: "Sign in with Google",
    successTitle: "Sign-in complete",
    successBody: "You can return to NetStitch.",
    googleFailed: "Google sign-in was not completed. Error code: {code}.",
  },
};

function oauthText(language) {
  return OAUTH_TEXT[oauthLanguage(language)];
}

function oauthLanguage(language) {
  return String(language || "").toLowerCase().startsWith("en") ? "en" : "ru";
}

function normalizeOauthUiLanguage(language) {
  return oauthLanguage(language) === "en" ? "en-en" : "ru-ru";
}

async function renderGoogleAuthPage(request, env) {
  const url = new URL(request.url);
  const state = String(url.searchParams.get("state") || "").trim();
  if (!state) {
    return renderAuthErrorPage(oauthText().missingState, 400);
  }

  const row = await loadOauthStateForPage(env.DB, state);
  const text = oauthText(row?.ui_language);
  if (!row) {
    return renderAuthErrorPage(text.expiredState, 400, row?.ui_language);
  }
  const captcha = await issueCaptchaChallenge(env.DB, "google_oauth", row.state);
  const browserChallenge = await issueBrowserVerificationChallenge(env.DB, "google_oauth", row.state);
  const lang = oauthLanguage(row.ui_language);
  const captchaArtwork = parseCaptchaArtwork(captcha.svg);
  const captchaWidth = Math.ceil(Number(captchaArtwork?.width || 238));

  return htmlResponse(`<!doctype html>
<html lang="${lang}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>NetStitch</title>
  <style>
    body { margin: 0; min-height: 100vh; display: grid; place-items: center; background: #151515; color: #f4f4f4; font: 14px system-ui, sans-serif; }
    .auth-shell { box-sizing: border-box; width: min(calc(var(--captcha-width) + 44px), calc(100vw - 32px)); display: grid; gap: 18px; justify-items: stretch; }
    main { box-sizing: border-box; width: 100%; position: relative; margin-top: 61px; border: 1px solid #f4f4f4; border-radius: 5px; background: #242424; padding: 36px 22px 22px; }
    h1 { font-weight: 400; font-size: 3em; margin: 0 0 16px; }
    .brand-title { text-align: center; }
    .brand-logo-row { position: absolute; left: 50%; top: 0; transform: translate(-50%, -65%); display: grid; place-items: center; width: 122px; height: 122px; margin: 0; }
    .brand-logo { width: 122px; height: 122px; object-fit: contain; display: block; }
    p { color: #b8c0c8; line-height: 1.45; }
    .confirm-text { margin: 12px 0 0; }
    .captcha-box { margin-top: 16px; display: grid; gap: 10px; }
    .captcha-grid { display: grid; grid-template-columns: 1fr; gap: 10px; align-items: start; }
    .captcha-controls { width: min(var(--captcha-width), 100%); display: grid; gap: 8px; align-content: start; }
    .captcha-art-button { margin: 0; padding: 0; border: 0; background: transparent; color: inherit; cursor: zoom-in; text-align: left; line-height: 0; }
    .captcha-art { width: max-content; max-width: 100%; line-height: 0; }
    .captcha-art svg { display: block; max-width: 100%; height: auto; }
    .captcha-sliced { position: relative; max-width: 100%; overflow: hidden; line-height: 0; background: #e7eef9; border-radius: 5px; }
    .captcha-piece { position: absolute; display: block; max-width: none; user-select: none; -webkit-user-drag: none; pointer-events: none; }
    label { display: block; color: #dbe3eb; font-weight: 700; }
    input[type="text"] { box-sizing: border-box; width: min(var(--captcha-width), 100%); min-height: 34px; border: 1px solid #3a3a3a; border-radius: 5px; background: #171717; color: #f4f4f4; padding: 0 10px; font: inherit; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; }
    .button-row { width: min(var(--captcha-width), 100%); display: grid; grid-template-columns: 34px minmax(0, 1fr); gap: 8px; align-items: center; }
    button { min-height: 34px; padding: 0 18px; border: 1px solid #2e8fd7; border-radius: 5px; background: #0d8de3; color: white; font-weight: 700; cursor: pointer; }
    button:disabled { opacity: 0.55; cursor: default; }
    .icon-button { width: 34px; padding: 0; font-size: 18px; line-height: 1; }
    .submit-button { width: 100%; }
    .secondary-button { border-color: #3a3a3a; background: #171717; }
    .captcha-zoom-backdrop { position: fixed; inset: 0; display: none; place-items: center; z-index: 10; background: rgba(12, 12, 12, 0.72); backdrop-filter: blur(6px); }
    .captcha-zoom-backdrop.is-open { display: grid; }
    .captcha-zoom-content { display: grid; gap: 12px; justify-items: center; color: #dbe3eb; }
    .captcha-zoom-panel { transform: scale(3); transform-origin: center; line-height: 0; box-shadow: 0 12px 36px rgba(0, 0, 0, 0.5); border-radius: 5px; }
    .captcha-zoom-hint { margin-top: 96px; color: #b8c0c8; font-size: 12px; }
    @media (max-width: 560px) {
      .captcha-zoom-panel { transform: scale(2); }
      .captcha-zoom-hint { margin-top: 56px; }
    }
  </style>
</head>
<body>
  <div class="auth-shell" style="--captcha-width: ${captchaWidth}px;">
    <main>
    <div class="brand-logo-row"><img class="brand-logo" src="${NETSTITCH_LOGO_DATA_URI}" alt="" aria-hidden="true"></div>
    <h1 class="brand-title">NetStitch</h1>
    <form method="post" action="/v1/auth/google/authorize">
      <input type="hidden" name="state" value="${escapeHtml(row.state)}">
      <input type="hidden" name="captcha_challenge_id" value="${escapeHtml(captcha.challenge_id)}">
      <input type="hidden" name="browser_challenge_id" value="${escapeHtml(browserChallenge.challenge_id)}">
      <input type="hidden" name="browser_nonce" value="${escapeHtml(browserChallenge.nonce)}">
      <input type="hidden" name="browser_counter" value="">
      <input type="hidden" name="browser_hash" value="">
      <input type="hidden" name="browser_memory_result" value="">
      <input type="hidden" name="browser_signals_json" value="">
      <input type="hidden" name="browser_workload_json" value="">
      <div class="captcha-box">
        <div class="captcha-grid">
          <div class="captcha-art-button" role="button" tabindex="0" data-captcha-open aria-label="${escapeHtml(text.captchaZoomLabel)}">
            <span class="captcha-art">${renderCaptchaArtwork(captcha.svg)}</span>
          </div>
          <div class="captcha-controls">
            <label for="captcha_answer">${escapeHtml(text.captchaLabel)}</label>
            <input id="captcha_answer" name="captcha_answer" type="text" inputmode="numeric" pattern="[0-9]*" maxlength="${CAPTCHA_LENGTH}" autocomplete="off" placeholder="${escapeHtml(text.captchaPlaceholder)}" autofocus required>
            <div class="button-row">
              <button class="secondary-button icon-button" type="button" data-captcha-refresh aria-label="${escapeHtml(text.captchaRefresh)}">&#8635;</button>
              <button class="submit-button" type="submit" data-captcha-submit disabled>${escapeHtml(text.captchaSubmit)}</button>
            </div>
          </div>
        </div>
      </div>
      <p class="confirm-text">${escapeHtml(text.confirm)}</p>
    </form>
    </main>
  </div>
  <div class="captcha-zoom-backdrop" data-captcha-zoom-backdrop aria-hidden="true">
    <div class="captcha-zoom-content" role="dialog" aria-modal="true" aria-label="${escapeHtml(text.captchaZoomLabel)}">
      <div class="captcha-zoom-panel" data-captcha-zoom-panel></div>
      <div class="captcha-zoom-hint">${escapeHtml(text.captchaZoomHint)}</div>
    </div>
  </div>
  <script>
(() => {
  const browserChallenge = ${JSON.stringify(browserChallenge)};
  const pageRenderedAt = performance.now();
  const interaction = {
    focus: 0,
    input: 0,
    change: 0,
    trusted: false,
    renderToActionMs: 0,
    pointerXPercent: null,
    pointerYPercent: null
  };
  document.addEventListener("focusin", () => { interaction.focus += 1; }, { passive: true });
  document.addEventListener("input", () => { interaction.input += 1; }, { passive: true });
  document.addEventListener("change", () => { interaction.change += 1; }, { passive: true });
  const refresh = document.querySelector("[data-captcha-refresh]");
  if (refresh) {
    refresh.addEventListener("click", () => {
      window.location.replace(window.location.href);
    });
  }
  const answer = document.getElementById("captcha_answer");
  const submit = document.querySelector("[data-captcha-submit]");
  const form = document.querySelector("form");
  if (answer) {
    setTimeout(() => {
      if (document.activeElement !== answer) answer.focus({ preventScroll: true });
    }, 0);
  }
  const readyAt = Date.now() + ${CAPTCHA_MIN_SOLVE_MS};
  let submitting = false;
  const canSubmit = () => Boolean(answer && Date.now() >= readyAt && /^\\d{${CAPTCHA_LENGTH}}$/.test(answer.value || ""));
  const updateSubmit = () => {
    if (submit) submit.disabled = submitting || !canSubmit();
  };
  if (answer) answer.addEventListener("input", updateSubmit);
  if (submit) {
    submit.addEventListener("pointerdown", (event) => {
      const rect = submit.getBoundingClientRect();
      if (rect.width > 0 && rect.height > 0) {
        interaction.pointerXPercent = Math.max(0, Math.min(100, ((event.clientX - rect.left) / rect.width) * 100));
        interaction.pointerYPercent = Math.max(0, Math.min(100, ((event.clientY - rect.top) / rect.height) * 100));
      }
      interaction.trusted = event.isTrusted === true;
    }, { passive: true });
  }
  if (form) {
    form.addEventListener("submit", async (event) => {
      event.preventDefault();
      interaction.trusted = interaction.trusted || event.isTrusted === true;
      interaction.renderToActionMs = Math.max(0, Math.floor(performance.now() - pageRenderedAt));
      if (!canSubmit() || submitting) {
        updateSubmit();
        return;
      }
      submitting = true;
      updateSubmit();
      try {
        const proof = await buildBrowserVerificationProof(browserChallenge, pageRenderedAt, interaction);
        form.elements.browser_counter.value = String(proof.proof.counter);
        form.elements.browser_hash.value = proof.proof.hash;
        form.elements.browser_memory_result.value = proof.proof.memory_result;
        form.elements.browser_signals_json.value = JSON.stringify(proof.signals);
        form.elements.browser_workload_json.value = JSON.stringify(proof.workload);
        HTMLFormElement.prototype.submit.call(form);
      } catch {
        submitting = false;
        updateSubmit();
      }
    });
  }
  updateSubmit();
  setTimeout(updateSubmit, ${CAPTCHA_MIN_SOLVE_MS});
  const source = document.querySelector("[data-captcha-open] .captcha-sliced");
  const opener = document.querySelector("[data-captcha-open]");
  const backdrop = document.querySelector("[data-captcha-zoom-backdrop]");
  const panel = document.querySelector("[data-captcha-zoom-panel]");
  if (!source || !opener || !backdrop || !panel) return;

  const close = () => {
    backdrop.classList.remove("is-open");
    backdrop.setAttribute("aria-hidden", "true");
    panel.replaceChildren();
  };
  const renderZoomTiles = () => {
    const root = source.cloneNode(false);
    let payload;
    try {
      payload = JSON.parse(source.dataset.captchaTiles || "{}");
    } catch {
      return source.cloneNode(true);
    }
    const tiles = Array.isArray(payload.tiles) ? payload.tiles : [];
    root.style.width = String(payload.width || 170) + "px";
    root.style.height = String(payload.height || 60) + "px";
    tiles.forEach((tile) => {
      if (!tile || !tile.src) return;
      const img = new Image();
      img.className = "captcha-piece";
      img.alt = "";
      img.decoding = "async";
      img.draggable = false;
      img.style.left = String(tile.x || 0) + "px";
      img.style.top = String(tile.y || 0) + "px";
      img.style.width = String((tile.width || 1) + 0.8) + "px";
      img.style.height = String((tile.height || 1) + 0.8) + "px";
      img.style.transform = "translate(" + (Math.random() * 0.4 - 0.2).toFixed(2) + "px," + (Math.random() * 0.4 - 0.2).toFixed(2) + "px)";
      img.src = tile.src;
      root.appendChild(img);
    });
    return root;
  };
  const open = () => {
    panel.replaceChildren(renderZoomTiles());
    backdrop.classList.add("is-open");
    backdrop.setAttribute("aria-hidden", "false");
  };
  opener.addEventListener("click", open);
  opener.addEventListener("keydown", (event) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      open();
    }
  });
  backdrop.addEventListener("click", (event) => {
    if (event.target === backdrop) close();
  });
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") close();
  });
})();

async function sha256Hex(input) {
  const bytes = new TextEncoder().encode(input);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest)).map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function storageAvailable(kind) {
  try {
    const storage = window[kind];
    const key = "__netstitch_browser_verification__";
    storage.setItem(key, "1");
    storage.removeItem(key);
    return true;
  } catch {
    return false;
  }
}

function collectWebGlSignals() {
  const startedAt = performance.now();
  try {
    const canvas = document.createElement("canvas");
    const gl = canvas.getContext("webgl") || canvas.getContext("experimental-webgl");
    if (!gl) return { webgl_vendor: "", webgl_renderer: "", elapsed_ms: Math.max(0, Math.floor(performance.now() - startedAt)) };
    const debugInfo = gl.getExtension("WEBGL_debug_renderer_info");
    if (!debugInfo) {
      return {
        webgl_vendor: String(gl.getParameter(gl.VENDOR) || ""),
        webgl_renderer: String(gl.getParameter(gl.RENDERER) || ""),
        elapsed_ms: Math.max(0, Math.floor(performance.now() - startedAt)),
      };
    }
    return {
      webgl_vendor: String(gl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL) || ""),
      webgl_renderer: String(gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL) || ""),
      elapsed_ms: Math.max(0, Math.floor(performance.now() - startedAt)),
    };
  } catch {
    return { webgl_vendor: "", webgl_renderer: "", elapsed_ms: Math.max(0, Math.floor(performance.now() - startedAt)) };
  }
}

async function collectCanvasHash() {
  const startedAt = performance.now();
  try {
    const canvas = document.createElement("canvas");
    canvas.width = 160;
    canvas.height = 48;
    const context = canvas.getContext("2d");
    if (!context) return { canvas_hash: "", elapsed_ms: Math.max(0, Math.floor(performance.now() - startedAt)) };
    context.fillStyle = "#14213d";
    context.fillRect(0, 0, canvas.width, canvas.height);
    context.fillStyle = "#fca311";
    context.font = "18px sans-serif";
    context.fillText("NetStitch browser check", 8, 28);
    context.strokeStyle = "#e5e5e5";
    context.beginPath();
    context.moveTo(4, 42);
    context.lineTo(156, 6);
    context.stroke();
    return {
      canvas_hash: await sha256Hex(canvas.toDataURL("image/png")),
      elapsed_ms: Math.max(0, Math.floor(performance.now() - startedAt)),
    };
  } catch {
    return { canvas_hash: "", elapsed_ms: Math.max(0, Math.floor(performance.now() - startedAt)) };
  }
}

const browserSignalKeys = [
  "canvas_hash", "color_depth", "cookie_enabled", "device_memory", "elapsed_ms",
  "hardware_concurrency", "interaction_action", "interaction_change_events",
  "interaction_focus_events", "interaction_input_events", "interaction_render_to_action_ms",
  "interaction_pointer_x_pct", "interaction_pointer_y_pct", "interaction_trusted",
  "language", "languages_count", "local_storage", "max_touch_points",
  "screen_height", "screen_width", "session_storage", "timezone", "timezone_offset_minutes",
  "user_agent", "visibility_state", "webdriver", "webgl_renderer", "webgl_vendor",
  "window_has_focus"
];

function normalizeSignalDigest(signals) {
  const normalized = {};
  for (const key of browserSignalKeys) normalized[key] = signals[key];
  return JSON.stringify(normalized);
}

function xorshift32Client(value) {
  let state = value >>> 0;
  state = (state ^ ((state << 13) >>> 0)) >>> 0;
  state = (state ^ (state >>> 17)) >>> 0;
  state = (state ^ ((state << 5) >>> 0)) >>> 0;
  return state >>> 0;
}

async function solveMemoryWalkClient(challenge) {
  const memory = challenge.memory;
  const bytes = Math.max(1024, Math.floor(memory.size_kib) * 1024);
  const wordCount = Math.max(256, Math.floor(bytes / 4));
  const rounds = Math.max(1, Math.min(16, Math.floor(memory.rounds)));
  const words = new Uint32Array(wordCount);
  let state = Number.parseInt((await sha256Hex(memory.seed + ":" + challenge.challenge_id + ":" + challenge.nonce)).slice(0, 8), 16) >>> 0;
  for (let index = 0; index < wordCount; index += 1) {
    state = xorshift32Client((state + index + 0x9e3779b9) >>> 0);
    words[index] = state;
  }
  let cursor = state % wordCount;
  let accumulator = 0x9e3779b9;
  const steps = wordCount * rounds;
  for (let step = 0; step < steps; step += 1) {
    const value = words[cursor] >>> 0;
    accumulator = (((accumulator ^ value) >>> 0) + ((cursor + step) >>> 0)) >>> 0;
    state = xorshift32Client((state ^ value ^ accumulator) >>> 0);
    cursor = (cursor + value + state) % wordCount;
  }
  return accumulator.toString(16).padStart(8, "0") + state.toString(16).padStart(8, "0");
}

function browserWorkloadSource() {
  return \`
const xorshift32 = (value) => {
  let state = value >>> 0;
  state = (state ^ ((state << 13) >>> 0)) >>> 0;
  state = (state ^ (state >>> 17)) >>> 0;
  state = (state ^ ((state << 5) >>> 0)) >>> 0;
  return state >>> 0;
};
const hex = (buffer) => Array.from(new Uint8Array(buffer)).map((byte) => byte.toString(16).padStart(2, "0")).join("");
const sha256 = async (input) => hex(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(input)));
const seedToUint32 = async (seed) => Number.parseInt((await sha256(seed)).slice(0, 8), 16) >>> 0;
const solveMemoryWalk = async (challenge) => {
  const memory = challenge.memory;
  const bytes = Math.max(1024, Math.floor(memory.size_kib) * 1024);
  const wordCount = Math.max(256, Math.floor(bytes / 4));
  const rounds = Math.max(1, Math.min(16, Math.floor(memory.rounds)));
  const words = new Uint32Array(wordCount);
  let state = await seedToUint32(memory.seed + ":" + challenge.challenge_id + ":" + challenge.nonce);
  for (let index = 0; index < wordCount; index += 1) {
    state = xorshift32((state + index + 0x9e3779b9) >>> 0);
    words[index] = state;
  }
  let cursor = state % wordCount;
  let accumulator = 0x9e3779b9;
  const steps = wordCount * rounds;
  for (let step = 0; step < steps; step += 1) {
    const value = words[cursor] >>> 0;
    accumulator = (((accumulator ^ value) >>> 0) + ((cursor + step) >>> 0)) >>> 0;
    state = xorshift32((state ^ value ^ accumulator) >>> 0);
    cursor = (cursor + value + state) % wordCount;
  }
  return accumulator.toString(16).padStart(8, "0") + state.toString(16).padStart(8, "0");
};
self.onmessage = async (event) => {
  const startedAt = performance.now();
  try {
    const { challenge, signalDigest } = event.data;
    const prefix = "0".repeat(Math.max(1, Math.min(5, Math.floor(challenge.difficulty))));
    let counter = 0;
    let hash = "";
    const powStartedAt = performance.now();
    for (; counter < 500000; counter += 1) {
      hash = await sha256(challenge.challenge_id + ":" + challenge.nonce + ":" + counter + ":" + signalDigest);
      if (hash.startsWith(prefix)) break;
    }
    if (!hash.startsWith(prefix)) throw new Error("browser-verification-failed");
    const powMs = Math.max(0, Math.floor(performance.now() - powStartedAt));
    const memoryStartedAt = performance.now();
    const memoryResult = await solveMemoryWalk(challenge);
    const memoryMs = Math.max(0, Math.floor(performance.now() - memoryStartedAt));
    self.postMessage({ ok: true, result: { proof: { counter, hash, memory_result: memoryResult }, timings: { pow_ms: powMs, memory_ms: memoryMs, worker_total_ms: Math.max(0, Math.floor(performance.now() - startedAt)) } } });
  } catch (error) {
    self.postMessage({ ok: false });
  }
};\`;
}

async function runBrowserWorkload(challenge, signalDigest) {
  if (typeof Worker === "undefined") {
    return { result: await runBrowserWorkloadOnMain(challenge, signalDigest), worker_used: false, worker_supported: false };
  }
  try {
    const blobUrl = URL.createObjectURL(new Blob([browserWorkloadSource()], { type: "text/javascript" }));
    const worker = new Worker(blobUrl);
    return await new Promise((resolve, reject) => {
      const timeoutId = window.setTimeout(() => {
        worker.terminate();
        URL.revokeObjectURL(blobUrl);
        reject(new Error("browser-verification-failed"));
      }, 30000);
      worker.onmessage = (event) => {
        window.clearTimeout(timeoutId);
        worker.terminate();
        URL.revokeObjectURL(blobUrl);
        if (event.data && event.data.ok && event.data.result) {
          resolve({ result: event.data.result, worker_used: true, worker_supported: true });
        } else {
          reject(new Error("browser-verification-failed"));
        }
      };
      worker.onerror = () => {
        window.clearTimeout(timeoutId);
        worker.terminate();
        URL.revokeObjectURL(blobUrl);
        reject(new Error("browser-verification-failed"));
      };
      worker.postMessage({ challenge, signalDigest });
    });
  } catch {
    return { result: await runBrowserWorkloadOnMain(challenge, signalDigest), worker_used: false, worker_supported: true };
  }
}

async function runBrowserWorkloadOnMain(challenge, signalDigest) {
  const startedAt = performance.now();
  const prefix = "0".repeat(Math.max(1, Math.min(5, Math.floor(challenge.difficulty))));
  let counter = 0;
  let hash = "";
  const powStartedAt = performance.now();
  for (; counter < 500000; counter += 1) {
    hash = await sha256Hex(challenge.challenge_id + ":" + challenge.nonce + ":" + counter + ":" + signalDigest);
    if (hash.startsWith(prefix)) break;
  }
  if (!hash.startsWith(prefix)) throw new Error("browser-verification-failed");
  const powMs = Math.max(0, Math.floor(performance.now() - powStartedAt));
  const memoryStartedAt = performance.now();
  const memoryResult = await solveMemoryWalkClient(challenge);
  const memoryMs = Math.max(0, Math.floor(performance.now() - memoryStartedAt));
  return {
    proof: { counter, hash, memory_result: memoryResult },
    timings: {
      pow_ms: powMs,
      memory_ms: memoryMs,
      worker_total_ms: Math.max(0, Math.floor(performance.now() - startedAt)),
    },
  };
}

async function buildBrowserVerificationProof(challenge, renderedAt, interaction) {
  const startedAt = performance.now();
  const webgl = collectWebGlSignals();
  const canvas = await collectCanvasHash();
  const dateTime = Intl.DateTimeFormat().resolvedOptions();
  const nav = navigator || {};
  const signals = {
    user_agent: nav.userAgent || "",
    language: nav.language || "",
    languages_count: Array.isArray(nav.languages) ? nav.languages.length : 0,
    timezone: dateTime.timeZone || "",
    timezone_offset_minutes: new Date().getTimezoneOffset(),
    screen_width: Math.floor((window.screen && window.screen.width) || window.innerWidth || 0),
    screen_height: Math.floor((window.screen && window.screen.height) || window.innerHeight || 0),
    color_depth: Math.floor((window.screen && window.screen.colorDepth) || 0),
    hardware_concurrency: Math.floor(nav.hardwareConcurrency || 0),
    device_memory: Number.isFinite(nav.deviceMemory) ? Number(nav.deviceMemory) : null,
    max_touch_points: Math.floor(nav.maxTouchPoints || 0),
    cookie_enabled: nav.cookieEnabled === true,
    local_storage: storageAvailable("localStorage"),
    session_storage: storageAvailable("sessionStorage"),
    webgl_vendor: webgl.webgl_vendor,
    webgl_renderer: webgl.webgl_renderer,
    canvas_hash: canvas.canvas_hash,
    visibility_state: document.visibilityState || "",
    elapsed_ms: Math.max(20, Math.floor(performance.now() - startedAt)),
    webdriver: nav.webdriver === true,
    interaction_action: "oauth_google",
    interaction_trusted: interaction.trusted === true,
    interaction_render_to_action_ms: Math.max(0, Math.floor(performance.now() - renderedAt)),
    interaction_focus_events: Math.max(0, Math.floor(interaction.focus || 0)),
    interaction_input_events: Math.max(0, Math.floor(interaction.input || 0)),
    interaction_change_events: Math.max(0, Math.floor(interaction.change || 0)),
    interaction_pointer_x_pct: Number.isFinite(interaction.pointerXPercent) ? Number(interaction.pointerXPercent.toFixed(2)) : null,
    interaction_pointer_y_pct: Number.isFinite(interaction.pointerYPercent) ? Number(interaction.pointerYPercent.toFixed(2)) : null,
    window_has_focus: document.hasFocus(),
  };
  const signalDigest = await sha256Hex(normalizeSignalDigest(signals));
  const workload = await runBrowserWorkload(challenge, signalDigest);
  const totalMs = Math.max(20, Math.floor(performance.now() - startedAt));
  return {
    signals,
    proof: workload.result.proof,
    workload: {
      timings: {
        canvas_ms: canvas.elapsed_ms,
        webgl_ms: webgl.elapsed_ms,
        pow_ms: workload.result.timings.pow_ms,
        memory_ms: workload.result.timings.memory_ms,
        worker_total_ms: workload.result.timings.worker_total_ms,
        total_ms: totalMs,
      },
      capabilities: {
        worker_supported: workload.worker_supported,
        worker_used: workload.worker_used,
        crypto_subtle: Boolean(crypto && crypto.subtle),
      },
    },
  };
}
  </script>
</body>
</html>`);
}

function renderAuthErrorPage(message, status = 400, language = null) {
  const lang = oauthLanguage(language);
  return htmlResponse(`<!doctype html>
<html lang="${lang}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>NetStitch</title>
  <style>
    body { margin: 0; min-height: 100vh; display: grid; place-items: center; background: #151515; color: #f4f4f4; font: 14px system-ui, sans-serif; }
    .auth-shell { width: min(460px, calc(100vw - 32px)); display: grid; gap: 18px; justify-items: stretch; }
    main { width: 100%; box-sizing: border-box; position: relative; margin-top: 61px; border: 1px solid #f4f4f4; border-radius: 5px; background: #242424; padding: 36px 22px 22px; }
    h1 { font-weight: 400; font-size: 3em; margin: 0 0 16px; }
    .brand-title { text-align: center; }
    .brand-logo-row { position: absolute; left: 50%; top: 0; transform: translate(-50%, -65%); display: grid; place-items: center; width: 122px; height: 122px; margin: 0; }
    .brand-logo { width: 122px; height: 122px; object-fit: contain; display: block; }
    p { color: #ff9b85; line-height: 1.45; }
  </style>
</head>
<body>
  <div class="auth-shell">
    <main>
    <div class="brand-logo-row"><img class="brand-logo" src="${NETSTITCH_LOGO_DATA_URI}" alt="" aria-hidden="true"></div>
    <h1 class="brand-title">NetStitch</h1>
    <p>${escapeHtml(message)}</p>
    </main>
  </div>
</body>
</html>`, status);
}

function renderCaptchaArtwork(payloadText) {
  const artwork = parseCaptchaArtwork(payloadText);
  if (!artwork) {
    return payloadText;
  }
  const elementId = `captcha_tiles_${randomToken(8)}`;
  const payload = JSON.stringify({
    width: artwork.width,
    height: artwork.height,
    tiles: artwork.tiles,
  });
  return `<div id="${elementId}" class="captcha-sliced" data-captcha-tiles="${escapeHtml(payload)}" aria-hidden="true"></div><script>
(() => {
  const root = document.getElementById(${JSON.stringify(elementId)});
  if (!root) return;
  let payload;
  try { payload = JSON.parse(root.dataset.captchaTiles || "{}"); } catch { return; }
  const tiles = Array.isArray(payload.tiles) ? payload.tiles : [];
  root.style.width = String(payload.width || 170) + "px";
  root.style.height = String(payload.height || 60) + "px";
  const order = tiles.map((_, index) => index);
  for (let index = order.length - 1; index > 0; index -= 1) {
    const swapIndex = Math.floor(Math.random() * (index + 1));
    [order[index], order[swapIndex]] = [order[swapIndex], order[index]];
  }
  order.forEach((tileIndex, sequence) => {
    const tile = tiles[tileIndex];
    if (!tile || !tile.src) return;
    const delay = 12 + sequence * (3 + Math.floor(Math.random() * 14));
    setTimeout(() => {
      const img = new Image();
      img.className = "captcha-piece";
      img.alt = "";
      img.decoding = "async";
      img.draggable = false;
      const jitterX = (Math.random() * 0.7 - 0.35).toFixed(2);
      const jitterY = (Math.random() * 0.7 - 0.35).toFixed(2);
      img.style.left = String(tile.x || 0) + "px";
      img.style.top = String(tile.y || 0) + "px";
      img.style.width = String((tile.width || 1) + 0.8) + "px";
      img.style.height = String((tile.height || 1) + 0.8) + "px";
      img.style.transform = "translate(" + jitterX + "px," + jitterY + "px)";
      img.src = tile.src;
      root.appendChild(img);
    }, delay);
  });
})();
</script>`;
}

function parseCaptchaArtwork(payloadText) {
  try {
    const artwork = JSON.parse(payloadText);
    if (
      artwork &&
      artwork.kind === "captcha_tiles_v1" &&
      Number.isFinite(Number(artwork.width)) &&
      Number.isFinite(Number(artwork.height)) &&
      Array.isArray(artwork.tiles)
    ) {
      return artwork;
    }
  } catch {
    return null;
  }
  return null;
}

async function authorizeGoogleAuth(request, env) {
  const form = await request.formData();
  const state = requireString(form.get("state"), "state");
  const row = await loadOauthState(env.DB, state);

  if (row.status !== "captcha_ok") {
    try {
      await verifyBrowserVerificationOrThrow(
        env.DB,
        request,
        form,
        "google_oauth",
        row.state
      );
      await verifyCaptchaOrThrow(
        env.DB,
        request,
        form.get("captcha_challenge_id"),
        form.get("captcha_answer"),
        "google_oauth",
        row.state
      );
      await markOauthCaptchaOk(env.DB, state);
    } catch (error) {
      await recordAuthFailure(
        env.DB,
        await authBucket(request, row.client_identifier, "google_start")
      );
      return renderAuthErrorPage(
        oauthText(row.ui_language).googleFailed.replace("{code}", "auth_flow_failed"),
        error instanceof HttpError ? error.status : 401,
        row.ui_language
      );
    }
  }

  const clientId = requireEnv(env, "NETSTITCH_GOOGLE_CLIENT_ID");
  const redirectUri = oauthRedirectUri(request, env);
  const challenge = await pkceChallenge(row.pkce_verifier);
  const params = new URLSearchParams({
    client_id: clientId,
    redirect_uri: redirectUri,
    response_type: "code",
    scope: "openid email profile",
    state,
    code_challenge: challenge,
    code_challenge_method: "S256",
    prompt: "select_account",
  });

  return Response.redirect(`https://accounts.google.com/o/oauth2/v2/auth?${params}`, 302);
}

async function markOauthCaptchaOk(db, state) {
  await db
    .prepare(
      `UPDATE oauth_states
       SET status = 'captcha_ok'
       WHERE state = ?`
    )
    .bind(state)
    .run();
}

async function issueBrowserVerificationChallenge(db, purpose, scope) {
  const timestamp = nowMs();
  const scopeHash = await browserVerificationScopeHash(scope);
  const challengeId = prefixedId("brv");
  const nonce = randomToken(24);
  const memorySeed = randomToken(16);

  await db.batch([
    db
      .prepare(
        `DELETE FROM browser_verification_challenges
         WHERE purpose = ?
           AND scope_hash = ?`
      )
      .bind(purpose, scopeHash),
    db
      .prepare(
        `INSERT INTO browser_verification_challenges
         (challenge_id, purpose, scope_hash, nonce, memory_seed, difficulty,
          memory_size_kib, memory_rounds, issued_at_ms, expires_at_ms, attempts)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0)`
      )
      .bind(
        challengeId,
        purpose,
        scopeHash,
        nonce,
        memorySeed,
        BROWSER_VERIFICATION_POW_DIFFICULTY,
        BROWSER_VERIFICATION_MEMORY_SIZE_KIB,
        BROWSER_VERIFICATION_MEMORY_ROUNDS,
        timestamp,
        timestamp + BROWSER_VERIFICATION_TTL_MS
      ),
  ]);

  return {
    challenge_id: challengeId,
    nonce,
    algorithm: "sha256",
    difficulty: BROWSER_VERIFICATION_POW_DIFFICULTY,
    purpose,
    expires_at_ms: timestamp + BROWSER_VERIFICATION_TTL_MS,
    memory: {
      algorithm: "seeded-memory-walk-v1",
      seed: memorySeed,
      size_kib: BROWSER_VERIFICATION_MEMORY_SIZE_KIB,
      rounds: BROWSER_VERIFICATION_MEMORY_ROUNDS,
    },
  };
}

async function verifyBrowserVerificationOrThrow(db, request, form, purpose, scope) {
  const challengeId = String(form.get("browser_challenge_id") || "").trim();
  const nonce = String(form.get("browser_nonce") || "").trim();
  const counter = Number(String(form.get("browser_counter") || "").trim());
  const hash = String(form.get("browser_hash") || "").trim().toLowerCase();
  const memoryResult = String(form.get("browser_memory_result") || "").trim().toLowerCase();
  const signals = parseHiddenJsonObject(form.get("browser_signals_json"));
  const workload = parseHiddenJsonObject(form.get("browser_workload_json"));

  if (
    !challengeId ||
    !nonce ||
    !Number.isInteger(counter) ||
    counter < 0 ||
    !/^[0-9a-f]{64}$/i.test(hash) ||
    !/^[0-9a-f]{16}$/i.test(memoryResult)
  ) {
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }

  const timestamp = nowMs();
  const scopeHash = await browserVerificationScopeHash(scope);
  const row = await db
    .prepare(
      `SELECT challenge_id, purpose, scope_hash, nonce, memory_seed, difficulty,
              memory_size_kib, memory_rounds, issued_at_ms, expires_at_ms,
              attempts, consumed_at_ms
       FROM browser_verification_challenges
       WHERE challenge_id = ?`
    )
    .bind(challengeId)
    .first();

  if (
    !row ||
    row.purpose !== purpose ||
    row.scope_hash !== scopeHash ||
    row.expires_at_ms <= timestamp ||
    row.consumed_at_ms ||
    !constantTimeStringEqual(row.nonce, nonce)
  ) {
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }

  const attempts = Number(row.attempts || 0) + 1;
  await db
    .prepare(
      `UPDATE browser_verification_challenges
       SET attempts = ?
       WHERE challenge_id = ?`
    )
    .bind(attempts, challengeId)
    .run();

  if (attempts > BROWSER_VERIFICATION_MAX_ATTEMPTS) {
    await consumeBrowserVerificationChallenge(db, challengeId, timestamp);
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }

  const signalDigest = await sha256Hex(normalizeBrowserSignalDigest(signals));
  const expectedHash = await sha256Hex(`${challengeId}:${nonce}:${counter}:${signalDigest}`);
  const difficulty = Math.max(1, Math.min(5, Number(row.difficulty || BROWSER_VERIFICATION_POW_DIFFICULTY)));
  if (!constantTimeStringEqual(expectedHash, hash) || !expectedHash.startsWith("0".repeat(difficulty))) {
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }

  const expectedMemory = await browserVerificationMemoryResult(row, challengeId, nonce);
  if (!constantTimeStringEqual(expectedMemory, memoryResult)) {
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }

  const requestUserAgent = String(request.headers.get("user-agent") || "").trim();
  const reportedUserAgent = String(signals.user_agent || "").trim();
  if (!requestUserAgent || !reportedUserAgent || !constantTimeStringEqual(requestUserAgent, reportedUserAgent)) {
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }

  const risk = evaluateBrowserVerificationRisk(signals, workload, requestUserAgent);
  if (risk.level === "hostile") {
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }
  if (risk.delay_ms > 0) {
    await sleep(risk.delay_ms);
  }

  await consumeBrowserVerificationChallenge(db, challengeId, timestamp);
}

async function consumeBrowserVerificationChallenge(db, challengeId, timestamp) {
  await db
    .prepare(
      `UPDATE browser_verification_challenges
       SET consumed_at_ms = ?
       WHERE challenge_id = ?`
    )
    .bind(timestamp, challengeId)
    .run();
}

async function completeGoogleAuth(request, env) {
  const url = new URL(request.url);
  const state = String(url.searchParams.get("state") || "").trim();
  if (!state) {
    return renderAuthErrorPage(oauthText().missingState, 400);
  }

  const callbackState = await loadOauthStateForCallback(env.DB, state);
  const text = oauthText(callbackState?.ui_language);
  if (!callbackState) {
    return renderAuthErrorPage(text.expiredState, 400);
  }
  if (callbackState.status === "completed" || callbackState.status === "consumed") {
    return renderAuthSuccessPage(callbackState.ui_language);
  }

  const googleError = optionalString(url.searchParams.get("error"));
  if (googleError) {
    const failureCode = normalizeFailureCode(`google_${googleError}`);
    await failOauthState(env.DB, state, failureCode);
    await recordAuthFailure(
      env.DB,
      await authBucket(request, callbackState.client_identifier, "google_start")
    );
    return renderAuthErrorPage(text.googleFailed.replace("{code}", failureCode), 401, callbackState.ui_language);
  }

  try {
    const code = requireString(url.searchParams.get("code"), "code");
    const row = callbackState;
    if (row.status !== "captcha_ok") {
      throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
    }

    const tokenInfo = await exchangeGoogleCode(request, env, code, row.pkce_verifier);
    const session = await upsertGoogleIdentitySession(env, row, tokenInfo);

    await env.DB
      .prepare(
        `UPDATE oauth_states
         SET status = 'completed', result_json = ?, used_at_ms = ?
         WHERE state = ?`
      )
      .bind(JSON.stringify(session), nowMs(), state)
      .run();

    return renderAuthSuccessPage(callbackState.ui_language);
  } catch (error) {
    const failureCode =
      error instanceof HttpError ? callbackFailureCode(error.code) : "auth_flow_failed";
    await failOauthState(env.DB, state, failureCode);
    await recordAuthFailure(
      env.DB,
      await authBucket(request, callbackState.client_identifier, "google_start")
    );
    return renderAuthErrorPage(
      text.googleFailed.replace("{code}", failureCode),
      error instanceof HttpError ? error.status : 401,
      callbackState.ui_language
    );
  }
}

function renderAuthSuccessPage(language = null) {
  const lang = oauthLanguage(language);
  const text = oauthText(language);
  return htmlResponse(`<!doctype html>
<html lang="${lang}"><head><meta charset="utf-8"><title>NetStitch</title>
<style>body{margin:0;min-height:100vh;display:grid;place-items:center;background:#151515;color:#f4f4f4;font:14px system-ui,sans-serif}.auth-shell{width:min(460px,calc(100vw - 32px));display:grid;gap:18px;justify-items:stretch}main{width:100%;box-sizing:border-box;position:relative;margin-top:61px;border:1px solid #f4f4f4;border-radius:5px;background:#242424;padding:36px 22px 22px}h1{font-weight:400;font-size:3em;margin:0 0 16px}.brand-title{text-align:center}.brand-logo-row{position:absolute;left:50%;top:0;transform:translate(-50%,-65%);display:grid;place-items:center;width:122px;height:122px;margin:0}.brand-logo{width:122px;height:122px;object-fit:contain;display:block}p{color:#b8c0c8}</style>
</head><body><div class="auth-shell"><main><div class="brand-logo-row"><img class="brand-logo" src="${NETSTITCH_LOGO_DATA_URI}" alt="" aria-hidden="true"></div><h1 class="brand-title">NetStitch</h1><p>${escapeHtml(text.successTitle)}</p><p>${escapeHtml(text.successBody)}</p></main></div></body></html>`);
}

async function pollAuthSession(request, env) {
  const body = await readJson(request);
  const state = requireString(body.state, "state");
  const pollSecret = requireString(body.poll_secret, "poll_secret");
  const pollSecretHash = await sha256Base64Url(pollSecret);
  const row = await env.DB
    .prepare(
      `SELECT state, poll_secret_hash, status, result_json, failure_code, expires_at_ms
       FROM oauth_states
       WHERE state = ?`
    )
    .bind(state)
    .first();

  if (!row) {
    throw new HttpError(401, "invalid_auth_poll", "Authentication flow failed");
  }
  if (row.expires_at_ms <= nowMs()) {
    throw new HttpError(401, "auth_state_expired", "Authentication flow failed");
  }
  if (row.poll_secret_hash !== pollSecretHash) {
    throw new HttpError(401, "auth_poll_secret_mismatch", "Authentication flow failed");
  }
  if (row.status === "completed" && row.result_json) {
    await env.DB
      .prepare(
        `UPDATE oauth_states
         SET status = 'consumed'
         WHERE state = ?`
      )
      .bind(state)
      .run();
    return jsonOk(parseJsonText(row.result_json));
  }
  if (row.status === "failed") {
    throw new HttpError(401, row.failure_code || "auth_flow_failed", "Authentication flow failed");
  }

  return jsonOk({ status: "pending" });
}

async function failOauthState(db, state, failureCode) {
  await db
    .prepare(
      `UPDATE oauth_states
       SET status = 'failed', failure_code = ?, used_at_ms = ?
       WHERE state = ?
         AND status NOT IN ('completed', 'consumed')`
    )
    .bind(failureCode, nowMs(), state)
    .run();
}

async function createSessionResponse(db, actor) {
  const timestamp = nowMs();
  const sessionId = prefixedId("ses");
  const sessionToken = randomToken(32);
  const tokenHash = await sha256Base64Url(sessionToken);
  const expiresAt = timestamp + SESSION_TTL_MS;

  await db
    .prepare(
      `INSERT INTO user_sessions
       (session_id, user_id, client_id, token_hash, created_at_ms, expires_at_ms)
       VALUES (?, ?, ?, ?, ?, ?)`
    )
    .bind(sessionId, actor.user_id, actor.client_id, tokenHash, timestamp, expiresAt)
    .run();

  return jsonOk({
    user_id: actor.user_id,
    login: actor.login,
    client_id: actor.client_id,
    key_id: actor.key_id,
    session_token: sessionToken,
    session_expires_at_ms: expiresAt,
    provider: actor.provider || null,
    display_name: actor.display_name || null,
    email: actor.email || null,
  });
}

async function loadOauthState(db, state) {
  const row = await db
    .prepare(
      `SELECT state, provider, client_identifier, client_public_key_jwk,
              pkce_verifier, ui_language, status, expires_at_ms
       FROM oauth_states
       WHERE state = ?`
    )
    .bind(state)
    .first();
  if (!row || row.expires_at_ms <= nowMs() || row.status === "consumed") {
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }
  return row;
}

async function loadOauthStateForPage(db, state) {
  const row = await db
    .prepare(
      `SELECT state, provider, client_identifier, client_public_key_jwk,
              pkce_verifier, ui_language, status, expires_at_ms
       FROM oauth_states
       WHERE state = ?`
    )
    .bind(state)
    .first();
  if (
    !row ||
    row.expires_at_ms <= nowMs() ||
    row.status === "consumed" ||
    row.status === "failed"
  ) {
    return null;
  }
  return row;
}

async function loadOauthStateForCallback(db, state) {
  const row = await db
    .prepare(
      `SELECT state, provider, client_identifier, client_public_key_jwk,
              pkce_verifier, ui_language, status, expires_at_ms
       FROM oauth_states
       WHERE state = ?`
    )
    .bind(state)
    .first();
  if (!row) {
    return null;
  }
  if (row.status !== "completed" && row.status !== "consumed" && row.expires_at_ms <= nowMs()) {
    return null;
  }
  return row;
}

async function assertAuthFailuresAllowed(db, bucket) {
  const timestamp = nowMs();
  const existing = await db
    .prepare(
      `SELECT window_ends_at_ms, used_count
       FROM auth_attempt_windows
       WHERE bucket_hash = ?`
    )
    .bind(bucket)
    .first();

  if (existing && existing.window_ends_at_ms > timestamp && existing.used_count >= AUTH_ATTEMPT_LIMIT) {
    throw new HttpError(429, "auth_rate_limited", "Too many failed sign-in attempts. Try again later.");
  }
}

async function recordOauthStartOrThrow(db, bucket) {
  const timestamp = nowMs();
  const existing = await db
    .prepare(
      `SELECT bucket_hash, window_started_at_ms, window_ends_at_ms, limit_count, used_count
       FROM oauth_start_windows
       WHERE bucket_hash = ?`
    )
    .bind(bucket)
    .first();

  let windowStarted = timestamp;
  let windowEnds = timestamp + OAUTH_START_WINDOW_MS;
  let usedCount = 0;

  if (existing && existing.window_ends_at_ms > timestamp) {
    windowStarted = existing.window_started_at_ms;
    windowEnds = existing.window_ends_at_ms;
    usedCount = Number(existing.used_count || 0);
  }

  if (usedCount >= OAUTH_START_LIMIT) {
    throw new HttpError(429, "auth_start_rate_limited", "Too many sign-in starts. Try again later.");
  }

  usedCount += 1;
  await db
    .prepare(
      `INSERT INTO oauth_start_windows
       (bucket_hash, window_started_at_ms, window_ends_at_ms, limit_count, used_count, updated_at_ms)
       VALUES (?, ?, ?, ?, ?, ?)
       ON CONFLICT(bucket_hash) DO UPDATE SET
         window_started_at_ms = excluded.window_started_at_ms,
         window_ends_at_ms = excluded.window_ends_at_ms,
         limit_count = excluded.limit_count,
         used_count = excluded.used_count,
         updated_at_ms = excluded.updated_at_ms`
    )
    .bind(bucket, windowStarted, windowEnds, OAUTH_START_LIMIT, usedCount, timestamp)
    .run();
}

async function oauthStartBucket(request, clientIdentifier, provider) {
  const ip = request.headers.get("cf-connecting-ip") || "unknown-ip";
  const userAgent = request.headers.get("user-agent") || "unknown-ua";
  const userAgentHash = await sha256Base64Url(userAgent);
  return sha256Base64Url(`oauth-start:${provider}:${clientIdentifier}:${ip}:${userAgentHash}`);
}

async function recordAuthFailure(db, bucket) {
  const timestamp = nowMs();
  const existing = await db
    .prepare(
      `SELECT bucket_hash, window_started_at_ms, window_ends_at_ms, limit_count, used_count
       FROM auth_attempt_windows
       WHERE bucket_hash = ?`
    )
    .bind(bucket)
    .first();

  let windowStarted = timestamp;
  let windowEnds = timestamp + AUTH_ATTEMPT_WINDOW_MS;
  let usedCount = 0;

  if (existing && existing.window_ends_at_ms > timestamp) {
    windowStarted = existing.window_started_at_ms;
    windowEnds = existing.window_ends_at_ms;
    usedCount = existing.used_count;
  }

  usedCount += 1;
  await db
    .prepare(
      `INSERT INTO auth_attempt_windows
       (bucket_hash, window_started_at_ms, window_ends_at_ms, limit_count, used_count, updated_at_ms)
       VALUES (?, ?, ?, ?, ?, ?)
       ON CONFLICT(bucket_hash) DO UPDATE SET
         window_started_at_ms = excluded.window_started_at_ms,
         window_ends_at_ms = excluded.window_ends_at_ms,
         limit_count = excluded.limit_count,
         used_count = excluded.used_count,
         updated_at_ms = excluded.updated_at_ms`
    )
    .bind(bucket, windowStarted, windowEnds, AUTH_ATTEMPT_LIMIT, usedCount, timestamp)
    .run();
}

async function authBucket(request, clientIdentifier, purpose) {
  const ip = request.headers.get("cf-connecting-ip") || "unknown-ip";
  return sha256Base64Url(`auth:${purpose}:${clientIdentifier}:${ip}`);
}

async function issueCaptchaChallenge(db, purpose, scope) {
  const timestamp = nowMs();
  const scopeHash = await captchaScopeHash(scope);
  const poolKey = await captchaPoolKey(purpose, scopeHash);
  let pool = await loadValidCaptchaPool(db, poolKey, purpose, scopeHash, timestamp);
  if (!pool) {
    await clearCaptchaPool(db, poolKey, null);
    pool = await generateCaptchaPool(db, poolKey, purpose, scopeHash, timestamp);
  }

  const nextIndex = pool.challengeIds.length === 0
    ? 0
    : Number(pool.nextIndex || 0) % pool.challengeIds.length;
  const challengeId = pool.challengeIds[nextIndex];
  const nextPoolIndex = pool.challengeIds.length <= 1 ? 0 : (nextIndex + 1) % pool.challengeIds.length;
  await db
    .prepare(
      `UPDATE captcha_refresh_pools
       SET next_index = ?, updated_at_ms = ?
       WHERE pool_key = ?`
    )
    .bind(nextPoolIndex, timestamp, poolKey)
    .run();

  const row = await db
    .prepare(
      `SELECT challenge_id, svg, expires_at_ms
       FROM captcha_challenges
       WHERE challenge_id = ?`
    )
    .bind(challengeId)
    .first();
  if (!row) {
    await clearCaptchaPool(db, poolKey, null);
    return issueCaptchaChallenge(db, purpose, scope);
  }

  return {
    challenge_id: row.challenge_id,
    svg: row.svg,
    expires_at_ms: row.expires_at_ms,
  };
}

async function loadValidCaptchaPool(db, poolKey, purpose, scopeHash, timestamp) {
  const poolRow = await db
    .prepare(
      `SELECT pool_key, challenge_ids_json, next_index, expires_at_ms
       FROM captcha_refresh_pools
       WHERE pool_key = ?
         AND purpose = ?
         AND scope_hash = ?`
    )
    .bind(poolKey, purpose, scopeHash)
    .first();
  if (!poolRow || poolRow.expires_at_ms <= timestamp) {
    return null;
  }

  let challengeIds;
  try {
    challengeIds = JSON.parse(poolRow.challenge_ids_json);
  } catch {
    return null;
  }
  if (!Array.isArray(challengeIds) || challengeIds.length !== CAPTCHA_REFRESH_POOL_SIZE) {
    return null;
  }

  const rows = await db
    .prepare(
      `SELECT challenge_id, purpose, scope_hash, svg, expires_at_ms, consumed_at_ms
       FROM captcha_challenges
       WHERE pool_key = ?`
    )
    .bind(poolKey)
    .all();
  const items = Array.isArray(rows.results) ? rows.results : [];
  if (items.length !== CAPTCHA_REFRESH_POOL_SIZE) {
    return null;
  }
  const byId = new Map(items.map((item) => [item.challenge_id, item]));
  for (const challengeId of challengeIds) {
    const item = byId.get(challengeId);
    if (
      !item ||
      item.purpose !== purpose ||
      item.scope_hash !== scopeHash ||
      item.expires_at_ms <= timestamp ||
      item.consumed_at_ms ||
      typeof item.svg !== "string" ||
      item.svg === "" ||
      !parseCaptchaArtwork(item.svg)
    ) {
      return null;
    }
  }

  return {
    challengeIds,
    nextIndex: poolRow.next_index,
  };
}

async function generateCaptchaPool(db, poolKey, purpose, scopeHash, timestamp) {
  const expiresAt = timestamp + CAPTCHA_TTL_MS;
  const challengeIds = [];
  const statements = [];
  for (let index = 0; index < CAPTCHA_REFRESH_POOL_SIZE; index += 1) {
    const challengeId = prefixedId("cap");
    const answer = randomCaptchaAnswer();
    const salt = randomToken(16);
    challengeIds.push(challengeId);
    statements.push(
      db
        .prepare(
          `INSERT INTO captcha_challenges
           (challenge_id, pool_key, purpose, scope_hash, salt, answer_hash, svg,
            issued_at_ms, expires_at_ms, attempts, max_attempts)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?)`
        )
        .bind(
          challengeId,
          poolKey,
          purpose,
          scopeHash,
          salt,
          await sha256Base64Url(`${salt}:${answer}`),
          JSON.stringify(buildCaptchaArtwork(answer)),
          timestamp,
          expiresAt,
          CAPTCHA_MAX_ATTEMPTS
        )
    );
  }

  statements.push(
    db
      .prepare(
        `INSERT INTO captcha_refresh_pools
         (pool_key, purpose, scope_hash, challenge_ids_json, next_index,
          created_at_ms, expires_at_ms, updated_at_ms)
         VALUES (?, ?, ?, ?, 0, ?, ?, ?)`
      )
      .bind(poolKey, purpose, scopeHash, JSON.stringify(challengeIds), timestamp, expiresAt, timestamp)
  );
  await db.batch(statements);
  return { challengeIds, nextIndex: 0 };
}

async function verifyCaptchaOrThrow(db, request, challengeIdValue, answerValue, purpose, scope) {
  const challengeId = String(challengeIdValue || "").trim();
  const answer = String(answerValue || "").trim().toUpperCase();
  const scopeHash = await captchaScopeHash(scope);
  const bucket = await captchaFailureBucket(request, purpose, scopeHash);

  await assertCaptchaFailureBudgetAvailable(db, bucket);

  if (!challengeId || !answer) {
    await recordCaptchaFailure(db, bucket);
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }

  const row = await db
    .prepare(
      `SELECT challenge_id, pool_key, purpose, scope_hash, salt, answer_hash,
              issued_at_ms, expires_at_ms, attempts, max_attempts, consumed_at_ms
       FROM captcha_challenges
       WHERE challenge_id = ?`
    )
    .bind(challengeId)
    .first();
  const timestamp = nowMs();
  if (
    !row ||
    row.purpose !== purpose ||
    row.scope_hash !== scopeHash ||
    row.expires_at_ms <= timestamp ||
    row.consumed_at_ms
  ) {
    await recordCaptchaFailure(db, bucket);
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }

  const attempts = Number(row.attempts || 0) + 1;
  await db
    .prepare(
      `UPDATE captcha_challenges
       SET attempts = ?
       WHERE challenge_id = ?`
    )
    .bind(attempts, challengeId)
    .run();

  const expectedHash = await sha256Base64Url(`${row.salt}:${answer}`);
  const solvedSlowEnough = timestamp - Number(row.issued_at_ms || 0) >= CAPTCHA_MIN_SOLVE_MS;
  const maxAttempts = Number(row.max_attempts || CAPTCHA_MAX_ATTEMPTS);
  if (attempts > maxAttempts || expectedHash !== row.answer_hash || !solvedSlowEnough) {
    if (attempts >= maxAttempts) {
      await db
        .prepare(
          `UPDATE captcha_challenges
           SET consumed_at_ms = ?
           WHERE challenge_id = ?`
        )
        .bind(timestamp, challengeId)
        .run();
    }
    await clearCaptchaPool(db, row.pool_key, attempts < maxAttempts ? challengeId : null);
    await recordCaptchaFailure(db, bucket);
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }

  await db
    .prepare(
      `UPDATE captcha_challenges
       SET consumed_at_ms = ?
       WHERE challenge_id = ?`
    )
    .bind(timestamp, challengeId)
    .run();
  await clearCaptchaPool(db, row.pool_key, null);
  await clearCaptchaFailures(db, bucket);
}

async function clearCaptchaPool(db, poolKey, preserveChallengeId) {
  if (preserveChallengeId) {
    await db
      .prepare(
        `DELETE FROM captcha_challenges
         WHERE pool_key = ?
           AND challenge_id <> ?`
      )
      .bind(poolKey, preserveChallengeId)
      .run();
  } else {
    await db
      .prepare(
        `DELETE FROM captcha_challenges
         WHERE pool_key = ?`
      )
      .bind(poolKey)
      .run();
  }
  await db
    .prepare(
      `DELETE FROM captcha_refresh_pools
       WHERE pool_key = ?`
    )
    .bind(poolKey)
    .run();
}

async function assertCaptchaFailureBudgetAvailable(db, bucket) {
  const timestamp = nowMs();
  const row = await db
    .prepare(
      `SELECT blocked_until_ms, window_ends_at_ms
       FROM captcha_failure_buckets
       WHERE bucket_hash = ?`
    )
    .bind(bucket)
    .first();
  if (row && row.window_ends_at_ms > timestamp && row.blocked_until_ms > timestamp) {
    throw new HttpError(429, "auth_flow_failed", "Authentication flow failed");
  }
}

async function recordCaptchaFailure(db, bucket) {
  const timestamp = nowMs();
  const existing = await db
    .prepare(
      `SELECT window_started_at_ms, window_ends_at_ms, attempt_count,
              consecutive_failures, blocked_until_ms
       FROM captcha_failure_buckets
       WHERE bucket_hash = ?`
    )
    .bind(bucket)
    .first();

  let windowStarted = timestamp;
  let windowEnds = timestamp + CAPTCHA_FAILURE_WINDOW_MS;
  let attemptCount = 0;
  let consecutiveFailures = 0;
  if (existing && existing.window_ends_at_ms > timestamp) {
    windowStarted = existing.window_started_at_ms;
    windowEnds = existing.window_ends_at_ms;
    attemptCount = Number(existing.attempt_count || 0);
    consecutiveFailures = Number(existing.consecutive_failures || 0);
  }

  attemptCount += 1;
  consecutiveFailures += 1;
  const backoffMs = Math.min(
    CAPTCHA_FAILURE_BACKOFF_MAX_MS,
    CAPTCHA_FAILURE_BACKOFF_BASE_MS * Math.max(1, 2 ** Math.max(0, consecutiveFailures - 1))
  );
  const blockedUntil = consecutiveFailures >= CAPTCHA_FAILURE_LOCKOUT_THRESHOLD
    ? timestamp + CAPTCHA_FAILURE_LOCKOUT_MS
    : timestamp + backoffMs;

  await db
    .prepare(
      `INSERT INTO captcha_failure_buckets
       (bucket_hash, window_started_at_ms, window_ends_at_ms, attempt_count,
        consecutive_failures, blocked_until_ms, last_attempt_at_ms)
       VALUES (?, ?, ?, ?, ?, ?, ?)
       ON CONFLICT(bucket_hash) DO UPDATE SET
         window_started_at_ms = excluded.window_started_at_ms,
         window_ends_at_ms = excluded.window_ends_at_ms,
         attempt_count = excluded.attempt_count,
         consecutive_failures = excluded.consecutive_failures,
         blocked_until_ms = excluded.blocked_until_ms,
         last_attempt_at_ms = excluded.last_attempt_at_ms`
    )
    .bind(bucket, windowStarted, windowEnds, attemptCount, consecutiveFailures, blockedUntil, timestamp)
    .run();
}

async function clearCaptchaFailures(db, bucket) {
  await db
    .prepare(
      `DELETE FROM captcha_failure_buckets
       WHERE bucket_hash = ?`
    )
    .bind(bucket)
    .run();
}

async function browserVerificationScopeHash(scope) {
  return sha256Base64Url(`browser:${String(scope || "").trim()}`);
}

const BROWSER_SIGNAL_DIGEST_KEYS = [
  "canvas_hash",
  "color_depth",
  "cookie_enabled",
  "device_memory",
  "elapsed_ms",
  "hardware_concurrency",
  "interaction_action",
  "interaction_change_events",
  "interaction_focus_events",
  "interaction_input_events",
  "interaction_render_to_action_ms",
  "interaction_pointer_x_pct",
  "interaction_pointer_y_pct",
  "interaction_trusted",
  "language",
  "languages_count",
  "local_storage",
  "max_touch_points",
  "screen_height",
  "screen_width",
  "session_storage",
  "timezone",
  "timezone_offset_minutes",
  "user_agent",
  "visibility_state",
  "webdriver",
  "webgl_renderer",
  "webgl_vendor",
  "window_has_focus",
];

function normalizeBrowserSignalDigest(signals) {
  const normalized = {};
  for (const key of BROWSER_SIGNAL_DIGEST_KEYS) {
    normalized[key] = signals[key];
  }
  return JSON.stringify(normalized);
}

function evaluateBrowserVerificationRisk(signals, workload, requestUserAgent) {
  let score = 0;
  const reasons = [];
  const lowerUserAgent = String(requestUserAgent || "").toLowerCase();
  for (const needle of ["headless", "phantom", "slimer", "selenium", "playwright", "curl", "wget", "python-requests"]) {
    if (lowerUserAgent.includes(needle)) {
      score += 70;
      reasons.push("automation_user_agent");
      break;
    }
  }
  if (signals.webdriver === true) {
    score += 70;
    reasons.push("webdriver_signal");
  }

  const basicScore = browserBasicSignalScore(signals);
  if (basicScore < BROWSER_VERIFICATION_MIN_SIGNAL_SCORE) {
    score += (BROWSER_VERIFICATION_MIN_SIGNAL_SCORE - basicScore) * 8;
    reasons.push("incomplete_browser_signals");
  }

  const capabilities = isPlainObject(workload.capabilities) ? workload.capabilities : {};
  const timings = isPlainObject(workload.timings) ? workload.timings : {};
  if (capabilities.crypto_subtle !== true) {
    score += 40;
    reasons.push("crypto_subtle_missing");
  }
  if (capabilities.worker_used !== true) {
    score += 15;
    reasons.push("worker_not_used");
  }
  if (capabilities.worker_supported !== true) {
    score += 15;
    reasons.push("worker_not_supported");
  }
  for (const key of ["pow_ms", "memory_ms", "worker_total_ms", "total_ms"]) {
    const value = browserTimingValue(timings[key]);
    if (value !== null && value <= 0) {
      score += 10;
      reasons.push(`impossible_timing_${key}`);
    }
  }
  if (String(signals.visibility_state || "") !== "visible") {
    score += 10;
    reasons.push("document_not_visible");
  }
  if (signals.window_has_focus !== true) {
    score += 10;
    reasons.push("window_not_focused");
  }
  if (signals.interaction_trusted !== true) {
    score += 25;
    reasons.push("untrusted_auth_action");
  }
  if (!positiveInt(signals.interaction_render_to_action_ms, 250, 86400000)) {
    score += 20;
    reasons.push("too_fast_auth_action");
  }
  const interactionEvents =
    Number(signals.interaction_focus_events || 0) +
    Number(signals.interaction_input_events || 0) +
    Number(signals.interaction_change_events || 0);
  if (interactionEvents <= 0) {
    score += 15;
    reasons.push("missing_interaction_events");
  }
  const pointerRisk = browserPointerRisk(signals);
  if (pointerRisk.score > 0) {
    score += pointerRisk.score;
    reasons.push(pointerRisk.reason);
  }

  const level =
    score >= BROWSER_VERIFICATION_HOSTILE_SCORE_THRESHOLD
      ? "hostile"
      : score >= BROWSER_VERIFICATION_SUSPICIOUS_SCORE_THRESHOLD
        ? "suspicious"
        : "normal";
  return {
    score,
    level,
    reasons: [...new Set(reasons)],
    delay_ms: level === "suspicious" ? BROWSER_VERIFICATION_TARPIT_MS : 0,
  };
}

function browserPointerRisk(signals) {
  const x = Number(signals.interaction_pointer_x_pct);
  const y = Number(signals.interaction_pointer_y_pct);
  if (!Number.isFinite(x) || !Number.isFinite(y)) {
    return { score: 45, reason: "missing_pointer_coordinates" };
  }
  const nearEdgeX = x <= 0.2 || x >= 99.8;
  const nearEdgeY = y <= 0.2 || y >= 99.8;
  if (nearEdgeX && nearEdgeY) {
    return { score: 55, reason: "corner_pointer_coordinates" };
  }
  if (nearEdgeX || nearEdgeY) {
    return { score: 45, reason: "edge_pointer_coordinates" };
  }
  if (Math.abs(x - 50) <= 0.15 && Math.abs(y - 50) <= 0.15) {
    return { score: 45, reason: "center_pointer_coordinates" };
  }
  return { score: 0, reason: "" };
}

function browserBasicSignalScore(signals) {
  let score = 0;
  score += positiveString(signals.language) ? 1 : 0;
  score += positiveInt(signals.languages_count, 1, 64) ? 1 : 0;
  score += positiveString(signals.timezone) ? 1 : 0;
  score += positiveInt(signals.screen_width, 320, 16384) && positiveInt(signals.screen_height, 240, 16384) ? 1 : 0;
  score += positiveInt(signals.color_depth, 16, 64) ? 1 : 0;
  score += positiveInt(signals.hardware_concurrency, 1, 256) ? 1 : 0;
  score += signals.cookie_enabled === true ? 1 : 0;
  score += signals.local_storage === true || signals.session_storage === true ? 1 : 0;
  score += positiveString(signals.webgl_vendor) || positiveString(signals.webgl_renderer) || positiveString(signals.canvas_hash) ? 1 : 0;
  score += positiveInt(signals.elapsed_ms, 10, BROWSER_VERIFICATION_TTL_MS) ? 1 : 0;
  return score;
}

function browserTimingValue(value) {
  return positiveInt(value, -86400000, 86400000) ? Number(value) : null;
}

async function browserVerificationMemoryResult(row, challengeId, nonce) {
  const bytes = Math.max(1024, Number(row.memory_size_kib || BROWSER_VERIFICATION_MEMORY_SIZE_KIB) * 1024);
  const wordCount = Math.max(256, Math.floor(bytes / 4));
  const rounds = Math.max(1, Math.min(16, Number(row.memory_rounds || BROWSER_VERIFICATION_MEMORY_ROUNDS)));
  const words = new Uint32Array(wordCount);
  let state = await seedToUint32(`${row.memory_seed}:${challengeId}:${nonce}`);

  for (let index = 0; index < wordCount; index += 1) {
    state = xorshift32((state + index + 0x9e3779b9) >>> 0);
    words[index] = state;
  }

  let cursor = state % wordCount;
  let accumulator = 0x9e3779b9;
  const steps = wordCount * rounds;
  for (let step = 0; step < steps; step += 1) {
    const value = words[cursor] >>> 0;
    accumulator = (((accumulator ^ value) >>> 0) + ((cursor + step) >>> 0)) >>> 0;
    state = xorshift32((state ^ value ^ accumulator) >>> 0);
    cursor = (cursor + value + state) % wordCount;
  }

  return accumulator.toString(16).padStart(8, "0") + state.toString(16).padStart(8, "0");
}

async function seedToUint32(seed) {
  return Number.parseInt((await sha256Hex(seed)).slice(0, 8), 16) >>> 0;
}

function xorshift32(value) {
  let state = value >>> 0;
  state = (state ^ ((state << 13) >>> 0)) >>> 0;
  state = (state ^ (state >>> 17)) >>> 0;
  state = (state ^ ((state << 5) >>> 0)) >>> 0;
  return state >>> 0;
}

async function captchaScopeHash(scope) {
  return sha256Base64Url(`scope:${String(scope || "").trim()}`);
}

async function captchaPoolKey(purpose, scopeHash) {
  return sha256Base64Url(`pool:${String(purpose || "").toLowerCase()}:${scopeHash || "_"}`);
}

async function captchaFailureBucket(request, purpose, scopeHash) {
  const ip = request.headers.get("cf-connecting-ip") || "unknown-ip";
  return sha256Base64Url(`captcha:${String(purpose || "").toLowerCase()}:${scopeHash || "_"}:${ip}`);
}

function randomCaptchaAnswer() {
  let answer = "";
  for (let index = 0; index < CAPTCHA_LENGTH; index += 1) {
    answer += CAPTCHA_ALPHABET[randomInt(0, CAPTCHA_ALPHABET.length - 1)];
  }
  return answer;
}

const CAPTCHA_GLYPH_SEGMENTS = {
  "2": ["a", "b", "g", "e", "d"],
  "3": ["a", "b", "g", "c", "d"],
  "4": ["f", "g", "b", "c"],
  "5": ["a", "f", "g", "c", "d"],
  "6": ["a", "f", "g", "c", "d", "e"],
  "7": ["a", "b", "c"],
  "8": ["a", "b", "c", "d", "e", "f", "g"],
  "9": ["a", "b", "c", "d", "f", "g"],
  S: ["a", "f", "g", "c", "d"],
  Z: ["a", "b", "g", "e", "d"],
  L: ["f", "e", "d"],
  G: ["a", "f", "e", "d", "c"],
};
const CAPTCHA_DECOY_GLYPHS = ["S", "Z", "L", "G"];

function buildCaptchaArtwork(answer) {
  const { width, height } = captchaDimensions(answer);
  const svg = buildCaptchaSvg(answer);
  return {
    kind: "captcha_tiles_v1",
    width,
    height,
    columns: CAPTCHA_TILE_COLUMNS,
    rows: CAPTCHA_TILE_ROWS,
    tiles: buildCaptchaTiles(svg, width, height),
  };
}

function captchaDimensions(answer) {
  const visualScale = 1.2;
  return {
    width: Math.round(Math.max(170, 42 + answer.length * 26) * visualScale),
    height: 60,
  };
}

function buildCaptchaSvg(answer) {
  const { width, height } = captchaDimensions(answer);
  const paddingX = 16;
  const slotWidth = ((width - paddingX * 2) / Math.max(answer.length, 1)) * 0.92;
  const digitPalette = ["#2563eb", "#15803d", "#c2410c", "#7c3aed", "#b91c1c", "#0f766e"];
  const noisePalette = ["#94a3b8", "#64748b", "#7c8ca1"];
  const digitParts = [];

  [...answer].forEach((char, index) => {
    const x = paddingX + slotWidth * index + randomInt(-4, 4);
    const y = 8 + randomInt(-3, 4);
    const rotate = randomInt(-12, 12);
    const skew = randomInt(18, 30);
    const scaleX = (randomInt(92, 110) / 100) * 1.2;
    const scaleY = (randomInt(88, 112) / 100) * 1.2;
    const fill = digitPalette[index % digitPalette.length];
    digitParts.push(
      `<g transform="translate(${fmt(x)} ${fmt(y)})"><g class="captcha-digit-slant" transform="rotate(${rotate} 9 18) skewX(${skew}) scale(${fmt(scaleX)} ${fmt(scaleY)})">${buildCaptchaGlyphShape(char, fill, false)}</g></g>`
    );
  });

  const decoyGlyphParts = [];
  if (randomInt(0, 100) < 76) {
    const glyph = CAPTCHA_DECOY_GLYPHS[randomInt(0, CAPTCHA_DECOY_GLYPHS.length - 1)];
    const x = paddingX + randomInt(0, Math.max(1, width - 44));
    const y = 8 + randomInt(-2, 6);
    const rotate = randomInt(-18, 18);
    const skew = randomInt(18, 32);
    const scaleX = (randomInt(92, 120) / 100) * 1.2;
    const scaleY = (randomInt(90, 118) / 100) * 1.2;
    const fill = digitPalette[randomInt(0, digitPalette.length - 1)];
    decoyGlyphParts.push(
      `<g class="captcha-decoy-letter" opacity="${fmt(randomInt(24, 36) / 100)}" transform="translate(${fmt(x)} ${fmt(y)})"><g transform="rotate(${rotate} 9 18) skewX(${skew}) scale(${fmt(scaleX)} ${fmt(scaleY)})" filter="url(#maskBlur)">${buildCaptchaGlyphShape(glyph, fill, true)}</g></g>`
    );
  }

  const noiseUnderParts = [];
  for (let index = 0; index < 10; index += 1) {
    const color = noisePalette[index % noisePalette.length];
    noiseUnderParts.push(
      `<path d="M ${randomInt(4, 18)} ${randomInt(8, height - 8)} C ${randomInt(30, 60)} ${randomInt(0, height)}, ${randomInt(80, 110)} ${randomInt(0, height)}, ${randomInt(120, width - 6)} ${randomInt(8, height - 8)}" stroke="${color}" stroke-width="1.15" stroke-opacity="${fmt(randomInt(24, 34) / 100)}" fill="none" filter="url(#noiseBlur)"/>`
    );
  }
  for (let index = 0; index < 22; index += 1) {
    const color = noisePalette[index % noisePalette.length];
    noiseUnderParts.push(
      `<circle cx="${randomInt(6, width - 6)}" cy="${randomInt(6, height - 6)}" r="${fmt(randomInt(4, 10) / 10)}" fill="${color}" fill-opacity="${fmt(randomInt(8, 16) / 100)}"/>`
    );
  }

  const noiseOverParts = [];
  for (let index = 0; index < 6; index += 1) {
    const color = digitPalette[index % digitPalette.length];
    noiseOverParts.push(
      `<path d="M ${randomInt(0, 18)} ${randomInt(8, height - 8)} C ${randomInt(36, 70)} ${randomInt(0, height)}, ${randomInt(86, 128)} ${randomInt(0, height)}, ${randomInt(136, width - 2)} ${randomInt(8, height - 8)}" stroke="${color}" stroke-width="${fmt(randomInt(12, 16) / 10)}" stroke-opacity="${fmt(randomInt(12, 22) / 100)}" fill="none" stroke-linecap="round" filter="url(#overlayBlur)"/>`
    );
  }

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" role="img" aria-hidden="true"><defs><linearGradient id="g" x1="0" x2="1" y1="0" y2="1"><stop offset="0%" stop-color="#f4f7fb"/><stop offset="100%" stop-color="#e7eef9"/></linearGradient><filter id="digitBlur" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="0.34"/></filter><filter id="maskBlur" x="-24%" y="-24%" width="148%" height="148%"><feGaussianBlur stdDeviation="0.95"/></filter><filter id="noiseBlur" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="0.55"/></filter><filter id="overlayBlur" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="0.68"/></filter></defs><rect width="${width}" height="${height}" rx="10" fill="url(#g)"/><rect x="1" y="1" width="${width - 2}" height="${height - 2}" rx="9" fill="none" stroke="#cbd5e1" stroke-opacity="0.65"/>${noiseUnderParts.join("")}${decoyGlyphParts.join("")}${digitParts.join("")}${noiseOverParts.join("")}</svg>`;
}

function buildCaptchaTiles(fullSvg, width, height) {
  const inner = fullSvg.slice(fullSvg.indexOf(">") + 1, fullSvg.lastIndexOf("</svg>"));
  const tiles = [];
  for (let row = 0; row < CAPTCHA_TILE_ROWS; row += 1) {
    for (let column = 0; column < CAPTCHA_TILE_COLUMNS; column += 1) {
      const x = (width / CAPTCHA_TILE_COLUMNS) * column;
      const y = (height / CAPTCHA_TILE_ROWS) * row;
      const tileWidth = column === CAPTCHA_TILE_COLUMNS - 1 ? width - x : width / CAPTCHA_TILE_COLUMNS;
      const tileHeight = row === CAPTCHA_TILE_ROWS - 1 ? height - y : height / CAPTCHA_TILE_ROWS;
      const tileSvg = `<svg xmlns="http://www.w3.org/2000/svg" width="${fmt(tileWidth)}" height="${fmt(tileHeight)}" viewBox="${fmt(x)} ${fmt(y)} ${fmt(tileWidth)} ${fmt(tileHeight)}" role="img" aria-hidden="true">${inner}</svg>`;
      tiles.push({
        x: Number(fmt(x)),
        y: Number(fmt(y)),
        width: Number(fmt(tileWidth)),
        height: Number(fmt(tileHeight)),
        src: svgDataUri(tileSvg),
      });
    }
  }
  return tiles;
}

function svgDataUri(svg) {
  return `data:image/svg+xml;base64,${btoa(svg)}`;
}

function buildCaptchaGlyphShape(glyph, fill, decoyGlyph) {
  const segments = CAPTCHA_GLYPH_SEGMENTS[glyph] || CAPTCHA_GLYPH_SEGMENTS["8"];
  const segmentRects = {
    a: [3, 0, 12, 3],
    b: [15, 2, 3, 14],
    c: [15, 18, 3, 14],
    d: [3, 32, 12, 3],
    e: [0, 18, 3, 14],
    f: [0, 2, 3, 14],
    g: [3, 16, 12, 3],
  };
  const coreParts = segments.map((segment) =>
    buildCaptchaSegmentFragments(distortCaptchaSegmentBox(segmentRects[segment], decoyGlyph), fill, false)
  );
  if (decoyGlyph) {
    return `<g class="captcha-decoy-glyph-core">${coreParts.join("")}</g>`;
  }

  const missingSegments = Object.keys(segmentRects).filter((segment) => !segments.includes(segment));
  const maskParts = [];
  if (missingSegments.length > 0) {
    shuffle(missingSegments);
    const decoyCount = Math.min(missingSegments.length, randomInt(2, 3));
    for (let index = 0; index < decoyCount; index += 1) {
      maskParts.push(
        buildCaptchaSegmentFragments(distortCaptchaSegmentBox(segmentRects[missingSegments[index]], true), fill, true)
      );
    }
  }
  if (randomInt(0, 100) < 85) {
    maskParts.push(
      `<path d="M ${randomInt(1, 6)} ${randomInt(4, 11)} Q ${randomInt(8, 12)} ${randomInt(10, 22)}, ${randomInt(11, 17)} ${randomInt(24, 33)}" stroke="${fill}" stroke-width="1.2" stroke-opacity="${fmt(randomInt(12, 20) / 100)}" fill="none" stroke-linecap="round" filter="url(#noiseBlur)"/>`
    );
  }
  if (randomInt(0, 100) < 80) {
    maskParts.push(
      `<path d="${polygonPath([
        [randomInt(2, 7), randomInt(6, 12)],
        [randomInt(9, 14), randomInt(8, 14)],
        [randomInt(11, 16), randomInt(20, 26)],
        [randomInt(4, 9), randomInt(18, 27)],
      ])}" fill="${fill}" fill-opacity="${fmt(randomInt(12, 22) / 100)}" filter="url(#maskBlur)"/>`
    );
  }

  return `<g class="captcha-digit-core">${coreParts.join("")}</g><g class="captcha-digit-mask" opacity="0.86">${maskParts.join("")}</g>`;
}

function buildCaptchaSegmentFragments(box, fill, decoy = false) {
  const [x, y, width, height] = box;
  const horizontal = width >= height;
  const fragmentCount = decoy ? randomInt(1, 2) : randomInt(2, 4);
  const gap = decoy ? 0 : randomInt(1, 2);
  const opacity = decoy ? randomInt(16, 30) / 100 : randomInt(70, 88) / 100;
  const parts = [];

  for (let index = 0; index < fragmentCount; index += 1) {
    let points;
    if (horizontal) {
      const fragmentWidth = (width - (fragmentCount - 1) * gap) / fragmentCount;
      const fragmentX = x + (fragmentWidth + gap) * index;
      points = [
        [fragmentX + jitter(0.8), y + jitter(0.5)],
        [fragmentX + fragmentWidth - 0.8 + jitter(0.8), y + jitter(0.5)],
        [fragmentX + fragmentWidth + 0.4 + jitter(0.6), y + height / 2 + jitter(0.5)],
        [fragmentX + fragmentWidth - 0.8 + jitter(0.8), y + height + jitter(0.5)],
        [fragmentX + jitter(0.8), y + height + jitter(0.5)],
        [fragmentX - 0.4 + jitter(0.6), y + height / 2 + jitter(0.5)],
      ];
    } else {
      const fragmentHeight = (height - (fragmentCount - 1) * gap) / fragmentCount;
      const fragmentY = y + (fragmentHeight + gap) * index;
      points = [
        [x + width / 2 + jitter(0.5), fragmentY - 0.4 + jitter(0.6)],
        [x + width + jitter(0.5), fragmentY + jitter(0.8)],
        [x + width + jitter(0.5), fragmentY + fragmentHeight - 0.8 + jitter(0.8)],
        [x + width / 2 + jitter(0.5), fragmentY + fragmentHeight + jitter(0.6)],
        [x + jitter(0.5), fragmentY + fragmentHeight - 0.8 + jitter(0.8)],
        [x + jitter(0.5), fragmentY + jitter(0.8)],
      ];
    }

    const filter = decoy
      ? ' filter="url(#maskBlur)"'
      : randomInt(0, 100) < 38
        ? ' filter="url(#digitBlur)"'
        : "";
    parts.push(`<path d="${polygonPath(points)}" fill="${fill}" fill-opacity="${fmt(opacity)}"${filter}/>`);
  }

  return parts.join("");
}

function distortCaptchaSegmentBox(box, decoy = false) {
  let [x, y, width, height] = box;
  const range = decoy ? 3 : 2;
  x += randomInt(-range, range);
  y += randomInt(-range, range);
  if (width >= height) {
    width = Math.max(8, width + randomInt(-3, 3));
    height = Math.max(3, height + randomInt(-2, 2));
  } else {
    width = Math.max(3, width + randomInt(-2, 2));
    height = Math.max(9, height + randomInt(-3, 3));
  }
  return [x, y, width, height];
}

function polygonPath(points) {
  return `${points
    .map(([x, y], index) => `${index === 0 ? "M" : "L"} ${fmt(x)} ${fmt(y)}`)
    .join(" ")} Z`;
}

function jitter(range) {
  return randomInt(Math.round(-range * 100), Math.round(range * 100)) / 100;
}

function shuffle(values) {
  for (let index = values.length - 1; index > 0; index -= 1) {
    const swapIndex = randomInt(0, index);
    [values[index], values[swapIndex]] = [values[swapIndex], values[index]];
  }
}

function fmt(value) {
  return Number(value).toFixed(2);
}

function oauthRedirectUri(request, env) {
  return (
    env.NETSTITCH_GOOGLE_REDIRECT_URI ||
    `${new URL(request.url).origin}/v1/auth/google/callback`
  );
}

async function exchangeGoogleCode(request, env, code, pkceVerifier) {
  const clientId = requireEnv(env, "NETSTITCH_GOOGLE_CLIENT_ID");
  const clientSecret = requireEnv(env, "NETSTITCH_GOOGLE_CLIENT_SECRET");
  const tokenResponse = await fetch("https://oauth2.googleapis.com/token", {
    method: "POST",
    headers: { "content-type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({
      code,
      client_id: clientId,
      client_secret: clientSecret,
      redirect_uri: oauthRedirectUri(request, env),
      grant_type: "authorization_code",
      code_verifier: pkceVerifier,
    }),
  });
  const tokenBody = await tokenResponse.json().catch(() => ({}));
  if (!tokenResponse.ok || typeof tokenBody.id_token !== "string") {
    throw new HttpError(
      401,
      normalizeFailureCode(`google_token_${tokenBody.error || "exchange_failed"}`),
      "Authentication flow failed"
    );
  }

  const infoResponse = await fetch(
    `https://oauth2.googleapis.com/tokeninfo?id_token=${encodeURIComponent(tokenBody.id_token)}`
  );
  const info = await infoResponse.json().catch(() => ({}));
  if (!infoResponse.ok || info.aud !== clientId || !info.sub) {
    throw new HttpError(401, "google_tokeninfo_failed", "Authentication flow failed");
  }
  if (info.iss !== "https://accounts.google.com" && info.iss !== "accounts.google.com") {
    throw new HttpError(401, "google_issuer_invalid", "Authentication flow failed");
  }

  return info;
}

function normalizeFailureCode(value) {
  return String(value || "auth_flow_failed")
    .toLowerCase()
    .replace(/[^a-z0-9_:-]+/g, "_")
    .replace(/_+/g, "_")
    .replace(/^_+|_+$/g, "")
    .slice(0, 80) || "auth_flow_failed";
}

function callbackFailureCode(code) {
  const normalized = normalizeFailureCode(code);
  if (normalized === "missing_field") {
    return "google_code_missing";
  }
  return normalized;
}

async function upsertGoogleIdentitySession(env, oauthState, googleInfo) {
  const db = env.DB;
  const timestamp = nowMs();
  const provider = "google";
  const subjectHash = await identityHmacBase64Url(env, `${provider}:${googleInfo.sub}`);
  const email = typeof googleInfo.email === "string" ? googleInfo.email.trim().toLowerCase() : "";
  const emailHash = email ? await identityHmacBase64Url(env, `email:${email}`) : null;
  const emailVerified = googleInfo.email_verified === "true" || googleInfo.email_verified === true;
  const displayName =
    typeof googleInfo.name === "string" && googleInfo.name.trim()
      ? googleInfo.name.trim().slice(0, 120)
      : null;
  let identity = await db
    .prepare(
      `SELECT user_id
       FROM user_identities
       WHERE provider = ?
         AND provider_subject_hash = ?`
    )
    .bind(provider, subjectHash)
    .first();

  let userId = identity?.user_id;
  if (!userId) {
    userId = prefixedId("usr");
    const login = `google_${subjectHash.slice(0, 24)}`;
    await db.batch([
      db
        .prepare(
          `INSERT INTO users
           (user_id, login, password_salt, password_hash, password_params_json, created_at_ms, last_login_at_ms)
           VALUES (?, ?, '', '', '{"algorithm":"oauth-only"}', ?, ?)`
        )
        .bind(userId, login, timestamp, timestamp),
      db
        .prepare(
          `INSERT INTO user_identities
           (provider, provider_subject_hash, user_id, email_hash, email_verified,
            display_name, created_at_ms, last_seen_at_ms)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)`
        )
        .bind(provider, subjectHash, userId, emailHash, emailVerified ? 1 : 0, null, timestamp, timestamp),
    ]);
  } else {
    await db.batch([
      db
        .prepare(
          `UPDATE user_identities
           SET email_hash = ?, email_verified = ?, display_name = ?, last_seen_at_ms = ?
           WHERE provider = ?
             AND provider_subject_hash = ?`
        )
        .bind(emailHash, emailVerified ? 1 : 0, null, timestamp, provider, subjectHash),
      db
        .prepare(
          `UPDATE users
           SET last_login_at_ms = ?
           WHERE user_id = ?`
        )
        .bind(timestamp, userId),
    ]);
  }

  let client = await db
    .prepare(
      `SELECT client_id
       FROM clients
       WHERE client_identifier = ?`
    )
    .bind(oauthState.client_identifier)
    .first();

  if (!client) {
    const clientId = prefixedId("cl");
    await db
      .prepare(
        `INSERT INTO clients (client_id, user_id, client_identifier, created_at_ms, last_seen_at_ms)
         VALUES (?, ?, ?, ?, ?)`
      )
      .bind(clientId, userId, oauthState.client_identifier, timestamp, timestamp)
      .run();
    client = { client_id: clientId };
  } else {
    await db
      .prepare(
        `UPDATE clients
         SET user_id = ?, last_seen_at_ms = ?
         WHERE client_id = ?`
      )
      .bind(userId, timestamp, client.client_id)
      .run();
  }

  const keyId = prefixedId("key");
  await db
    .prepare(
      `INSERT INTO client_keys
       (key_id, client_id, user_id, public_key_jwk, status, created_at_ms)
       VALUES (?, ?, ?, ?, 'active', ?)`
    )
    .bind(keyId, client.client_id, userId, oauthState.client_public_key_jwk, timestamp)
    .run();

  await audit(db, "user", "google_oauth_login", userId, userId, {
    client_id: client.client_id,
    provider,
  });

  const response = await createSessionResponse(db, {
    user_id: userId,
    login: "Google",
    client_id: client.client_id,
    key_id: keyId,
    provider,
    display_name: displayName,
    email: email || null,
  });
  return parseJsonText(await response.text());
}

function requireEnv(env, name) {
  const value = env[name] || "";
  if (!value) {
    throw new HttpError(500, "missing_auth_config", `${name} is not configured`);
  }
  return value;
}

function htmlResponse(html, status = 200) {
  return new Response(html, {
    status,
    headers: {
      "content-type": "text/html; charset=utf-8",
      "cache-control": "no-store",
    },
  });
}

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

async function pkceChallenge(verifier) {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(verifier));
  return base64Url(new Uint8Array(digest));
}

function normalizeTag(value, field = "tag") {
  const tag = requireString(value, field).trim().toLowerCase();
  if (tag.length < 1 || tag.length > TAG_MAX_LENGTH) {
    throw new HttpError(400, "invalid_tag", `Tag must be 1..${TAG_MAX_LENGTH} characters`);
  }
  if (!/^[a-z0-9 ._-]+$/.test(tag) || !/[a-z0-9]/.test(tag)) {
    throw new HttpError(400, "invalid_tag", "Tag may contain ASCII letters, digits, spaces, '-', '_' and '.' and must include a letter or digit");
  }
  return tag;
}

function normalizeObservationTags(value) {
  if (value === null || value === undefined) {
    return [];
  }
  if (!Array.isArray(value)) {
    throw new HttpError(400, "invalid_tags", "tags must be an array");
  }
  const tags = [...new Set(value.map((tag) => normalizeTag(tag)))];
  if (tags.length > TAGS_PER_OBSERVATION_LIMIT) {
    throw new HttpError(400, "too_many_row_tags", `One observation may have at most ${TAGS_PER_OBSERVATION_LIMIT} tags`);
  }
  return tags.sort();
}

async function userTagCount(db, userId) {
  const row = await db
    .prepare(`SELECT COUNT(*) AS count FROM user_tags WHERE user_id = ? AND expires_at_ms > ?`)
    .bind(userId, nowMs())
    .first();
  return Number(row?.count || 0);
}

async function ensureUserTags(db, userId, tags, timestamp) {
  if (!tags.length) {
    return userTagCount(db, userId);
  }
  const uniqueTags = [...new Set(tags)].sort();
  const placeholders = uniqueTags.map(() => "?").join(",");
  const existing = await db
    .prepare(`SELECT tag FROM user_tags WHERE user_id = ? AND expires_at_ms > ? AND tag IN (${placeholders})`)
    .bind(userId, timestamp, ...uniqueTags)
    .all();
  const existingNames = new Set((existing.results || []).map((row) => row.tag));
  const currentCount = await userTagCount(db, userId);
  const newCount = uniqueTags.filter((tag) => !existingNames.has(tag)).length;
  if (currentCount + newCount > TAGS_PER_USER_LIMIT) {
    throw new HttpError(409, "tag_limit_reached", `An account may use at most ${TAGS_PER_USER_LIMIT} active tags`);
  }
  await db.batch(
    uniqueTags.map((tag) => db
      .prepare(
        `INSERT INTO user_tags(user_id, tag, created_at_ms, updated_at_ms, expires_at_ms)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(user_id, tag) DO UPDATE SET
           updated_at_ms = excluded.updated_at_ms,
           expires_at_ms = MAX(user_tags.expires_at_ms, excluded.expires_at_ms)`
      )
      .bind(userId, tag, timestamp, timestamp, timestamp + RETENTION_MS))
  );
  return currentCount + newCount;
}

async function listTags(request, url, db) {
  await cleanupExpiredRows(db, nowMs());
  const query = normalizeSearch(url.searchParams.get("query") || "").toLowerCase();
  const pattern = `%${query}%`;
  const actor = await optionalUser(request, db);
  const actorUserId = actor?.user_id || "";
  const result = await db
    .prepare(
      `SELECT tag,
              COUNT(DISTINCT user_id) AS user_count,
              MAX(CASE WHEN user_id = ? THEN 1 ELSE 0 END) AS is_own
       FROM user_tags
       WHERE expires_at_ms > ? AND tag LIKE ?
       GROUP BY tag
       ORDER BY is_own DESC, tag ASC
       LIMIT 100`
    )
    .bind(actorUserId, nowMs(), pattern)
    .all();
  const ownCount = actor ? await userTagCount(db, actor.user_id) : 0;
  return jsonOk({
    items: (result.results || []).map((item) => ({
      tag: item.tag,
      user_count: Number(item.user_count || 0),
      is_own: Number(item.is_own || 0) > 0,
    })),
    own_count: ownCount,
    limit: TAGS_PER_USER_LIMIT,
  });
}

async function deleteTag(request, db, actor) {
  const body = await readJson(request);
  const tag = normalizeTag(body.tag);
  const result = await db.batch([
    db.prepare(`DELETE FROM observation_tags WHERE tag_user_id = ? AND tag = ?`).bind(actor.user_id, tag),
    db.prepare(`DELETE FROM user_tags WHERE user_id = ? AND tag = ?`).bind(actor.user_id, tag),
  ]);
  await audit(db, "tag", "delete", tag, actor.user_id, { tag });
  return jsonOk({ deleted: Number(result?.[1]?.meta?.changes || 0), own_count: await userTagCount(db, actor.user_id), limit: TAGS_PER_USER_LIMIT });
}

async function listApps(url, db) {
  const query = normalizeSearch(url.searchParams.get("query") || "");
  const publisher = normalizeSearch(url.searchParams.get("publisher") || "");
  const source = normalizeSearch(url.searchParams.get("source") || "");
  const tagQuery = normalizeSearch(url.searchParams.get("tag") || "").toLowerCase();
  const limit = boundedLimit(url.searchParams.get("limit"), 50, 100);
  const conditions = [`c.status = 'active'`];
  const params = [];

  if (query) {
    const pattern = `%${query}%`;
    conditions.push(`c.display_name LIKE ?`);
    params.push(pattern);
  }
  if (publisher) {
    const pattern = `%${publisher}%`;
    conditions.push(`c.publisher_name LIKE ?`);
    params.push(pattern);
  }
  if (source) {
    conditions.push(`p.author_signature_norm LIKE ?`);
    params.push(`%${source.toLowerCase()}%`);
  }
  if (tagQuery) {
    conditions.push(
      `EXISTS (
         SELECT 1 FROM observation_tags ot
         WHERE ot.visibility = 'public'
           AND ot.app_id = r.app_id
           AND ot.owner_user_id = ''
           AND ot.ip = r.ip
           AND ot.port = r.port
           AND LOWER(ot.protocol) = LOWER(r.protocol)
           AND ot.expires_at_ms > ?
           AND ot.tag LIKE ?
       )`
    );
    params.push(nowMs(), `%${tagQuery}%`);
  }

  const result = await db
    .prepare(
      `SELECT c.app_id,
              c.display_name,
              c.publisher_name,
              c.short_app_id,
              c.updated_at_ms,
              COUNT(DISTINCT r.ip || ':' || r.port || ':' || LOWER(r.protocol)) AS endpoint_count,
              COUNT(DISTINCT r.ip || ':' || r.port || ':' || LOWER(r.protocol)) AS available_row_count,
              COUNT(DISTINCT r.author_user_id) AS author_count,
              GROUP_CONCAT(DISTINCT p.author_signature) AS author_signatures,
              MAX(r.last_seen_ms) AS last_seen_ms
       FROM app_catalog c
       LEFT JOIN observation_author_rows r
              ON r.app_id = c.app_id
             AND LOWER(r.visibility) = 'public'
             AND r.expires_at_ms > ?
       LEFT JOIN author_profiles p ON p.author_user_id = r.author_user_id
       WHERE ${conditions.join(" AND ")}
       GROUP BY c.app_id, c.display_name, c.publisher_name, c.short_app_id, c.updated_at_ms
       HAVING COUNT(DISTINCT r.ip || ':' || r.port || ':' || LOWER(r.protocol)) > 0
       ORDER BY c.display_name ASC, c.publisher_name ASC, c.app_id ASC
       LIMIT ?`
    )
    .bind(nowMs(), ...params, limit)
    .all();

  const items = (result.results || []).map((row) => {
    const { author_signatures: authorSignatures, ...app } = row;
    return {
      ...app,
      authors: splitAuthorSignatures(authorSignatures),
    };
  });

  return jsonOk({ items });
}

function splitAuthorSignatures(value) {
  if (!value) {
    return [];
  }
  return String(value)
    .split(",")
    .map((item) => item.trim())
    .filter((item, index, items) => item && items.indexOf(item) === index);
}

async function listUserApps(db, userId, tagQuery = "") {
  const result = await db
    .prepare(
      `WITH app_totals AS (
         SELECT app_id,
                LOWER(visibility) AS visibility,
                COUNT(DISTINCT ip || ':' || port || ':' || LOWER(protocol)) AS total_endpoint_count
         FROM observation_author_rows
         WHERE expires_at_ms > ?
         GROUP BY app_id, LOWER(visibility)
       )
       SELECT c.app_id,
              c.display_name,
              c.publisher_name,
              p.author_signature,
              CASE LOWER(r.visibility)
                WHEN 'private' THEN 'private'
                ELSE 'public'
              END AS visibility,
              COUNT(DISTINCT r.ip || ':' || r.port || ':' || LOWER(r.protocol)) AS endpoint_count,
              COUNT(DISTINCT r.ip || ':' || r.port || ':' || LOWER(r.protocol)) AS available_row_count,
              COALESCE(t.total_endpoint_count, 0) AS total_endpoint_count,
              MAX(r.updated_at_ms) AS last_uploaded_at_ms
       FROM observation_author_rows r
       JOIN app_catalog c ON c.app_id = r.app_id
       LEFT JOIN author_profiles p ON p.author_user_id = r.author_user_id
       LEFT JOIN app_totals t ON t.app_id = r.app_id AND t.visibility = LOWER(r.visibility)
       WHERE r.author_user_id = ?
         AND r.expires_at_ms > ?
         AND (? = '' OR EXISTS (
           SELECT 1 FROM observation_tags ot
           WHERE ot.visibility = LOWER(r.visibility)
             AND ot.app_id = r.app_id
             AND ot.owner_user_id = CASE WHEN LOWER(r.visibility) = 'private' THEN r.author_user_id ELSE '' END
             AND ot.ip = r.ip
             AND ot.port = r.port
             AND LOWER(ot.protocol) = LOWER(r.protocol)
             AND ot.expires_at_ms > ?
             AND ot.tag LIKE ?
         ))
       GROUP BY c.app_id, c.display_name, c.publisher_name, p.author_signature, LOWER(r.visibility), t.total_endpoint_count
       ORDER BY last_uploaded_at_ms DESC, c.display_name ASC`
    )
    .bind(nowMs(), userId, nowMs(), tagQuery, nowMs(), `%${tagQuery}%`)
    .all();

  return jsonOk({ items: result.results || [] });
}

async function getQuota(url, db) {
  const clientIdentifier = requireString(
    url.searchParams.get("client_identifier"),
    "client_identifier"
  );
  const upload = await quotaSnapshot(db, clientIdentifier, "upload", false);
  const download = await quotaSnapshot(db, clientIdentifier, "download", false);
  return jsonOk({ upload, download });
}

async function getObservations(request, url, db) {
  await cleanupExpiredRows(db, nowMs());

  const appId = requireString(url.searchParams.get("app_id"), "app_id");
  const clientIdentifier = requireString(
    url.searchParams.get("client_identifier"),
    "client_identifier"
  );
  const operationId = normalizeQuotaOperationId(url.searchParams.get("operation_id"));
  const authorUserId = url.searchParams.get("author_user_id");
  const visibilityScope = normalizeObservationVisibilityScope(url.searchParams.get("visibility"));
  const limit = boundedLimit(url.searchParams.get("limit"), 500, 1000);
  const offset = boundedOffset(url.searchParams.get("offset"));
  const ipQuery = normalizeSearch(url.searchParams.get("ip") || "");
  const domainQuery = normalizeSearch(url.searchParams.get("domain") || "");
  const portQuery = normalizeSearch(url.searchParams.get("port") || "");
  const sourceQuery = normalizeSearch(url.searchParams.get("source") || "");
  const tagQuery = normalizeSearch(url.searchParams.get("tag") || "").toLowerCase();
  const protocolQuery = normalizeSearch(url.searchParams.get("protocol") || "").toLowerCase();
  const conditions = [`r.app_id = ?`, `r.expires_at_ms > ?`];
  const params = [appId, nowMs()];

  if (protocolQuery) {
    if (!["tcp", "udp", "other"].includes(protocolQuery)) {
      throw new HttpError(400, "invalid_protocol", "Protocol filter must be tcp, udp or other");
    }
    conditions.push(`LOWER(r.protocol) = ?`);
    params.push(protocolQuery);
  }
  if (ipQuery) {
    conditions.push(`r.ip LIKE ?`);
    params.push(`%${ipQuery}%`);
  }
  if (domainQuery) {
    conditions.push(`(COALESCE(r.domain_verified, '') LIKE ? OR COALESCE(r.domain_raw, '') LIKE ?)`);
    params.push(`%${domainQuery}%`, `%${domainQuery}%`);
  }
  if (portQuery) {
    conditions.push(`CAST(r.port AS TEXT) LIKE ?`);
    params.push(`%${portQuery}%`);
  }
  if (sourceQuery) {
    conditions.push(`p.author_signature_norm LIKE ?`);
    params.push(`%${sourceQuery.toLowerCase()}%`);
  }
  if (tagQuery) {
    conditions.push(
      `EXISTS (
         SELECT 1 FROM observation_tags ot
         WHERE ot.visibility = LOWER(r.visibility)
           AND ot.app_id = r.app_id
           AND ot.owner_user_id = CASE WHEN LOWER(r.visibility) = 'private' THEN r.author_user_id ELSE '' END
           AND ot.ip = r.ip
           AND ot.port = r.port
           AND LOWER(ot.protocol) = LOWER(r.protocol)
           AND ot.expires_at_ms > ?
           AND ot.tag LIKE ?
       )`
    );
    params.push(nowMs(), `%${tagQuery}%`);
  }

  let actor = null;
  if (authorUserId || visibilityScope === "private") {
    actor = await requireUser(request, db);
  } else if (visibilityScope === "all") {
    actor = await optionalUser(request, db);
  }

  if (authorUserId) {
    if (actor.user_id !== authorUserId) {
      throw new HttpError(403, "author_scope_forbidden", "Author scoped download requires the same logged in user");
    }
    conditions.push(`r.author_user_id = ?`);
    params.push(authorUserId);
    if (visibilityScope !== "all") {
      conditions.push(`LOWER(r.visibility) = ?`);
      params.push(visibilityScope);
    }
  } else if (visibilityScope === "private") {
    conditions.push(`LOWER(r.visibility) = ?`);
    params.push("private");
    conditions.push(`r.author_user_id = ?`);
    params.push(actor.user_id);
  } else if (visibilityScope === "all") {
    if (actor) {
      conditions.push(`(LOWER(r.visibility) = 'public' OR (LOWER(r.visibility) = 'private' AND r.author_user_id = ?))`);
      params.push(actor.user_id);
    } else {
      conditions.push(`LOWER(r.visibility) = ?`);
      params.push("public");
    }
  } else {
    conditions.push(`LOWER(r.visibility) = ?`);
    params.push("public");
  }

  const spendDownloadQuota = operationId
    ? await quotaOperationRequiresSpend(db, clientIdentifier, "download", operationId)
    : offset === 0;
  const downloadQuota = await quotaSnapshot(db, clientIdentifier, "download", spendDownloadQuota);
  if (operationId && spendDownloadQuota) {
    await recordQuotaOperation(db, clientIdentifier, "download", operationId, downloadQuota.window_ends_at_ms);
  }

  const countResult = await db
    .prepare(
      `SELECT COUNT(*) AS total
       FROM (
         SELECT r.app_id,
                LOWER(r.visibility) AS visibility,
                r.ip,
                r.port,
                LOWER(r.protocol) AS protocol
         FROM observation_author_rows r
         LEFT JOIN author_profiles p ON p.author_user_id = r.author_user_id
         WHERE ${conditions.join(" AND ")}
         GROUP BY r.app_id, LOWER(r.visibility), r.ip, r.port, LOWER(r.protocol)
       ) collapsed_observations`
    )
    .bind(...params)
    .first();
  const total = Number(countResult?.total || 0);

  const result = await db
    .prepare(
      `SELECT r.app_id,
              CASE
                WHEN SUM(CASE WHEN LOWER(r.visibility) = 'private' THEN 1 ELSE 0 END) > 0 THEN 'private'
                ELSE 'public'
              END AS visibility,
              COALESCE(GROUP_CONCAT(DISTINCT p.author_signature), '') AS author_signature,
              r.ip AS remote_ip,
              r.port AS remote_port,
              CASE LOWER(r.protocol)
                WHEN 'tcp' THEN 'Tcp'
                WHEN 'udp' THEN 'Udp'
                ELSE 'Other'
              END AS protocol,
              CASE
                WHEN SUM(COALESCE(r.successful_hits, 0)) > 0 THEN 'Established'
                WHEN SUM(COALESCE(r.failed_hits, 0)) > 0 THEN 'Failed'
                ELSE CASE LOWER(MAX(r.connection_state))
                  WHEN 'attempting' THEN 'Attempting'
                  WHEN 'established' THEN 'Established'
                  WHEN 'closing' THEN 'Closing'
                  WHEN 'failed' THEN 'Failed'
                  ELSE 'Unknown'
                END
              END AS connection_state,
              SUM(COALESCE(r.requests, 0)) AS requests,
              MIN(r.first_seen_ms) AS first_seen_ms,
              MAX(r.last_seen_ms) AS last_seen_ms,
              SUM(COALESCE(r.failed_hits, 0)) AS failed_hits,
              SUM(COALESCE(r.successful_hits, 0)) AS successful_hits,
              MAX(r.domain_raw) AS domain_raw,
              MAX(r.domain_verified) AS domain_verified,
              CASE
                WHEN SUM(CASE WHEN LOWER(r.domain_status) = 'verified' THEN 1 ELSE 0 END) > 0 THEN 'Verified'
                ELSE CASE LOWER(MAX(r.domain_status))
                  WHEN 'mismatch' THEN 'Mismatch'
                  WHEN 'unresolved' THEN 'Unresolved'
                  WHEN 'invalid' THEN 'Invalid'
                  ELSE 'None'
                END
              END AS domain_status,
              CASE
                WHEN SUM(CASE WHEN LOWER(r.trust_level) = 'communityverified' THEN 1 ELSE 0 END) > 0 THEN 'CommunityVerified'
                WHEN SUM(CASE WHEN LOWER(r.trust_level) = 'verifieduploadcandidate' THEN 1 ELSE 0 END) > 0 THEN 'VerifiedUploadCandidate'
                WHEN SUM(CASE WHEN LOWER(r.trust_level) = 'cloudimportuntrusted' THEN 1 ELSE 0 END) > 0 THEN 'CloudImportUntrusted'
                ELSE 'BlockedOrSuspect'
              END AS trust_level,
              CASE
                WHEN SUM(CASE WHEN LOWER(r.source_kind) = 'verifiedupload' THEN 1 ELSE 0 END) > 0 THEN 'VerifiedUpload'
                WHEN SUM(CASE WHEN LOWER(r.source_kind) = 'cloudimport' THEN 1 ELSE 0 END) > 0 THEN 'CloudImport'
                ELSE 'CloudImportUntrusted'
              END AS source_kind,
              MAX(r.publisher_key) AS app_signature_key,
              MAX(r.app_signature_subject) AS app_signature_subject,
              MAX(r.app_signature_issuer) AS app_signature_issuer
       FROM observation_author_rows r
       LEFT JOIN author_profiles p ON p.author_user_id = r.author_user_id
       WHERE ${conditions.join(" AND ")}
       GROUP BY r.app_id,
                LOWER(r.visibility),
                r.ip,
                r.port,
                LOWER(r.protocol)
       ORDER BY MAX(r.last_seen_ms) DESC,
                LOWER(r.visibility) ASC,
                r.app_id ASC,
                r.ip ASC,
                r.port ASC,
                LOWER(r.protocol) ASC,
                MIN(r.first_seen_ms) ASC,
                SUM(COALESCE(r.requests, 0)) ASC
       LIMIT ? OFFSET ?`
    )
    .bind(...params, limit, offset)
    .all();

  if (authorUserId) {
    return jsonOk({ scope: "author", app_id: appId, total, limit, offset, rows: result.results || [] });
  }
  return jsonOk({ scope: "community", app_id: appId, total, limit, offset, rows: result.results || [] });
}

async function appendObservations(request, db, actor) {
  await cleanupExpiredRows(db, nowMs());

  const rawBody = await request.text();
  await verifySignedRequestJwt(request, db, actor, rawBody, UPLOAD_SCOPE);

  const body = parseJsonText(rawBody);
  const clientIdentifier = requireString(body.client_identifier, "client_identifier");
  const operationId = normalizeQuotaOperationId(body.operation_id);
  const authorSignature = normalizeAuthorSignature(body.author_signature);
  const visibility = normalizeObservationVisibility(body.visibility) || "public";
  const appProof = body.app_proof || {};
  const appId = requireString(appProof.app_id, "app_proof.app_id");
  const rows = Array.isArray(body.rows) ? body.rows : [];

  if (rows.length === 0 || rows.length > OBSERVATION_BATCH_LIMIT) {
    throw new HttpError(400, "invalid_batch_size", `Rows count must be 1..${OBSERVATION_BATCH_LIMIT}`);
  }

  const uploadMode = uploadModeForRows(rows);
  if (uploadMode === "verified") {
    requireVerifiedAppProof(appProof);
  }
  await ensureAppNotBlocked(db, appId);
  await claimAuthorSignature(db, actor.user_id, authorSignature);
  const spendUploadQuota = operationId
    ? await quotaOperationRequiresSpend(db, clientIdentifier, "upload", operationId)
    : true;
  const uploadQuota = await quotaSnapshot(db, clientIdentifier, "upload", spendUploadQuota);
  if (operationId && spendUploadQuota) {
    await recordQuotaOperation(db, clientIdentifier, "upload", operationId, uploadQuota.window_ends_at_ms);
  }

  const timestamp = nowMs();
  const normalizedRows = rows.map((row) =>
    normalizeObservationRow(row, appId, timestamp, uploadMode, appProof, visibility)
  );
  const usedTags = [...new Set(normalizedRows.flatMap((row) => row.tags))].sort();
  await ensureUserTags(db, actor.user_id, usedTags, timestamp);
  const submissionId = prefixedId("sub");
  const appCatalogEntry = appCatalogEntryForUpload(appProof, appId);
  const statements = [
    upsertAppCatalog(db, appCatalogEntry, timestamp),
    db
      .prepare(
        `INSERT INTO observation_submissions
         (submission_id, app_id, author_user_id, client_id, row_count, created_at_ms)
         VALUES (?, ?, ?, ?, ?, ?)`
      )
      .bind(submissionId, appId, actor.user_id, actor.client_id, rows.length, timestamp),
  ];

  for (const normalized of normalizedRows) {
    if (visibility === "public") {
      statements.push(upsertCommunityObservation(db, normalized));
    }
    statements.push(upsertAuthorObservation(db, normalized, actor.user_id));
    for (const tag of normalized.tags) {
      statements.push(upsertObservationTag(db, normalized, actor.user_id, tag));
    }
  }

  await db.batch(statements);
  await audit(db, "submission", "append", submissionId, actor.user_id, {
    app_id: appId,
    client_id: actor.client_id,
    upload_mode: uploadMode,
    visibility,
    author_signature_norm: authorSignature.norm,
    row_count: rows.length,
    tag_count: usedTags.length,
  });

  return jsonOk({
    submission_id: submissionId,
    accepted_rows: rows.length,
    author_signature: authorSignature.display,
    visibility,
    quota: await quotaSnapshot(db, clientIdentifier, "upload", false),
  });
}

function normalizeAuthorSignature(value) {
  const display = requireString(value, "author_signature").trim();
  if (display.length === 0 || display.length > 32) {
    throw new HttpError(400, "invalid_author_signature", "Nickname must be 1..32 characters");
  }
  if (!/^[A-Za-z0-9!.@#$%^&*()_+=\-~]+$/.test(display)) {
    throw new HttpError(400, "invalid_author_signature", "Nickname contains unsupported characters");
  }
  if (isReservedAuthorSignature(display)) {
    throw new HttpError(409, "nickname_blocked", "Nickname is reserved");
  }
  return {
    display,
    norm: display.toLowerCase(),
  };
}

function normalizeObservationVisibility(value) {
  if (value === null || value === undefined || String(value).trim() === "") {
    return null;
  }
  const normalized = String(value).trim().toLowerCase();
  if (normalized === "public") {
    return "public";
  }
  if (normalized === "private") {
    return "private";
  }
  throw new HttpError(400, "invalid_visibility", "Observation visibility must be public or private");
}

function normalizeObservationVisibilityScope(value) {
  if (value === null || value === undefined || String(value).trim() === "") {
    return "public";
  }
  const normalized = String(value).trim().toLowerCase();
  if (normalized === "all" || normalized === "public" || normalized === "private") {
    return normalized;
  }
  throw new HttpError(400, "invalid_visibility", "Observation visibility must be all, public or private");
}

function reservedAuthorSignatureNorm(value) {
  return String(value || "")
    .trim()
    .toLowerCase();
}

function isReservedAuthorSignature(value) {
  return RESERVED_AUTHOR_SIGNATURES.includes(reservedAuthorSignatureNorm(value));
}

async function checkAuthorSignatureAvailability(request, url, db) {
  let authorSignature;
  try {
    authorSignature = normalizeAuthorSignature(url.searchParams.get("nickname") || "");
  } catch (error) {
    if (error instanceof HttpError) {
      const status = error.code === "nickname_blocked" ? "blocked" : "invalid";
      return jsonOk({ status, code: error.code, message: error.message });
    }
    throw error;
  }

  const actor = await optionalUser(request, db);
  const owner = await db
    .prepare(
      `SELECT author_user_id
       FROM author_profiles
       WHERE author_signature_norm = ?`
    )
    .bind(authorSignature.norm)
    .first();

  if (owner && (!actor || owner.author_user_id !== actor.user_id)) {
    return jsonOk({
      status: "taken",
      code: "nickname_taken",
      message: "Nickname is already registered",
    });
  }

  return jsonOk({
    status: "accepted",
    code: "nickname_accepted",
    message: "Nickname is available",
  });
}

async function claimAuthorSignature(db, userId, authorSignature) {
  const owner = await db
    .prepare(
      `SELECT author_user_id
       FROM author_profiles
       WHERE author_signature_norm = ?`
    )
    .bind(authorSignature.norm)
    .first();

  if (owner && owner.author_user_id !== userId) {
    throw new HttpError(409, "nickname_taken", "Nickname is already registered");
  }

  const timestamp = nowMs();
  await db
    .prepare(
      `INSERT INTO author_profiles
       (author_user_id, author_signature, author_signature_norm, created_at_ms, updated_at_ms)
       VALUES (?, ?, ?, ?, ?)
       ON CONFLICT(author_user_id) DO UPDATE SET
         author_signature = excluded.author_signature,
         author_signature_norm = excluded.author_signature_norm,
         updated_at_ms = excluded.updated_at_ms`
    )
    .bind(userId, authorSignature.display, authorSignature.norm, timestamp, timestamp)
    .run();
}

function upsertCommunityObservation(db, row) {
  return db
    .prepare(
      `INSERT INTO observations
       (app_id, visibility, publisher_key, ip, port, protocol, connection_state, requests, first_seen_ms, last_seen_ms,
        failed_hits, successful_hits, domain_raw, domain_verified, domain_status, trust_level,
        source_kind, app_signature_subject, app_signature_issuer, expires_at_ms, updated_at_ms)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
       ON CONFLICT(visibility, app_id, publisher_key, ip, port, protocol) DO UPDATE SET
         connection_state = excluded.connection_state,
         requests = observations.requests + excluded.requests,
         first_seen_ms = MIN(observations.first_seen_ms, excluded.first_seen_ms),
         last_seen_ms = MAX(observations.last_seen_ms, excluded.last_seen_ms),
         failed_hits = observations.failed_hits + excluded.failed_hits,
         successful_hits = observations.successful_hits + excluded.successful_hits,
         domain_raw = COALESCE(excluded.domain_raw, observations.domain_raw),
         domain_verified = COALESCE(excluded.domain_verified, observations.domain_verified),
         domain_status = excluded.domain_status,
         trust_level = excluded.trust_level,
         source_kind = excluded.source_kind,
         app_signature_subject = COALESCE(excluded.app_signature_subject, observations.app_signature_subject),
         app_signature_issuer = COALESCE(excluded.app_signature_issuer, observations.app_signature_issuer),
         expires_at_ms = excluded.expires_at_ms,
         updated_at_ms = excluded.updated_at_ms`
    )
    .bind(...observationBindValues(row));
}

function upsertAuthorObservation(db, row, authorUserId) {
  return db
    .prepare(
      `INSERT INTO observation_author_rows
       (app_id, visibility, author_user_id, publisher_key, ip, port, protocol, connection_state, requests, first_seen_ms,
        last_seen_ms, failed_hits, successful_hits, domain_raw, domain_verified, domain_status,
        trust_level, source_kind, app_signature_subject, app_signature_issuer, expires_at_ms, updated_at_ms)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
       ON CONFLICT(visibility, app_id, author_user_id, publisher_key, ip, port, protocol) DO UPDATE SET
         connection_state = excluded.connection_state,
         requests = observation_author_rows.requests + excluded.requests,
         first_seen_ms = MIN(observation_author_rows.first_seen_ms, excluded.first_seen_ms),
         last_seen_ms = MAX(observation_author_rows.last_seen_ms, excluded.last_seen_ms),
         failed_hits = observation_author_rows.failed_hits + excluded.failed_hits,
         successful_hits = observation_author_rows.successful_hits + excluded.successful_hits,
         domain_raw = COALESCE(excluded.domain_raw, observation_author_rows.domain_raw),
         domain_verified = COALESCE(excluded.domain_verified, observation_author_rows.domain_verified),
         domain_status = excluded.domain_status,
         trust_level = excluded.trust_level,
         source_kind = excluded.source_kind,
         app_signature_subject = COALESCE(excluded.app_signature_subject, observation_author_rows.app_signature_subject),
         app_signature_issuer = COALESCE(excluded.app_signature_issuer, observation_author_rows.app_signature_issuer),
         expires_at_ms = excluded.expires_at_ms,
         updated_at_ms = excluded.updated_at_ms`
    )
    .bind(row.app_id, row.visibility, authorUserId, ...observationBindValues(row).slice(2));
}

function upsertObservationTag(db, row, authorUserId, tag) {
  const ownerUserId = row.visibility === "private" ? authorUserId : "";
  return db
    .prepare(
      `INSERT INTO observation_tags
       (visibility, app_id, owner_user_id, ip, port, protocol, tag_user_id, tag, created_at_ms, updated_at_ms, expires_at_ms)
       SELECT ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
       WHERE EXISTS (
         SELECT 1 FROM observation_tags
         WHERE visibility = ? AND app_id = ? AND owner_user_id = ?
           AND ip = ? AND port = ? AND protocol = ? AND tag = ?
       ) OR (
         SELECT COUNT(DISTINCT tag) FROM observation_tags
         WHERE visibility = ? AND app_id = ? AND owner_user_id = ?
           AND ip = ? AND port = ? AND protocol = ?
           AND expires_at_ms > ?
       ) < ?
       ON CONFLICT(visibility, app_id, owner_user_id, ip, port, protocol, tag_user_id, tag) DO UPDATE SET
         updated_at_ms = excluded.updated_at_ms,
         expires_at_ms = MAX(observation_tags.expires_at_ms, excluded.expires_at_ms)`
    )
    .bind(
      row.visibility,
      row.app_id,
      ownerUserId,
      row.ip,
      row.port,
      row.protocol,
      authorUserId,
      tag,
      row.updated_at_ms,
      row.updated_at_ms,
      row.expires_at_ms,
      row.visibility,
      row.app_id,
      ownerUserId,
      row.ip,
      row.port,
      row.protocol,
      tag,
      row.visibility,
      row.app_id,
      ownerUserId,
      row.ip,
      row.port,
      row.protocol,
      row.updated_at_ms,
      TAGS_PER_OBSERVATION_LIMIT
    );
}

function observationBindValues(row) {
  return [
    row.app_id,
    row.visibility,
    row.publisher_key,
    row.ip,
    row.port,
    row.protocol,
    row.connection_state,
    row.requests,
    row.first_seen_ms,
    row.last_seen_ms,
    row.failed_hits,
    row.successful_hits,
    row.domain_raw,
    row.domain_verified,
    row.domain_status,
    row.trust_level,
    row.source_kind,
    row.app_signature_subject,
    row.app_signature_issuer,
    row.expires_at_ms,
    row.updated_at_ms,
  ];
}

function uploadModeForRows(rows) {
  const sourceKinds = new Set(rows.map((row) => normalizeEnum(row.source_kind, "VerifiedUpload")));
  if (sourceKinds.size !== 1) {
    throw new HttpError(400, "mixed_source_kind", "Upload batch must not mix source kinds");
  }
  const sourceKind = [...sourceKinds][0];
  if (sourceKind === "VerifiedUpload") {
    return "verified";
  }
  if (sourceKind === "CloudImportUntrusted") {
    return "untrusted_import";
  }
  throw new HttpError(400, "invalid_source_kind", "Unsupported source kind for upload");
}

function requireVerifiedAppProof(appProof) {
  if (
    appProof.status !== "Verified" ||
    appProof.signature_chain_valid !== true ||
    typeof appProof.leaf_spki_sha256 !== "string" ||
    appProof.leaf_spki_sha256.trim() === ""
  ) {
    throw new HttpError(403, "app_not_verified", "Upload requires verified application proof");
  }
}

async function ensureAppNotBlocked(db, appId) {
  const app = await db
    .prepare(`SELECT status FROM app_catalog WHERE app_id = ?`)
    .bind(appId)
    .first();

  if (app && app.status === "blocked") {
    throw new HttpError(403, "app_blocked", "Application is blocked");
  }
}

function appCatalogEntryForUpload(appProof, appId) {
  const displayName =
    optionalString(appProof.display_name) ||
    displayNameFromProof(appProof) ||
    appId;
  const publisherName = optionalString(appProof.publisher_name);
  const shortAppId =
    optionalString(appProof.short_app_id) ||
    shortAppIdFromDisplay(displayName, appId);

  return {
    app_id: appId,
    display_name: displayName.slice(0, 160),
    publisher_name: publisherName ? publisherName.slice(0, 160) : null,
    short_app_id: shortAppId.slice(0, 80),
  };
}

function upsertAppCatalog(db, app, timestamp) {
  return db
    .prepare(
      `INSERT INTO app_catalog
       (app_id, display_name, publisher_name, short_app_id, status, created_at_ms, updated_at_ms)
       VALUES (?, ?, ?, ?, 'active', ?, ?)
       ON CONFLICT(app_id) DO UPDATE SET
         display_name = CASE
           WHEN excluded.display_name <> '' THEN excluded.display_name
           ELSE app_catalog.display_name
         END,
         publisher_name = COALESCE(excluded.publisher_name, app_catalog.publisher_name),
         short_app_id = COALESCE(excluded.short_app_id, app_catalog.short_app_id),
         status = CASE
           WHEN app_catalog.status = 'blocked' THEN app_catalog.status
           ELSE 'active'
         END,
         updated_at_ms = excluded.updated_at_ms`
    )
    .bind(
      app.app_id,
      app.display_name,
      app.publisher_name,
      app.short_app_id,
      timestamp,
      timestamp
    );
}

function displayNameFromProof(appProof) {
  const subject = optionalString(appProof.subject);
  if (!subject) {
    return null;
  }
  const product = subject.match(/ProductName=([^;,]+)/i);
  if (product && product[1].trim()) {
    return product[1].trim();
  }
  const company = subject.match(/CompanyName=([^;,]+)/i);
  if (company && company[1].trim()) {
    return company[1].trim();
  }
  return null;
}

function shortAppIdFromDisplay(displayName, appId) {
  const base = String(displayName || "")
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "")
    .slice(0, 48);
  const suffix = String(appId || "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "")
    .slice(-8);
  if (base && suffix) {
    return `${base}-${suffix}`;
  }
  return base || suffix || "app";
}

function normalizeObservationRow(row, appId, timestamp, uploadMode, appProof, visibility) {
  if (row.app_id !== appId) {
    throw new HttpError(400, "app_id_mismatch", "All rows must match app_proof.app_id");
  }

  const port = Number(row.remote_port);
  const firstSeen = Number(row.first_seen_ms);
  const lastSeen = Number(row.last_seen_ms);

  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new HttpError(400, "invalid_port", "remote_port must be 1..65535");
  }
  if (!Number.isFinite(firstSeen) || !Number.isFinite(lastSeen) || lastSeen < firstSeen) {
    throw new HttpError(400, "invalid_observation_time", "Observation timestamps are invalid");
  }

  const sourceKind =
    uploadMode === "verified" ? "VerifiedUpload" : "CloudImportUntrusted";
  const publisherKey = publisherKeyForUpload(appProof, uploadMode);

  return {
    app_id: appId,
    visibility,
    publisher_key: publisherKey,
    tags: normalizeObservationTags(row.tags),
    ip: normalizePublicIp(row.remote_ip),
    port,
    protocol: normalizeEnum(row.protocol, "other").toLowerCase(),
    connection_state: normalizeEnum(row.connection_state, "Unknown"),
    requests: nonNegativeInteger(row.requests),
    first_seen_ms: firstSeen,
    last_seen_ms: lastSeen,
    failed_hits: nonNegativeInteger(row.failed_hits),
    successful_hits: nonNegativeInteger(row.successful_hits),
    domain_raw: optionalString(row.domain_raw),
    domain_verified: optionalString(row.domain_verified),
    domain_status: normalizeEnum(row.domain_status, "None"),
    trust_level:
      uploadMode === "verified" ? "VerifiedUploadCandidate" : "CloudImportUntrusted",
    source_kind: sourceKind,
    app_signature_subject: optionalString(appProof.subject),
    app_signature_issuer: optionalString(appProof.issuer),
    expires_at_ms: lastSeen + RETENTION_MS,
    updated_at_ms: timestamp,
  };
}

function publisherKeyForUpload(appProof, uploadMode) {
  const key = optionalString(appProof.leaf_spki_sha256);
  if (key) {
    return key;
  }
  return uploadMode === "verified" ? "verified:unknown" : "untrusted:unknown";
}

function normalizePublicIp(value) {
  const ip = requireString(value, "remote_ip").toLowerCase();
  if (isBlockedIpv4(ip) || isBlockedIpv6(ip)) {
    throw new HttpError(400, "blocked_ip", "Private, local, documentation and other non-public IP ranges are not accepted");
  }
  return ip;
}

function isBlockedIpv4(value) {
  const parts = value.split(".");
  if (parts.length !== 4) {
    return false;
  }

  const octets = parts.map((part) => Number(part));
  if (octets.some((octet) => !Number.isInteger(octet) || octet < 0 || octet > 255)) {
    throw new HttpError(400, "invalid_ip", "remote_ip must be a valid IP literal");
  }

  const [a, b, c] = octets;
  return (
    a === 0 ||
    a === 10 ||
    a === 127 ||
    a >= 224 ||
    (a === 100 && b >= 64 && b <= 127) ||
    (a === 169 && b === 254) ||
    (a === 172 && b >= 16 && b <= 31) ||
    (a === 192 && b === 168) ||
    (a === 192 && b === 0 && c === 2) ||
    (a === 198 && b === 18) ||
    (a === 198 && b === 19) ||
    (a === 198 && b === 51 && c === 100) ||
    (a === 203 && b === 0 && c === 113)
  );
}

function isBlockedIpv6(value) {
  if (!value.includes(":")) {
    return false;
  }

  const normalized = value.replace(/^\[|\]$/g, "");
  return (
    normalized === "::" ||
    normalized === "::1" ||
    normalized.startsWith("fc") ||
    normalized.startsWith("fd") ||
    normalized.startsWith("fe80:") ||
    normalized.startsWith("2001:db8:")
  );
}

async function quotaSnapshot(db, clientIdentifier, operationKind, spend) {
  const timestamp = nowMs();
  if (QUOTA_LIMIT === 0) {
    return {
      operation: operationKind,
      limit_count: 0,
      used_count: 0,
      window_started_at_ms: timestamp,
      window_ends_at_ms: timestamp + QUOTA_WINDOW_MS,
      synced_at_ms: timestamp,
    };
  }
  const existing = await db
    .prepare(
      `SELECT client_identifier, operation_kind, window_started_at_ms, window_ends_at_ms,
              limit_count, used_count, updated_at_ms
       FROM client_quota_windows
       WHERE client_identifier = ?
         AND operation_kind = ?`
    )
    .bind(clientIdentifier, operationKind)
    .first();

  let windowStarted = timestamp;
  let windowEnds = timestamp + QUOTA_WINDOW_MS;
  let usedCount = 0;

  if (existing && existing.window_ends_at_ms > timestamp) {
    windowStarted = existing.window_started_at_ms;
    windowEnds = existing.window_ends_at_ms;
    usedCount = existing.used_count;
  }

  if (spend) {
    if (usedCount >= QUOTA_LIMIT) {
      throw new HttpError(429, "quota_exhausted", "Cloud operation quota is exhausted");
    }
    usedCount += 1;
  }

  if (!existing || existing.window_ends_at_ms <= timestamp || spend) {
    await db
      .prepare(
        `INSERT INTO client_quota_windows
         (client_identifier, operation_kind, window_started_at_ms, window_ends_at_ms,
          limit_count, used_count, updated_at_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(client_identifier, operation_kind) DO UPDATE SET
           window_started_at_ms = excluded.window_started_at_ms,
           window_ends_at_ms = excluded.window_ends_at_ms,
           limit_count = excluded.limit_count,
           used_count = excluded.used_count,
           updated_at_ms = excluded.updated_at_ms`
      )
      .bind(clientIdentifier, operationKind, windowStarted, windowEnds, QUOTA_LIMIT, usedCount, timestamp)
      .run();
  }

  return {
    operation: operationKind,
    limit_count: QUOTA_LIMIT,
    used_count: usedCount,
    window_started_at_ms: windowStarted,
    window_ends_at_ms: windowEnds,
    synced_at_ms: timestamp,
  };
}

function normalizeQuotaOperationId(value) {
  const operationId = optionalString(value);
  if (!operationId) {
    return null;
  }
  if (!/^[A-Za-z0-9_.:-]{1,128}$/.test(operationId)) {
    throw new HttpError(400, "invalid_operation_id", "Operation id contains unsupported characters");
  }
  return operationId;
}

async function quotaOperationRequiresSpend(db, clientIdentifier, operationKind, operationId) {
  const timestamp = nowMs();
  const existing = await db
    .prepare(
      `SELECT window_ends_at_ms
       FROM client_quota_operations
       WHERE client_identifier = ?
         AND operation_kind = ?
         AND operation_id = ?`
    )
    .bind(clientIdentifier, operationKind, operationId)
    .first();

  return !(existing && existing.window_ends_at_ms > timestamp);
}

async function recordQuotaOperation(db, clientIdentifier, operationKind, operationId, windowEndsAtMs) {
  const timestamp = nowMs();
  await db
    .prepare(
      `INSERT INTO client_quota_operations
       (client_identifier, operation_kind, operation_id, window_ends_at_ms, created_at_ms, updated_at_ms)
       VALUES (?, ?, ?, ?, ?, ?)
       ON CONFLICT(client_identifier, operation_kind, operation_id) DO UPDATE SET
         window_ends_at_ms = excluded.window_ends_at_ms,
         updated_at_ms = excluded.updated_at_ms`
    )
    .bind(clientIdentifier, operationKind, operationId, windowEndsAtMs, timestamp, timestamp)
    .run();
}

async function requireUser(request, db) {
  const authorization = request.headers.get("authorization") || "";
  const match = authorization.match(/^Bearer\s+(.+)$/i);
  if (!match) {
    throw new HttpError(401, "auth_required", "Bearer session token is required");
  }

  const tokenHash = await sha256Base64Url(match[1]);
  const actor = await db
    .prepare(
      `SELECT s.user_id, u.login, s.client_id
       FROM user_sessions s
       JOIN users u ON u.user_id = s.user_id
       WHERE s.token_hash = ?
         AND s.revoked_at_ms IS NULL
         AND s.expires_at_ms > ?`
    )
    .bind(tokenHash, nowMs())
    .first();

  if (!actor) {
    throw new HttpError(401, "invalid_session", "Session token is invalid or expired");
  }

  return actor;
}

async function optionalUser(request, db) {
  const authorization = request.headers.get("authorization") || "";
  if (!/^Bearer\s+.+$/i.test(authorization)) {
    return null;
  }
  return await requireUser(request, db);
}

async function verifySignedRequestJwt(request, db, actor, rawBody, requiredScope) {
  const token = requireString(request.headers.get("x-netstitch-jwt"), "x-netstitch-jwt");
  const parts = token.split(".");
  if (parts.length !== 3) {
    throw new HttpError(401, "invalid_jwt", "JWT must use compact serialization");
  }

  const header = parseJsonText(decodeBase64UrlText(parts[0]));
  const claims = parseJsonText(decodeBase64UrlText(parts[1]));
  const signedData = new TextEncoder().encode(`${parts[0]}.${parts[1]}`);
  const signature = base64UrlToBytes(parts[2]);

  if (header.alg !== "ES256" || header.typ !== "JWT") {
    throw new HttpError(401, "invalid_jwt_header", "JWT header must use ES256 and typ JWT");
  }

  const keyId = requireString(header.kid, "jwt.kid");
  const keyRow = await db
    .prepare(
      `SELECT key_id, public_key_jwk
       FROM client_keys
       WHERE key_id = ?
         AND client_id = ?
         AND user_id = ?
         AND status = 'active'`
    )
    .bind(keyId, actor.client_id, actor.user_id)
    .first();

  if (!keyRow) {
    throw new HttpError(401, "unknown_key", "JWT key is not active for this session");
  }

  const publicKey = await importEs256PublicKey(keyRow.public_key_jwk);
  const verified = await crypto.subtle.verify(
    { name: "ECDSA", hash: "SHA-256" },
    publicKey,
    signature,
    signedData
  );

  if (!verified) {
    throw new HttpError(401, "invalid_jwt_signature", "JWT signature is invalid");
  }

  const nowSeconds = Math.floor(nowMs() / 1000);
  const bodyHash = await sha256Base64Url(rawBody);
  const scope = requireString(claims.scope, "jwt.scope");
  const jti = requireString(claims.jti, "jwt.jti");
  const exp = Number(claims.exp);
  const nbf = Number(claims.nbf);
  const iat = Number(claims.iat);

  if (claims.aud !== JWT_AUDIENCE) {
    throw new HttpError(401, "invalid_jwt_audience", "JWT audience is invalid");
  }
  if (claims.sub !== actor.client_id || claims.iss !== `netstitch-client:${actor.client_id}`) {
    throw new HttpError(401, "invalid_jwt_subject", "JWT subject is invalid");
  }
  if (!scope.split(/\s+/).includes(requiredScope)) {
    throw new HttpError(403, "invalid_jwt_scope", "JWT scope is not sufficient");
  }
  if (!Number.isFinite(exp) || !Number.isFinite(nbf) || !Number.isFinite(iat)) {
    throw new HttpError(401, "invalid_jwt_time", "JWT time claims are invalid");
  }
  if (nbf > nowSeconds + JWT_CLOCK_SKEW_SECONDS || iat > nowSeconds + JWT_CLOCK_SKEW_SECONDS) {
    throw new HttpError(
      401,
      "jwt_clock_skew",
      "JWT is not valid yet. Check that the system clock is synchronized"
    );
  }
  if (exp <= nowSeconds) {
    throw new HttpError(401, "expired_jwt", "JWT is not currently valid");
  }
  if (exp - iat > 15 * 60) {
    throw new HttpError(401, "jwt_ttl_too_long", "JWT TTL must not exceed 15 minutes");
  }
  if (claims.body_sha256 !== bodyHash) {
    throw new HttpError(401, "body_hash_mismatch", "JWT body hash does not match request body");
  }

  const replay = await db
    .prepare(
      `SELECT jti
       FROM jwt_replay_cache
       WHERE jti = ?`
    )
    .bind(jti)
    .first();

  if (replay) {
    throw new HttpError(409, "jwt_replay", "JWT jti was already used");
  }

    await db
      .prepare(
        `INSERT INTO jwt_replay_cache
       (jti, client_id, expires_at_ms, created_at_ms)
       VALUES (?, ?, ?, ?)`
      )
    .bind(jti, actor.client_id, exp * 1000, nowMs())
    .run();
}

async function importEs256PublicKey(publicKeyJwkText) {
  let jwk;
  try {
    jwk = JSON.parse(publicKeyJwkText);
  } catch {
    throw new HttpError(401, "invalid_public_key", "Stored public key is not valid JWK JSON");
  }

  if (jwk.kty !== "EC" || jwk.crv !== "P-256") {
    throw new HttpError(401, "unsupported_public_key", "JWT key must be EC P-256");
  }

  return crypto.subtle.importKey(
    "jwk",
    jwk,
    { name: "ECDSA", namedCurve: "P-256" },
    false,
    ["verify"]
  );
}

async function cleanupExpiredRows(db, timestamp) {
  await db.batch([
    db
      .prepare(
        `DELETE FROM observation_tags
         WHERE rowid IN (
           SELECT rowid FROM observation_tags
           WHERE expires_at_ms <= ?
           LIMIT ?
         )`
      )
      .bind(timestamp, CLEANUP_LIMIT),
    db
      .prepare(
        `DELETE FROM user_tags
         WHERE rowid IN (
           SELECT ut.rowid FROM user_tags ut
           WHERE ut.expires_at_ms <= ?
             AND NOT EXISTS (
               SELECT 1 FROM observation_tags ot
               WHERE ot.tag_user_id = ut.user_id AND ot.tag = ut.tag
             )
           LIMIT ?
         )`
      )
      .bind(timestamp, CLEANUP_LIMIT),
    db
      .prepare(
        `DELETE FROM observations
         WHERE rowid IN (
           SELECT rowid FROM observations
           WHERE expires_at_ms <= ?
           LIMIT ?
         )`
      )
      .bind(timestamp, CLEANUP_LIMIT),
    db
      .prepare(
        `DELETE FROM observation_author_rows
         WHERE rowid IN (
           SELECT rowid FROM observation_author_rows
           WHERE expires_at_ms <= ?
           LIMIT ?
         )`
      )
      .bind(timestamp, CLEANUP_LIMIT),
    db
      .prepare(
        `DELETE FROM jwt_replay_cache
         WHERE rowid IN (
           SELECT rowid FROM jwt_replay_cache
           WHERE expires_at_ms <= ?
           LIMIT ?
         )`
      )
      .bind(timestamp, CLEANUP_LIMIT),
    db
      .prepare(
        `DELETE FROM oauth_start_windows
         WHERE rowid IN (
           SELECT rowid FROM oauth_start_windows
           WHERE window_ends_at_ms <= ?
           LIMIT ?
         )`
      )
      .bind(timestamp, CLEANUP_LIMIT),
    db
      .prepare(
        `DELETE FROM client_quota_operations
         WHERE rowid IN (
           SELECT rowid FROM client_quota_operations
           WHERE window_ends_at_ms <= ?
           LIMIT ?
         )`
      )
      .bind(timestamp, CLEANUP_LIMIT),
    db
      .prepare(
        `DELETE FROM captcha_challenges
         WHERE rowid IN (
           SELECT rowid FROM captcha_challenges
           WHERE expires_at_ms <= ?
              OR consumed_at_ms IS NOT NULL
           LIMIT ?
         )`
      )
      .bind(timestamp, CLEANUP_LIMIT),
    db
      .prepare(
        `DELETE FROM captcha_refresh_pools
         WHERE rowid IN (
           SELECT rowid FROM captcha_refresh_pools
           WHERE expires_at_ms <= ?
           LIMIT ?
         )`
      )
      .bind(timestamp, CLEANUP_LIMIT),
    db
      .prepare(
        `DELETE FROM captcha_failure_buckets
         WHERE rowid IN (
           SELECT rowid FROM captcha_failure_buckets
           WHERE window_ends_at_ms <= ?
           LIMIT ?
         )`
      )
      .bind(timestamp, CLEANUP_LIMIT),
    db
      .prepare(
        `DELETE FROM browser_verification_challenges
         WHERE rowid IN (
           SELECT rowid FROM browser_verification_challenges
           WHERE expires_at_ms <= ?
              OR consumed_at_ms IS NOT NULL
           LIMIT ?
         )`
      )
      .bind(timestamp, CLEANUP_LIMIT),
  ]);
}

async function audit(db, entityType, actionType, entityId, actorId, payload) {
  await db
    .prepare(
      `INSERT INTO audit_events
       (event_id, created_at_ms, entity_type, action_type, entity_id, actor_id, payload_json)
       VALUES (?, ?, ?, ?, ?, ?, ?)`
    )
    .bind(
      prefixedId("evt"),
      nowMs(),
      entityType,
      actionType,
      entityId,
      actorId,
      JSON.stringify(payload || {})
    )
    .run();
}

async function readJson(request) {
  try {
    return parseJsonText(await request.text());
  } catch {
    throw new HttpError(400, "invalid_json", "Request body must be JSON");
  }
}

function parseJsonText(value) {
  let body;
  try {
    body = JSON.parse(value);
  } catch {
    throw new HttpError(400, "invalid_json", "Request body must be JSON");
  }
  if (!body || typeof body !== "object" || Array.isArray(body)) {
    throw new HttpError(400, "invalid_json", "Request JSON must be an object");
  }
  return body;
}

function parseHiddenJsonObject(value) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }
  try {
    const parsed = JSON.parse(value);
    if (!isPlainObject(parsed)) {
      throw new Error("not-object");
    }
    return parsed;
  } catch {
    throw new HttpError(401, "auth_flow_failed", "Authentication flow failed");
  }
}

function isPlainObject(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function requireString(value, field) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new HttpError(400, "missing_field", `${field} is required`);
  }
  return value.trim();
}

function optionalString(value) {
  return typeof value === "string" && value.trim() !== "" ? value.trim() : null;
}

function normalizeSearch(value) {
  return String(value || "").trim().slice(0, 80);
}

function normalizeEnum(value, fallback) {
  return typeof value === "string" && value.trim() !== "" ? value.trim() : fallback;
}

function nonNegativeInteger(value) {
  const number = Number(value || 0);
  if (!Number.isFinite(number) || number < 0) {
    return 0;
  }
  return Math.floor(number);
}

function positiveString(value) {
  return typeof value === "string" && value.trim() !== "";
}

function positiveInt(value, min, max) {
  const number = Number(value);
  return Number.isFinite(number) && Number.isInteger(number) && number >= min && number <= max;
}

function boundedLimit(value, fallback, max) {
  const number = Number(value || fallback);
  if (!Number.isInteger(number) || number <= 0) {
    return fallback;
  }
  return Math.min(number, max);
}

function boundedOffset(value) {
  const number = Number(value || 0);
  if (!Number.isInteger(number) || number < 0) {
    return 0;
  }
  return Math.min(number, 1000000);
}

async function sha256Base64Url(value) {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value));
  return base64Url(new Uint8Array(digest));
}

async function identityHmacBase64Url(env, value) {
  const pepper = requireEnv(env, "NETSTITCH_IDENTITY_PEPPER");
  const key = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(pepper),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"]
  );
  const signature = await crypto.subtle.sign(
    "HMAC",
    key,
    new TextEncoder().encode(String(value || ""))
  );
  return base64Url(new Uint8Array(signature));
}

async function sha256Hex(value) {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(value));
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function constantTimeStringEqual(left, right) {
  const a = String(left || "");
  const b = String(right || "");
  if (a.length !== b.length) {
    return false;
  }
  let diff = 0;
  for (let index = 0; index < a.length; index += 1) {
    diff |= a.charCodeAt(index) ^ b.charCodeAt(index);
  }
  return diff === 0;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, Math.max(0, Number(ms) || 0)));
}

function randomToken(byteLength) {
  const bytes = new Uint8Array(byteLength);
  crypto.getRandomValues(bytes);
  return base64Url(bytes);
}

function randomInt(min, max) {
  const lower = Math.ceil(min);
  const upper = Math.floor(max);
  if (upper <= lower) {
    return lower;
  }
  const range = upper - lower + 1;
  const maxValue = Math.floor(0xffffffff / range) * range;
  const bytes = new Uint32Array(1);
  do {
    crypto.getRandomValues(bytes);
  } while (bytes[0] >= maxValue);
  return lower + (bytes[0] % range);
}

function base64Url(bytes) {
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function base64UrlToBytes(value) {
  const base64 = value.replace(/-/g, "+").replace(/_/g, "/").padEnd(
    Math.ceil(value.length / 4) * 4,
    "="
  );
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}

function decodeBase64UrlText(value) {
  return new TextDecoder().decode(base64UrlToBytes(value));
}

function prefixedId(prefix) {
  return `${prefix}_${crypto.randomUUID()}`;
}

function nowMs() {
  return Date.now();
}

function jsonOk(payload, status = 200) {
  return new Response(JSON.stringify(payload), {
    status,
    headers: { ...JSON_HEADERS, ...corsHeaders() },
  });
}

function jsonError(status, code, message) {
  return new Response(JSON.stringify({ error: { code, message } }), {
    status,
    headers: { ...JSON_HEADERS, ...corsHeaders() },
  });
}

function corsHeaders() {
  return {
    "access-control-allow-origin": "*",
    "access-control-allow-methods": "GET,POST,OPTIONS",
    "access-control-allow-headers": "content-type,authorization,x-netstitch-jwt",
  };
}

class HttpError extends Error {
  constructor(status, code, message) {
    super(message);
    this.status = status;
    this.code = code;
  }
}
