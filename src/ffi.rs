//! Win32-, GDI+- und DWM-Bindings (nur was das Programm braucht).

#![allow(non_snake_case, non_camel_case_types, clippy::upper_case_acronyms, dead_code)]

use std::ffi::c_void;

pub type HWND = isize;
pub type HANDLE = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;
pub type WNDPROC = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;
pub type HOOKPROC = unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: u32,
    pub pt: POINT,
}

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: u32,
    pub style: u32,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HANDLE,
    pub hIcon: HANDLE,
    pub hCursor: HANDLE,
    pub hbrBackground: HANDLE,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
    pub hIconSm: HANDLE,
}

#[repr(C)]
pub struct MOUSEINPUT {
    pub dx: i32,
    pub dy: i32,
    pub mouseData: u32,
    pub dwFlags: u32,
    pub time: u32,
    pub dwExtraInfo: usize,
}

#[repr(C)]
pub struct INPUT {
    pub kind: u32,
    pub mi: MOUSEINPUT,
}

#[repr(C)]
pub struct GUID {
    pub a: u32,
    pub b: u16,
    pub c: u16,
    pub d: [u8; 8],
}

#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cbSize: u32,
    pub hWnd: HWND,
    pub uID: u32,
    pub uFlags: u32,
    pub uCallbackMessage: u32,
    pub hIcon: HANDLE,
    pub szTip: [u16; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [u16; 256],
    pub uVersion: u32,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: u32,
    pub guidItem: GUID,
    pub hBalloonIcon: HANDLE,
}

#[repr(C)]
pub struct INITCOMMONCONTROLSEX {
    pub dwSize: u32,
    pub dwICC: u32,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HANDLE,
    pub fErase: i32,
    pub rcPaint: RECT,
    pub fRestore: i32,
    pub fIncUpdate: i32,
    pub rgbReserved: [u8; 32],
}

#[repr(C)]
pub struct DRAWITEMSTRUCT {
    pub CtlType: u32,
    pub CtlID: u32,
    pub itemID: u32,
    pub itemAction: u32,
    pub itemState: u32,
    pub hwndItem: HWND,
    pub hDC: HANDLE,
    pub rcItem: RECT,
    pub itemData: usize,
}

#[repr(C)]
pub struct MEASUREITEMSTRUCT {
    pub CtlType: u32,
    pub CtlID: u32,
    pub itemID: u32,
    pub itemWidth: u32,
    pub itemHeight: u32,
    pub itemData: usize,
}

#[repr(C)]
pub struct TRACKMOUSEEVENT {
    pub cbSize: u32,
    pub dwFlags: u32,
    pub hwndTrack: HWND,
    pub dwHoverTime: u32,
}

#[repr(C)]
pub struct KBDLLHOOKSTRUCT {
    pub vkCode: u32,
    pub scanCode: u32,
    pub flags: u32,
    pub time: u32,
    pub dwExtraInfo: usize,
}

#[repr(C)]
pub struct ICONINFO {
    pub fIcon: i32,
    pub xHotspot: u32,
    pub yHotspot: u32,
    pub hbmMask: HANDLE,
    pub hbmColor: HANDLE,
}

#[repr(C)]
pub struct BITMAPINFOHEADER {
    pub biSize: u32,
    pub biWidth: i32,
    pub biHeight: i32,
    pub biPlanes: u16,
    pub biBitCount: u16,
    pub biCompression: u32,
    pub biSizeImage: u32,
    pub biXPelsPerMeter: i32,
    pub biYPelsPerMeter: i32,
    pub biClrUsed: u32,
    pub biClrImportant: u32,
}

#[repr(C)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [u32; 1],
}

#[repr(C)]
pub struct OSVERSIONINFOW {
    pub dwOSVersionInfoSize: u32,
    pub dwMajorVersion: u32,
    pub dwMinorVersion: u32,
    pub dwBuildNumber: u32,
    pub dwPlatformId: u32,
    pub szCSDVersion: [u16; 128],
}

#[repr(C)]
pub struct GdiplusStartupInput {
    pub GdiplusVersion: u32,
    pub DebugEventCallback: usize,
    pub SuppressBackgroundThread: i32,
    pub SuppressExternalCodecs: i32,
}

#[link(name = "user32")]
extern "system" {
    pub fn RegisterClassExW(wc: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(
        ex: u32, class: *const u16, name: *const u16, style: u32,
        x: i32, y: i32, w: i32, h: i32,
        parent: HWND, menu: isize, inst: HANDLE, param: *mut c_void,
    ) -> HWND;
    pub fn DefWindowProcW(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn CallWindowProcW(prev: isize, h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn SetWindowLongPtrW(h: HWND, index: i32, value: isize) -> isize;
    pub fn GetMessageW(msg: *mut MSG, h: HWND, min: u32, max: u32) -> i32;
    pub fn GetMessageExtraInfo() -> isize;
    pub fn IsDialogMessageW(h: HWND, msg: *mut MSG) -> i32;
    pub fn TranslateMessage(msg: *const MSG) -> i32;
    pub fn DispatchMessageW(msg: *const MSG) -> LRESULT;
    pub fn PostQuitMessage(code: i32);
    pub fn PostMessageW(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> i32;
    pub fn SendMessageW(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn DestroyWindow(h: HWND) -> i32;
    pub fn ShowWindow(h: HWND, cmd: i32) -> i32;
    pub fn SetForegroundWindow(h: HWND) -> i32;
    pub fn SetFocus(h: HWND) -> HWND;
    pub fn SetWindowTextW(h: HWND, s: *const u16) -> i32;
    pub fn GetWindowTextW(h: HWND, buf: *mut u16, max: i32) -> i32;
    pub fn GetDlgItem(h: HWND, id: i32) -> HWND;
    pub fn InvalidateRect(h: HWND, r: *const RECT, erase: i32) -> i32;
    pub fn RedrawWindow(h: HWND, r: *const RECT, rgn: HANDLE, flags: u32) -> i32;
    pub fn AdjustWindowRectEx(r: *mut RECT, style: u32, menu: i32, ex: u32) -> i32;
    pub fn SetWindowPos(h: HWND, after: HWND, x: i32, y: i32, w: i32, h2: i32, flags: u32) -> i32;
    pub fn MoveWindow(h: HWND, x: i32, y: i32, w: i32, h2: i32, repaint: i32) -> i32;
    pub fn GetClientRect(h: HWND, r: *mut RECT) -> i32;
    pub fn GetWindowRect(h: HWND, r: *mut RECT) -> i32;
    pub fn BeginPaint(h: HWND, ps: *mut PAINTSTRUCT) -> HANDLE;
    pub fn EndPaint(h: HWND, ps: *const PAINTSTRUCT) -> i32;
    pub fn FillRect(dc: HANDLE, r: *const RECT, brush: HANDLE) -> i32;
    pub fn DrawTextW(dc: HANDLE, s: *const u16, n: i32, r: *mut RECT, fmt: u32) -> i32;
    pub fn DrawIconEx(dc: HANDLE, x: i32, y: i32, icon: HANDLE, w: i32, h: i32, step: u32, brush: HANDLE, flags: u32) -> i32;
    pub fn CreateIconIndirect(ii: *const ICONINFO) -> HANDLE;
    pub fn RegisterHotKey(h: HWND, id: i32, mods: u32, vk: u32) -> i32;
    pub fn UnregisterHotKey(h: HWND, id: i32) -> i32;
    pub fn SendInput(n: u32, inputs: *const INPUT, size: i32) -> u32;
    pub fn GetCursorPos(p: *mut POINT) -> i32;
    pub fn SetCursorPos(x: i32, y: i32) -> i32;
    pub fn GetDC(h: HWND) -> HANDLE;
    pub fn ReleaseDC(h: HWND, dc: HANDLE) -> i32;
    pub fn SetTimer(h: HWND, id: usize, ms: u32, f: *const c_void) -> usize;
    pub fn LoadCursorW(inst: HANDLE, name: *const u16) -> HANDLE;
    pub fn CreatePopupMenu() -> HANDLE;
    pub fn AppendMenuW(menu: HANDLE, flags: u32, id: usize, text: *const u16) -> i32;
    pub fn TrackPopupMenu(menu: HANDLE, flags: u32, x: i32, y: i32, r: i32, h: HWND, rc: *const RECT) -> i32;
    pub fn DestroyMenu(menu: HANDLE) -> i32;
    pub fn MessageBoxW(h: HWND, text: *const u16, caption: *const u16, kind: u32) -> i32;
    pub fn SetProcessDPIAware() -> i32;
    pub fn GetSystemMetrics(index: i32) -> i32;
    pub fn GetKeyState(vk: i32) -> i16;
    pub fn GetAsyncKeyState(vk: i32) -> i16;
    pub fn GetKeyNameTextW(l: i32, buf: *mut u16, n: i32) -> i32;
    pub fn MapVirtualKeyW(code: u32, kind: u32) -> u32;
    pub fn SetWindowsHookExW(id: i32, f: HOOKPROC, module: HANDLE, thread: u32) -> HANDLE;
    pub fn CallNextHookEx(h: HANDLE, code: i32, w: WPARAM, l: LPARAM) -> LRESULT;
    pub fn UnhookWindowsHookEx(h: HANDLE) -> i32;
    pub fn TrackMouseEvent(t: *mut TRACKMOUSEEVENT) -> i32;
    pub fn OpenClipboard(h: HWND) -> i32;
    pub fn EmptyClipboard() -> i32;
    pub fn SetClipboardData(fmt: u32, h: HANDLE) -> HANDLE;
    pub fn CloseClipboard() -> i32;
}

#[link(name = "gdi32")]
extern "system" {
    pub fn GetPixel(dc: HANDLE, x: i32, y: i32) -> u32;
    pub fn GetDeviceCaps(dc: HANDLE, index: i32) -> i32;
    pub fn CreateSolidBrush(color: u32) -> HANDLE;
    pub fn DeleteObject(obj: HANDLE) -> i32;
    pub fn SelectObject(dc: HANDLE, obj: HANDLE) -> HANDLE;
    pub fn CreateCompatibleDC(dc: HANDLE) -> HANDLE;
    pub fn CreateCompatibleBitmap(dc: HANDLE, w: i32, h: i32) -> HANDLE;
    pub fn DeleteDC(dc: HANDLE) -> i32;
    pub fn BitBlt(dst: HANDLE, x: i32, y: i32, w: i32, h: i32, src: HANDLE, sx: i32, sy: i32, rop: u32) -> i32;
    pub fn SetBkMode(dc: HANDLE, mode: i32) -> i32;
    pub fn SetBkColor(dc: HANDLE, c: u32) -> u32;
    pub fn SetTextColor(dc: HANDLE, c: u32) -> u32;
    pub fn CreateDIBSection(dc: HANDLE, bi: *const BITMAPINFO, usage: u32, bits: *mut *mut c_void, section: HANDLE, offset: u32) -> HANDLE;
    pub fn CreateBitmap(w: i32, h: i32, planes: u32, bpp: u32, bits: *const c_void) -> HANDLE;
    pub fn CreateFontW(
        h: i32, w: i32, esc: i32, orient: i32, weight: i32,
        italic: u32, underline: u32, strike: u32, charset: u32,
        out_prec: u32, clip_prec: u32, quality: u32, pitch: u32, face: *const u16,
    ) -> HANDLE;
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GetModuleHandleW(name: *const u16) -> HANDLE;
    pub fn GlobalAlloc(flags: u32, bytes: usize) -> HANDLE;
    pub fn GlobalLock(h: HANDLE) -> *mut c_void;
    pub fn GlobalUnlock(h: HANDLE) -> i32;
    pub fn LoadLibraryW(name: *const u16) -> HANDLE;
    pub fn GetProcAddress(module: HANDLE, name: *const u8) -> usize;
}

#[link(name = "shell32")]
extern "system" {
    pub fn Shell_NotifyIconW(msg: u32, data: *const NOTIFYICONDATAW) -> i32;
}

#[link(name = "comctl32")]
extern "system" {
    pub fn InitCommonControlsEx(p: *const INITCOMMONCONTROLSEX) -> i32;
}

#[link(name = "dwmapi")]
extern "system" {
    pub fn DwmSetWindowAttribute(h: HWND, attr: u32, value: *const c_void, size: u32) -> i32;
}

#[link(name = "uxtheme")]
extern "system" {
    pub fn SetWindowTheme(h: HWND, app: *const u16, idlist: *const u16) -> i32;
}

#[link(name = "advapi32")]
extern "system" {
    pub fn RegGetValueW(
        key: isize, sub: *const u16, value: *const u16, flags: u32,
        kind: *mut u32, data: *mut c_void, size: *mut u32,
    ) -> i32;
}

#[link(name = "ntdll")]
extern "system" {
    pub fn RtlGetVersion(info: *mut OSVERSIONINFOW) -> i32;
}

#[link(name = "gdiplus")]
extern "system" {
    pub fn GdiplusStartup(token: *mut usize, input: *const GdiplusStartupInput, output: *mut c_void) -> i32;
    pub fn GdipCreateFromHDC(dc: HANDLE, g: *mut isize) -> i32;
    pub fn GdipDeleteGraphics(g: isize) -> i32;
    pub fn GdipSetSmoothingMode(g: isize, mode: i32) -> i32;
    pub fn GdipSetPixelOffsetMode(g: isize, mode: i32) -> i32;
    pub fn GdipCreateSolidFill(argb: u32, brush: *mut isize) -> i32;
    pub fn GdipDeleteBrush(b: isize) -> i32;
    pub fn GdipCreatePen1(argb: u32, width: f32, unit: i32, pen: *mut isize) -> i32;
    pub fn GdipSetPenStartCap(pen: isize, cap: i32) -> i32;
    pub fn GdipSetPenEndCap(pen: isize, cap: i32) -> i32;
    pub fn GdipDeletePen(p: isize) -> i32;
    pub fn GdipCreatePath(fill: i32, path: *mut isize) -> i32;
    pub fn GdipDeletePath(p: isize) -> i32;
    pub fn GdipAddPathArc(p: isize, x: f32, y: f32, w: f32, h: f32, start: f32, sweep: f32) -> i32;
    pub fn GdipClosePathFigure(p: isize) -> i32;
    pub fn GdipFillPath(g: isize, b: isize, p: isize) -> i32;
    pub fn GdipDrawPath(g: isize, pen: isize, p: isize) -> i32;
    pub fn GdipFillEllipse(g: isize, b: isize, x: f32, y: f32, w: f32, h: f32) -> i32;
    pub fn GdipDrawLine(g: isize, pen: isize, x1: f32, y1: f32, x2: f32, y2: f32) -> i32;
}

// ───── Konstanten ─────
pub const WM_NULL: u32 = 0x0000;
pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_ACTIVATE: u32 = 0x0006;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_ERASEBKGND: u32 = 0x0014;
pub const WM_SETFONT: u32 = 0x0030;
pub const WM_DRAWITEM: u32 = 0x002B;
pub const WM_MEASUREITEM: u32 = 0x002C;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_KEYUP: u32 = 0x0101;
pub const WM_CHAR: u32 = 0x0102;
pub const WM_SYSKEYDOWN: u32 = 0x0104;
pub const WM_SYSKEYUP: u32 = 0x0105;
pub const WM_SYSCHAR: u32 = 0x0106;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_SYSCOMMAND: u32 = 0x0112;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_CTLCOLOREDIT: u32 = 0x0133;
pub const WM_CTLCOLORLISTBOX: u32 = 0x0134;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_LBUTTONDBLCLK: u32 = 0x0203;
pub const WM_RBUTTONUP: u32 = 0x0205;
pub const WM_MOUSELEAVE: u32 = 0x02A3;
pub const WM_HOTKEY: u32 = 0x0312;
pub const WM_APP: u32 = 0x8000;
pub const SC_MINIMIZE: usize = 0xF020;

pub const WS_OVERLAPPED: u32 = 0x0000_0000;
pub const WS_CAPTION: u32 = 0x00C0_0000;
pub const WS_SYSMENU: u32 = 0x0008_0000;
pub const WS_MINIMIZEBOX: u32 = 0x0002_0000;
pub const WS_CLIPCHILDREN: u32 = 0x0200_0000;
pub const WS_CHILD: u32 = 0x4000_0000;
pub const WS_VISIBLE: u32 = 0x1000_0000;
pub const WS_TABSTOP: u32 = 0x0001_0000;
pub const WS_VSCROLL: u32 = 0x0020_0000;
pub const CW_USEDEFAULT: i32 = 0x8000_0000_u32 as i32;
pub const GWLP_WNDPROC: i32 = -4;

pub const BS_OWNERDRAW: u32 = 0xB;
pub const ES_AUTOHSCROLL: u32 = 0x80;
pub const LBS_NOTIFY: u32 = 0x1;
pub const LBS_OWNERDRAWFIXED: u32 = 0x10;
pub const LBS_HASSTRINGS: u32 = 0x40;
pub const LBS_NOINTEGRALHEIGHT: u32 = 0x100;

pub const BN_CLICKED: u32 = 0;
pub const BN_DBLCLK: u32 = 5;
pub const LBN_DBLCLK: u32 = 2;
pub const EN_SETFOCUS: u32 = 0x0100;
pub const EN_KILLFOCUS: u32 = 0x0200;
pub const EN_CHANGE: u32 = 0x0300;
pub const LB_ADDSTRING: u32 = 0x0180;
pub const LB_RESETCONTENT: u32 = 0x0184;
pub const LB_SETCURSEL: u32 = 0x0186;
pub const LB_GETCURSEL: u32 = 0x0188;

pub const ODT_LISTBOX: u32 = 2;
pub const ODT_BUTTON: u32 = 4;
pub const ODS_SELECTED: u32 = 0x1;
pub const ODS_FOCUS: u32 = 0x10;
pub const ODS_NOFOCUSRECT: u32 = 0x200;

pub const MOD_ALT: u32 = 0x1;
pub const MOD_CONTROL: u32 = 0x2;
pub const MOD_SHIFT: u32 = 0x4;
pub const MOD_NOREPEAT: u32 = 0x4000;

pub const MOUSEEVENTF_LEFTDOWN: u32 = 0x02;
pub const MOUSEEVENTF_LEFTUP: u32 = 0x04;
pub const MOUSEEVENTF_RIGHTDOWN: u32 = 0x08;
pub const MOUSEEVENTF_RIGHTUP: u32 = 0x10;

pub const NIM_ADD: u32 = 0;
pub const NIM_MODIFY: u32 = 1;
pub const NIM_DELETE: u32 = 2;
pub const NIF_MESSAGE: u32 = 1;
pub const NIF_ICON: u32 = 2;
pub const NIF_TIP: u32 = 4;

pub const MF_STRING: u32 = 0x0;
pub const MF_CHECKED: u32 = 0x8;
pub const MF_SEPARATOR: u32 = 0x800;
pub const TPM_RIGHTBUTTON: u32 = 0x2;
pub const TPM_RETURNCMD: u32 = 0x100;

pub const SW_HIDE: i32 = 0;
pub const SW_SHOW: i32 = 5;
pub const SW_RESTORE: i32 = 9;
pub const SWP_NOMOVE: u32 = 0x2;
pub const SWP_NOZORDER: u32 = 0x4;
pub const SWP_NOACTIVATE: u32 = 0x10;
pub const RDW_INVALIDATE: u32 = 0x1;
pub const RDW_ERASE: u32 = 0x4;
pub const RDW_ALLCHILDREN: u32 = 0x80;
pub const RDW_FRAME: u32 = 0x400;

pub const DT_CENTER: u32 = 0x1;
pub const DT_RIGHT: u32 = 0x2;
pub const DT_VCENTER: u32 = 0x4;
pub const DT_SINGLELINE: u32 = 0x20;
pub const DT_CALCRECT: u32 = 0x400;
pub const DT_NOPREFIX: u32 = 0x800;
pub const DT_END_ELLIPSIS: u32 = 0x8000;

pub const SRCCOPY: u32 = 0x00CC_0020;
pub const TME_LEAVE: u32 = 0x2;
pub const WH_KEYBOARD_LL: i32 = 13;
pub const IDC_ARROW: usize = 32512;
pub const LOGPIXELSX: i32 = 88;
pub const SM_CXICON: i32 = 11;
pub const SM_CXSMICON: i32 = 49;
pub const MB_ICONWARNING: u32 = 0x30;
pub const DI_NORMAL: u32 = 3;
pub const CF_UNICODETEXT: u32 = 13;
pub const GMEM_MOVEABLE: u32 = 0x2;
pub const HKEY_CURRENT_USER: isize = 0x8000_0001_u32 as i32 as isize;
pub const RRF_RT_REG_DWORD: u32 = 0x10;

// ───── Datei-Dialoge ─────

#[repr(C)]
pub struct OPENFILENAMEW {
    pub lStructSize: u32,
    pub hwndOwner: HWND,
    pub hInstance: HANDLE,
    pub lpstrFilter: *const u16,
    pub lpstrCustomFilter: *mut u16,
    pub nMaxCustFilter: u32,
    pub nFilterIndex: u32,
    pub lpstrFile: *mut u16,
    pub nMaxFile: u32,
    pub lpstrFileTitle: *mut u16,
    pub nMaxFileTitle: u32,
    pub lpstrInitialDir: *const u16,
    pub lpstrTitle: *const u16,
    pub Flags: u32,
    pub nFileOffset: u16,
    pub nFileExtension: u16,
    pub lpstrDefExt: *const u16,
    pub lCustData: isize,
    pub lpfnHook: usize,
    pub lpTemplateName: *const u16,
    pub pvReserved: *mut c_void,
    pub dwReserved: u32,
    pub FlagsEx: u32,
}

#[link(name = "comdlg32")]
extern "system" {
    pub fn GetSaveFileNameW(ofn: *mut OPENFILENAMEW) -> i32;
    pub fn GetOpenFileNameW(ofn: *mut OPENFILENAMEW) -> i32;
}

pub const OFN_OVERWRITEPROMPT: u32 = 0x2;
pub const OFN_NOCHANGEDIR: u32 = 0x8;
pub const OFN_PATHMUSTEXIST: u32 = 0x800;
pub const OFN_FILEMUSTEXIST: u32 = 0x1000;

// ───── Absturz-Diagnose ─────

#[repr(C)]
pub struct EXCEPTION_RECORD {
    pub ExceptionCode: u32,
    pub ExceptionFlags: u32,
    pub ExceptionRecord: *mut EXCEPTION_RECORD,
    pub ExceptionAddress: usize,
    pub NumberParameters: u32,
    pub ExceptionInformation: [usize; 15],
}

#[repr(C)]
pub struct EXCEPTION_POINTERS {
    pub ExceptionRecord: *const EXCEPTION_RECORD,
    pub ContextRecord: *mut c_void,
}

pub type EXCEPTION_FILTER = unsafe extern "system" fn(*const EXCEPTION_POINTERS) -> i32;

#[link(name = "kernel32")]
extern "system" {
    pub fn SetUnhandledExceptionFilter(f: EXCEPTION_FILTER) -> usize;
}
