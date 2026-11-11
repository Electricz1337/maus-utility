//! Maus-Utility – Autoclicker, Dauerklick, Farb-Trigger, Klickfolgen und Cursor-Info.
//! Reines Win32 + GDI+ ueber FFI, keine externen Crates.

#![windows_subsystem = "windows"]
#![allow(clippy::too_many_arguments, clippy::missing_safety_doc)]

mod ffi;

use ffi::*;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicIsize, AtomicU32, AtomicU64, Ordering::SeqCst};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

// ═════════════════════════ Farben (an die Claude-App angelehnt) ═════════════════════════

#[derive(Clone, Copy)]
struct Pal {
    bg: u32,
    card: u32,
    border: u32,
    input: u32,
    input_border: u32,
    text: u32,
    muted: u32,
    accent: u32,
    accent_hover: u32,
    accent_text: u32,
    accent_soft: u32,
    sec: u32,
    sec_hover: u32,
    sel: u32,
}

const LIGHT: Pal = Pal {
    bg: 0xF0EEE6,
    card: 0xFAF9F5,
    border: 0xE3E0D5,
    input: 0xFFFFFF,
    input_border: 0xD6D3C7,
    text: 0x1F1E1D,
    muted: 0x73716A,
    accent: 0xD97757,
    accent_hover: 0xC6613F,
    accent_text: 0xFFFFFF,
    accent_soft: 0xF6E3DA,
    sec: 0xEDEAE0,
    sec_hover: 0xE2DED2,
    sel: 0xEFEBE1,
};

const DARK: Pal = Pal {
    bg: 0x1F1E1D,
    card: 0x262624,
    border: 0x3A3936,
    input: 0x30302E,
    input_border: 0x48463F,
    text: 0xF2F0E8,
    muted: 0xA29F96,
    accent: 0xD97757,
    accent_hover: 0xE38B6C,
    accent_text: 0xFFFFFF,
    accent_soft: 0x47312A,
    sec: 0x34332F,
    sec_hover: 0x403E3A,
    sel: 0x3B3A36,
};

static DARK_MODE: AtomicBool = AtomicBool::new(false);

fn pal() -> Pal {
    if DARK_MODE.load(SeqCst) { DARK } else { LIGHT }
}

/// 0xRRGGBB -> COLORREF (0x00BBGGRR)
fn cref(rgb: u32) -> u32 {
    ((rgb & 0xFF) << 16) | (rgb & 0xFF00) | ((rgb >> 16) & 0xFF)
}

/// COLORREF -> 0xRRGGBB
fn rgb_of(c: u32) -> u32 {
    cref(c)
}

fn argb(rgb: u32) -> u32 {
    0xFF00_0000 | rgb
}

// ═════════════════════════ IDs ═════════════════════════

const ID_A_M1: i32 = 101;
const ID_A_M2: i32 = 102;
const ID_A_INT: i32 = 103;
const ID_A_INT_U: i32 = 104;
const ID_A_JIT: i32 = 105;
const ID_A_JIT_U: i32 = 106;
const ID_A_HK: i32 = 107;
const ID_A_GO: i32 = 108;

const ID_H_M1: i32 = 201;
const ID_H_M2: i32 = 202;
const ID_H_HK: i32 = 203;
const ID_H_GO: i32 = 204;
const ID_H_DUR: i32 = 205;
const ID_H_DUR_U: i32 = 206;

const ID_T_X: i32 = 301;
const ID_T_Y: i32 = 302;
const ID_T_HEX: i32 = 303;
const ID_T_TOL: i32 = 304;
const ID_T_OFF: i32 = 305;
const ID_T_INT: i32 = 306;
const ID_T_INT_U: i32 = 307;
const ID_T_M1: i32 = 308;
const ID_T_M2: i32 = 309;
const ID_T_ONCE: i32 = 310;
const ID_T_RESTORE: i32 = 311;
const ID_T_HK: i32 = 312;
const ID_T_GO: i32 = 313;
const ID_T_COOL: i32 = 314;
const ID_T_COOL_U: i32 = 315;

const ID_I_SW: i32 = 401;
const ID_I_LIST: i32 = 402;
const ID_I_COPY: i32 = 403;
const ID_I_TOTRIG: i32 = 404;
const ID_I_CLEAR: i32 = 405;

const ID_S_LIST: i32 = 501;
const ID_S_ADD1: i32 = 502;
const ID_S_ADD2: i32 = 503;
const ID_S_WAIT: i32 = 504;
const ID_S_WAIT_U: i32 = 505;
const ID_S_ADDW: i32 = 506;
const ID_S_UP: i32 = 507;
const ID_S_DOWN: i32 = 508;
const ID_S_DEL: i32 = 509;
const ID_S_CLEAR: i32 = 510;
const ID_S_LOOP: i32 = 511;
const ID_S_ROUNDS: i32 = 512;
const ID_S_HK: i32 = 513;
const ID_S_GO: i32 = 514;
const ID_S_SAVE: i32 = 515;
const ID_S_LOAD: i32 = 516;

const ID_SEQ_TOGGLE: i32 = 601;
const ID_THEME: i32 = 602;
const ID_INFO_TOGGLE: i32 = 603;

const TM_SHOW: i32 = 1001;
const TM_AUTO: i32 = 1002;
const TM_HOLD: i32 = 1003;
const TM_TRIG: i32 = 1004;
const TM_SEQ: i32 = 1005;
const TM_EXIT: i32 = 1006;

const WM_TRAY: u32 = WM_APP + 1;
const WM_GRAB: u32 = WM_APP + 2;

/// Markiert eigene, simulierte Klicks (damit sie die eigenen Buttons nicht ausloesen).
const MAGIC: usize = 0x4D41_5553;

// ═════════════════════════ Layout (logische Pixel bei 96 DPI) ═════════════════════════

const W_COLLAPSED: i32 = 804;
const W_EXPANDED: i32 = 1156;
const H_COLLAPSED: i32 = 678;
const H_EXPANDED: i32 = 788;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Primary,
    Secondary,
    Segment,
    Dropdown,
    Hotkey,
    Switch,
    Icon,
    Ghost,
}

struct Btn {
    id: i32,
    kind: Kind,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    seq: bool,
}

const fn b(id: i32, kind: Kind, x: i32, y: i32, w: i32, seq: bool) -> Btn {
    Btn { id, kind, x, y, w, h: 30, seq }
}

const BTNS: &[Btn] = &[
    // Autoclicker
    b(ID_A_M1, Kind::Segment, 132, 108, 120, false),
    b(ID_A_M2, Kind::Segment, 260, 108, 120, false),
    b(ID_A_INT_U, Kind::Dropdown, 260, 146, 120, false),
    b(ID_A_JIT_U, Kind::Dropdown, 260, 184, 120, false),
    b(ID_A_HK, Kind::Hotkey, 132, 222, 120, false),
    b(ID_A_GO, Kind::Primary, 260, 222, 120, false),
    // Dauerklick
    b(ID_H_M1, Kind::Segment, 132, 326, 120, false),
    b(ID_H_M2, Kind::Segment, 260, 326, 120, false),
    b(ID_H_DUR_U, Kind::Dropdown, 260, 364, 120, false),
    b(ID_H_HK, Kind::Hotkey, 132, 402, 120, false),
    b(ID_H_GO, Kind::Primary, 260, 402, 120, false),
    // Farb-Trigger
    b(ID_T_COOL_U, Kind::Dropdown, 652, 260, 120, false),
    b(ID_T_INT_U, Kind::Dropdown, 652, 298, 120, false),
    b(ID_T_M1, Kind::Segment, 524, 336, 120, false),
    b(ID_T_M2, Kind::Segment, 652, 336, 120, false),
    b(ID_T_ONCE, Kind::Switch, 424, 374, 170, false),
    b(ID_T_RESTORE, Kind::Switch, 604, 374, 168, false),
    b(ID_T_HK, Kind::Hotkey, 524, 412, 120, false),
    b(ID_T_GO, Kind::Primary, 652, 412, 120, false),
    // Cursor-Info
    b(ID_I_SW, Kind::Switch, 652, 475, 120, false),
    b(ID_I_COPY, Kind::Secondary, 628, 516, 144, false),
    b(ID_I_TOTRIG, Kind::Secondary, 628, 551, 144, false),
    b(ID_I_CLEAR, Kind::Secondary, 628, 586, 144, false),
    // Klickfolge
    b(ID_S_ADD1, Kind::Secondary, 816, 384, 150, true),
    b(ID_S_ADD2, Kind::Secondary, 974, 384, 150, true),
    b(ID_S_WAIT_U, Kind::Dropdown, 914, 422, 90, true),
    b(ID_S_ADDW, Kind::Secondary, 1012, 422, 112, true),
    b(ID_S_UP, Kind::Secondary, 816, 460, 60, true),
    b(ID_S_DOWN, Kind::Secondary, 884, 460, 60, true),
    b(ID_S_DEL, Kind::Secondary, 952, 460, 88, true),
    b(ID_S_CLEAR, Kind::Secondary, 1048, 460, 76, true),
    b(ID_S_SAVE, Kind::Secondary, 816, 498, 150, true),
    b(ID_S_LOAD, Kind::Secondary, 974, 498, 150, true),
    b(ID_S_LOOP, Kind::Switch, 816, 548, 110, true),
    b(ID_S_HK, Kind::Hotkey, 900, 586, 108, true),
    b(ID_S_GO, Kind::Primary, 1016, 586, 108, true),
    // Kopfzeile (Position wird dynamisch gesetzt)
    Btn { id: ID_SEQ_TOGGLE, kind: Kind::Ghost, x: 0, y: 15, w: 190, h: 30, seq: false },
    Btn { id: ID_THEME, kind: Kind::Icon, x: 0, y: 12, w: 36, h: 36, seq: false },
    Btn { id: ID_INFO_TOGGLE, kind: Kind::Ghost, x: 16, y: 640, w: 170, h: 28, seq: false },
];

struct Ed {
    id: i32,
    x: i32,
    y: i32,
    w: i32,
    prefix: &'static str,
    pw: i32,
    def: &'static str,
    seq: bool,
}

const fn e(id: i32, x: i32, y: i32, w: i32, prefix: &'static str, pw: i32, def: &'static str, seq: bool) -> Ed {
    Ed { id, x, y, w, prefix, pw, def, seq }
}

const EDS: &[Ed] = &[
    e(ID_A_INT, 132, 146, 120, "", 0, "100", false),
    e(ID_A_JIT, 132, 184, 120, "±", 14, "0", false),
    e(ID_T_X, 524, 108, 120, "X", 16, "0", false),
    e(ID_T_Y, 652, 108, 120, "Y", 16, "0", false),
    e(ID_T_HEX, 524, 146, 120, "Hex #", 42, "D97757", false),
    e(ID_T_TOL, 524, 184, 120, "", 0, "10", false),
    e(ID_T_OFF, 524, 222, 120, "±", 14, "0", false),
    e(ID_T_COOL, 524, 260, 120, "", 0, "0", false),
    e(ID_T_INT, 524, 298, 120, "", 0, "100", false),
    e(ID_H_DUR, 132, 364, 120, "", 0, "0", false),
    e(ID_S_WAIT, 816, 422, 90, "", 0, "1", true),
    e(ID_S_ROUNDS, 1044, 548, 80, "", 0, "0", true),
];

struct ListDef {
    id: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    seq: bool,
}

const LISTS: &[ListDef] = &[
    ListDef { id: ID_I_LIST, x: 316, y: 516, w: 300, h: 100, seq: false },
    ListDef { id: ID_S_LIST, x: 816, y: 108, w: 308, h: 266, seq: true },
];

#[derive(Clone, Copy, PartialEq)]
enum St {
    None,
    Auto,
    Hold,
    Trig,
    Seq,
}

struct Card {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    title: &'static str,
    status: St,
    seq: bool,
}

const CARDS: &[Card] = &[
    Card { x: 16, y: 60, w: 380, h: 206, title: "Autoclicker", status: St::Auto, seq: false },
    Card { x: 16, y: 278, w: 380, h: 178, title: "Dauerklick", status: St::Hold, seq: false },
    Card { x: 408, y: 60, w: 380, h: 396, title: "Farb-Trigger", status: St::Trig, seq: false },
    Card { x: 16, y: 468, w: 772, h: 162, title: "Cursor-Info", status: St::None, seq: false },
    Card { x: 800, y: 60, w: 340, h: 570, title: "Klickfolge", status: St::Seq, seq: true },
];

/// style: 0 = normal, 1 = gedaempft, 2 = gedaempft rechtsbuendig
struct Label {
    x: i32,
    y: i32,
    w: i32,
    text: &'static str,
    style: u8,
    seq: bool,
}

const fn l(x: i32, y: i32, w: i32, text: &'static str, style: u8, seq: bool) -> Label {
    Label { x, y, w, text, style, seq }
}

const LABELS: &[Label] = &[
    l(32, 108, 96, "Maustaste", 0, false),
    l(32, 146, 96, "Intervall", 0, false),
    l(32, 184, 96, "Zufall-Offset", 0, false),
    l(32, 222, 96, "Hotkey", 0, false),
    l(32, 326, 96, "Maustaste", 0, false),
    l(32, 364, 96, "Loslassen nach", 0, false),
    l(32, 402, 96, "Hotkey", 0, false),
    l(424, 108, 96, "Position", 0, false),
    l(424, 146, 96, "Farbe", 0, false),
    l(424, 184, 96, "Toleranz", 0, false),
    l(424, 222, 96, "Klick-Offset", 0, false),
    l(424, 260, 96, "Cooldown", 0, false),
    l(424, 298, 96, "Prüfintervall", 0, false),
    l(424, 336, 96, "Maustaste", 0, false),
    l(424, 412, 96, "Hotkey", 0, false),
    l(690, 146, 82, "Zielfarbe", 1, false),
    l(652, 184, 120, "0–255 je Kanal", 1, false),
    l(652, 222, 120, "Pixel, zufällig", 1, false),
    l(32, 586, 280, "Doppelklick auf Eintrag → Farb-Trigger", 1, false),
    l(920, 548, 116, "Durchläufe (0 = ∞)", 2, true),
    l(816, 586, 80, "Hotkey", 0, true),
];

// ═════════════════════════ Zustand ═════════════════════════

static MAIN: AtomicIsize = AtomicIsize::new(0);
static SCALE_PCT: AtomicI32 = AtomicI32::new(100);
static RNG: AtomicU64 = AtomicU64::new(0x9E37_79B9_7F4A_7C15);
static ORIG_BTN_PROC: AtomicIsize = AtomicIsize::new(0);
static HOVER: AtomicIsize = AtomicIsize::new(0);
static FOCUS_ED: AtomicI32 = AtomicI32::new(0);
static INPUT_BRUSH: AtomicIsize = AtomicIsize::new(0);
static KB_HOOK: AtomicIsize = AtomicIsize::new(0);
static SEQ_OPEN: AtomicBool = AtomicBool::new(false);
static INFO_OPEN: AtomicBool = AtomicBool::new(false);

struct Fonts {
    normal: HANDLE,
    small: HANDLE,
    semi: HANDLE,
    btn: HANDLE,
    title: HANDLE,
    mono: HANDLE,
}
static FONTS: OnceLock<Fonts> = OnceLock::new();

struct Icons {
    big: HANDLE,
    small: HANDLE,
    header: HANDLE,
}
static ICONS: OnceLock<Icons> = OnceLock::new();

fn fonts() -> &'static Fonts {
    FONTS.get().expect("fonts")
}

// Einheiten: 0 = ms, 1 = s, 2 = min, 3 = h
static U_A_INT: AtomicU32 = AtomicU32::new(0);
static U_A_JIT: AtomicU32 = AtomicU32::new(0);
static U_T_INT: AtomicU32 = AtomicU32::new(0);
static U_S_WAIT: AtomicU32 = AtomicU32::new(1);
static U_T_COOL: AtomicU32 = AtomicU32::new(1);
static U_H_DUR: AtomicU32 = AtomicU32::new(1);
const UNIT_LONG: [&str; 4] = ["Millisekunden", "Sekunden", "Minuten", "Stunden"];
const UNIT_FACTOR: [f64; 4] = [1.0, 1000.0, 60_000.0, 3_600_000.0];

fn unit_of(id: i32) -> &'static AtomicU32 {
    match id {
        ID_A_INT_U => &U_A_INT,
        ID_A_JIT_U => &U_A_JIT,
        ID_T_INT_U => &U_T_INT,
        ID_T_COOL_U => &U_T_COOL,
        ID_H_DUR_U => &U_H_DUR,
        _ => &U_S_WAIT,
    }
}

// Hotkeys: (vk, mods) fuer Autoclicker, Dauerklick, Farb-Trigger, Klickfolge
static HOTKEYS: Mutex<[(u32, u32); 4]> = Mutex::new([(0x75, 0), (0x76, 0), (0x78, 0), (0x77, 0)]);
static HK_FAIL: Mutex<String> = Mutex::new(String::new());
static CAPTURE: AtomicI32 = AtomicI32::new(0);
const HK_BUTTONS: [i32; 4] = [ID_A_HK, ID_H_HK, ID_T_HK, ID_S_HK];

// Autoclicker
static AUTO_RUN: AtomicBool = AtomicBool::new(false);
static AUTO_GEN: AtomicU32 = AtomicU32::new(0);
static AUTO_BTN: AtomicU32 = AtomicU32::new(0);
static AUTO_INTERVAL: AtomicU64 = AtomicU64::new(100);
static AUTO_JITTER: AtomicU64 = AtomicU64::new(0);
static AUTO_COUNT: AtomicU64 = AtomicU64::new(0);

// Dauerklick
static HOLD_ON: AtomicBool = AtomicBool::new(false);
static HOLD_SEL: AtomicU32 = AtomicU32::new(0);
static HOLD_ACTIVE: AtomicU32 = AtomicU32::new(0);
static HOLD_GEN: AtomicU32 = AtomicU32::new(0);

// Farb-Trigger
static TRIG_RUN: AtomicBool = AtomicBool::new(false);
static TRIG_GEN: AtomicU32 = AtomicU32::new(0);
static TRIG_BTN: AtomicU32 = AtomicU32::new(0);
static TRIG_ONCE: AtomicBool = AtomicBool::new(true);
static TRIG_RESTORE: AtomicBool = AtomicBool::new(true);
static TRIG_HITS: AtomicU64 = AtomicU64::new(0);
static TRIG_COOLING: AtomicBool = AtomicBool::new(false);

// Cursor-Info
static INFO_ON: AtomicBool = AtomicBool::new(false);
static INFO_X: AtomicI32 = AtomicI32::new(0);
static INFO_Y: AtomicI32 = AtomicI32::new(0);
static INFO_COLOR: AtomicU32 = AtomicU32::new(u32::MAX);
static MERK: Mutex<Vec<(i32, i32, u32)>> = Mutex::new(Vec::new());

// Klickfolge
#[derive(Clone, Copy)]
enum Step {
    Click(u32),
    Wait(u64),
}
static SEQ: Mutex<Vec<Step>> = Mutex::new(Vec::new());
static SEQ_RUN: AtomicBool = AtomicBool::new(false);
static SEQ_GEN: AtomicU32 = AtomicU32::new(0);
static SEQ_STEP: AtomicI32 = AtomicI32::new(-1);
static SEQ_LEN: AtomicU32 = AtomicU32::new(0);
static SEQ_ROUND: AtomicU32 = AtomicU32::new(0);
static SEQ_ROUNDS_TOTAL: AtomicU32 = AtomicU32::new(0);
static SEQ_LOOP: AtomicBool = AtomicBool::new(false);
static SEQ_DRAWN_STEP: AtomicI32 = AtomicI32::new(-2);

static SNAP_MAIN: Mutex<String> = Mutex::new(String::new());
static SNAP_BTN: Mutex<String> = Mutex::new(String::new());
static TRAY_TIP: Mutex<String> = Mutex::new(String::new());

// ═════════════════════════ Hilfsfunktionen ═════════════════════════

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn s(v: i32) -> i32 {
    v * SCALE_PCT.load(SeqCst) / 100
}

fn sf(v: i32) -> f32 {
    s(v) as f32
}

fn main_hwnd() -> HWND {
    MAIN.load(SeqCst)
}

fn item(id: i32) -> HWND {
    unsafe { GetDlgItem(main_hwnd(), id) }
}

fn get_text(id: i32) -> String {
    let mut buf = [0u16; 256];
    let n = unsafe { GetWindowTextW(item(id), buf.as_mut_ptr(), buf.len() as i32) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

fn set_text(id: i32, text: &str) {
    let w = wide(text);
    unsafe { SetWindowTextW(item(id), w.as_ptr()) };
}

fn msg_box(text: &str) {
    let t = wide(text);
    let c = wide("Maus-Utility");
    unsafe { MessageBoxW(main_hwnd(), t.as_ptr(), c.as_ptr(), MB_ICONWARNING) };
}

fn rand_below(n: u64) -> u64 {
    let mut x = RNG.load(SeqCst);
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    RNG.store(x, SeqCst);
    x.wrapping_mul(0x2545_F491_4F6C_DD1D) % n.max(1)
}

/// Zufallswert im Bereich [-r, +r]
fn rand_pm(r: i64) -> i64 {
    if r <= 0 { 0 } else { rand_below((2 * r + 1) as u64) as i64 - r }
}

fn sleep_while(ms: u64, alive: &dyn Fn() -> bool) -> bool {
    let end = Instant::now() + Duration::from_millis(ms);
    loop {
        if !alive() {
            return false;
        }
        let now = Instant::now();
        if now >= end {
            return true;
        }
        std::thread::sleep((end - now).min(Duration::from_millis(10)));
    }
}

fn parse_num(text: &str) -> Option<f64> {
    text.trim()
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v >= 0.0)
}

/// Liest ein Zahlenfeld zusammen mit seiner Einheit und liefert Millisekunden.
fn read_dur(edit: i32, unit_btn: i32) -> Option<u64> {
    let u = unit_of(unit_btn).load(SeqCst) as usize;
    parse_num(&get_text(edit)).map(|v| (v * UNIT_FACTOR[u]).round().min(1e15) as u64)
}

fn parse_hex(text: &str) -> Option<u32> {
    let t: String = text
        .trim()
        .trim_start_matches(|c: char| c.is_ascii_alphabetic() && !c.is_ascii_hexdigit() || c == ' ')
        .chars()
        .filter(|c| *c != '#' && !c.is_whitespace())
        .collect();
    let t = t.strip_prefix("Hex").or_else(|| t.strip_prefix("hex")).unwrap_or(&t);
    if t.len() == 6 { u32::from_str_radix(t, 16).ok() } else { None }
}

fn fmt_dur(ms: u64) -> String {
    if ms >= 3_600_000 && ms.is_multiple_of(3_600_000) {
        format!("{} h", ms / 3_600_000)
    } else if ms >= 60_000 && ms.is_multiple_of(60_000) {
        format!("{} min", ms / 60_000)
    } else if ms >= 1000 {
        let mut t = format!("{:.3}", ms as f64 / 1000.0);
        while t.ends_with('0') {
            t.pop();
        }
        if t.ends_with('.') {
            t.pop();
        }
        format!("{} s", t.replace('.', ","))
    } else {
        format!("{} ms", ms)
    }
}

fn key_name(vk: u32, mods: u32) -> String {
    if vk == 0 {
        return "–".into();
    }
    let mut out = String::new();
    if mods & MOD_CONTROL != 0 {
        out += "Strg+";
    }
    if mods & MOD_ALT != 0 {
        out += "Alt+";
    }
    if mods & MOD_SHIFT != 0 {
        out += "Umschalt+";
    }
    let key = match vk {
        0x70..=0x87 => format!("F{}", vk - 0x6F),
        0x30..=0x39 | 0x41..=0x5A => (vk as u8 as char).to_string(),
        0x60..=0x69 => format!("Num {}", vk - 0x60),
        _ => unsafe {
            let mut sc = MapVirtualKeyW(vk, 0);
            if matches!(vk, 0x21..=0x2E | 0x5B | 0x5C | 0x6F | 0x90) {
                sc |= 0x100;
            }
            let mut buf = [0u16; 64];
            let n = GetKeyNameTextW((sc << 16) as i32, buf.as_mut_ptr(), 64);
            if n > 0 {
                String::from_utf16_lossy(&buf[..n as usize])
            } else {
                format!("Taste {}", vk)
            }
        },
    };
    out + &key
}

fn read_pixel(x: i32, y: i32) -> u32 {
    unsafe {
        let dc = GetDC(0);
        let c = GetPixel(dc, x, y);
        ReleaseDC(0, dc);
        c
    }
}

fn copy_to_clipboard(text: &str) {
    let w = wide(text);
    unsafe {
        if OpenClipboard(main_hwnd()) == 0 {
            return;
        }
        EmptyClipboard();
        let h = GlobalAlloc(GMEM_MOVEABLE, w.len() * 2);
        if h != 0 {
            let p = GlobalLock(h) as *mut u16;
            if !p.is_null() {
                std::ptr::copy_nonoverlapping(w.as_ptr(), p, w.len());
                GlobalUnlock(h);
                SetClipboardData(CF_UNICODETEXT, h);
            }
        }
        CloseClipboard();
    }
}

// ═════════════════════════ Maus-Eingaben ═════════════════════════

fn down_flag(b: u32) -> u32 {
    if b == 0 { MOUSEEVENTF_LEFTDOWN } else { MOUSEEVENTF_RIGHTDOWN }
}

fn up_flag(b: u32) -> u32 {
    if b == 0 { MOUSEEVENTF_LEFTUP } else { MOUSEEVENTF_RIGHTUP }
}

fn send_mouse(flags: &[u32]) {
    let inputs: Vec<INPUT> = flags
        .iter()
        .map(|&f| INPUT {
            kind: 0,
            mi: MOUSEINPUT { dx: 0, dy: 0, mouseData: 0, dwFlags: f, time: 0, dwExtraInfo: MAGIC },
        })
        .collect();
    unsafe { SendInput(inputs.len() as u32, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32) };
}

fn click(b: u32) {
    send_mouse(&[down_flag(b), up_flag(b)]);
}

fn click_at(x: i32, y: i32, b: u32, restore: bool) {
    unsafe {
        let mut old = POINT::default();
        GetCursorPos(&mut old);
        SetCursorPos(x, y);
        std::thread::sleep(Duration::from_millis(5));
        click(b);
        if restore {
            std::thread::sleep(Duration::from_millis(15));
            SetCursorPos(old.x, old.y);
        }
    }
}

// ═════════════════════════ Funktionen ═════════════════════════

fn read_auto_settings() {
    AUTO_INTERVAL.store(read_dur(ID_A_INT, ID_A_INT_U).unwrap_or(100).max(1), SeqCst);
    AUTO_JITTER.store(read_dur(ID_A_JIT, ID_A_JIT_U).unwrap_or(0), SeqCst);
}

fn toggle_auto() {
    if AUTO_RUN.swap(false, SeqCst) {
        return;
    }
    read_auto_settings();
    let gen = AUTO_GEN.fetch_add(1, SeqCst) + 1;
    AUTO_COUNT.store(0, SeqCst);
    AUTO_RUN.store(true, SeqCst);
    std::thread::spawn(move || {
        let alive = || AUTO_RUN.load(SeqCst) && AUTO_GEN.load(SeqCst) == gen;
        while alive() {
            click(AUTO_BTN.load(SeqCst));
            AUTO_COUNT.fetch_add(1, SeqCst);
            let base = AUTO_INTERVAL.load(SeqCst).max(1) as i64;
            let jitter = AUTO_JITTER.load(SeqCst) as i64;
            let delay = (base + rand_pm(jitter)).max(1) as u64;
            if !sleep_while(delay, &alive) {
                break;
            }
        }
    });
}

fn toggle_hold() {
    if HOLD_ON.swap(false, SeqCst) {
        send_mouse(&[up_flag(HOLD_ACTIVE.load(SeqCst))]);
    } else {
        let b = HOLD_SEL.load(SeqCst);
        let dur = read_dur(ID_H_DUR, ID_H_DUR_U).unwrap_or(0);
        let gen = HOLD_GEN.fetch_add(1, SeqCst) + 1;
        HOLD_ACTIVE.store(b, SeqCst);
        HOLD_ON.store(true, SeqCst);
        send_mouse(&[down_flag(b)]);
        if dur > 0 {
            // nach Ablauf automatisch loslassen
            std::thread::spawn(move || {
                let alive = || HOLD_ON.load(SeqCst) && HOLD_GEN.load(SeqCst) == gen;
                if sleep_while(dur, &alive) && HOLD_GEN.load(SeqCst) == gen && HOLD_ON.swap(false, SeqCst) {
                    send_mouse(&[up_flag(b)]);
                }
            });
        }
    }
}

fn toggle_trigger() {
    if TRIG_RUN.swap(false, SeqCst) {
        return;
    }
    let x = get_text(ID_T_X).trim().parse::<i32>().ok();
    let y = get_text(ID_T_Y).trim().parse::<i32>().ok();
    let color = parse_hex(&get_text(ID_T_HEX));
    let interval = read_dur(ID_T_INT, ID_T_INT_U);
    let (Some(x), Some(y), Some(color), Some(interval)) = (x, y, color, interval) else {
        msg_box("Bitte Position (X/Y), Farbe als Hex-Code (z. B. D97757) und Prüfintervall prüfen.");
        return;
    };
    let tol = get_text(ID_T_TOL).trim().parse::<u32>().unwrap_or(0).min(255) as i32;
    let off = get_text(ID_T_OFF).trim().parse::<i64>().unwrap_or(0).clamp(0, 2000);
    let interval = interval.max(10);
    let cooldown = read_dur(ID_T_COOL, ID_T_COOL_U).unwrap_or(0);
    let btn = TRIG_BTN.load(SeqCst);
    let once = TRIG_ONCE.load(SeqCst);
    let restore = TRIG_RESTORE.load(SeqCst);
    let (tr, tg, tb) = (((color >> 16) & 0xFF) as i32, ((color >> 8) & 0xFF) as i32, (color & 0xFF) as i32);

    let gen = TRIG_GEN.fetch_add(1, SeqCst) + 1;
    TRIG_HITS.store(0, SeqCst);
    TRIG_RUN.store(true, SeqCst);
    std::thread::spawn(move || {
        let alive = || TRIG_RUN.load(SeqCst) && TRIG_GEN.load(SeqCst) == gen;
        let mut was_hit = false;
        while alive() {
            let c = read_pixel(x, y);
            let hit = c != u32::MAX && {
                let (r, g, b) = ((c & 0xFF) as i32, ((c >> 8) & 0xFF) as i32, ((c >> 16) & 0xFF) as i32);
                (r - tr).abs() <= tol && (g - tg).abs() <= tol && (b - tb).abs() <= tol
            };
            if hit && (!once || !was_hit) {
                click_at(x + rand_pm(off) as i32, y + rand_pm(off) as i32, btn, restore);
                TRIG_HITS.fetch_add(1, SeqCst);
                if cooldown > 0 {
                    TRIG_COOLING.store(true, SeqCst);
                    let ok = sleep_while(cooldown, &alive);
                    TRIG_COOLING.store(false, SeqCst);
                    if !ok {
                        break;
                    }
                }
            }
            was_hit = hit;
            if !sleep_while(interval, &alive) {
                break;
            }
        }
    });
}

fn toggle_seq() {
    if SEQ_RUN.swap(false, SeqCst) {
        return;
    }
    let steps = SEQ.lock().unwrap().clone();
    if steps.is_empty() {
        msg_box("Die Klickfolge ist leer. Füge zuerst Schritte hinzu.");
        return;
    }
    let rounds: u32 = if SEQ_LOOP.load(SeqCst) {
        get_text(ID_S_ROUNDS).trim().parse().unwrap_or(0)
    } else {
        1
    };
    let gen = SEQ_GEN.fetch_add(1, SeqCst) + 1;
    SEQ_LEN.store(steps.len() as u32, SeqCst);
    SEQ_ROUNDS_TOTAL.store(rounds, SeqCst);
    SEQ_ROUND.store(0, SeqCst);
    SEQ_RUN.store(true, SeqCst);
    std::thread::spawn(move || {
        let alive = || SEQ_RUN.load(SeqCst) && SEQ_GEN.load(SeqCst) == gen;
        let mut round = 0u32;
        'outer: loop {
            round += 1;
            SEQ_ROUND.store(round, SeqCst);
            let mut waited = false;
            for (i, st) in steps.iter().enumerate() {
                if !alive() {
                    break 'outer;
                }
                SEQ_STEP.store(i as i32, SeqCst);
                match *st {
                    Step::Click(b) => click(b),
                    Step::Wait(ms) => {
                        waited |= ms > 0;
                        if !sleep_while(ms, &alive) {
                            break 'outer;
                        }
                    }
                }
            }
            if rounds != 0 && round >= rounds {
                break;
            }
            if !waited && !sleep_while(10, &alive) {
                break;
            }
        }
        if SEQ_GEN.load(SeqCst) == gen {
            SEQ_RUN.store(false, SeqCst);
            SEQ_STEP.store(-1, SeqCst);
        }
    });
}

// ───── Listen ─────

fn list_sel(id: i32) -> i32 {
    unsafe { SendMessageW(item(id), LB_GETCURSEL, 0, 0) as i32 }
}

fn fill_list(id: i32, count: usize, sel: i32) {
    let h = item(id);
    let empty = wide(" ");
    unsafe {
        SendMessageW(h, LB_RESETCONTENT, 0, 0);
        for _ in 0..count {
            SendMessageW(h, LB_ADDSTRING, 0, empty.as_ptr() as LPARAM);
        }
        if sel >= 0 && (sel as usize) < count {
            SendMessageW(h, LB_SETCURSEL, sel as usize, 0);
        }
    }
}

fn add_step(st: Step) {
    let mut v = SEQ.lock().unwrap();
    let sel = list_sel(ID_S_LIST);
    let idx = if sel >= 0 && (sel as usize) < v.len() { sel as usize + 1 } else { v.len() };
    v.insert(idx, st);
    let n = v.len();
    drop(v);
    fill_list(ID_S_LIST, n, idx as i32);
}

fn edit_seq(id: i32) {
    let mut v = SEQ.lock().unwrap();
    let sel = list_sel(ID_S_LIST);
    let valid = sel >= 0 && (sel as usize) < v.len();
    let new_sel = match id {
        ID_S_DEL if valid => {
            v.remove(sel as usize);
            sel.min(v.len() as i32 - 1)
        }
        ID_S_UP if valid && sel > 0 => {
            v.swap(sel as usize, sel as usize - 1);
            sel - 1
        }
        ID_S_DOWN if valid && (sel as usize) + 1 < v.len() => {
            v.swap(sel as usize, sel as usize + 1);
            sel + 1
        }
        ID_S_CLEAR => {
            v.clear();
            -1
        }
        _ => return,
    };
    let n = v.len();
    drop(v);
    fill_list(ID_S_LIST, n, new_sel);
}

// ───── Klickfolgen speichern / laden ─────

const SEQ_FILTER: &str = "Klickfolge (*.mausfolge)\0*.mausfolge\0Textdatei (*.txt)\0*.txt\0Alle Dateien (*.*)\0*.*\0\0";

/// Zeigt den Speichern- oder Oeffnen-Dialog und liefert den gewaehlten Pfad.
fn file_dialog(hwnd: HWND, save: bool) -> Option<std::path::PathBuf> {
    let filter: Vec<u16> = SEQ_FILTER.encode_utf16().collect();
    let mut file = [0u16; 1024];
    if save {
        for (i, c) in "Klickfolge.mausfolge".encode_utf16().enumerate() {
            file[i] = c;
        }
    }
    let title = wide(if save { "Klickfolge speichern" } else { "Klickfolge laden" });
    let ext = wide("mausfolge");
    let mut ofn: OPENFILENAMEW = unsafe { std::mem::zeroed() };
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = hwnd;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.nFilterIndex = 1;
    ofn.lpstrFile = file.as_mut_ptr();
    ofn.nMaxFile = file.len() as u32;
    ofn.lpstrTitle = title.as_ptr();
    ofn.lpstrDefExt = ext.as_ptr();
    ofn.Flags = OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR | if save { OFN_OVERWRITEPROMPT } else { OFN_FILEMUSTEXIST };
    let ok = unsafe { if save { GetSaveFileNameW(&mut ofn) } else { GetOpenFileNameW(&mut ofn) } };
    if ok == 0 {
        return None;
    }
    let n = file.iter().position(|&c| c == 0).unwrap_or(file.len());
    Some(std::path::PathBuf::from(String::from_utf16_lossy(&file[..n])))
}

fn save_seq(hwnd: HWND) {
    let steps = SEQ.lock().unwrap().clone();
    if steps.is_empty() {
        msg_box("Die Klickfolge ist leer – es gibt nichts zu speichern.");
        return;
    }
    let Some(path) = file_dialog(hwnd, true) else { return };
    let mut out = String::from("# Maus-Utility Klickfolge\r\n");
    out += &format!("loop={}\r\n", SEQ_LOOP.load(SeqCst) as u8);
    out += &format!("rounds={}\r\n", get_text(ID_S_ROUNDS).trim().parse::<u32>().unwrap_or(0));
    for st in &steps {
        match *st {
            Step::Click(b) => out += &format!("click {}\r\n", b + 1),
            Step::Wait(ms) => out += &format!("wait {}\r\n", ms),
        }
    }
    if let Err(e) = std::fs::write(&path, out) {
        msg_box(&format!("Speichern fehlgeschlagen:\n{}", e));
    }
}

/// Liest eine gespeicherte Klickfolge. Unbekannte Zeilen werden uebersprungen.
fn parse_seq(text: &str) -> (Vec<Step>, Option<bool>, Option<u32>, usize) {
    let (mut steps, mut looped, mut rounds, mut bad) = (Vec::new(), None, None, 0);
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let lower = line.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("loop=") {
            looped = Some(v.trim() == "1");
        } else if let Some(v) = lower.strip_prefix("rounds=") {
            rounds = v.trim().parse().ok();
        } else if let Some(v) = lower.strip_prefix("click") {
            match v.trim() {
                "1" => steps.push(Step::Click(0)),
                "2" => steps.push(Step::Click(1)),
                _ => bad += 1,
            }
        } else if let Some(v) = lower.strip_prefix("wait") {
            match v.trim().parse::<u64>() {
                Ok(ms) => steps.push(Step::Wait(ms)),
                Err(_) => bad += 1,
            }
        } else {
            bad += 1;
        }
    }
    (steps, looped, rounds, bad)
}

fn load_seq(hwnd: HWND) {
    let Some(path) = file_dialog(hwnd, false) else { return };
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            msg_box(&format!("Laden fehlgeschlagen:\n{}", e));
            return;
        }
    };
    let (steps, looped, rounds, bad) = parse_seq(&text);
    if steps.is_empty() {
        msg_box("In dieser Datei wurde keine gültige Klickfolge gefunden.");
        return;
    }
    SEQ_RUN.store(false, SeqCst);
    let n = steps.len();
    *SEQ.lock().unwrap() = steps;
    fill_list(ID_S_LIST, n, -1);
    if let Some(lp) = looped {
        SEQ_LOOP.store(lp, SeqCst);
    }
    if let Some(r) = rounds {
        set_text(ID_S_ROUNDS, &r.to_string());
    }
    invalidate_buttons();
    if bad > 0 {
        msg_box(&format!("{} unbekannte Zeile(n) wurden übersprungen.", bad));
    }
}

fn grab_cursor_value() {
    if !INFO_ON.load(SeqCst) {
        return;
    }
    let mut p = POINT::default();
    unsafe { GetCursorPos(&mut p) };
    let c = read_pixel(p.x, p.y);
    if c == u32::MAX {
        return;
    }
    let mut m = MERK.lock().unwrap();
    m.insert(0, (p.x, p.y, c));
    m.truncate(200);
    let n = m.len();
    drop(m);
    fill_list(ID_I_LIST, n, 0);
}

fn merk_selected() -> Option<(i32, i32, u32)> {
    let m = MERK.lock().unwrap();
    let sel = list_sel(ID_I_LIST);
    if sel >= 0 { m.get(sel as usize).copied() } else { m.first().copied() }
}

fn merk_to_trigger() {
    if let Some((x, y, c)) = merk_selected() {
        set_text(ID_T_X, &x.to_string());
        set_text(ID_T_Y, &y.to_string());
        set_text(ID_T_HEX, &format!("{:06X}", rgb_of(c)));
    }
}

// ═════════════════════════ Hotkeys ═════════════════════════

fn register_hotkeys() {
    let hwnd = main_hwnd();
    let keys = *HOTKEYS.lock().unwrap();
    let names = ["Autoclicker", "Dauerklick", "Farb-Trigger", "Klickfolge"];
    let mut failed = Vec::new();
    for (i, (vk, mods)) in keys.iter().enumerate() {
        unsafe {
            UnregisterHotKey(hwnd, i as i32 + 1);
            if *vk != 0 && RegisterHotKey(hwnd, i as i32 + 1, mods | MOD_NOREPEAT, *vk) == 0 {
                failed.push(names[i]);
            }
        }
    }
    *HK_FAIL.lock().unwrap() = failed.join(", ");
}

fn unregister_hotkeys() {
    for i in 1..=4 {
        unsafe { UnregisterHotKey(main_hwnd(), i) };
    }
}

fn start_capture(id: i32) {
    if CAPTURE.load(SeqCst) == id {
        end_capture();
        return;
    }
    unregister_hotkeys();
    CAPTURE.store(id, SeqCst);
    unsafe { SetFocus(item(id)) };
    invalidate_buttons();
}

fn end_capture() {
    CAPTURE.store(0, SeqCst);
    register_hotkeys();
    invalidate_buttons();
    unsafe { InvalidateRect(main_hwnd(), null(), 0) };
}

/// Wird aus der Nachrichtenschleife aufgerufen, solange eine Hotkey-Aufnahme laeuft.
fn capture_key(vk: u32) {
    let id = CAPTURE.load(SeqCst);
    let Some(idx) = HK_BUTTONS.iter().position(|&b| b == id) else {
        return;
    };
    match vk {
        0x10 | 0x11 | 0x12 | 0xA0..=0xA5 | 0x5B | 0x5C => return, // nur Modifier
        0x1B => {}                                                 // Esc: abbrechen
        0x08 | 0x2E => HOTKEYS.lock().unwrap()[idx] = (0, 0),      // Entf/Backspace: loeschen
        _ => {
            let mut mods = 0;
            unsafe {
                if GetKeyState(0x11) < 0 {
                    mods |= MOD_CONTROL;
                }
                if GetKeyState(0x10) < 0 {
                    mods |= MOD_SHIFT;
                }
                if GetKeyState(0x12) < 0 {
                    mods |= MOD_ALT;
                }
            }
            if mods == MOD_CONTROL && vk == 0x43 {
                msg_box("Strg+C ist für das Merken der Cursor-Werte reserviert.");
            } else {
                HOTKEYS.lock().unwrap()[idx] = (vk, mods);
            }
        }
    }
    end_capture();
}

unsafe extern "system" fn kb_hook(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code == 0 && (wp as u32 == WM_KEYDOWN || wp as u32 == WM_SYSKEYDOWN) && INFO_ON.load(SeqCst) {
        let k = &*(lp as *const KBDLLHOOKSTRUCT);
        if k.vkCode == 0x43 && GetAsyncKeyState(0x11) < 0 {
            PostMessageW(main_hwnd(), WM_GRAB, 0, 0);
        }
    }
    CallNextHookEx(0, code, wp, lp)
}

// ═════════════════════════ Icon (Maus, selbst gerendert) ═════════════════════════

fn sd_round_box(px: f32, py: f32, bx: f32, by: f32, r: f32) -> f32 {
    let qx = px.abs() - bx + r;
    let qy = py.abs() - by + r;
    let ox = qx.max(0.0);
    let oy = qy.max(0.0);
    (ox * ox + oy * oy).sqrt() + qx.max(qy).min(0.0) - r
}

fn sd_capsule(px: f32, py: f32, half: f32, r: f32) -> f32 {
    let cy = py.clamp(-half, half);
    (px * px + (py - cy) * (py - cy)).sqrt() - r
}

fn mouse_sample(u: f32, v: f32) -> Option<u32> {
    let outline = 0x9E4428;
    let d = sd_round_box(u - 0.5, v - 0.52, 0.31, 0.45, 0.29);
    if d > 0.0 {
        return None;
    }
    if d > -0.06 {
        return Some(outline);
    }
    let split = 0.45;
    let wd = sd_capsule(u - 0.5, v - 0.28, 0.07, 0.052);
    if wd < 0.0 {
        return Some(if wd > -0.028 { outline } else { 0xFFFFFF });
    }
    if ((u - 0.5).abs() < 0.026 && v < split) || (v - split).abs() < 0.026 {
        return Some(outline);
    }
    if v < split && u < 0.5 {
        return Some(0xF4B7A0);
    }
    Some(0xD97757)
}

fn make_mouse_icon(size: i32) -> HANDLE {
    let n = size.max(16) as usize;
    let ss = 4usize;
    let mut px = vec![0u32; n * n];
    for y in 0..n {
        for x in 0..n {
            let (mut r, mut g, mut bl, mut a) = (0f32, 0f32, 0f32, 0f32);
            for sy in 0..ss {
                for sx in 0..ss {
                    let u = (x as f32 + (sx as f32 + 0.5) / ss as f32) / n as f32;
                    let v = (y as f32 + (sy as f32 + 0.5) / ss as f32) / n as f32;
                    if let Some(c) = mouse_sample(u, v) {
                        r += ((c >> 16) & 0xFF) as f32;
                        g += ((c >> 8) & 0xFF) as f32;
                        bl += (c & 0xFF) as f32;
                        a += 1.0;
                    }
                }
            }
            if a > 0.0 {
                let alpha = (a / (ss * ss) as f32 * 255.0).round() as u32;
                px[y * n + x] = (alpha << 24)
                    | (((r / a).round() as u32) << 16)
                    | (((g / a).round() as u32) << 8)
                    | ((bl / a).round() as u32);
            }
        }
    }
    unsafe {
        let bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: n as i32,
                biHeight: -(n as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [0],
        };
        let mut bits: *mut c_void = null_mut();
        let dc = GetDC(0);
        let color = CreateDIBSection(dc, &bi, 0, &mut bits, 0, 0);
        ReleaseDC(0, dc);
        if color == 0 || bits.is_null() {
            return 0;
        }
        std::ptr::copy_nonoverlapping(px.as_ptr(), bits as *mut u32, n * n);
        let mask_bits = vec![0u8; n.div_ceil(16) * 2 * n];
        let mask = CreateBitmap(n as i32, n as i32, 1, 1, mask_bits.as_ptr() as *const c_void);
        let ii = ICONINFO { fIcon: 1, xHotspot: 0, yHotspot: 0, hbmMask: mask, hbmColor: color };
        let icon = CreateIconIndirect(&ii);
        DeleteObject(mask);
        DeleteObject(color);
        icon
    }
}

// ═════════════════════════ Zeichnen ═════════════════════════

/// Kleine GDI+-Huelle fuer weiche (antialiased) Formen.
struct Gfx(isize);

impl Gfx {
    fn new(dc: HANDLE) -> Gfx {
        let mut g = 0;
        unsafe {
            GdipCreateFromHDC(dc, &mut g);
            GdipSetSmoothingMode(g, 4);
            GdipSetPixelOffsetMode(g, 4);
        }
        Gfx(g)
    }

    fn path(x: f32, y: f32, w: f32, h: f32, r: f32) -> isize {
        let mut p = 0;
        let r = r.min(w / 2.0).min(h / 2.0).max(0.5);
        let d = r * 2.0;
        unsafe {
            GdipCreatePath(0, &mut p);
            GdipAddPathArc(p, x, y, d, d, 180.0, 90.0);
            GdipAddPathArc(p, x + w - d, y, d, d, 270.0, 90.0);
            GdipAddPathArc(p, x + w - d, y + h - d, d, d, 0.0, 90.0);
            GdipAddPathArc(p, x, y + h - d, d, d, 90.0, 90.0);
            GdipClosePathFigure(p);
        }
        p
    }

    fn rrect(&self, x: f32, y: f32, w: f32, h: f32, r: f32, fill: Option<u32>, border: Option<u32>) {
        unsafe {
            if let Some(f) = fill {
                let p = Self::path(x, y, w, h, r);
                let mut br = 0;
                GdipCreateSolidFill(argb(f), &mut br);
                GdipFillPath(self.0, br, p);
                GdipDeleteBrush(br);
                GdipDeletePath(p);
            }
            if let Some(bc) = border {
                let p = Self::path(x + 0.5, y + 0.5, w - 1.0, h - 1.0, r - 0.5);
                let mut pen = 0;
                GdipCreatePen1(argb(bc), 1.0, 2, &mut pen);
                GdipDrawPath(self.0, pen, p);
                GdipDeletePen(pen);
                GdipDeletePath(p);
            }
        }
    }

    fn ellipse(&self, x: f32, y: f32, w: f32, h: f32, fill: u32) {
        unsafe {
            let mut br = 0;
            GdipCreateSolidFill(argb(fill), &mut br);
            GdipFillEllipse(self.0, br, x, y, w, h);
            GdipDeleteBrush(br);
        }
    }

    fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: u32) {
        unsafe {
            let mut pen = 0;
            GdipCreatePen1(argb(color), width, 2, &mut pen);
            GdipSetPenStartCap(pen, 2);
            GdipSetPenEndCap(pen, 2);
            GdipDrawLine(self.0, pen, x1, y1, x2, y2);
            GdipDeletePen(pen);
        }
    }

    fn chevron_down(&self, cx: f32, cy: f32, color: u32) {
        let k = sf(4);
        self.line(cx - k, cy - k / 2.0, cx, cy + k / 2.0, sf(1) * 1.6, color);
        self.line(cx, cy + k / 2.0, cx + k, cy - k / 2.0, sf(1) * 1.6, color);
    }

    fn chevron_vert(&self, cx: f32, cy: f32, down: bool, color: u32) {
        let k = sf(4);
        let d = if down { k / 2.0 } else { -k / 2.0 };
        self.line(cx - k, cy - d, cx, cy + d, sf(1) * 1.6, color);
        self.line(cx, cy + d, cx + k, cy - d, sf(1) * 1.6, color);
    }

    fn chevron_side(&self, cx: f32, cy: f32, right: bool, color: u32) {
        let k = sf(4);
        let d = if right { k / 2.0 } else { -k / 2.0 };
        self.line(cx - d, cy - k, cx + d, cy, sf(1) * 1.6, color);
        self.line(cx + d, cy, cx - d, cy + k, sf(1) * 1.6, color);
    }
}

impl Drop for Gfx {
    fn drop(&mut self) {
        unsafe { GdipDeleteGraphics(self.0) };
    }
}

/// Doppelpuffer: zeichnet in eine Bitmap und kopiert sie am Ende auf einmal.
struct Buffer {
    dc: HANDLE,
    bmp: HANDLE,
    old: HANDLE,
    w: i32,
    h: i32,
}

impl Buffer {
    fn new(target: HANDLE, w: i32, h: i32) -> Buffer {
        unsafe {
            let dc = CreateCompatibleDC(target);
            let bmp = CreateCompatibleBitmap(target, w.max(1), h.max(1));
            let old = SelectObject(dc, bmp);
            SetBkMode(dc, 1);
            Buffer { dc, bmp, old, w, h }
        }
    }

    fn fill(&self, rgb: u32) {
        unsafe {
            let br = CreateSolidBrush(cref(rgb));
            let rc = RECT { left: 0, top: 0, right: self.w, bottom: self.h };
            FillRect(self.dc, &rc, br);
            DeleteObject(br);
        }
    }

    fn blit(&self, target: HANDLE, x: i32, y: i32) {
        unsafe { BitBlt(target, x, y, self.w, self.h, self.dc, 0, 0, SRCCOPY) };
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            DeleteObject(self.bmp);
            DeleteDC(self.dc);
        }
    }
}

fn text(dc: HANDLE, t: &str, font: HANDLE, color: u32, x: i32, y: i32, w: i32, h: i32, flags: u32) {
    // Leere Texte nie an Windows uebergeben (ein leerer Vec hat keinen gueltigen Zeiger)
    if t.is_empty() {
        return;
    }
    let w16 = wide(t);
    let mut rc = RECT { left: x, top: y, right: x + w, bottom: y + h };
    unsafe {
        SelectObject(dc, font);
        SetTextColor(dc, cref(color));
        DrawTextW(dc, w16.as_ptr(), w16.len() as i32 - 1, &mut rc, flags | DT_NOPREFIX);
    }
}

fn text_width(dc: HANDLE, t: &str, font: HANDLE) -> i32 {
    if t.is_empty() {
        return 0;
    }
    let w16 = wide(t);
    let mut rc = RECT::default();
    unsafe {
        SelectObject(dc, font);
        DrawTextW(dc, w16.as_ptr(), w16.len() as i32 - 1, &mut rc, DT_CALCRECT | DT_SINGLELINE | DT_NOPREFIX);
    }
    rc.right - rc.left
}

const VC: u32 = DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS;

fn status_of(st: St) -> Option<(String, bool)> {
    Some(match st {
        St::None => return None,
        St::Auto => {
            if AUTO_RUN.load(SeqCst) {
                (format!("Läuft · {} Klicks", AUTO_COUNT.load(SeqCst)), true)
            } else {
                ("Gestoppt".into(), false)
            }
        }
        St::Hold => {
            if HOLD_ON.load(SeqCst) {
                (format!("Hält Mouse {}", HOLD_ACTIVE.load(SeqCst) + 1), true)
            } else {
                ("Aus".into(), false)
            }
        }
        St::Trig => {
            if TRIG_RUN.load(SeqCst) {
                let what = if TRIG_COOLING.load(SeqCst) { "Cooldown" } else { "Überwacht" };
                (format!("{} · {} Treffer", what, TRIG_HITS.load(SeqCst)), true)
            } else {
                ("Gestoppt".into(), false)
            }
        }
        St::Seq => {
            if SEQ_RUN.load(SeqCst) {
                let total = SEQ_ROUNDS_TOTAL.load(SeqCst);
                let t = if total == 0 { "∞".to_string() } else { total.to_string() };
                (
                    format!(
                        "Schritt {}/{} · Runde {}/{}",
                        SEQ_STEP.load(SeqCst) + 1,
                        SEQ_LEN.load(SeqCst),
                        SEQ_ROUND.load(SeqCst),
                        t
                    ),
                    true,
                )
            } else {
                ("Gestoppt".into(), false)
            }
        }
    })
}

fn info_lines() -> [String; 3] {
    if !INFO_ON.load(SeqCst) {
        return ["Anzeige aus".into(), "Schalter oben rechts".into(), "aktivieren".into()];
    }
    let c = INFO_COLOR.load(SeqCst);
    let (x, y) = (INFO_X.load(SeqCst), INFO_Y.load(SeqCst));
    if c == u32::MAX {
        return [format!("X {}   Y {}", x, y), "Farbe nicht lesbar".into(), String::new()];
    }
    let rgb = rgb_of(c);
    [
        format!("X {}   Y {}", x, y),
        format!("RGB {}, {}, {}", rgb >> 16, (rgb >> 8) & 0xFF, rgb & 0xFF),
        format!("Hex #{:06X}", rgb),
    ]
}

fn footer_text() -> String {
    let fail = HK_FAIL.lock().unwrap().clone();
    if fail.is_empty() {
        String::new()
    } else {
        format!("Achtung: Hotkey belegt oder doppelt vergeben – {}", fail)
    }
}

const HELP_LEFT: [&str; 3] = [
    "Minimieren schickt das Programm in den Infobereich (Tray).",
    "Strg+C merkt den Cursor-Wert, solange Cursor-Info aktiv ist.",
    "Klickfolgen werden als .mausfolge-Datei gespeichert.",
];
const HELP_RIGHT: [&str; 3] = [
    "Hotkey ändern: Feld anklicken, Taste drücken. Esc = abbrechen, Entf = keiner.",
    "Loslassen nach 0 = erst per Hotkey. Cooldown 0 = keine Pause.",
    "Simulierte Klicks lösen die Buttons dieses Programms nicht aus.",
];

fn paint_main(hwnd: HWND) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let dc = BeginPaint(hwnd, &mut ps);
        let mut rc = RECT::default();
        GetClientRect(hwnd, &mut rc);
        let buf = Buffer::new(dc, rc.right, rc.bottom);
        let m = buf.dc;
        let p = pal();
        let f = fonts();
        let open = SEQ_OPEN.load(SeqCst);
        buf.fill(p.bg);

        // Vorab: Status-Texte messen (fuer die Punkt-Position)
        let mut statuses = Vec::new();
        for c in CARDS.iter().filter(|c| open || !c.seq) {
            if let Some((t, on)) = status_of(c.status) {
                let tw = text_width(m, &t, f.small);
                statuses.push((c, t, on, tw));
            }
        }

        // ── Formen ──
        {
            let g = Gfx::new(m);
            for c in CARDS.iter().filter(|c| open || !c.seq) {
                g.rrect(sf(c.x), sf(c.y), sf(c.w), sf(c.h), sf(14), Some(p.card), Some(p.border));
            }
            for (c, _, on, tw) in &statuses {
                let right = s(c.x + c.w - 16);
                let cy = sf(c.y + 25);
                let d = sf(8);
                let x = (right - tw - s(8)) as f32 - d;
                g.ellipse(x, cy - d / 2.0, d, d, if *on { p.accent } else { p.input_border });
            }
            let focus = FOCUS_ED.load(SeqCst);
            for ed in EDS.iter().filter(|e| open || !e.seq) {
                let border = if ed.id == focus { p.accent } else { p.input_border };
                g.rrect(sf(ed.x), sf(ed.y), sf(ed.w), sf(30), sf(8), Some(p.input), Some(border));
            }
            for li in LISTS.iter().filter(|l| open || !l.seq) {
                g.rrect(sf(li.x), sf(li.y), sf(li.w), sf(li.h), sf(10), Some(p.input), Some(p.input_border));
            }
            // Cursor-Farbfeld
            let ic = INFO_COLOR.load(SeqCst);
            let sw = if INFO_ON.load(SeqCst) && ic != u32::MAX { rgb_of(ic) } else { p.input };
            g.rrect(sf(32), sf(516), sf(44), sf(44), sf(10), Some(sw), Some(p.input_border));
            // Zielfarbe des Triggers
            let tc = parse_hex(&get_text(ID_T_HEX));
            g.rrect(sf(652), sf(146), sf(30), sf(30), sf(8), Some(tc.unwrap_or(p.input)), Some(p.input_border));
            if tc.is_none() {
                g.line(sf(658), sf(170), sf(676), sf(152), sf(1) * 1.5, p.muted);
            }
            if open {
                g.line(sf(816), sf(538), sf(1124), sf(538), 1.0, p.border);
            }
            if INFO_OPEN.load(SeqCst) {
                let cw = rc.right as f32;
                g.rrect(sf(16), sf(680), cw - sf(32), sf(96), sf(14), Some(p.card), Some(p.border));
                let col_w = (rc.right - s(64)) / 2;
                for col in 0..2 {
                    for i in 0..3 {
                        let x = s(30) + col * (col_w + s(8));
                        g.ellipse(x as f32, sf(702 + i * 22), sf(5), sf(5), p.accent);
                    }
                }
            }
        }

        // ── Text ──
        if let Some(ic) = ICONS.get() {
            DrawIconEx(m, s(18), s(14), ic.header, s(32), s(32), 0, 0, DI_NORMAL);
        }
        text(m, "Maus-Utility", f.title, p.text, s(60), s(10), s(300), s(40), VC);

        for c in CARDS.iter().filter(|c| open || !c.seq) {
            text(m, c.title, f.semi, p.text, s(c.x + 16), s(c.y + 12), s(c.w - 32), s(26), VC);
        }
        // Hinweis neben "Cursor-Info"
        let tw = text_width(m, "Cursor-Info", f.semi);
        text(m, "Strg+C speichert den aktuellen Wert", f.small, p.muted, s(32) + tw + s(12), s(480), s(300), s(26), VC);

        for (c, t, on, tw) in &statuses {
            let right = s(c.x + c.w - 16);
            text(m, t, f.small, if *on { p.accent } else { p.muted }, right - tw, s(c.y + 12), *tw, s(26), VC);
        }

        for lb in LABELS.iter().filter(|l| open || !l.seq) {
            let (font, color, flags) = match lb.style {
                0 => (f.normal, p.text, VC),
                1 => (f.small, p.muted, VC),
                _ => (f.small, p.muted, VC | DT_RIGHT),
            };
            text(m, lb.text, font, color, s(lb.x), s(lb.y), s(lb.w), s(30), flags);
        }

        for ed in EDS.iter().filter(|e| (open || !e.seq) && !e.prefix.is_empty()) {
            text(m, ed.prefix, f.normal, p.muted, s(ed.x + 10), s(ed.y), s(ed.pw + 4), s(30), VC);
        }

        let lines = info_lines();
        let on = INFO_ON.load(SeqCst);
        for (i, ln) in lines.iter().enumerate() {
            let (font, color) = if on { (f.mono, p.text) } else { (f.small, p.muted) };
            text(m, ln, font, color, s(88), s(514 + i as i32 * 21), s(220), s(21), VC);
        }

        let ft = footer_text();
        text(m, &ft, f.small, p.accent, s(196), s(640), rc.right - s(212), s(28), VC);
        if INFO_OPEN.load(SeqCst) {
            let col_w = (rc.right - s(64)) / 2;
            for (col, lines) in [HELP_LEFT, HELP_RIGHT].iter().enumerate() {
                for (i, ln) in lines.iter().enumerate() {
                    let x = s(40) + col as i32 * (col_w + s(8));
                    text(m, ln, f.small, p.muted, x, s(694 + i as i32 * 22), col_w - s(24), s(22), VC);
                }
            }
        }

        buf.blit(dc, 0, 0);
        drop(buf);
        EndPaint(hwnd, &ps);
    }
}

fn btn_def(id: i32) -> Option<&'static Btn> {
    BTNS.iter().find(|b| b.id == id)
}

fn btn_label(id: i32) -> String {
    let running = |b: bool, on: &str, off: &str| if b { on.to_string() } else { off.to_string() };
    match id {
        ID_A_M1 | ID_H_M1 | ID_T_M1 => "Mouse 1".into(),
        ID_A_M2 | ID_H_M2 | ID_T_M2 => "Mouse 2".into(),
        ID_A_INT_U | ID_A_JIT_U | ID_T_INT_U | ID_S_WAIT_U | ID_T_COOL_U | ID_H_DUR_U => {
            let u = unit_of(id).load(SeqCst) as usize;
            if id == ID_S_WAIT_U { ["ms", "s", "min", "h"][u].into() } else { UNIT_LONG[u].into() }
        }
        ID_A_HK | ID_H_HK | ID_T_HK | ID_S_HK => {
            if CAPTURE.load(SeqCst) == id {
                "Taste drücken…".into()
            } else {
                let idx = HK_BUTTONS.iter().position(|&b| b == id).unwrap_or(0);
                let (vk, mods) = HOTKEYS.lock().unwrap()[idx];
                key_name(vk, mods)
            }
        }
        ID_A_GO => running(AUTO_RUN.load(SeqCst), "Stoppen", "Starten"),
        ID_H_GO => running(HOLD_ON.load(SeqCst), "Loslassen", "Halten"),
        ID_T_GO => running(TRIG_RUN.load(SeqCst), "Stoppen", "Starten"),
        ID_S_GO => running(SEQ_RUN.load(SeqCst), "Stoppen", "Starten"),
        ID_T_ONCE => "Nur 1× je Treffer".into(),
        ID_T_RESTORE => "Cursor zurück".into(),
        ID_I_SW => "Anzeigen".into(),
        ID_S_LOOP => "Loopen".into(),
        ID_I_COPY => "Kopieren".into(),
        ID_I_TOTRIG => "→ Farb-Trigger".into(),
        ID_I_CLEAR => "Leeren".into(),
        ID_S_ADD1 => "+ Mouse 1".into(),
        ID_S_ADD2 => "+ Mouse 2".into(),
        ID_S_ADDW => "+ Warten".into(),
        ID_S_UP => "↑".into(),
        ID_S_DOWN => "↓".into(),
        ID_S_DEL => "Entfernen".into(),
        ID_S_CLEAR => "Leeren".into(),
        ID_S_SAVE => "Speichern …".into(),
        ID_S_LOAD => "Laden …".into(),
        ID_INFO_TOGGLE => running(INFO_OPEN.load(SeqCst), "Hinweise ausblenden", "Hinweise einblenden"),
        ID_SEQ_TOGGLE => running(SEQ_OPEN.load(SeqCst), "Klickfolge ausblenden", "Klickfolge einblenden"),
        _ => String::new(),
    }
}

fn btn_active(id: i32) -> bool {
    match id {
        ID_A_M1 => AUTO_BTN.load(SeqCst) == 0,
        ID_A_M2 => AUTO_BTN.load(SeqCst) == 1,
        ID_H_M1 => HOLD_SEL.load(SeqCst) == 0,
        ID_H_M2 => HOLD_SEL.load(SeqCst) == 1,
        ID_T_M1 => TRIG_BTN.load(SeqCst) == 0,
        ID_T_M2 => TRIG_BTN.load(SeqCst) == 1,
        ID_A_GO => AUTO_RUN.load(SeqCst),
        ID_H_GO => HOLD_ON.load(SeqCst),
        ID_T_GO => TRIG_RUN.load(SeqCst),
        ID_S_GO => SEQ_RUN.load(SeqCst),
        ID_T_ONCE => TRIG_ONCE.load(SeqCst),
        ID_T_RESTORE => TRIG_RESTORE.load(SeqCst),
        ID_I_SW => INFO_ON.load(SeqCst),
        ID_S_LOOP => SEQ_LOOP.load(SeqCst),
        ID_SEQ_TOGGLE => SEQ_OPEN.load(SeqCst),
        ID_INFO_TOGGLE => INFO_OPEN.load(SeqCst),
        _ => false,
    }
}

fn draw_button(dis: &DRAWITEMSTRUCT) {
    let id = dis.CtlID as i32;
    let Some(def) = btn_def(id) else { return };
    let rc = dis.rcItem;
    let (w, h) = (rc.right - rc.left, rc.bottom - rc.top);
    let p = pal();
    let f = fonts();
    let header = matches!(def.kind, Kind::Icon | Kind::Ghost);
    let base = if header { p.bg } else { p.card };
    let hover = HOVER.load(SeqCst) == dis.hwndItem;
    let pressed = dis.itemState & ODS_SELECTED != 0;
    let focus = dis.itemState & ODS_FOCUS != 0 && dis.itemState & ODS_NOFOCUSRECT == 0;
    let active = btn_active(id);
    let label = btn_label(id);
    let (wf, hf) = (w as f32, h as f32);
    let r = sf(8);

    let buf = Buffer::new(dis.hDC, w, h);
    let m = buf.dc;
    buf.fill(base);

    let mut text_color = p.text;
    let mut font = f.normal;
    let mut align = DT_CENTER;
    let mut text_x = 0;

    {
        let g = Gfx::new(m);
        match def.kind {
            Kind::Primary => {
                font = f.btn;
                if active {
                    g.rrect(0.0, 0.0, wf, hf, r, Some(p.accent_soft), Some(p.accent));
                    text_color = p.accent;
                } else {
                    let c = if hover || pressed { p.accent_hover } else { p.accent };
                    g.rrect(0.0, 0.0, wf, hf, r, Some(c), None);
                    text_color = p.accent_text;
                }
            }
            Kind::Secondary => {
                let c = if hover || pressed { p.sec_hover } else { p.sec };
                g.rrect(0.0, 0.0, wf, hf, r, Some(c), None);
            }
            Kind::Segment => {
                font = f.btn;
                if active {
                    g.rrect(0.0, 0.0, wf, hf, r, Some(p.accent), None);
                    text_color = p.accent_text;
                } else {
                    let c = if hover || pressed { p.sec_hover } else { p.sec };
                    g.rrect(0.0, 0.0, wf, hf, r, Some(c), Some(p.border));
                    text_color = p.muted;
                }
            }
            Kind::Dropdown => {
                let c = if hover || pressed { p.sec_hover } else { p.sec };
                g.rrect(0.0, 0.0, wf, hf, r, Some(c), None);
                g.chevron_down(wf - sf(16), hf / 2.0, p.muted);
                align = 0;
                text_x = s(12);
            }
            Kind::Hotkey => {
                let capturing = CAPTURE.load(SeqCst) == id;
                let border = if capturing { p.accent } else if hover { p.muted } else { p.input_border };
                g.rrect(0.0, 0.0, wf, hf, r, Some(p.input), Some(border));
                if capturing {
                    text_color = p.accent;
                }
                font = f.btn;
            }
            Kind::Switch => {
                let (pw, ph) = (sf(36), sf(20));
                let y = (hf - ph) / 2.0;
                let track = if active { p.accent } else if hover { p.muted } else { p.input_border };
                g.rrect(0.0, y, pw, ph, ph / 2.0, Some(track), None);
                let k = ph - sf(4);
                let kx = if active { pw - k - sf(2) } else { sf(2) };
                g.ellipse(kx, y + sf(2), k, k, 0xFFFFFF);
                align = 0;
                text_x = s(46);
            }
            Kind::Icon => {
                let bgc = if hover || pressed { p.sec } else { p.bg };
                if hover || pressed {
                    g.ellipse(0.0, 0.0, wf, hf, bgc);
                }
                let (cx, cy) = (wf / 2.0, hf / 2.0);
                if DARK_MODE.load(SeqCst) {
                    // Sonne -> zu Hell wechseln
                    let rr = sf(4) + 0.5;
                    g.ellipse(cx - rr, cy - rr, rr * 2.0, rr * 2.0, p.text);
                    for i in 0..8 {
                        let a = i as f32 * std::f32::consts::FRAC_PI_4;
                        let (c, sn) = (a.cos(), a.sin());
                        g.line(cx + c * sf(7), cy + sn * sf(7), cx + c * sf(9), cy + sn * sf(9), sf(1) * 1.6, p.text);
                    }
                } else {
                    // Mond -> zu Dunkel wechseln
                    let rr = sf(7);
                    g.ellipse(cx - rr, cy - rr, rr * 2.0, rr * 2.0, p.text);
                    let r2 = sf(6);
                    g.ellipse(cx - r2 + sf(4), cy - r2 - sf(3), r2 * 2.0, r2 * 2.0, bgc);
                }
            }
            Kind::Ghost => {
                if hover || pressed {
                    g.rrect(0.0, 0.0, wf, hf, r, Some(p.sec), None);
                }
                if id == ID_INFO_TOGGLE {
                    g.chevron_vert(wf - sf(16), hf / 2.0, !active, p.muted);
                } else {
                    g.chevron_side(wf - sf(16), hf / 2.0, !active, p.muted);
                }
                align = 0;
                text_x = s(12);
                text_color = p.muted;
            }
        }
        if focus && def.kind != Kind::Switch {
            g.rrect(0.0, 0.0, wf, hf, r, None, Some(p.accent));
        }
    }

    if !label.is_empty() {
        let tw = if def.kind == Kind::Dropdown || def.kind == Kind::Ghost { w - text_x - s(28) } else { w - text_x };
        text(m, &label, font, text_color, text_x, 0, tw, h, VC | align);
    }
    buf.blit(dis.hDC, rc.left, rc.top);
}

fn draw_list_item(dis: &DRAWITEMSTRUCT) {
    let rc = dis.rcItem;
    let (w, h) = (rc.right - rc.left, rc.bottom - rc.top);
    if w <= 0 || h <= 0 {
        return;
    }
    let p = pal();
    let f = fonts();
    let buf = Buffer::new(dis.hDC, w, h);
    let m = buf.dc;
    buf.fill(p.input);
    let idx = dis.itemID as i32;
    let selected = dis.itemState & ODS_SELECTED != 0;
    let (wf, hf) = (w as f32, h as f32);

    if dis.CtlID as i32 == ID_S_LIST {
        let step = SEQ.lock().unwrap().get(idx.max(0) as usize).copied();
        if let Some(st) = step.filter(|_| idx >= 0) {
            let running = SEQ_RUN.load(SeqCst) && SEQ_STEP.load(SeqCst) == idx;
            {
                let g = Gfx::new(m);
                if running {
                    g.rrect(sf(2), sf(2), wf - sf(4), hf - sf(4), sf(6), Some(p.accent_soft), None);
                    g.rrect(sf(4), sf(7), sf(3), hf - sf(14), sf(2), Some(p.accent), None);
                } else if selected {
                    g.rrect(sf(2), sf(2), wf - sf(4), hf - sf(4), sf(6), Some(p.sel), None);
                }
                if let Step::Click(_) = st {
                    g.ellipse(sf(38), hf / 2.0 - sf(3), sf(6), sf(6), p.accent);
                } else {
                    g.ellipse(sf(38), hf / 2.0 - sf(3), sf(6), sf(6), p.input_border);
                }
            }
            text(m, &(idx + 1).to_string(), f.small, p.muted, s(10), 0, s(22), h, VC | DT_RIGHT);
            let (t, c) = match st {
                Step::Click(b) => (format!("Klick  Mouse {}", b + 1), p.text),
                Step::Wait(ms) => (format!("Warten  {}", fmt_dur(ms)), p.muted),
            };
            let font = if running { f.btn } else { f.normal };
            text(m, &t, font, if running { p.accent } else { c }, s(52), 0, w - s(60), h, VC);
        }
    } else {
        let entry = MERK.lock().unwrap().get(idx.max(0) as usize).copied();
        if let Some((x, y, c)) = entry.filter(|_| idx >= 0) {
            {
                let g = Gfx::new(m);
                if selected {
                    g.rrect(sf(2), sf(2), wf - sf(4), hf - sf(4), sf(6), Some(p.sel), None);
                }
                g.rrect(sf(10), hf / 2.0 - sf(8), sf(16), sf(16), sf(4), Some(rgb_of(c)), Some(p.input_border));
            }
            text(m, &format!("X {}  Y {}", x, y), f.normal, p.text, s(34), 0, s(130), h, VC);
            text(m, &format!("Hex #{:06X}", rgb_of(c)), f.mono, p.muted, s(150), 0, w - s(160), h, VC | DT_RIGHT);
        }
    }
    buf.blit(dis.hDC, rc.left, rc.top);
}

fn invalidate_buttons() {
    for b in BTNS {
        unsafe { InvalidateRect(item(b.id), null(), 0) };
    }
}

// ═════════════════════════ Theme & Layout ═════════════════════════

fn windows_build() -> u32 {
    unsafe {
        let mut v: OSVERSIONINFOW = std::mem::zeroed();
        v.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;
        RtlGetVersion(&mut v);
        v.dwBuildNumber
    }
}

fn system_prefers_dark() -> bool {
    let sub = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let val = wide("AppsUseLightTheme");
    let mut data: u32 = 1;
    let mut size: u32 = 4;
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER, sub.as_ptr(), val.as_ptr(), RRF_RT_REG_DWORD,
            null_mut(), &mut data as *mut u32 as *mut c_void, &mut size,
        )
    };
    r == 0 && data == 0
}

/// Dunkle Kontextmenues ueber die (inoffizielle) uxtheme-Funktion, ab Windows 10 1903.
fn set_menu_theme(dark: bool) {
    static FNS: OnceLock<(usize, usize)> = OnceLock::new();
    let (set_mode, flush) = *FNS.get_or_init(|| unsafe {
        if windows_build() < 18362 {
            return (0, 0);
        }
        let name = wide("uxtheme.dll");
        let h = LoadLibraryW(name.as_ptr());
        if h == 0 {
            return (0, 0);
        }
        (GetProcAddress(h, 135usize as *const u8), GetProcAddress(h, 136usize as *const u8))
    });
    unsafe {
        if set_mode != 0 {
            let f: extern "system" fn(i32) -> i32 = std::mem::transmute(set_mode);
            f(if dark { 2 } else { 3 });
        }
        if flush != 0 {
            let f: extern "system" fn() = std::mem::transmute(flush);
            f();
        }
    }
}

fn apply_theme(hwnd: HWND) {
    let dark = DARK_MODE.load(SeqCst);
    let p = pal();
    unsafe {
        let old = INPUT_BRUSH.swap(CreateSolidBrush(cref(p.input)), SeqCst);
        if old != 0 {
            DeleteObject(old);
        }
        let v: i32 = dark as i32;
        let vp = &v as *const i32 as *const c_void;
        if DwmSetWindowAttribute(hwnd, 20, vp, 4) != 0 {
            DwmSetWindowAttribute(hwnd, 19, vp, 4);
        }
        // Windows 11: Titelleiste in Fensterfarbe
        let cap = cref(p.bg);
        let txt = cref(p.text);
        let brd = cref(p.border);
        DwmSetWindowAttribute(hwnd, 35, &cap as *const u32 as *const c_void, 4);
        DwmSetWindowAttribute(hwnd, 36, &txt as *const u32 as *const c_void, 4);
        DwmSetWindowAttribute(hwnd, 34, &brd as *const u32 as *const c_void, 4);
        set_menu_theme(dark);
        let theme = wide(if dark { "DarkMode_Explorer" } else { "Explorer" });
        for li in LISTS {
            SetWindowTheme(item(li.id), theme.as_ptr(), null());
        }
        RedrawWindow(hwnd, null(), 0, RDW_INVALIDATE | RDW_ERASE | RDW_ALLCHILDREN | RDW_FRAME);
    }
}

fn window_style() -> u32 {
    WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_CLIPCHILDREN
}

fn outer_size(cw: i32, ch: i32) -> (i32, i32) {
    let mut rc = RECT { left: 0, top: 0, right: s(cw), bottom: s(ch) };
    unsafe { AdjustWindowRectEx(&mut rc, window_style(), 0, 0) };
    (rc.right - rc.left, rc.bottom - rc.top)
}

fn apply_layout(hwnd: HWND) {
    let open = SEQ_OPEN.load(SeqCst);
    let cw = if open { W_EXPANDED } else { W_COLLAPSED };
    unsafe {
        let show = if open { SW_SHOW } else { SW_HIDE };
        for bt in BTNS.iter().filter(|b| b.seq) {
            ShowWindow(item(bt.id), show);
        }
        for ed in EDS.iter().filter(|e| e.seq) {
            ShowWindow(item(ed.id), show);
        }
        for li in LISTS.iter().filter(|l| l.seq) {
            ShowWindow(item(li.id), show);
        }
        MoveWindow(item(ID_THEME), s(cw - 16 - 36), s(12), s(36), s(36), 1);
        MoveWindow(item(ID_SEQ_TOGGLE), s(cw - 16 - 36 - 8 - 190), s(15), s(190), s(30), 1);
        let ch = if INFO_OPEN.load(SeqCst) { H_EXPANDED } else { H_COLLAPSED };
        let (w, h) = outer_size(cw, ch);
        SetWindowPos(hwnd, 0, 0, 0, w, h, SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);
        InvalidateRect(hwnd, null(), 0);
    }
    invalidate_buttons();
}

// ═════════════════════════ Oberflaeche aufbauen ═════════════════════════

unsafe extern "system" fn btn_proc(h: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        // Eigene simulierte Klicks loesen keine eigenen Buttons aus
        WM_LBUTTONDOWN | WM_LBUTTONUP | WM_LBUTTONDBLCLK if GetMessageExtraInfo() as usize == MAGIC => return 0,
        WM_MOUSEMOVE => {
            let old = HOVER.swap(h, SeqCst);
            if old != h {
                if old != 0 {
                    InvalidateRect(old, null(), 0);
                }
                InvalidateRect(h, null(), 0);
                let mut t = TRACKMOUSEEVENT {
                    cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: h,
                    dwHoverTime: 0,
                };
                TrackMouseEvent(&mut t);
            }
        }
        WM_MOUSELEAVE => {
            if HOVER.compare_exchange(h, 0, SeqCst, SeqCst).is_ok() {
                InvalidateRect(h, null(), 0);
            }
        }
        WM_ERASEBKGND => return 1,
        _ => {}
    }
    CallWindowProcW(ORIG_BTN_PROC.load(SeqCst), h, msg, wp, lp)
}

fn create_child(parent: HWND, class: &str, txt: &str, style: u32, x: i32, y: i32, w: i32, h: i32, id: i32) -> HWND {
    let c = wide(class);
    let t = wide(txt);
    unsafe {
        let hwnd = CreateWindowExW(
            0, c.as_ptr(), t.as_ptr(), WS_CHILD | WS_VISIBLE | style,
            x, y, w, h, parent, id as isize, GetModuleHandleW(null()), null_mut(),
        );
        SendMessageW(hwnd, WM_SETFONT, fonts().normal as usize, 1);
        hwnd
    }
}

fn build_ui(p: HWND) {
    for bt in BTNS {
        let h = create_child(p, "BUTTON", "", BS_OWNERDRAW | WS_TABSTOP, s(bt.x), s(bt.y), s(bt.w), s(bt.h), bt.id);
        unsafe {
            let old = SetWindowLongPtrW(h, GWLP_WNDPROC, btn_proc as WNDPROC as usize as isize);
            ORIG_BTN_PROC.store(old, SeqCst);
        }
    }
    let eh = 18;
    for ed in EDS {
        let x = ed.x + 10 + if ed.pw > 0 { ed.pw + 4 } else { 0 };
        let w = ed.w - (x - ed.x) - 10;
        create_child(p, "EDIT", ed.def, ES_AUTOHSCROLL | WS_TABSTOP, s(x), s(ed.y) + (s(30) - s(eh)) / 2, s(w), s(eh), ed.id);
    }
    for li in LISTS {
        create_child(
            p, "LISTBOX", "",
            LBS_OWNERDRAWFIXED | LBS_HASSTRINGS | LBS_NOTIFY | LBS_NOINTEGRALHEIGHT | WS_VSCROLL | WS_TABSTOP,
            s(li.x + 4), s(li.y + 4), s(li.w - 8), s(li.h - 8), li.id,
        );
    }
}

// ═════════════════════════ Tray ═════════════════════════

fn tray_data(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut d: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    d.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    d.hWnd = hwnd;
    d.uID = 1;
    d
}

fn set_tip(d: &mut NOTIFYICONDATAW, tip: &str) {
    for (i, c) in tip.encode_utf16().take(127).enumerate() {
        d.szTip[i] = c;
    }
}

fn tray_add(hwnd: HWND) {
    let mut d = tray_data(hwnd);
    d.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    d.uCallbackMessage = WM_TRAY;
    d.hIcon = ICONS.get().map(|i| i.small).unwrap_or(0);
    set_tip(&mut d, "Maus-Utility");
    unsafe { Shell_NotifyIconW(NIM_ADD, &d) };
}

fn tray_update_tip(hwnd: HWND) {
    let mut tip = String::from("Maus-Utility");
    if AUTO_RUN.load(SeqCst) {
        tip.push_str("\nAutoclicker läuft");
    }
    if HOLD_ON.load(SeqCst) {
        tip.push_str("\nDauerklick aktiv");
    }
    if TRIG_RUN.load(SeqCst) {
        tip.push_str("\nFarb-Trigger aktiv");
    }
    if SEQ_RUN.load(SeqCst) {
        tip.push_str("\nKlickfolge läuft");
    }
    let mut last = TRAY_TIP.lock().unwrap();
    if *last != tip {
        let mut d = tray_data(hwnd);
        d.uFlags = NIF_TIP;
        set_tip(&mut d, &tip);
        unsafe { Shell_NotifyIconW(NIM_MODIFY, &d) };
        *last = tip;
    }
}

fn show_main(hwnd: HWND) {
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        ShowWindow(hwnd, SW_RESTORE);
        SetForegroundWindow(hwnd);
    }
}

fn popup(hwnd: HWND, items: &[(i32, &str, bool)], x: i32, y: i32) -> i32 {
    unsafe {
        let m = CreatePopupMenu();
        for (id, t, checked) in items {
            if *id == 0 {
                AppendMenuW(m, MF_SEPARATOR, 0, null());
            } else {
                let w = wide(t);
                AppendMenuW(m, MF_STRING | if *checked { MF_CHECKED } else { 0 }, *id as usize, w.as_ptr());
            }
        }
        SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(m, TPM_RETURNCMD | TPM_RIGHTBUTTON, x, y, 0, hwnd, null());
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(m);
        cmd
    }
}

fn tray_menu(hwnd: HWND) {
    let mut p = POINT::default();
    unsafe { GetCursorPos(&mut p) };
    let cmd = popup(
        hwnd,
        &[
            (TM_SHOW, "Fenster anzeigen", false),
            (0, "", false),
            (TM_AUTO, "Autoclicker", AUTO_RUN.load(SeqCst)),
            (TM_HOLD, "Dauerklick", HOLD_ON.load(SeqCst)),
            (TM_TRIG, "Farb-Trigger", TRIG_RUN.load(SeqCst)),
            (TM_SEQ, "Klickfolge", SEQ_RUN.load(SeqCst)),
            (0, "", false),
            (TM_EXIT, "Beenden", false),
        ],
        p.x,
        p.y,
    );
    match cmd {
        TM_SHOW => show_main(hwnd),
        TM_AUTO => toggle_auto(),
        TM_HOLD => toggle_hold(),
        TM_TRIG => toggle_trigger(),
        TM_SEQ => toggle_seq(),
        TM_EXIT => unsafe {
            DestroyWindow(hwnd);
        },
        _ => {}
    }
}

fn unit_menu(hwnd: HWND, id: i32) {
    let mut rc = RECT::default();
    unsafe { GetWindowRect(item(id), &mut rc) };
    let cur = unit_of(id).load(SeqCst) as i32;
    let items: Vec<(i32, &str, bool)> = UNIT_LONG.iter().enumerate().map(|(i, t)| (i as i32 + 1, *t, i as i32 == cur)).collect();
    let cmd = popup(hwnd, &items, rc.left, rc.bottom + s(2));
    if (1..=4).contains(&cmd) {
        unit_of(id).store(cmd as u32 - 1, SeqCst);
        read_auto_settings();
        unsafe { InvalidateRect(item(id), null(), 0) };
    }
}

// ═════════════════════════ Ereignisse ═════════════════════════

fn on_timer(hwnd: HWND) {
    if INFO_ON.load(SeqCst) {
        let mut pt = POINT::default();
        unsafe { GetCursorPos(&mut pt) };
        INFO_X.store(pt.x, SeqCst);
        INFO_Y.store(pt.y, SeqCst);
        INFO_COLOR.store(read_pixel(pt.x, pt.y), SeqCst);
    }

    // Hauptfenster nur neu zeichnen, wenn sich etwas Sichtbares geaendert hat
    let snap = format!(
        "{:?}|{:?}|{:?}|{:?}|{:?}|{}|{}",
        status_of(St::Auto),
        status_of(St::Hold),
        status_of(St::Trig),
        status_of(St::Seq),
        info_lines(),
        INFO_COLOR.load(SeqCst),
        footer_text()
    );
    {
        let mut last = SNAP_MAIN.lock().unwrap();
        if *last != snap {
            *last = snap;
            unsafe { InvalidateRect(hwnd, null(), 0) };
        }
    }

    let mut bsnap = String::new();
    for bt in BTNS {
        bsnap.push_str(&btn_label(bt.id));
        bsnap.push(if btn_active(bt.id) { '1' } else { '0' });
    }
    {
        let mut last = SNAP_BTN.lock().unwrap();
        if *last != bsnap {
            *last = bsnap;
            invalidate_buttons();
        }
    }

    let step = if SEQ_RUN.load(SeqCst) { SEQ_STEP.load(SeqCst) } else { -1 };
    if SEQ_DRAWN_STEP.swap(step, SeqCst) != step {
        unsafe { InvalidateRect(item(ID_S_LIST), null(), 0) };
    }

    tray_update_tip(hwnd);
}

fn on_command(hwnd: HWND, id: i32, code: u32) {
    let clicked = code == BN_CLICKED || code == BN_DBLCLK;
    if clicked && CAPTURE.load(SeqCst) != 0 && !HK_BUTTONS.contains(&id) {
        end_capture();
    }
    match id {
        ID_A_GO if clicked => toggle_auto(),
        ID_H_GO if clicked => toggle_hold(),
        ID_T_GO if clicked => toggle_trigger(),
        ID_S_GO if clicked => toggle_seq(),

        ID_A_M1 | ID_A_M2 if clicked => AUTO_BTN.store((id == ID_A_M2) as u32, SeqCst),
        ID_H_M1 | ID_H_M2 if clicked => HOLD_SEL.store((id == ID_H_M2) as u32, SeqCst),
        ID_T_M1 | ID_T_M2 if clicked => TRIG_BTN.store((id == ID_T_M2) as u32, SeqCst),

        ID_T_ONCE if clicked => {
            TRIG_ONCE.fetch_xor(true, SeqCst);
        }
        ID_T_RESTORE if clicked => {
            TRIG_RESTORE.fetch_xor(true, SeqCst);
        }
        ID_S_LOOP if clicked => {
            SEQ_LOOP.fetch_xor(true, SeqCst);
        }
        ID_I_SW if clicked => {
            INFO_ON.fetch_xor(true, SeqCst);
            INFO_COLOR.store(u32::MAX, SeqCst);
        }

        ID_A_INT_U | ID_A_JIT_U | ID_T_INT_U | ID_S_WAIT_U | ID_T_COOL_U | ID_H_DUR_U if clicked => unit_menu(hwnd, id),
        ID_A_HK | ID_H_HK | ID_T_HK | ID_S_HK if clicked => start_capture(id),

        ID_A_INT | ID_A_JIT if code == EN_CHANGE => read_auto_settings(),
        ID_T_HEX if code == EN_CHANGE => unsafe {
            InvalidateRect(hwnd, null(), 0);
        },
        _ if code == EN_SETFOCUS && EDS.iter().any(|e| e.id == id) => {
            FOCUS_ED.store(id, SeqCst);
            unsafe { InvalidateRect(hwnd, null(), 0) };
        }
        _ if code == EN_KILLFOCUS && EDS.iter().any(|e| e.id == id) => {
            FOCUS_ED.store(0, SeqCst);
            unsafe { InvalidateRect(hwnd, null(), 0) };
        }

        ID_S_ADD1 if clicked => add_step(Step::Click(0)),
        ID_S_ADD2 if clicked => add_step(Step::Click(1)),
        ID_S_ADDW if clicked => match read_dur(ID_S_WAIT, ID_S_WAIT_U) {
            Some(ms) => add_step(Step::Wait(ms)),
            None => msg_box("Bitte eine gültige Wartezeit eingeben, z. B. 10 oder 0,5."),
        },
        ID_S_UP | ID_S_DOWN | ID_S_DEL | ID_S_CLEAR if clicked => edit_seq(id),
        ID_S_SAVE if clicked => save_seq(hwnd),
        ID_S_LOAD if clicked => load_seq(hwnd),

        ID_I_COPY if clicked => {
            if let Some((x, y, c)) = merk_selected() {
                let rgb = rgb_of(c);
                copy_to_clipboard(&format!(
                    "X {}, Y {}, RGB {}, {}, {}, Hex #{:06X}",
                    x, y, rgb >> 16, (rgb >> 8) & 0xFF, rgb & 0xFF, rgb
                ));
            }
        }
        ID_I_TOTRIG if clicked => merk_to_trigger(),
        ID_I_LIST if code == LBN_DBLCLK => merk_to_trigger(),
        ID_I_CLEAR if clicked => {
            MERK.lock().unwrap().clear();
            fill_list(ID_I_LIST, 0, -1);
        }

        ID_SEQ_TOGGLE if clicked => {
            SEQ_OPEN.fetch_xor(true, SeqCst);
            apply_layout(hwnd);
        }
        ID_INFO_TOGGLE if clicked => {
            INFO_OPEN.fetch_xor(true, SeqCst);
            apply_layout(hwnd);
        }
        ID_THEME if clicked => {
            DARK_MODE.fetch_xor(true, SeqCst);
            apply_theme(hwnd);
        }
        _ => {}
    }
    if clicked {
        invalidate_buttons();
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            MAIN.store(hwnd, SeqCst);
            build_ui(hwnd);
            apply_theme(hwnd);
            apply_layout(hwnd);
            tray_add(hwnd);
            register_hotkeys();
            read_auto_settings();
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, kb_hook, GetModuleHandleW(null()), 0);
            KB_HOOK.store(hook, SeqCst);
            SetTimer(hwnd, 1, 50, null());
            0
        }
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            paint_main(hwnd);
            0
        }
        WM_MEASUREITEM => {
            let mis = &mut *(lp as *mut MEASUREITEMSTRUCT);
            mis.itemHeight = s(28) as u32;
            1
        }
        WM_DRAWITEM => {
            let dis = &*(lp as *const DRAWITEMSTRUCT);
            if dis.CtlType == ODT_BUTTON {
                draw_button(dis);
            } else if dis.CtlType == ODT_LISTBOX {
                draw_list_item(dis);
            }
            1
        }
        WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
            let p = pal();
            SetTextColor(wp as HANDLE, cref(p.text));
            SetBkColor(wp as HANDLE, cref(p.input));
            INPUT_BRUSH.load(SeqCst)
        }
        WM_COMMAND => {
            on_command(hwnd, (wp & 0xFFFF) as i32, ((wp >> 16) & 0xFFFF) as u32);
            0
        }
        WM_HOTKEY => {
            match wp as i32 {
                1 => toggle_auto(),
                2 => toggle_hold(),
                3 => toggle_trigger(),
                4 => toggle_seq(),
                _ => {}
            }
            0
        }
        WM_GRAB => {
            grab_cursor_value();
            0
        }
        WM_TIMER => {
            on_timer(hwnd);
            0
        }
        WM_ACTIVATE => {
            if wp & 0xFFFF == 0 && CAPTURE.load(SeqCst) != 0 {
                end_capture();
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_SYSCOMMAND if (wp & 0xFFF0) == SC_MINIMIZE => {
            ShowWindow(hwnd, SW_HIDE);
            0
        }
        WM_TRAY => {
            match (lp as u32) & 0xFFFF {
                WM_LBUTTONUP => show_main(hwnd),
                WM_RBUTTONUP => tray_menu(hwnd),
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            AUTO_RUN.store(false, SeqCst);
            SEQ_RUN.store(false, SeqCst);
            TRIG_RUN.store(false, SeqCst);
            if HOLD_ON.swap(false, SeqCst) {
                send_mouse(&[up_flag(HOLD_ACTIVE.load(SeqCst))]);
            }
            unregister_hotkeys();
            let hook = KB_HOOK.swap(0, SeqCst);
            if hook != 0 {
                UnhookWindowsHookEx(hook);
            }
            let d = tray_data(hwnd);
            Shell_NotifyIconW(NIM_DELETE, &d);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}

// ───── Absturz-Diagnose ─────

fn crash_report(msg: &str) {
    let path = std::env::current_exe()
        .map(|p| p.with_file_name("maus-utility-absturz.txt"))
        .unwrap_or_else(|_| "maus-utility-absturz.txt".into());
    let _ = std::fs::write(&path, msg);
    let t = wide(&format!("Das Programm ist abgestürzt.\n\n{}\n\nDetails stehen in:\n{}", msg, path.display()));
    let c = wide("Maus-Utility – Fehler");
    unsafe { MessageBoxW(0, t.as_ptr(), c.as_ptr(), 0x10) };
}

unsafe extern "system" fn crash_filter(info: *const EXCEPTION_POINTERS) -> i32 {
    let (code, addr) = if info.is_null() || (*info).ExceptionRecord.is_null() {
        (0, 0)
    } else {
        let r = &*(*info).ExceptionRecord;
        (r.ExceptionCode, r.ExceptionAddress)
    };
    let base = GetModuleHandleW(null()) as usize;
    crash_report(&format!(
        "Ausnahme 0x{:08X} an Adresse 0x{:X} (Offset im Programm 0x{:X})",
        code,
        addr,
        addr.wrapping_sub(base)
    ));
    1
}

fn install_crash_handlers() {
    std::panic::set_hook(Box::new(|info| {
        crash_report(&format!("Rust-Panic: {}", info));
    }));
    unsafe { SetUnhandledExceptionFilter(crash_filter) };
}

fn make_font(pt10: i32, weight: i32, face: &str, dpi: i32) -> HANDLE {
    let f = wide(face);
    // pt10 = Punktgroesse * 10
    unsafe { CreateFontW(-(pt10 * dpi / 720), 0, 0, 0, weight, 0, 0, 0, 1, 0, 0, 5, 0, f.as_ptr()) }
}

fn main() {
    install_crash_handlers();
    unsafe {
        SetProcessDPIAware();
        let icc = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: 0x4000,
        };
        InitCommonControlsEx(&icc);

        let mut token = 0usize;
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            DebugEventCallback: 0,
            SuppressBackgroundThread: 0,
            SuppressExternalCodecs: 0,
        };
        GdiplusStartup(&mut token, &input, null_mut());

        let dc = GetDC(0);
        let dpi = GetDeviceCaps(dc, LOGPIXELSX).max(96);
        ReleaseDC(0, dc);
        SCALE_PCT.store(dpi * 100 / 96, SeqCst);

        let _ = FONTS.set(Fonts {
            normal: make_font(95, 400, "Segoe UI", dpi),
            small: make_font(85, 400, "Segoe UI", dpi),
            semi: make_font(110, 600, "Segoe UI Semibold", dpi),
            btn: make_font(95, 600, "Segoe UI Semibold", dpi),
            title: make_font(170, 400, "Georgia", dpi),
            mono: make_font(95, 400, "Consolas", dpi),
        });
        let _ = ICONS.set(Icons {
            big: make_mouse_icon(GetSystemMetrics(SM_CXICON)),
            small: make_mouse_icon(GetSystemMetrics(SM_CXSMICON)),
            header: make_mouse_icon(s(32)),
        });

        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);
        RNG.store(seed | 1, SeqCst);
        DARK_MODE.store(system_prefers_dark(), SeqCst);

        let inst = GetModuleHandleW(null());
        let icons = ICONS.get().expect("icons");
        let class = wide("MausUtilityWnd");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: wndproc,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: inst,
            hIcon: icons.big,
            hCursor: LoadCursorW(0, IDC_ARROW as *const u16),
            hbrBackground: 0,
            lpszMenuName: null(),
            lpszClassName: class.as_ptr(),
            hIconSm: icons.small,
        };
        RegisterClassExW(&wc);

        let (w, h) = outer_size(W_COLLAPSED, H_COLLAPSED);
        let title = wide("Maus-Utility");
        let hwnd = CreateWindowExW(
            0, class.as_ptr(), title.as_ptr(), window_style(),
            CW_USEDEFAULT, CW_USEDEFAULT, w, h, 0, 0, inst, null_mut(),
        );
        ShowWindow(hwnd, SW_SHOW);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            let cap = CAPTURE.load(SeqCst);
            if cap != 0 {
                match msg.message {
                    WM_KEYDOWN | WM_SYSKEYDOWN => {
                        capture_key(msg.wParam as u32);
                        continue;
                    }
                    WM_KEYUP | WM_SYSKEYUP | WM_CHAR | WM_SYSCHAR => continue,
                    WM_LBUTTONDOWN if msg.hwnd != item(cap) => end_capture(),
                    _ => {}
                }
            }
            if IsDialogMessageW(hwnd, &mut msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
