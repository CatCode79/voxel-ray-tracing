//= IMPORTS ==================================================================

use windows_sys::Win32::{
    System::SystemServices::LANG_KOREAN, UI::Input::KeyboardAndMouse::GetKeyboardLayout,
};

use crate::{primarylangid, unsigned_loword};

//= KEY CODE (took from winit 0.29.15) =======================================

/// Code representing the location of a physical key
///
/// This mostly conforms to the UI Events Specification's [`KeyboardEvent.code`] with a few
/// exceptions:
/// - The keys that the specification calls "MetaLeft" and "MetaRight" are named "SuperLeft" and
///   "SuperRight" here.
/// - The key that the specification calls "Super" is reported as `Unidentified` here.
///
/// [`KeyboardEvent.code`]: https://w3c.github.io/uievents-code/#code-value-tables
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyCode {
    /// <kbd>`</kbd> on a US keyboard. This is also called a backtick or grave.
    /// This is the <kbd>半角</kbd>/<kbd>全角</kbd>/<kbd>漢字</kbd>
    /// (hankaku/zenkaku/kanji) key on Japanese keyboards
    Backquote,
    /// Used for both the US <kbd>\\</kbd> (on the 101-key layout) and also for the key
    /// located between the <kbd>"</kbd> and <kbd>Enter</kbd> keys on row C of the 102-,
    /// 104- and 106-key layouts.
    /// Labeled <kbd>#</kbd> on a UK (102) keyboard.
    Backslash,
    /// <kbd>[</kbd> on a US keyboard.
    BracketLeft,
    /// <kbd>]</kbd> on a US keyboard.
    BracketRight,
    /// <kbd>,</kbd> on a US keyboard.
    Comma,
    /// <kbd>0</kbd> on a US keyboard.
    Digit0,
    /// <kbd>1</kbd> on a US keyboard.
    Digit1,
    /// <kbd>2</kbd> on a US keyboard.
    Digit2,
    /// <kbd>3</kbd> on a US keyboard.
    Digit3,
    /// <kbd>4</kbd> on a US keyboard.
    Digit4,
    /// <kbd>5</kbd> on a US keyboard.
    Digit5,
    /// <kbd>6</kbd> on a US keyboard.
    Digit6,
    /// <kbd>7</kbd> on a US keyboard.
    Digit7,
    /// <kbd>8</kbd> on a US keyboard.
    Digit8,
    /// <kbd>9</kbd> on a US keyboard.
    Digit9,
    /// <kbd>=</kbd> on a US keyboard.
    Equal,
    /// Located between the left <kbd>Shift</kbd> and <kbd>Z</kbd> keys.
    /// Labeled <kbd>\\</kbd> on a UK keyboard.
    IntlBackslash,
    /// Located between the <kbd>/</kbd> and right <kbd>Shift</kbd> keys.
    /// Labeled <kbd>\\</kbd> (ro) on a Japanese keyboard.
    IntlRo,
    /// Located between the <kbd>=</kbd> and <kbd>Backspace</kbd> keys.
    /// Labeled <kbd>¥</kbd> (yen) on a Japanese keyboard. <kbd>\\</kbd> on a
    /// Russian keyboard.
    IntlYen,
    /// <kbd>a</kbd> on a US keyboard.
    /// Labeled <kbd>q</kbd> on an AZERTY (e.g., French) keyboard.
    KeyA,
    /// <kbd>b</kbd> on a US keyboard.
    KeyB,
    /// <kbd>c</kbd> on a US keyboard.
    KeyC,
    /// <kbd>d</kbd> on a US keyboard.
    KeyD,
    /// <kbd>e</kbd> on a US keyboard.
    KeyE,
    /// <kbd>f</kbd> on a US keyboard.
    KeyF,
    /// <kbd>g</kbd> on a US keyboard.
    KeyG,
    /// <kbd>h</kbd> on a US keyboard.
    KeyH,
    /// <kbd>i</kbd> on a US keyboard.
    KeyI,
    /// <kbd>j</kbd> on a US keyboard.
    KeyJ,
    /// <kbd>k</kbd> on a US keyboard.
    KeyK,
    /// <kbd>l</kbd> on a US keyboard.
    KeyL,
    /// <kbd>m</kbd> on a US keyboard.
    KeyM,
    /// <kbd>n</kbd> on a US keyboard.
    KeyN,
    /// <kbd>o</kbd> on a US keyboard.
    KeyO,
    /// <kbd>p</kbd> on a US keyboard.
    KeyP,
    /// <kbd>q</kbd> on a US keyboard.
    /// Labeled <kbd>a</kbd> on an AZERTY (e.g., French) keyboard.
    KeyQ,
    /// <kbd>r</kbd> on a US keyboard.
    KeyR,
    /// <kbd>s</kbd> on a US keyboard.
    KeyS,
    /// <kbd>t</kbd> on a US keyboard.
    KeyT,
    /// <kbd>u</kbd> on a US keyboard.
    KeyU,
    /// <kbd>v</kbd> on a US keyboard.
    KeyV,
    /// <kbd>w</kbd> on a US keyboard.
    /// Labeled <kbd>z</kbd> on an AZERTY (e.g., French) keyboard.
    KeyW,
    /// <kbd>x</kbd> on a US keyboard.
    KeyX,
    /// <kbd>y</kbd> on a US keyboard.
    /// Labeled <kbd>z</kbd> on a QWERTZ (e.g., German) keyboard.
    KeyY,
    /// <kbd>z</kbd> on a US keyboard.
    /// Labeled <kbd>w</kbd> on an AZERTY (e.g., French) keyboard, and <kbd>y</kbd> on a
    /// QWERTZ (e.g., German) keyboard.
    KeyZ,
    /// <kbd>-</kbd> on a US keyboard.
    Minus,
    /// <kbd>.</kbd> on a US keyboard.
    Period,
    /// <kbd>'</kbd> on a US keyboard.
    Quote,
    /// <kbd>;</kbd> on a US keyboard.
    Semicolon,
    /// <kbd>/</kbd> on a US keyboard.
    Slash,
    /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    AltLeft,
    /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    /// This is labeled <kbd>AltGr</kbd> on many keyboard layouts.
    AltRight,
    /// <kbd>Backspace</kbd> or <kbd>⌫</kbd>.
    /// Labeled <kbd>Delete</kbd> on Apple keyboards.
    Backspace,
    /// <kbd>CapsLock</kbd> or <kbd>⇪</kbd>
    CapsLock,
    /// The application context menu key, which is typically found between the right
    /// <kbd>Super</kbd> key and the right <kbd>Control</kbd> key.
    ContextMenu,
    /// <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlLeft,
    /// <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlRight,
    /// <kbd>Enter</kbd> or <kbd>↵</kbd>. Labeled <kbd>Return</kbd> on Apple keyboards.
    Enter,
    /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperLeft,
    /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperRight,
    /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftLeft,
    /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftRight,
    /// <kbd> </kbd> (space)
    Space,
    /// <kbd>Tab</kbd> or <kbd>⇥</kbd>
    Tab,
    /// Japanese: <kbd>変</kbd> (henkan)
    Convert,
    /// Japanese: <kbd>カタカナ</kbd>/<kbd>ひらがな</kbd>/<kbd>ローマ字</kbd> (katakana/hiragana/romaji)
    KanaMode,
    /// Korean: HangulMode <kbd>한/영</kbd> (han/yeong)
    ///
    /// Japanese (Mac keyboard): <kbd>か</kbd> (kana)
    Lang1,
    /// Korean: Hanja <kbd>한</kbd> (hanja)
    ///
    /// Japanese (Mac keyboard): <kbd>英</kbd> (eisu)
    Lang2,
    /// Japanese (word-processing keyboard): Katakana
    Lang3,
    /// Japanese (word-processing keyboard): Hiragana
    Lang4,
    /// Japanese (word-processing keyboard): Zenkaku/Hankaku
    Lang5,
    /// Japanese: <kbd>無変換</kbd> (muhenkan)
    NonConvert,
    /// <kbd>⌦</kbd>. The forward delete key.
    /// Note that on Apple keyboards, the key labelled <kbd>Delete</kbd> on the main part of
    /// the keyboard is encoded as [`Backspace`].
    ///
    /// [`Backspace`]: Self::Backspace
    Delete,
    /// <kbd>Page Down</kbd>, <kbd>End</kbd>, or <kbd>↘</kbd>
    End,
    /// <kbd>Help</kbd>. Not present on standard PC keyboards.
    Help,
    /// <kbd>Home</kbd> or <kbd>↖</kbd>
    Home,
    /// <kbd>Insert</kbd> or <kbd>Ins</kbd>. Not present on Apple keyboards.
    Insert,
    /// <kbd>Page Down</kbd>, <kbd>PgDn</kbd>, or <kbd>⇟</kbd>
    PageDown,
    /// <kbd>Page Up</kbd>, <kbd>PgUp</kbd>, or <kbd>⇞</kbd>
    PageUp,
    /// <kbd>↓</kbd>
    ArrowDown,
    /// <kbd>←</kbd>
    ArrowLeft,
    /// <kbd>→</kbd>
    ArrowRight,
    /// <kbd>↑</kbd>
    ArrowUp,
    /// On the Mac, this is used for the numpad <kbd>Clear</kbd> key.
    NumLock,
    /// <kbd>0 Ins</kbd> on a keyboard. <kbd>0</kbd> on a phone or remote control
    Numpad0,
    /// <kbd>1 End</kbd> on a keyboard. <kbd>1</kbd> or <kbd>1 QZ</kbd> on a phone or remote control
    Numpad1,
    /// <kbd>2 ↓</kbd> on a keyboard. <kbd>2 ABC</kbd> on a phone or remote control
    Numpad2,
    /// <kbd>3 PgDn</kbd> on a keyboard. <kbd>3 DEF</kbd> on a phone or remote control
    Numpad3,
    /// <kbd>4 ←</kbd> on a keyboard. <kbd>4 GHI</kbd> on a phone or remote control
    Numpad4,
    /// <kbd>5</kbd> on a keyboard. <kbd>5 JKL</kbd> on a phone or remote control
    Numpad5,
    /// <kbd>6 →</kbd> on a keyboard. <kbd>6 MNO</kbd> on a phone or remote control
    Numpad6,
    /// <kbd>7 Home</kbd> on a keyboard. <kbd>7 PQRS</kbd> or <kbd>7 PRS</kbd> on a phone
    /// or remote control
    Numpad7,
    /// <kbd>8 ↑</kbd> on a keyboard. <kbd>8 TUV</kbd> on a phone or remote control
    Numpad8,
    /// <kbd>9 PgUp</kbd> on a keyboard. <kbd>9 WXYZ</kbd> or <kbd>9 WXY</kbd> on a phone
    /// or remote control
    Numpad9,
    /// <kbd>+</kbd>
    NumpadAdd,
    /// Found on the Microsoft Natural Keyboard.
    NumpadBackspace,
    /// <kbd>C</kbd> or <kbd>A</kbd> (All Clear). Also for use with numpads that have a
    /// <kbd>Clear</kbd> key that is separate from the <kbd>NumLock</kbd> key. On the Mac, the
    /// numpad <kbd>Clear</kbd> key is encoded as [`NumLock`].
    ///
    /// [`NumLock`]: Self::NumLock
    NumpadClear,
    /// <kbd>C</kbd> (Clear Entry)
    NumpadClearEntry,
    /// <kbd>,</kbd> (thousands separator). For locales where the thousands separator
    /// is a "." (e.g., Brazil), this key may generate a <kbd>.</kbd>.
    NumpadComma,
    /// <kbd>. Del</kbd>. For locales where the decimal separator is "," (e.g.,
    /// Brazil), this key may generate a <kbd>,</kbd>.
    NumpadDecimal,
    /// <kbd>/</kbd>
    NumpadDivide,
    NumpadEnter,
    /// <kbd>=</kbd>
    NumpadEqual,
    /// <kbd>#</kbd> on a phone or remote control device. This key is typically found
    /// below the <kbd>9</kbd> key and to the right of the <kbd>0</kbd> key.
    NumpadHash,
    /// <kbd>M</kbd> Add current entry to the value stored in memory.
    NumpadMemoryAdd,
    /// <kbd>M</kbd> Clear the value stored in memory.
    NumpadMemoryClear,
    /// <kbd>M</kbd> Replace the current entry with the value stored in memory.
    NumpadMemoryRecall,
    /// <kbd>M</kbd> Replace the value stored in memory with the current entry.
    NumpadMemoryStore,
    /// <kbd>M</kbd> Subtract current entry from the value stored in memory.
    NumpadMemorySubtract,
    /// <kbd>*</kbd> on a keyboard. For use with numpads that provide mathematical
    /// operations (<kbd>+</kbd>, <kbd>-</kbd> <kbd>*</kbd> and <kbd>/</kbd>).
    ///
    /// Use `NumpadStar` for the <kbd>*</kbd> key on phones and remote controls.
    NumpadMultiply,
    /// <kbd>(</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenLeft,
    /// <kbd>)</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenRight,
    /// <kbd>*</kbd> on a phone or remote control device.
    ///
    /// This key is typically found below the <kbd>7</kbd> key and to the left of
    /// the <kbd>0</kbd> key.
    ///
    /// Use <kbd>"NumpadMultiply"</kbd> for the <kbd>*</kbd> key on
    /// numeric keypads.
    NumpadStar,
    /// <kbd>-</kbd>
    NumpadSubtract,
    /// <kbd>Esc</kbd> or <kbd>⎋</kbd>
    Escape,
    /// <kbd>Fn</kbd> This is typically a hardware key that does not generate a separate code.
    Fn,
    /// <kbd>FLock</kbd> or <kbd>FnLock</kbd>. Function Lock key. Found on the Microsoft
    /// Natural Keyboard.
    FnLock,
    /// <kbd>PrtScr SysRq</kbd> or <kbd>Print Screen</kbd>
    PrintScreen,
    /// <kbd>Scroll Lock</kbd>
    ScrollLock,
    /// <kbd>Pause Break</kbd>
    Pause,
    /// Some laptops place this key to the left of the <kbd>↑</kbd> key.
    ///
    /// This also the "back" button (triangle) on Android.
    BrowserBack,
    BrowserFavorites,
    /// Some laptops place this key to the right of the <kbd>↑</kbd> key.
    BrowserForward,
    /// The "home" button on Android.
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    /// <kbd>Eject</kbd> or <kbd>⏏</kbd>. This key is placed in the function section on some Apple
    /// keyboards.
    Eject,
    /// Sometimes labelled <kbd>My Computer</kbd> on the keyboard
    LaunchApp1,
    /// Sometimes labelled <kbd>Calculator</kbd> on the keyboard
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    /// This key is placed in the function section on some Apple keyboards, replacing the
    /// <kbd>Eject</kbd> key.
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    // Legacy modifier key. Also called "Super" in certain places.
    Meta,
    // Legacy modifier key.
    Hyper,
    Turbo,
    Abort,
    Resume,
    Suspend,
    /// Found on Sun’s USB keyboard.
    Again,
    /// Found on Sun’s USB keyboard.
    Copy,
    /// Found on Sun’s USB keyboard.
    Cut,
    /// Found on Sun’s USB keyboard.
    Find,
    /// Found on Sun’s USB keyboard.
    Open,
    /// Found on Sun’s USB keyboard.
    Paste,
    /// Found on Sun’s USB keyboard.
    Props,
    /// Found on Sun’s USB keyboard.
    Select,
    /// Found on Sun’s USB keyboard.
    Undo,
    /// Use for dedicated <kbd>ひらがな</kbd> key found on some Japanese word processing keyboards.
    Hiragana,
    /// Use for dedicated <kbd>カタカナ</kbd> key found on some Japanese word processing keyboards.
    Katakana,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F1,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F2,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F3,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F4,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F5,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F6,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F7,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F8,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F9,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F10,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F11,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F12,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F13,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F14,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F15,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F16,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F17,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F18,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F19,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F20,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F21,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F22,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F23,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F24,
    /// General-purpose function key.
    F25,
    /// General-purpose function key.
    F26,
    /// General-purpose function key.
    F27,
    /// General-purpose function key.
    F28,
    /// General-purpose function key.
    F29,
    /// General-purpose function key.
    F30,
    /// General-purpose function key.
    F31,
    /// General-purpose function key.
    F32,
    /// General-purpose function key.
    F33,
    /// General-purpose function key.
    F34,
    /// General-purpose function key.
    F35,
}

//= SCAN CODE CONVERSION (took from winit 0.29.15) ===========================

#[allow(unused)]
pub(crate) fn to_scancode(keycode: KeyCode) -> Option<u16> {
    // See `from_scancode` for more info

    let hkl = unsafe { GetKeyboardLayout(0) };

    let primary_lang_id = primarylangid(unsigned_loword(hkl as u32));
    let is_korean = primary_lang_id as u32 == LANG_KOREAN;

    match keycode {
        KeyCode::Backquote => Some(0x0029),
        KeyCode::Backslash => Some(0x002B),
        KeyCode::Backspace => Some(0x000E),
        KeyCode::BracketLeft => Some(0x001A),
        KeyCode::BracketRight => Some(0x001B),
        KeyCode::Comma => Some(0x0033),
        KeyCode::Digit0 => Some(0x000B),
        KeyCode::Digit1 => Some(0x0002),
        KeyCode::Digit2 => Some(0x0003),
        KeyCode::Digit3 => Some(0x0004),
        KeyCode::Digit4 => Some(0x0005),
        KeyCode::Digit5 => Some(0x0006),
        KeyCode::Digit6 => Some(0x0007),
        KeyCode::Digit7 => Some(0x0008),
        KeyCode::Digit8 => Some(0x0009),
        KeyCode::Digit9 => Some(0x000A),
        KeyCode::Equal => Some(0x000D),
        KeyCode::IntlBackslash => Some(0x0056),
        KeyCode::IntlRo => Some(0x0073),
        KeyCode::IntlYen => Some(0x007D),
        KeyCode::KeyA => Some(0x001E),
        KeyCode::KeyB => Some(0x0030),
        KeyCode::KeyC => Some(0x002E),
        KeyCode::KeyD => Some(0x0020),
        KeyCode::KeyE => Some(0x0012),
        KeyCode::KeyF => Some(0x0021),
        KeyCode::KeyG => Some(0x0022),
        KeyCode::KeyH => Some(0x0023),
        KeyCode::KeyI => Some(0x0017),
        KeyCode::KeyJ => Some(0x0024),
        KeyCode::KeyK => Some(0x0025),
        KeyCode::KeyL => Some(0x0026),
        KeyCode::KeyM => Some(0x0032),
        KeyCode::KeyN => Some(0x0031),
        KeyCode::KeyO => Some(0x0018),
        KeyCode::KeyP => Some(0x0019),
        KeyCode::KeyQ => Some(0x0010),
        KeyCode::KeyR => Some(0x0013),
        KeyCode::KeyS => Some(0x001F),
        KeyCode::KeyT => Some(0x0014),
        KeyCode::KeyU => Some(0x0016),
        KeyCode::KeyV => Some(0x002F),
        KeyCode::KeyW => Some(0x0011),
        KeyCode::KeyX => Some(0x002D),
        KeyCode::KeyY => Some(0x0015),
        KeyCode::KeyZ => Some(0x002C),
        KeyCode::Minus => Some(0x000C),
        KeyCode::Period => Some(0x0034),
        KeyCode::Quote => Some(0x0028),
        KeyCode::Semicolon => Some(0x0027),
        KeyCode::Slash => Some(0x0035),
        KeyCode::AltLeft => Some(0x0038),
        KeyCode::AltRight => Some(0xE038),
        KeyCode::CapsLock => Some(0x003A),
        KeyCode::ContextMenu => Some(0xE05D),
        KeyCode::ControlLeft => Some(0x001D),
        KeyCode::ControlRight => Some(0xE01D),
        KeyCode::Enter => Some(0x001C),
        KeyCode::SuperLeft => Some(0xE05B),
        KeyCode::SuperRight => Some(0xE05C),
        KeyCode::ShiftLeft => Some(0x002A),
        KeyCode::ShiftRight => Some(0x0036),
        KeyCode::Space => Some(0x0039),
        KeyCode::Tab => Some(0x000F),
        KeyCode::Convert => Some(0x0079),
        KeyCode::Lang1 => {
            if is_korean {
                Some(0xE0F2)
            } else {
                Some(0x0072)
            }
        }
        KeyCode::Lang2 => {
            if is_korean {
                Some(0xE0F1)
            } else {
                Some(0x0071)
            }
        }
        KeyCode::KanaMode => Some(0x0070),
        KeyCode::NonConvert => Some(0x007B),
        KeyCode::Delete => Some(0xE053),
        KeyCode::End => Some(0xE04F),
        KeyCode::Home => Some(0xE047),
        KeyCode::Insert => Some(0xE052),
        KeyCode::PageDown => Some(0xE051),
        KeyCode::PageUp => Some(0xE049),
        KeyCode::ArrowDown => Some(0xE050),
        KeyCode::ArrowLeft => Some(0xE04B),
        KeyCode::ArrowRight => Some(0xE04D),
        KeyCode::ArrowUp => Some(0xE048),
        KeyCode::NumLock => Some(0xE045),
        KeyCode::Numpad0 => Some(0x0052),
        KeyCode::Numpad1 => Some(0x004F),
        KeyCode::Numpad2 => Some(0x0050),
        KeyCode::Numpad3 => Some(0x0051),
        KeyCode::Numpad4 => Some(0x004B),
        KeyCode::Numpad5 => Some(0x004C),
        KeyCode::Numpad6 => Some(0x004D),
        KeyCode::Numpad7 => Some(0x0047),
        KeyCode::Numpad8 => Some(0x0048),
        KeyCode::Numpad9 => Some(0x0049),
        KeyCode::NumpadAdd => Some(0x004E),
        KeyCode::NumpadComma => Some(0x007E),
        KeyCode::NumpadDecimal => Some(0x0053),
        KeyCode::NumpadDivide => Some(0xE035),
        KeyCode::NumpadEnter => Some(0xE01C),
        KeyCode::NumpadEqual => Some(0x0059),
        KeyCode::NumpadMultiply => Some(0x0037),
        KeyCode::NumpadSubtract => Some(0x004A),
        KeyCode::Escape => Some(0x0001),
        KeyCode::F1 => Some(0x003B),
        KeyCode::F2 => Some(0x003C),
        KeyCode::F3 => Some(0x003D),
        KeyCode::F4 => Some(0x003E),
        KeyCode::F5 => Some(0x003F),
        KeyCode::F6 => Some(0x0040),
        KeyCode::F7 => Some(0x0041),
        KeyCode::F8 => Some(0x0042),
        KeyCode::F9 => Some(0x0043),
        KeyCode::F10 => Some(0x0044),
        KeyCode::F11 => Some(0x0057),
        KeyCode::F12 => Some(0x0058),
        KeyCode::F13 => Some(0x0064),
        KeyCode::F14 => Some(0x0065),
        KeyCode::F15 => Some(0x0066),
        KeyCode::F16 => Some(0x0067),
        KeyCode::F17 => Some(0x0068),
        KeyCode::F18 => Some(0x0069),
        KeyCode::F19 => Some(0x006A),
        KeyCode::F20 => Some(0x006B),
        KeyCode::F21 => Some(0x006C),
        KeyCode::F22 => Some(0x006D),
        KeyCode::F23 => Some(0x006E),
        KeyCode::F24 => Some(0x0076),
        KeyCode::PrintScreen => Some(0xE037),
        //KeyCode::PrintScreen => Some(0x0054), // Alt + PrintScreen
        KeyCode::ScrollLock => Some(0x0046),
        KeyCode::Pause => Some(0x0045),
        //KeyCode::Pause => Some(0xE046), // Ctrl + Pause
        KeyCode::BrowserBack => Some(0xE06A),
        KeyCode::BrowserFavorites => Some(0xE066),
        KeyCode::BrowserForward => Some(0xE069),
        KeyCode::BrowserHome => Some(0xE032),
        KeyCode::BrowserRefresh => Some(0xE067),
        KeyCode::BrowserSearch => Some(0xE065),
        KeyCode::BrowserStop => Some(0xE068),
        KeyCode::LaunchApp1 => Some(0xE06B),
        KeyCode::LaunchApp2 => Some(0xE021),
        KeyCode::LaunchMail => Some(0xE06C),
        KeyCode::MediaPlayPause => Some(0xE022),
        KeyCode::MediaSelect => Some(0xE06D),
        KeyCode::MediaStop => Some(0xE024),
        KeyCode::MediaTrackNext => Some(0xE019),
        KeyCode::MediaTrackPrevious => Some(0xE010),
        KeyCode::Power => Some(0xE05E),
        KeyCode::AudioVolumeDown => Some(0xE02E),
        KeyCode::AudioVolumeMute => Some(0xE020),
        KeyCode::AudioVolumeUp => Some(0xE030),
        _ => None,
    }
}

// See: https://www.win.tue.nl/~aeb/linux/kbd/scancodes-1.html
// and: https://www.w3.org/TR/uievents-code/
// and: The widget/NativeKeyToDOMCodeName.h file in the firefox source
pub(crate) fn from_scancode(scancode: u16) -> Option<KeyCode> {
    match scancode {
        0x0029 => Some(KeyCode::Backquote),
        0x002B => Some(KeyCode::Backslash),
        0x000E => Some(KeyCode::Backspace),
        0x001A => Some(KeyCode::BracketLeft),
        0x001B => Some(KeyCode::BracketRight),
        0x0033 => Some(KeyCode::Comma),
        0x000B => Some(KeyCode::Digit0),
        0x0002 => Some(KeyCode::Digit1),
        0x0003 => Some(KeyCode::Digit2),
        0x0004 => Some(KeyCode::Digit3),
        0x0005 => Some(KeyCode::Digit4),
        0x0006 => Some(KeyCode::Digit5),
        0x0007 => Some(KeyCode::Digit6),
        0x0008 => Some(KeyCode::Digit7),
        0x0009 => Some(KeyCode::Digit8),
        0x000A => Some(KeyCode::Digit9),
        0x000D => Some(KeyCode::Equal),
        0x0056 => Some(KeyCode::IntlBackslash),
        0x0073 => Some(KeyCode::IntlRo),
        0x007D => Some(KeyCode::IntlYen),
        0x001E => Some(KeyCode::KeyA),
        0x0030 => Some(KeyCode::KeyB),
        0x002E => Some(KeyCode::KeyC),
        0x0020 => Some(KeyCode::KeyD),
        0x0012 => Some(KeyCode::KeyE),
        0x0021 => Some(KeyCode::KeyF),
        0x0022 => Some(KeyCode::KeyG),
        0x0023 => Some(KeyCode::KeyH),
        0x0017 => Some(KeyCode::KeyI),
        0x0024 => Some(KeyCode::KeyJ),
        0x0025 => Some(KeyCode::KeyK),
        0x0026 => Some(KeyCode::KeyL),
        0x0032 => Some(KeyCode::KeyM),
        0x0031 => Some(KeyCode::KeyN),
        0x0018 => Some(KeyCode::KeyO),
        0x0019 => Some(KeyCode::KeyP),
        0x0010 => Some(KeyCode::KeyQ),
        0x0013 => Some(KeyCode::KeyR),
        0x001F => Some(KeyCode::KeyS),
        0x0014 => Some(KeyCode::KeyT),
        0x0016 => Some(KeyCode::KeyU),
        0x002F => Some(KeyCode::KeyV),
        0x0011 => Some(KeyCode::KeyW),
        0x002D => Some(KeyCode::KeyX),
        0x0015 => Some(KeyCode::KeyY),
        0x002C => Some(KeyCode::KeyZ),
        0x000C => Some(KeyCode::Minus),
        0x0034 => Some(KeyCode::Period),
        0x0028 => Some(KeyCode::Quote),
        0x0027 => Some(KeyCode::Semicolon),
        0x0035 => Some(KeyCode::Slash),
        0x0038 => Some(KeyCode::AltLeft),
        0xE038 => Some(KeyCode::AltRight),
        0x003A => Some(KeyCode::CapsLock),
        0xE05D => Some(KeyCode::ContextMenu),
        0x001D => Some(KeyCode::ControlLeft),
        0xE01D => Some(KeyCode::ControlRight),
        0x001C => Some(KeyCode::Enter),
        0xE05B => Some(KeyCode::SuperLeft),
        0xE05C => Some(KeyCode::SuperRight),
        0x002A => Some(KeyCode::ShiftLeft),
        0x0036 => Some(KeyCode::ShiftRight),
        0x0039 => Some(KeyCode::Space),
        0x000F => Some(KeyCode::Tab),
        0x0079 => Some(KeyCode::Convert),
        0x0072 => Some(KeyCode::Lang1), // for non-Korean layout
        0xE0F2 => Some(KeyCode::Lang1), // for Korean layout
        0x0071 => Some(KeyCode::Lang2), // for non-Korean layout
        0xE0F1 => Some(KeyCode::Lang2), // for Korean layout
        0x0070 => Some(KeyCode::KanaMode),
        0x007B => Some(KeyCode::NonConvert),
        0xE053 => Some(KeyCode::Delete),
        0xE04F => Some(KeyCode::End),
        0xE047 => Some(KeyCode::Home),
        0xE052 => Some(KeyCode::Insert),
        0xE051 => Some(KeyCode::PageDown),
        0xE049 => Some(KeyCode::PageUp),
        0xE050 => Some(KeyCode::ArrowDown),
        0xE04B => Some(KeyCode::ArrowLeft),
        0xE04D => Some(KeyCode::ArrowRight),
        0xE048 => Some(KeyCode::ArrowUp),
        0xE045 => Some(KeyCode::NumLock),
        0x0052 => Some(KeyCode::Numpad0),
        0x004F => Some(KeyCode::Numpad1),
        0x0050 => Some(KeyCode::Numpad2),
        0x0051 => Some(KeyCode::Numpad3),
        0x004B => Some(KeyCode::Numpad4),
        0x004C => Some(KeyCode::Numpad5),
        0x004D => Some(KeyCode::Numpad6),
        0x0047 => Some(KeyCode::Numpad7),
        0x0048 => Some(KeyCode::Numpad8),
        0x0049 => Some(KeyCode::Numpad9),
        0x004E => Some(KeyCode::NumpadAdd),
        0x007E => Some(KeyCode::NumpadComma),
        0x0053 => Some(KeyCode::NumpadDecimal),
        0xE035 => Some(KeyCode::NumpadDivide),
        0xE01C => Some(KeyCode::NumpadEnter),
        0x0059 => Some(KeyCode::NumpadEqual),
        0x0037 => Some(KeyCode::NumpadMultiply),
        0x004A => Some(KeyCode::NumpadSubtract),
        0x0001 => Some(KeyCode::Escape),
        0x003B => Some(KeyCode::F1),
        0x003C => Some(KeyCode::F2),
        0x003D => Some(KeyCode::F3),
        0x003E => Some(KeyCode::F4),
        0x003F => Some(KeyCode::F5),
        0x0040 => Some(KeyCode::F6),
        0x0041 => Some(KeyCode::F7),
        0x0042 => Some(KeyCode::F8),
        0x0043 => Some(KeyCode::F9),
        0x0044 => Some(KeyCode::F10),
        0x0057 => Some(KeyCode::F11),
        0x0058 => Some(KeyCode::F12),
        0x0064 => Some(KeyCode::F13),
        0x0065 => Some(KeyCode::F14),
        0x0066 => Some(KeyCode::F15),
        0x0067 => Some(KeyCode::F16),
        0x0068 => Some(KeyCode::F17),
        0x0069 => Some(KeyCode::F18),
        0x006A => Some(KeyCode::F19),
        0x006B => Some(KeyCode::F20),
        0x006C => Some(KeyCode::F21),
        0x006D => Some(KeyCode::F22),
        0x006E => Some(KeyCode::F23),
        0x0076 => Some(KeyCode::F24),
        0xE037 => Some(KeyCode::PrintScreen),
        0x0054 => Some(KeyCode::PrintScreen), // Alt + PrintScreen
        0x0046 => Some(KeyCode::ScrollLock),
        0x0045 => Some(KeyCode::Pause),
        0xE046 => Some(KeyCode::Pause), // Ctrl + Pause
        0xE06A => Some(KeyCode::BrowserBack),
        0xE066 => Some(KeyCode::BrowserFavorites),
        0xE069 => Some(KeyCode::BrowserForward),
        0xE032 => Some(KeyCode::BrowserHome),
        0xE067 => Some(KeyCode::BrowserRefresh),
        0xE065 => Some(KeyCode::BrowserSearch),
        0xE068 => Some(KeyCode::BrowserStop),
        0xE06B => Some(KeyCode::LaunchApp1),
        0xE021 => Some(KeyCode::LaunchApp2),
        0xE06C => Some(KeyCode::LaunchMail),
        0xE022 => Some(KeyCode::MediaPlayPause),
        0xE06D => Some(KeyCode::MediaSelect),
        0xE024 => Some(KeyCode::MediaStop),
        0xE019 => Some(KeyCode::MediaTrackNext),
        0xE010 => Some(KeyCode::MediaTrackPrevious),
        0xE05E => Some(KeyCode::Power),
        0xE02E => Some(KeyCode::AudioVolumeDown),
        0xE020 => Some(KeyCode::AudioVolumeMute),
        0xE030 => Some(KeyCode::AudioVolumeUp),
        _ => None,
    }
}
