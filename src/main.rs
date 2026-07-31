#![windows_subsystem = "windows"]
#![allow(static_mut_refs)]

use std::ffi::c_void;
use std::fs;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Mutex, OnceLock};

use muda::{Menu, MenuItem};
use tray_icon::menu::MenuEvent;
use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent};
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::Console::AllocConsole;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Accessibility::*;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

struct Config {
    sound: SoundConfig,
    triggers: TriggerConfig,
    debug: DebugConfig,
}

struct SoundConfig {
    path: String,
}

struct TriggerConfig {
    words: Vec<String>,
    case_sensitive: bool,
    boundary_vkeys: Vec<u16>,
}

struct DebugConfig {
    enabled: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);
static AUDIO_ENABLED: AtomicBool = AtomicBool::new(true);

struct AudioState {
    _handle: rodio::MixerDeviceSink,
    player: rodio::Player,
}
static AUDIO_STATE: OnceLock<Mutex<Option<AudioState>>> = OnceLock::new();

fn parse_vk_code(s: &str) -> Option<u16> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    
    // Try hex (e.g. 0x20)
    if s.to_lowercase().starts_with("0x") {
        if let Ok(val) = u16::from_str_radix(&s[2..], 16) {
            return Some(val);
        }
    }
    
    // Try decimal (e.g. 32)
    if let Ok(val) = s.parse::<u16>() {
        return Some(val);
    }
    
    // Try Windows VK Name
    match s.to_uppercase().as_str() {
        "VK_LBUTTON" => Some(VK_LBUTTON.0),
        "VK_RBUTTON" => Some(VK_RBUTTON.0),
        "VK_CANCEL" => Some(VK_CANCEL.0),
        "VK_MBUTTON" => Some(VK_MBUTTON.0),
        "VK_XBUTTON1" => Some(VK_XBUTTON1.0),
        "VK_XBUTTON2" => Some(VK_XBUTTON2.0),
        "VK_BACK" => Some(VK_BACK.0),
        "VK_TAB" => Some(VK_TAB.0),
        "VK_CLEAR" => Some(VK_CLEAR.0),
        "VK_RETURN" | "VK_ENTER" => Some(VK_RETURN.0),
        "VK_SHIFT" => Some(VK_SHIFT.0),
        "VK_CONTROL" => Some(VK_CONTROL.0),
        "VK_MENU" | "VK_ALT" => Some(VK_MENU.0),
        "VK_PAUSE" => Some(VK_PAUSE.0),
        "VK_CAPITAL" | "VK_CAPSLOCK" => Some(VK_CAPITAL.0),
        "VK_KANA" => Some(VK_KANA.0),
        "VK_HANGUEL" | "VK_HANGEUL" => Some(VK_HANGEUL.0),
        "VK_HANGUL" => Some(VK_HANGUL.0),
        "VK_IME_ON" => Some(VK_IME_ON.0),
        "VK_JUNJA" => Some(VK_JUNJA.0),
        "VK_FINAL" => Some(VK_FINAL.0),
        "VK_HANJA" => Some(VK_HANJA.0),
        "VK_KANJI" => Some(VK_KANJI.0),
        "VK_IME_OFF" => Some(VK_IME_OFF.0),
        "VK_ESCAPE" => Some(VK_ESCAPE.0),
        "VK_CONVERT" => Some(VK_CONVERT.0),
        "VK_NONCONVERT" => Some(VK_NONCONVERT.0),
        "VK_ACCEPT" => Some(VK_ACCEPT.0),
        "VK_MODECHANGE" => Some(VK_MODECHANGE.0),
        "VK_SPACE" => Some(VK_SPACE.0),
        "VK_PRIOR" | "VK_PAGEUP" => Some(VK_PRIOR.0),
        "VK_NEXT" | "VK_PAGEDOWN" => Some(VK_NEXT.0),
        "VK_END" => Some(VK_END.0),
        "VK_HOME" => Some(VK_HOME.0),
        "VK_LEFT" => Some(VK_LEFT.0),
        "VK_UP" => Some(VK_UP.0),
        "VK_RIGHT" => Some(VK_RIGHT.0),
        "VK_DOWN" => Some(VK_DOWN.0),
        "VK_SELECT" => Some(VK_SELECT.0),
        "VK_PRINT" => Some(VK_PRINT.0),
        "VK_EXECUTE" => Some(VK_EXECUTE.0),
        "VK_SNAPSHOT" | "VK_PRINTSCREEN" => Some(VK_SNAPSHOT.0),
        "VK_INSERT" => Some(VK_INSERT.0),
        "VK_DELETE" => Some(VK_DELETE.0),
        "VK_HELP" => Some(VK_HELP.0),
        "VK_LWIN" => Some(VK_LWIN.0),
        "VK_RWIN" => Some(VK_RWIN.0),
        "VK_APPS" => Some(VK_APPS.0),
        "VK_SLEEP" => Some(VK_SLEEP.0),
        "VK_NUMPAD0" => Some(VK_NUMPAD0.0),
        "VK_NUMPAD1" => Some(VK_NUMPAD1.0),
        "VK_NUMPAD2" => Some(VK_NUMPAD2.0),
        "VK_NUMPAD3" => Some(VK_NUMPAD3.0),
        "VK_NUMPAD4" => Some(VK_NUMPAD4.0),
        "VK_NUMPAD5" => Some(VK_NUMPAD5.0),
        "VK_NUMPAD6" => Some(VK_NUMPAD6.0),
        "VK_NUMPAD7" => Some(VK_NUMPAD7.0),
        "VK_NUMPAD8" => Some(VK_NUMPAD8.0),
        "VK_NUMPAD9" => Some(VK_NUMPAD9.0),
        "VK_MULTIPLY" => Some(VK_MULTIPLY.0),
        "VK_ADD" => Some(VK_ADD.0),
        "VK_SEPARATOR" => Some(VK_SEPARATOR.0),
        "VK_SUBTRACT" => Some(VK_SUBTRACT.0),
        "VK_DECIMAL" => Some(VK_DECIMAL.0),
        "VK_DIVIDE" => Some(VK_DIVIDE.0),
        "VK_F1" => Some(VK_F1.0),
        "VK_F2" => Some(VK_F2.0),
        "VK_F3" => Some(VK_F3.0),
        "VK_F4" => Some(VK_F4.0),
        "VK_F5" => Some(VK_F5.0),
        "VK_F6" => Some(VK_F6.0),
        "VK_F7" => Some(VK_F7.0),
        "VK_F8" => Some(VK_F8.0),
        "VK_F9" => Some(VK_F9.0),
        "VK_F10" => Some(VK_F10.0),
        "VK_F11" => Some(VK_F11.0),
        "VK_F12" => Some(VK_F12.0),
        "VK_F13" => Some(VK_F13.0),
        "VK_F14" => Some(VK_F14.0),
        "VK_F15" => Some(VK_F15.0),
        "VK_F16" => Some(VK_F16.0),
        "VK_F17" => Some(VK_F17.0),
        "VK_F18" => Some(VK_F18.0),
        "VK_F19" => Some(VK_F19.0),
        "VK_F20" => Some(VK_F20.0),
        "VK_F21" => Some(VK_F21.0),
        "VK_F22" => Some(VK_F22.0),
        "VK_F23" => Some(VK_F23.0),
        "VK_F24" => Some(VK_F24.0),
        "VK_NUMLOCK" => Some(VK_NUMLOCK.0),
        "VK_SCROLL" => Some(VK_SCROLL.0),
        "VK_LSHIFT" => Some(VK_LSHIFT.0),
        "VK_RSHIFT" => Some(VK_RSHIFT.0),
        "VK_LCONTROL" => Some(VK_LCONTROL.0),
        "VK_RCONTROL" => Some(VK_RCONTROL.0),
        "VK_LMENU" => Some(VK_LMENU.0),
        "VK_RMENU" => Some(VK_RMENU.0),
        "VK_BROWSER_BACK" => Some(VK_BROWSER_BACK.0),
        "VK_BROWSER_FORWARD" => Some(VK_BROWSER_FORWARD.0),
        "VK_BROWSER_REFRESH" => Some(VK_BROWSER_REFRESH.0),
        "VK_BROWSER_STOP" => Some(VK_BROWSER_STOP.0),
        "VK_BROWSER_SEARCH" => Some(VK_BROWSER_SEARCH.0),
        "VK_BROWSER_FAVORITES" => Some(VK_BROWSER_FAVORITES.0),
        "VK_BROWSER_HOME" => Some(VK_BROWSER_HOME.0),
        "VK_VOLUME_MUTE" => Some(VK_VOLUME_MUTE.0),
        "VK_VOLUME_DOWN" => Some(VK_VOLUME_DOWN.0),
        "VK_VOLUME_UP" => Some(VK_VOLUME_UP.0),
        "VK_MEDIA_NEXT_TRACK" => Some(VK_MEDIA_NEXT_TRACK.0),
        "VK_MEDIA_PREV_TRACK" => Some(VK_MEDIA_PREV_TRACK.0),
        "VK_MEDIA_STOP" => Some(VK_MEDIA_STOP.0),
        "VK_MEDIA_PLAY_PAUSE" => Some(VK_MEDIA_PLAY_PAUSE.0),
        "VK_LAUNCH_MAIL" => Some(VK_LAUNCH_MAIL.0),
        "VK_LAUNCH_MEDIA_SELECT" => Some(VK_LAUNCH_MEDIA_SELECT.0),
        "VK_LAUNCH_APP1" => Some(VK_LAUNCH_APP1.0),
        "VK_LAUNCH_APP2" => Some(VK_LAUNCH_APP2.0),
        "VK_OEM_1" => Some(VK_OEM_1.0),
        "VK_OEM_PLUS" => Some(VK_OEM_PLUS.0),
        "VK_OEM_COMMA" => Some(VK_OEM_COMMA.0),
        "VK_OEM_MINUS" => Some(VK_OEM_MINUS.0),
        "VK_OEM_PERIOD" => Some(VK_OEM_PERIOD.0),
        "VK_OEM_2" => Some(VK_OEM_2.0),
        "VK_OEM_3" => Some(VK_OEM_3.0),
        "VK_OEM_4" => Some(VK_OEM_4.0),
        "VK_OEM_5" => Some(VK_OEM_5.0),
        "VK_OEM_6" => Some(VK_OEM_6.0),
        "VK_OEM_7" => Some(VK_OEM_7.0),
        "VK_OEM_8" => Some(VK_OEM_8.0),
        "VK_OEM_102" => Some(VK_OEM_102.0),
        "VK_PROCESSKEY" => Some(VK_PROCESSKEY.0),
        "VK_PACKET" => Some(VK_PACKET.0),
        "VK_ATTN" => Some(VK_ATTN.0),
        "VK_CRSEL" => Some(VK_CRSEL.0),
        "VK_EXSEL" => Some(VK_EXSEL.0),
        "VK_EREOF" => Some(VK_EREOF.0),
        "VK_PLAY" => Some(VK_PLAY.0),
        "VK_ZOOM" => Some(VK_ZOOM.0),
        "VK_NONAME" => Some(VK_NONAME.0),
        "VK_PA1" => Some(VK_PA1.0),
        "VK_OEM_CLEAR" => Some(VK_OEM_CLEAR.0),
        "VK_GAMEPAD_A" => Some(VK_GAMEPAD_A.0),
        "VK_GAMEPAD_B" => Some(VK_GAMEPAD_B.0),
        "VK_GAMEPAD_X" => Some(VK_GAMEPAD_X.0),
        "VK_GAMEPAD_Y" => Some(VK_GAMEPAD_Y.0),
        "VK_GAMEPAD_RIGHT_SHOULDER" => Some(VK_GAMEPAD_RIGHT_SHOULDER.0),
        "VK_GAMEPAD_LEFT_SHOULDER" => Some(VK_GAMEPAD_LEFT_SHOULDER.0),
        "VK_GAMEPAD_LEFT_TRIGGER" => Some(VK_GAMEPAD_LEFT_TRIGGER.0),
        "VK_GAMEPAD_RIGHT_TRIGGER" => Some(VK_GAMEPAD_RIGHT_TRIGGER.0),
        "VK_GAMEPAD_DPAD_UP" => Some(VK_GAMEPAD_DPAD_UP.0),
        "VK_GAMEPAD_DPAD_DOWN" => Some(VK_GAMEPAD_DPAD_DOWN.0),
        "VK_GAMEPAD_DPAD_LEFT" => Some(VK_GAMEPAD_DPAD_LEFT.0),
        "VK_GAMEPAD_DPAD_RIGHT" => Some(VK_GAMEPAD_DPAD_RIGHT.0),
        "VK_GAMEPAD_MENU" => Some(VK_GAMEPAD_MENU.0),
        "VK_GAMEPAD_VIEW" => Some(VK_GAMEPAD_VIEW.0),
        "VK_GAMEPAD_LEFT_THUMBSTICK_BUTTON" => Some(VK_GAMEPAD_LEFT_THUMBSTICK_BUTTON.0),
        "VK_GAMEPAD_RIGHT_THUMBSTICK_BUTTON" => Some(VK_GAMEPAD_RIGHT_THUMBSTICK_BUTTON.0),
        "VK_GAMEPAD_LEFT_THUMBSTICK_UP" => Some(VK_GAMEPAD_LEFT_THUMBSTICK_UP.0),
        "VK_GAMEPAD_LEFT_THUMBSTICK_DOWN" => Some(VK_GAMEPAD_LEFT_THUMBSTICK_DOWN.0),
        "VK_GAMEPAD_LEFT_THUMBSTICK_RIGHT" => Some(VK_GAMEPAD_LEFT_THUMBSTICK_RIGHT.0),
        "VK_GAMEPAD_LEFT_THUMBSTICK_LEFT" => Some(VK_GAMEPAD_LEFT_THUMBSTICK_LEFT.0),
        "VK_GAMEPAD_RIGHT_THUMBSTICK_UP" => Some(VK_GAMEPAD_RIGHT_THUMBSTICK_UP.0),
        "VK_GAMEPAD_RIGHT_THUMBSTICK_DOWN" => Some(VK_GAMEPAD_RIGHT_THUMBSTICK_DOWN.0),
        "VK_GAMEPAD_RIGHT_THUMBSTICK_RIGHT" => Some(VK_GAMEPAD_RIGHT_THUMBSTICK_RIGHT.0),
        "VK_GAMEPAD_RIGHT_THUMBSTICK_LEFT" => Some(VK_GAMEPAD_RIGHT_THUMBSTICK_LEFT.0),
        _ => None,
    }
}

fn load_config() -> Config {
    let config_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("settings.ini")))
        .unwrap_or_else(|| PathBuf::from("settings.ini"));

    let mut sound_path = String::from("sounds/snd_ominous_music.wav");
    let mut words = vec![String::from("proceed")];
    let mut case_sensitive = false;
    let mut boundary_vkeys = vec![VK_SPACE.0, VK_RETURN.0, VK_TAB.0];
    let mut debug_enabled = false;

    if let Ok(content) = fs::read_to_string(&config_path) {
        let mut current_section = String::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].trim().to_lowercase();
                continue;
            }
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_lowercase();
                let val = line[pos + 1..].trim();
                
                let val = if (val.starts_with('"') && val.ends_with('"')) || (val.starts_with('\'') && val.ends_with('\'')) {
                    &val[1..val.len() - 1]
                } else {
                    val
                };

                match current_section.as_str() {
                    "sound" => {
                        if key == "path" {
                            sound_path = val.to_string();
                        }
                    }
                    "triggers" => {
                        if key == "words" {
                            words = val.split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                        } else if key == "case_sensitive" {
                            case_sensitive = val.to_lowercase() == "true" || val == "1";
                        } else if key == "boundary_keys" {
                            let parsed: Vec<u16> = val.split(',')
                                .map(|s| s.trim())
                                .filter_map(parse_vk_code)
                                .collect();
                            if !parsed.is_empty() {
                                boundary_vkeys = parsed;
                            }
                        }
                    }
                    "debug" => {
                        if key == "enabled" {
                            debug_enabled = val.to_lowercase() == "true" || val == "1";
                        }
                    }
                    _ => {}
                }
            }
        }
    } else {
        eprintln!("[Simple-Jingle] settings.ini not found at {:?}", config_path);
    }

    Config {
        sound: SoundConfig { path: sound_path },
        triggers: TriggerConfig { words, case_sensitive, boundary_vkeys },
        debug: DebugConfig { enabled: debug_enabled },
    }
}

// ---------------------------------------------------------------------------
// Word detection buffer — static mut, only accessed from hook callback thread
// ---------------------------------------------------------------------------

const BUFFER_SIZE: usize = 64;

struct WordBuffer {
    buf: [u8; BUFFER_SIZE],
    pos: usize,
}

impl WordBuffer {
    const fn new() -> Self {
        Self {
            buf: [0u8; BUFFER_SIZE],
            pos: 0,
        }
    }

    fn push_byte(&mut self, b: u8) {
        if self.pos < BUFFER_SIZE {
            self.buf[self.pos] = b;
            self.pos += 1;
        } else {
            self.clear();
        }
    }

    fn pop(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
            self.buf[self.pos] = 0;
        }
    }

    fn clear(&mut self) {
        for b in &mut self.buf {
            unsafe {
                std::ptr::write_volatile(b, 0);
            }
        }
        self.pos = 0;
    }

    fn as_str(&self) -> Option<&str> {
        if self.pos == 0 {
            return None;
        }
        std::str::from_utf8(&self.buf[..self.pos]).ok()
    }
}

static mut WORD_BUFFER: WordBuffer = WordBuffer::new();

// Cross-thread signal: UI automation thread sets this, hook callback checks and clears
static CLEAR_SIGNAL: AtomicBool = AtomicBool::new(false);
static IS_PASSWORD_FIELD: AtomicBool = AtomicBool::new(false);

fn signal_clear() {
    CLEAR_SIGNAL.store(true, Ordering::Release);
}

// ---------------------------------------------------------------------------
// Audio playback (hound + rodio, spawned thread so hook returns fast)
// ---------------------------------------------------------------------------

struct WavSource {
    samples: Vec<f32>,
    sample_rate: std::num::NonZeroU32,
    channels: std::num::NonZeroU16,
    pos: usize,
}

impl rodio::Source for WavSource {
    fn current_span_len(&self) -> Option<usize> {
        Some(self.samples.len() - self.pos)
    }

    fn channels(&self) -> std::num::NonZeroU16 {
        self.channels
    }

    fn sample_rate(&self) -> std::num::NonZeroU32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        let frames = self.samples.len() as u64 / self.channels.get() as u64;
        Some(std::time::Duration::from_secs_f64(
            frames as f64 / self.sample_rate.get() as f64,
        ))
    }
}

impl Iterator for WavSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.pos < self.samples.len() {
            let s = self.samples[self.pos];
            self.pos += 1;
            Some(s)
        } else {
            None
        }
    }
}

fn load_wav_samples(path: &Path) -> Result<WavSource, String> {
    let mut reader = hound::WavReader::open(path).map_err(|e| format!("open: {}", e))?;
    let spec = reader.spec();

    let samples: Vec<f32> = match spec.bits_per_sample {
        16 => reader
            .samples::<i16>()
            .map(|s| s.map(|v| v as f32 / 32768.0))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("read: {}", e))?,
        24 => reader
            .samples::<i32>()
            .map(|s| s.map(|v| (v >> 8) as f32 / 8388608.0))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("read: {}", e))?,
        8 => reader
            .samples::<i8>()
            .map(|s| s.map(|v| v as f32 / 128.0))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("read: {}", e))?,
        _ => return Err(format!("unsupported {}-bit", spec.bits_per_sample)),
    };

    Ok(WavSource {
        samples,
        sample_rate: std::num::NonZeroU32::new(spec.sample_rate).unwrap_or(std::num::NonZeroU32::new(44100).unwrap()),
        channels: std::num::NonZeroU16::new(spec.channels).unwrap_or(std::num::NonZeroU16::new(1).unwrap()),
        pos: 0,
    })
}

fn play_sound_async(config_path: String) {
    std::thread::spawn(move || {
        let sound_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join(&config_path)))
            .unwrap_or_else(|| PathBuf::from(&config_path));

        let source = match load_wav_samples(&sound_path) {
            Ok(s) => s,
            Err(e) => {
                if DEBUG_ENABLED.load(Ordering::Relaxed) {
                    eprintln!("[Simple-Jingle] WAV error: {}", e);
                }
                return;
            }
        };

        let state_mutex = AUDIO_STATE.get_or_init(|| Mutex::new(None));
        let mut state = state_mutex.lock().unwrap();

        if state.is_none() {
            match rodio::DeviceSinkBuilder::open_default_sink() {
                Ok(mut handle) => {
                    handle.log_on_drop(false);
                    let player = rodio::Player::connect_new(handle.mixer());
                    *state = Some(AudioState {
                        _handle: handle,
                        player,
                    });
                }
                Err(e) => {
                    if DEBUG_ENABLED.load(Ordering::Relaxed) {
                        eprintln!("[Simple-Jingle] Could not open audio device: {:?}", e);
                    }
                    return;
                }
            }
        }

        if let Some(audio) = state.as_ref() {
            audio.player.stop();
            audio.player.append(source);
        }
    });
}

// ---------------------------------------------------------------------------
// Window title blacklist (for password-field heuristics)
// ---------------------------------------------------------------------------

const BLACKLIST_KEYWORDS: &[&str] = &[
    "password",
    "login",
    "sign in",
    "credentials",
    "bitwarden",
    "1password",
    "keepass",
    "lastpass",
    "vault",
    "unlock",
    "master password",
    "encryption key",
    "private key",
    "secret key",
    "otp",
    "2fa",
    "authenticator",
];

fn is_blacklisted_window(hwnd: HWND) -> bool {
    if hwnd.is_invalid() {
        return false;
    }
    unsafe {
        let mut title_buf = [0u16; 256];
        let len = GetWindowTextW(hwnd, &mut title_buf);
        if len == 0 {
            return false;
        }
        let title = String::from_utf16_lossy(&title_buf[..len as usize]);
        let title_lower = title.to_lowercase();
        BLACKLIST_KEYWORDS
            .iter()
            .any(|kw| title_lower.contains(kw))
    }
}

// ---------------------------------------------------------------------------
// Keyboard hook
// ---------------------------------------------------------------------------

static HOOK_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static LAST_HWND: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

fn is_word_boundary(vk: u32) -> bool {
    let v = vk as u16;
    if let Some(config) = CONFIG.get() {
        config.triggers.boundary_vkeys.contains(&v)
    } else {
        v == VK_SPACE.0 || v == VK_RETURN.0 || v == VK_TAB.0
    }
}

fn is_letter(vk: u32) -> bool {
    let v = vk as u16;
    (VK_A.0..=VK_Z.0).contains(&v)
}

fn is_digit_or_symbol(vk: u32) -> bool {
    let v = vk as u16;
    (VK_0.0..=VK_9.0).contains(&v)
        || (VK_OEM_1.0..=VK_OEM_8.0).contains(&v)
        || v == VK_MULTIPLY.0
        || v == VK_ADD.0
        || v == VK_SEPARATOR.0
        || v == VK_SUBTRACT.0
        || v == VK_DECIMAL.0
        || v == VK_DIVIDE.0
}

fn is_navigation_key(vk: u32) -> bool {
    let v = vk as u16;
    v == VK_LEFT.0
        || v == VK_RIGHT.0
        || v == VK_UP.0
        || v == VK_DOWN.0
        || v == VK_HOME.0
        || v == VK_END.0
        || v == VK_PRIOR.0
        || v == VK_NEXT.0
        || v == VK_DELETE.0
        || v == VK_ESCAPE.0
}

unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code >= 0 && wparam.0 == WM_KEYDOWN as usize {
        let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let vk = kb.vkCode;

        // Check cross-thread clear signal first
        if CLEAR_SIGNAL.swap(false, Ordering::Acquire) {
            WORD_BUFFER.clear();
        }

        // Modifier combos — ignore shortcuts
        let ctrl = GetAsyncKeyState(VK_CONTROL.0.into());
        let alt = GetAsyncKeyState(VK_MENU.0.into());
        let lwin = GetAsyncKeyState(VK_LWIN.0.into());
        let rwin = GetAsyncKeyState(VK_RWIN.0.into());
        if (ctrl | alt | lwin | rwin) & 0x8000u16 as i16 != 0 {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        // Focus change — clear buffer
        let hwnd = GetForegroundWindow();
        if hwnd.0 != LAST_HWND.load(Ordering::Relaxed) {
            LAST_HWND.store(hwnd.0, Ordering::Relaxed);
            WORD_BUFFER.clear();
        }

        // Blacklisted window (password managers, login pages, etc.)
        if is_blacklisted_window(hwnd) {
            WORD_BUFFER.clear();
            return CallNextHookEx(None, code, wparam, lparam);
        }

        // Password protection check
        if IS_PASSWORD_FIELD.load(Ordering::Acquire) {
            WORD_BUFFER.clear();
            return CallNextHookEx(None, code, wparam, lparam);
        }

        let vk_u16 = vk as u16;

        if is_letter(vk) {
            let letter = (vk_u16 - VK_A.0 + b'a' as u16) as u8;
            WORD_BUFFER.push_byte(letter);

            #[cfg(debug_assertions)]
            if let Some(w) = WORD_BUFFER.as_str() {
                println!("[Simple-Jingle] buffer: \"{}\"", w);
            }
        } else if is_word_boundary(vk) {
            if let Some(word) = WORD_BUFFER.as_str() {
                if let Some(config) = CONFIG.get() {
                    let matched = config.triggers.words.iter().any(|trigger| {
                        if config.triggers.case_sensitive {
                            word == trigger.as_str()
                        } else {
                            word.eq_ignore_ascii_case(trigger.as_str())
                        }
                    });

                    if matched {
                        if DEBUG_ENABLED.load(Ordering::Relaxed) {
                            println!("[Simple-Jingle] TRIGGER DETECTED: \"{}\"", word);
                        }
                        if AUDIO_ENABLED.load(Ordering::Relaxed) {
                            play_sound_async(config.sound.path.clone());
                        }
                    }
                }
            }
            WORD_BUFFER.clear();
        } else if vk_u16 == VK_BACK.0 {
            WORD_BUFFER.pop();
        } else if is_digit_or_symbol(vk) || is_navigation_key(vk) {
            WORD_BUFFER.clear();
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

// ---------------------------------------------------------------------------
// Tray icon
// ---------------------------------------------------------------------------

fn create_icon() -> Icon {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default();

    for ext in &["png", "jpg", "jpeg"] {
        let icon_path = exe_dir.join(format!("icon.{}", ext));
        if let Ok(img) = image::open(&icon_path) {
            let img = img.to_rgba8();
            let (w, h) = img.dimensions();
            let data = img.into_raw();
            if let Ok(icon) = Icon::from_rgba(data, w, h) {
                return icon;
            }
        }
    }

    let size = 32u32;
    let mut data = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let cx = size as i32 / 2;
            let cy = size as i32 / 2;
            let dx = x as i32 - cx;
            let dy = y as i32 - cy;
            if dx * dx + dy * dy < (cx - 2) * (cx - 2) {
                data[idx] = 100;
                data[idx + 1] = 180;
                data[idx + 2] = 255;
                data[idx + 3] = 255;
            }
        }
    }
    Icon::from_rgba(data, size, size).unwrap()
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn run_app() {
    let config = load_config();
    DEBUG_ENABLED.store(config.debug.enabled, Ordering::Relaxed);
    CONFIG.set(config).ok();

    if DEBUG_ENABLED.load(Ordering::Relaxed) {
        unsafe {
            let _ = AllocConsole();
        }
        println!("[Simple-Jingle] Debug mode ON");
        println!("[Simple-Jingle] Hook installed — type a trigger word then press Space");
        println!("[Simple-Jingle] Trigger words: {:?}", CONFIG.get().unwrap().triggers.words);
    }

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }

    // Background thread: monitor focused window for blacklist / focus changes / UI Automation password fields
    std::thread::spawn(|| {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let automation: Result<IUIAutomation, _> = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER);
            let mut last_hwnd = HWND(ptr::null_mut());
            loop {
                std::thread::sleep(std::time::Duration::from_millis(150));
                let hwnd = GetForegroundWindow();
                if hwnd != last_hwnd {
                    last_hwnd = hwnd;
                    signal_clear();
                }

                // Check title heuristics
                if is_blacklisted_window(hwnd) {
                    IS_PASSWORD_FIELD.store(true, Ordering::Release);
                    signal_clear();
                    continue;
                }

                // Check UI Automation focus element IsPassword
                let mut is_pass = false;
                if let Ok(ref auto) = automation {
                    if let Ok(el) = auto.GetFocusedElement() {
                        if let Ok(val) = el.CurrentIsPassword() {
                            is_pass = val.as_bool();
                        }
                    }
                }

                if is_pass {
                    IS_PASSWORD_FIELD.store(true, Ordering::Release);
                    signal_clear();
                } else {
                    IS_PASSWORD_FIELD.store(false, Ordering::Release);
                }
            }
        }
    });

    // Tray menu
    let menu = Menu::new();
    let toggle_item = MenuItem::new("Disable", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    menu.append(&toggle_item).unwrap();
    menu.append(&muda::PredefinedMenuItem::separator())
        .unwrap();
    menu.append(&quit_item).unwrap();

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Simple-Jingle - Listening for trigger words")
        .with_icon(create_icon())
        .build()
        .unwrap();

    // Install keyboard hook
    unsafe {
        match SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0) {
            Ok(h) => {
                HOOK_HANDLE.store(h.0, Ordering::SeqCst);
                if DEBUG_ENABLED.load(Ordering::Relaxed) {
                    println!("[Simple-Jingle] Keyboard hook OK");
                }
            }
            Err(e) => {
                eprintln!("[Simple-Jingle] Hook FAILED: {:?}", e);
                return;
            }
        }
    }

    // Message loop
    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();

    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            if let Ok(event) = menu_channel.try_recv() {
                if event.id == toggle_item.id() {
                    let enabled = AUDIO_ENABLED.load(Ordering::Relaxed);
                    AUDIO_ENABLED.store(!enabled, Ordering::Relaxed);
                    if enabled {
                        toggle_item.set_text("Enable");
                        tray_icon.set_tooltip(Some("Simple-Jingle - Disabled")).unwrap();
                        if DEBUG_ENABLED.load(Ordering::Relaxed) {
                            println!("[Simple-Jingle] Disabled");
                        }
                    } else {
                        toggle_item.set_text("Disable");
                        tray_icon
                            .set_tooltip(Some("Simple-Jingle - Listening"))
                            .unwrap();
                        if DEBUG_ENABLED.load(Ordering::Relaxed) {
                            println!("[Simple-Jingle] Enabled");
                        }
                    }
                } else if event.id == quit_item.id() {
                    break;
                }
            }
            let _ = tray_channel.try_recv();
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    // Cleanup
    unsafe {
        let ptr = HOOK_HANDLE.swap(ptr::null_mut(), Ordering::SeqCst);
        if !ptr.is_null() {
            let _ = UnhookWindowsHookEx(HHOOK(ptr));
        }
        WORD_BUFFER.clear();
    }

    if DEBUG_ENABLED.load(Ordering::Relaxed) {
        println!("[Simple-Jingle] Exiting");
    }
}

fn main() {
    run_app();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn simulate_key(vk: u16) {
        let kb = KBDLLHOOKSTRUCT {
            vkCode: vk as u32,
            scanCode: 0,
            flags: KBDLLHOOKSTRUCT_FLAGS(0),
            time: 0,
            dwExtraInfo: 0,
        };
        unsafe {
            let _ = keyboard_hook_proc(0, WPARAM(WM_KEYDOWN as usize), LPARAM(&kb as *const _ as isize));
        }
    }

    #[test]
    fn test_word_buffer() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            WORD_BUFFER.clear();
            WORD_BUFFER.push_byte(b'h');
            WORD_BUFFER.push_byte(b'i');
            assert_eq!(WORD_BUFFER.as_str(), Some("hi"));
            WORD_BUFFER.clear();
            assert_eq!(WORD_BUFFER.as_str(), None);
        }
    }

    #[test]
    fn test_backspace() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            WORD_BUFFER.clear();
            WORD_BUFFER.push_byte(b'a');
            WORD_BUFFER.push_byte(b'b');
            WORD_BUFFER.pop();
            assert_eq!(WORD_BUFFER.as_str(), Some("a"));
        }
    }

    #[test]
    fn test_overflow() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            WORD_BUFFER.clear();
            for _ in 0..65 {
                WORD_BUFFER.push_byte(b'x');
            }
            assert_eq!(WORD_BUFFER.as_str(), None);
        }
    }

    #[test]
    fn test_hook_letters() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            WORD_BUFFER.clear();
            IS_PASSWORD_FIELD.store(false, Ordering::Release);
            simulate_key(VK_A.0);
            simulate_key(VK_B.0);
            assert_eq!(WORD_BUFFER.as_str(), Some("ab"));
        }
    }

    #[test]
    fn test_hook_password_field() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            WORD_BUFFER.clear();
            WORD_BUFFER.push_byte(b'a');
            IS_PASSWORD_FIELD.store(true, Ordering::Release);
            simulate_key(VK_B.0); // Typing in password field
            assert_eq!(WORD_BUFFER.as_str(), None); // Buffer should be cleared and key ignored
            IS_PASSWORD_FIELD.store(false, Ordering::Release);
        }
    }

    #[test]
    fn test_hook_digit_wipe() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            WORD_BUFFER.clear();
            simulate_key(VK_A.0);
            simulate_key(VK_3.0); // Digit
            assert_eq!(WORD_BUFFER.as_str(), None); // Buffer wiped
        }
    }

    #[test]
    fn test_hook_navigation_wipe() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            WORD_BUFFER.clear();
            simulate_key(VK_A.0);
            simulate_key(VK_LEFT.0); // Left Arrow
            assert_eq!(WORD_BUFFER.as_str(), None); // Buffer wiped
        }
    }
}
