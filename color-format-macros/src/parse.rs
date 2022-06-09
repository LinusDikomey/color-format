use std::{str::CharIndices, iter::Peekable};

use crate::codes::{Color, BasicColor, BaseColor, Intensity, RgbColor};


#[derive(Debug)]
pub enum Cmd {
    Color { color: Color, background: bool },
    Bold,
    Faint,
    Italic,
    Underline,
    Blink,
    Reverse,
    Conceal,
    Strike,

}
struct CmdParser<'a> {
    s: &'a str,
}
impl<'a> CmdParser<'a> {
    pub fn parse_commands(s: &'a str) -> Vec<Cmd> {
        s.split(';').map(str::trim).map(|s| Self { s }.parse_command()).collect()
    }

    fn skip_if(&mut self, prefix: &str) -> bool {
        if self.s.starts_with(prefix) {
            self.s = &self.s[prefix.len() ..];
            true
        } else { false }
    }
    fn skip_if_any<'s>(&mut self, prefixes: impl IntoIterator<Item = &'s str>) -> bool {
        prefixes.into_iter().any(|s| self.skip_if(s))
    }
    fn get_base_color(s: &str) -> Option<BaseColor> {
        Some(match s {
            "k" | "black" => BaseColor::Black,
            "r" | "red" => BaseColor::Red,
            "g" | "green" => BaseColor::Green,
            "y" | "yellow" => BaseColor::Yellow,
            "b" | "blue" => BaseColor::Blue,
            "m" | "magenta" => BaseColor::Magenta,
            "c" | "cyan" => BaseColor::Cyan,
            "w" | "white" => BaseColor::White,
            _ => return None
        })
    }

    fn parse_command(&mut self) -> Cmd {
        let background = self.skip_if_any(["_", "bg:"]);
        let intensity = if self.skip_if("bright-") { Intensity::Bright }
        else if self.s.ends_with('!') {
            self.s = &self.s[..self.s.len() - 1];
            Intensity::Bright
        } else { Intensity::Normal };
        match Self::get_base_color(self.s) {
            Some(base) => Cmd::Color {
                color: Color::Basic(BasicColor::new(base, intensity)),
                background
            },
            None => {
                if intensity == Intensity::Bright {
                    panic!("Invalid tag, 'bright-'/'!' can only be applied to basic colors and \
                        '{}' is not one of them", self.s);
                }
                if self.skip_if("rgb(") {
                    let end = self.s.find(')').expect("Missing ')' in rgb tag");
                    let args = &self.s[..end];
                    let mut split = args.split(',');
                    let msg = "Not enough arguments in rgb tag";
                    let parse_msg = "Invalid component in rgb tag, value from 0 to 255 expected";
                    let r = split.next().expect(msg).parse().expect(parse_msg);
                    let g = split.next().expect(msg).parse().expect(parse_msg);
                    let b = split.next().expect(msg).parse().expect(parse_msg);
                    if split.next().is_some() {
                        panic!("Too many components in an rgb tag. 3 components were expected");
                    }
                    return Cmd::Color { color: Color::Rgb(RgbColor { r, g, b }), background };
                }
                if background {
                    panic!("Invalid tag, 'bg:'/'_' can only be applied to basic colors and \
                        '{}' is not one of them", self.s);
                }
                match self.s {
                    "bold" | "s" => Cmd::Bold,
                    "faint" | "f" => Cmd::Faint,
                    "italic" | "i" => Cmd::Italic,
                    "underline" | "u" => Cmd::Underline,
                    "blink" => Cmd::Blink,
                    "reverse" => Cmd::Reverse,
                    "conceal" => Cmd::Conceal,
                    "strike" => Cmd::Strike,
                    other => panic!("Unknown tag '{}'", other)
                }
            }
        }
        
    }
}

pub enum StringPart<'a> {
    String(&'a str),
    StartCmd(Vec<Cmd>),
    EndCmd,
}

pub struct StringParser<'a> {
    s: &'a str,
    chars: Peekable<CharIndices<'a>>,
}
impl<'a> StringParser<'a> {
    pub fn new(s: &'a str) -> Self {
        Self { s, chars: s.char_indices().peekable() }
    }
    fn skip_whitespace(&mut self) {
        while let Some((_, c)) = self.chars.peek() {
            if !c.is_whitespace() { break }
            self.chars.next();
        }
    }
}
impl<'a> Iterator for StringParser<'a> {
    type Item = StringPart<'a>;
    /// returns start index and the command
    fn next(&mut self) -> Option<StringPart<'a>> {
        match self.chars.next() {
            Some((_, '#')) => {
                match self.chars.peek() {
                    Some((_, '#')) => {
                        self.chars.next().unwrap();
                        return Some(StringPart::String("#"));
                    }
                    Some((_, '<')) => {
                        self.chars.next().unwrap();
                        return Some(StringPart::String("<"));
                    }
                    Some((_, '>')) => {
                        self.chars.next().unwrap();
                        return Some(StringPart::String(">"));
                    }
                    _ => ()
                }
                self.skip_whitespace();
                let tag_start = self.chars.peek().expect("Color tag expected").0;
                
                let tag_end = loop {
                    match self.chars.next().expect("Color tag expected") {
                        (end, '<') => break end,
                        (end, c) if c.is_whitespace() => {
                            self.skip_whitespace();
                            if !matches!(self.chars.next(), Some((_, '<'))) {
                                panic!("'<' expected after color tag");
                            }
                            break end
                        }
                        _ => ()
                    }
                };
                let tag = &self.s[tag_start..tag_end];
                Some(StringPart::StartCmd(CmdParser::parse_commands(tag)))
            }
            Some((_, '>')) => Some(StringPart::EndCmd),
            Some((str_start, _)) => {
                Some(StringPart::String(loop {
                    match self.chars.peek() {
                        Some((end, '#' | '<' | '>')) => {
                            break &self.s[str_start..*end];
                        }
                        None => break &self.s[str_start..],
                        _ => {
                            self.chars.next().unwrap();
                        }
                    }
                }))
            }
            None => None
        }
        
    }
}
