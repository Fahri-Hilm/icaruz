# ICARUZ Privacy Notes

## Offline first

- The launcher works without an account and without a network call for local tasks: detect GTA, validate nickname format, plan launch, open settings.
- Server join needs network only to that game server. No launcher analytics call is required to play.
- Saved data lives on the PC: `%LOCALAPPDATA%/Icaruz/data/icaruz.db`, config, and logs under `%LOCALAPPDATA%/Icaruz/`.

## Telemetry: opt in only

- Default: off. No usage events, no crash upload, no update ping without consent.
- If a telemetry toggle ships later, it must be off on first run, explain what is sent in plain words, and allow full use of the app while off.
- Any future sender must exclude: full paths, user folder names, IPs, ports, nicknames, tokens, and file hashes outside the opted in scope.

## Passwords and Credential Manager

- Game server passwords, if ever stored, go to Windows Credential Manager, never to SQLite, TOML, logs, or crash reports.
- Memory handling: zero out password buffers after use where the language allows it, never bind a password to a log line or an error message.
- If Credential Manager is unavailable, ask per session and do not write a fallback file.

## Logs are redacted

- Log files under `%LOCALAPPDATA%/Icaruz/logs/` redact: user profile path segments, IPs, ports, nicknames beyond format validation, tokens, and password fields.
- Crash reports under `%LOCALAPPDATA%/Icaruz/crash-reports/` follow the same redaction.
- The Crash dialog Copy button copies the visible error text only, with a note that paths and tokens are hidden. It never attaches a full log file automatically.
- Report template for users: describe what happened, paste the copied error text, add launcher version and Windows build. No full log paste unless the user reviews and approves it.

## Current honest state

- Rust core stores settings in local SQLite and uses OS paths above. No telemetry sender exists in this tree.
- Credential Manager wiring for saved passwords is planned, not present yet. Until then, do not persist server passwords.
- Redaction helpers should be covered by tests before any upload button ships.
