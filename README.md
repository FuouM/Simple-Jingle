# Simple-Jingle

A zero-trust, offline background application that plays a jingle when you type a trigger word. Built with strict anti-keylogger design patterns to guarantee total privacy and security.

## What It Does

- Detects configurable trigger words (default: `proceed`) as you type. Triggering is checked when a word boundary is reached (by typing `Space`, `Enter` / `Return`, or `Tab`).
- Plays a sound file (WAV) when a trigger word is completed.
- Runs silently in the system tray — no console or terminal window by default.
- 100% offline, zero network capabilities, zero file writes, and zero data persistence.

## Security & Privacy Safeguards

Simple-Jingle is designed as a *non-retentive* input watcher. It implements multiple defense-in-depth safety layers to ensure it cannot act as a keylogger:
1. **Windows UI Automation Bypass**: Queries COM accessibility interfaces. If the focused input field is a password field (`IsPassword` is true), logging is instantly suspended and memory is zeroized.
2. **Window Title Heuristic Blacklist**: Automatically zeroizes and suspends logging in sensitive screens (e.g. windows containing "password", "login", "bitwarden", "keepass", "vault", etc.).
3. **Digit & Symbol Instant Clears**: Typing any digit (`0-9`) or special character immediately wipes the character buffer, ensuring no complex credentials can reside in memory.
4. **Navigation Key Wiping**: Pressing arrows (`Left`/`Right`/`Up`/`Down`), `Home`, `End`, `Delete`, `Escape`, `PageUp`, or `PageDown` instantly zeroizes the buffer.
5. **No System Modifiers**: Keystrokes typed while `Ctrl`, `Alt`, or `Win` keys are down are completely ignored to avoid capturing keyboard shortcuts.
6. **Hardened Compiler Profile**: Compiled with `panic = "abort"`, link-time optimization (LTO), and symbol stripping to prevent memory dumps or reverse engineering.

See [SECURITY.md](SECURITY.md) for full details on verification.

---

## Quick Start

1. **Build and Start** the app:
   ```powershell
   .\dev.ps1
   ```
2. The app compiles, copies assets, and starts silently in your **system tray** (hidden under the `^` overflow menu next to the Windows clock).
3. Right-click the system tray icon to **Enable/Disable** triggers or **Quit**.
4. Type `proceed` followed by `Space`, `Enter`, or `Tab` in any text editor — the ominous sound plays.

---

## Configuration (`settings.ini`)

Customize the jingle, trigger words, or debug console settings in the `settings.ini` file located next to the executable:

```ini
[sound]
path = sounds/snd_ominous_music.wav    ; Relative path to the audio file

[triggers]
words = proceed,start,play             ; Comma-separated list of trigger words
case_sensitive = false                 ; Case-insensitive matching
boundary_keys = VK_SPACE,VK_RETURN,VK_TAB   ; Windows Virtual-Key code names (e.g. VK_SPACE, VK_RETURN, VK_TAB) or hex codes (e.g. 0x20)

[debug]
enabled = false                        ; Set to true to allocate a diagnostic console
```

---

## Packaging a Portable Release

To package a standalone, portable release:
```powershell
.\build.ps1
```
This compiles the release executable and copies all required portable assets (`simple-jingle.exe`, `settings.ini`, `sounds/`) into the `release/` folder in the project root. 

You can move this `release` folder anywhere (USB drive, custom directory) and run `simple-jingle.exe` portably. To uninstall, simply delete the folder.

### Windows SmartScreen & Antivirus Note
Because the pre-compiled releases are portable and unsigned, Windows SmartScreen or antivirus software may flag the executable as unrecognized when run for the first time.
* To run the pre-built release, click **"More info"** on the SmartScreen dialog and select **"Run anyway"**.
* Alternatively (and recommended for total trust), you can **build the executable yourself** from source using the steps above. This guarantees that only the audited code from this repository is compiled into your binary.

## Verification & Testing

Verify the security rules and memory zeroization programmatically using unit tests:
```powershell
. .\env.ps1
cargo test
```

## License

MIT
