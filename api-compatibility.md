# Desktop/Webapp API-Kompatibilität — historical integration audit

> **Scope:** This is a July 2026 audit snapshot that references a separate local
> `gregWeb.Modstore` checkout. It is planning evidence, not a stable public API
> specification and not proof that the named web routes are deployed. Confirm
> request/response contracts against the owning web repository before changing
> desktop code or documenting a feature as released.

Stand: `v1.6.1-pre.13`

## Desktop-API-Vertrag

| Zweck | Desktop-Aufruf | Webapp-Status |
|---|---|---|
| Browser-Login starten | `GET /auth/login` auf `https://datacentermods.home` | Implementiert in `src/app/auth/login/route.ts` |
| Code gegen Session tauschen | `POST /auth/token` auf `https://api.datacentermods.home` | Implementiert in `src/app/auth/token/route.ts` |
| Refresh prüfen | `POST /auth/token` mit `state=refresh` | Implementiert; prüft die Better-Auth-Session gegen Ablauf/Widerruf |
| Logout | `POST /auth/logout` mit Bearer-Sessiontoken | Implementiert in `src/app/auth/logout/route.ts` |
| Webapp-Session | Better Auth `/api/auth/*` | Bestehende Webapp-Authentifizierung bleibt unverändert |

## Lokaler Desktop-Testmodus

Der Starter `/usr/local/bin/gregmodmanager-local-test` setzt:

- `MODSTORE_WEB_URL=https://datacentermods.home`
- `MODSTORE_API_URL=https://api.datacentermods.home`
- `IS_LOCAL_TEST_BUILD=TRUE`

Die lokale Webapp muss danach neu gebaut/gestartet werden. Außerdem müssen `datacentermods.home` und `api.datacentermods.home` auf die lokale Reverse-Proxy-Adresse zeigen; aktuell ist `api.datacentermods.home` auf diesem System noch nicht auflösbar.

## Nicht-Webapp-APIs

Steam Workshop Upload, Update, Download und Progress laufen über Steamworks/ISteamUGC und benötigen keine Webapp-Route. Lokale Workspace-, GameRoot- und Projektbearbeitung laufen ausschließlich im Desktop-Client.

Die vollständige Modstore-Funktionsliste und die noch nicht im Desktop-Client verdrahteten Webapp-Funktionen stehen in [`functions.md`](./functions.md).

## Audit-Ergebnis 2026-07-29

Die Auth-Routen sind im Webapp-Quellcode vorhanden und die bestehenden Desktop-Verträge bleiben unverändert. Für Katalog, Moddetails, Favoriten, Watchlist, Bewertungen, Kommentare, Collections und Creator-Releases existiert im Desktop aktuell kein gleichwertiger allgemeiner API-Client. Diese Funktionen dürfen nicht als fertig gelten, bis Request-/Response-Modelle und Tests ergänzt sind.

Der neue Game-Adapter ist lokal und benötigt keine Webapp-Route. Workshop-Download, Upload und Fortschritt bleiben gemäß Vertrag native Steamworks-Funktionen.
