# Security Policy — Simple-Jingle

## What This App Does

- Detects when you type a configurable trigger word (default: "proceed")
- Plays a sound file when the trigger word is detected
- Shows a system tray icon for enable/disable/quit
- Runs as a portable executable with no installation

## What This App Does NOT Do

- Does **NOT** log keystrokes to any file
- Does **NOT** write any files to disk
- Does **NOT** make any network connections
- Does **NOT** read or write the clipboard
- Does **NOT** inject DLLs into other processes
- Does **NOT** require admin privileges
- Does **NOT** encrypt or obfuscate its buffer (transparency over obscurity)
- Does **NOT** persist any typed data — buffer is zeroized on every word boundary, focus change, digit/symbol entry, and password field detection

## Zero-Trust Guarantees

| Guarantee | Implementation |
|-----------|----------------|
| No keystroke logging | Fixed 64-byte stack-allocated buffer, zeroized on every clear |
| Zeroize-on-drop | Buffer struct implements `Zeroize` and `ZeroizeOnDrop` |
| Password field bypass | Queries focused control via Windows UI Automation; suspends and zeroizes buffer if `IsPassword` is true |
| Window title blacklist | Checks active window title for keywords like "password", "login", "bitwarden", etc. |
| Digit/symbol wipe | Any digit or symbol character instantly clears and zeroizes the buffer |
| Navigation key wipe | Any arrow/cursor or reset key press instantly clears and zeroizes the buffer |
| Modifier filtering | Keystrokes with Ctrl/Alt/Win held are ignored to prevent tracking shortcuts |
| Crash protection | Compiled with `panic = "abort"` to prevent stack unwinding or memory dumping |
| No file writes | Only `fs::read` for `settings.ini` |
| No network | Zero network crates in `Cargo.toml` |
| No admin rights | `WH_KEYBOARD_LL` works without elevation |
| No DLL injection | Hook is local to the process |
| No console window | `#![windows_subsystem = "windows"]` |
| No encryption/obscurity | Plain buffer, securely zeroized — transparent by design |

## How to Verify

1. **Programmatic Tests** — run `cargo test` to execute unit tests verifying each security rule (zeroization, password field bypass, modifier ignoring, focus-change resetting, and digit/symbol wiping).
2. **Read the source** — the entire app is in `src/main.rs` (~600 lines, single file).
3. **Process Monitor** — run [Process Monitor](https://learn.microsoft.com/en-us/sysinternals/downloads/procmon) to verify zero file writes.
4. **Wireshark / Fiddler** — verify zero network traffic.
5. **Debug mode** — set `enabled = true` in `settings.ini` to watch detection events in real time.
6. **Build from source** — `cargo build --release`.

## Dependencies

All dependencies are well-known, auditable, and have no network capabilities:

| Crate | Purpose | Downloads |
|-------|---------|-----------|
| `windows` | Keyboard hook, COM, UI Automation | 280M+ |
| `rodio` | Audio playback via WASAPI | 9.5M+ |
| `tray-icon` | System tray icon | 21M+ |
| `muda` | Tray menu items | 26M+ |
| `hound` | WAV decoding | 4M+ |
| `zeroize` | Secure memory clearing | 587M+ |
| `image` | Dynamic icon loading | 100M+ |

## Reporting Vulnerabilities

If you find a security issue, please open a GitHub issue with details.
