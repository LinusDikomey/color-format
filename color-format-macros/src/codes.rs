
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseColor {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intensity { Normal, Bright }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasicColor { pub base: BaseColor, pub intensity: Intensity }
impl BasicColor {
    pub fn new(base: BaseColor, intensity: Intensity) -> Self {
        Self { base, intensity }
    }
    pub fn ansi(self, background: bool) -> u8 {
        let offset = match (self.intensity, background) {
            (Intensity::Normal, false) => Code::SetForegroundBase,
            (Intensity::Normal, true) => Code::SetBackgroundBase,
            (Intensity::Bright, false) => Code::SetBrightForegroundBase,
            (Intensity::Bright, true) => Code::SetBrightBackgroundBase,
        } as u8;
        offset + self.base as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Normal,
    Basic(BasicColor),
    Rgb(RgbColor)
}
impl Color {
    pub fn ansi(self, s: &mut String, background: bool) {
        match self {
            Color::Normal => add_ansi_code(s, [
                if background { Code::DefaultBackground } else { Code::DefaultForeground } as u8
            ]),
            Color::Basic(basic) => add_ansi_code(s, [basic.ansi(background)]),
            Color::Rgb(RgbColor { r, g, b }) => add_ansi_code(s, [
                if background { Code::SetBackground } else { Code::SetForeground } as u8,
                2, r, g, b
            ]),
        }
    }
}
impl Default for Color {
    fn default() -> Self { Self::Normal }
}

// pub const RESET: u8 = 0;
#[repr(u8)]
pub enum Code {
    Reset = 0,
    Bold = 1,
    Faint = 2,
    Italic = 3,
    Underline = 4,
    Blink = 5,
    Reverse = 7,
    Conceal = 8,
    Strike = 9,
    NoBoldness = 22,
    NoItalic = 23,
    NoUnderline = 24,
    NoBlink = 25,
    NoReverse = 27,
    NoConceal = 28,
    NoStrike = 29,
    SetForegroundBase = 30,
    SetForeground = 38,
    DefaultForeground = 39,
    SetBackgroundBase = 40,
    SetBackground = 48,
    DefaultBackground = 49,
    SetBrightForegroundBase = 90,
    SetBrightBackgroundBase = 100,
}
pub fn add_ansi_code(s: &mut String, params: impl IntoIterator<Item = u8>) {
    s.push_str("\u{1b}[");
    let mut params = params.into_iter();
    if let Some(first) = params.next() {
        s.push_str(&format!("{}", first));
    }
    for param in params {
        s.push(';');
        s.push_str(&format!("{}", param));
    }
    s.push('m');  
}