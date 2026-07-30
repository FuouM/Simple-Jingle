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
Because the pre-compiled releases are portable and unsigned, Windows SmartScreen or antivirus software may flag the executable as unrecognized or malicious on first run.

* **VirusTotal Audit:** https://www.virustotal.com/gui/file/a177ed14d7bcb6c38d0209c95082c9338993b4aa0512d1a6cf99170255bda8a4/detection
  * *Microsoft (Defender):* Flags heuristically as `Trojan:Win32/Wacatac.B!ml` (False Positive)
  * *Bkav Pro:* Flags as `W32.Malware.27B289C3` (False Positive)
  * *SecureAge:* Flags as `Malicious` (False Positive)
  * *Others (e.g., Acronis, Kaspersky, CrowdStrike):* 100% clean / Undetected.

#### Why does it trigger flags like "Wacatac.B!ml"?
Antivirus machine learning models (`!ml` indicates machine learning heuristic detection) flag this application due to its core keyboard integration patterns:
1. **Low-Level Keyboard Hook (`WH_KEYBOARD_LL`):** The program registers a system-wide hook to detect trigger word completions. Heuristic engines flag this because keyboard hooks are heavily utilized by keylogger malware.
2. **Background Subsytem (No Window):** The program is compiled without a standard console window (`#![windows_subsystem = "windows"]`) so it runs silently in the system tray. Unseen background processes that monitor keys resemble Trojan/stealth spyware behavior.
3. **Active Window Auditing:** It uses Windows UI Automation and COM interfaces to query active foreground window titles and input fields (to instantly disable logging on password fields). Querying focus contexts across different applications is flagged by heuristic models.

* **To run the pre-built release:** Click **"More info"** on the SmartScreen dialog and select **"Run anyway"** (or add an exception to your antivirus).
* **To avoid flags completely:** We highly recommend you **build the executable yourself** from source using the steps above. This guarantees that only the audited code from this repository is compiled into your local binary, which prevents generic reputation flags.

## Verification & Testing

Verify the security rules and memory zeroization programmatically using unit tests:
```powershell
. .\env.ps1
cargo test
```

## LLM Use Disclosure

This repository was developed with assistant support from the following large language models:
* **Xiaomi: MiMo-V2.5** via *OpenCode* — used for initial implementation plan and initial implementation.
* **Gemini 3.5 Flash (Low)** via *Antigravity* — used for security audits, bug fixes, features, and overall code polish.

## License

MIT
