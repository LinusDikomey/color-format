use std::{str::{CharIndices, FromStr}, iter::Peekable};
use crate::codes::{Color, BaseColor, BasicColor};
use proc_macro::Span;
use quote::quote;
use syn::{Ident, Expr, LitStr, punctuated::Punctuated, Token};


pub(crate) fn colored_macro(f: Option<Expr>, fmt: LitStr, args: Punctuated<Expr, Token![,]>, emitted_macro: &str)
-> proc_macro::TokenStream {
    let (fmt_str, unformatted_str) = colored_fmt_string(fmt.value());
    let macro_ident = Ident::new(emitted_macro, Span::call_site().into());
    let fmt_args = args.iter();
    let fmt_args2 = args.iter();
    let f = f.map_or_else(|| quote!{}, |f| quote!{ #f, });
    quote! {
        if ::color_format::config::CONFIG.colorize() {
            #macro_ident!(#f #fmt_str, #(#fmt_args),*)
        } else {
            #macro_ident!(#f #unformatted_str, #(#fmt_args2),*)
        }
    }.into()
}

#[derive(Debug)]
enum Cmd {
    Color(Color)
}
impl FromStr for Cmd {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "red" => Cmd::Color(Color::Basic(BasicColor::normal(BaseColor::Red))),
            "green" => Cmd::Color(Color::Basic(BasicColor::normal(BaseColor::Green))),
            "blue" => Cmd::Color(Color::Basic(BasicColor::normal(BaseColor::Blue))),
            _ => return Err(())
        })
    }
}

struct StringParser<'a> {
    s: &'a str,
    chars: Peekable<CharIndices<'a>>,
}
impl<'a> StringParser<'a> {
    fn skip_whitespace(&mut self) {
        while let Some((_, ' ' | '\t' | '\n' | '\r')) = self.chars.peek() {
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
                        (end, ' ' | '\t' | '\n' | '\r') => {
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
                Some(StringPart::StartCmd(tag.parse()
                    .unwrap_or_else(|_| panic!("Unknown tag '{}'", tag))))
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
enum StringPart<'a> {
    String(&'a str),
    StartCmd(Cmd),
    EndCmd,
}

#[derive(Debug, Clone, Copy)]
struct State {
    fg_color: Color,
    bg_color: Color,
}
impl State {
    fn diff(&self, other: &Self) -> StateDiff {
        macro_rules! diff {
            ($($member: ident)*) => {
                StateDiff {
                    $(
                        $member: (self.$member != other.$member).then(|| self.$member)
                    ),*
                } 
            };
        }
        diff!(fg_color bg_color)
    }
}
impl Default for State {
    fn default() -> Self {
        Self {
            fg_color: Color::Normal,
            bg_color: Color::Normal,
        }
    }
}
#[derive(Default, Debug, Clone, Copy)]
struct StateDiff {
    fg_color: Option<Color>,
    bg_color: Option<Color>,
}
impl StateDiff {
    fn apply(&self, s: &mut String) {
        if let Some(fg_color) = self.fg_color { fg_color.ansi(s, false) }
        if let Some(bg_color) = self.bg_color { bg_color.ansi(s, true) }
    }
}

/// Takes in a format string literal possibly containing color escapes like #green { ... }
/// and converts them to a string with ansi escapes. Also returns a String with all escapes just taken out.
fn colored_fmt_string(s: String) -> (String, String) {
    let parser = StringParser { s: &s, chars: s.char_indices().peekable() };
    let mut out_str = String::new();
    let mut unformatted = String::new();
    let mut states = Vec::new();
    let mut applied_state = State::default();
    let mut unapplied_state = State::default();
    for item in parser {
        match item {
            StringPart::String(s) => {
                let diff = unapplied_state.diff(&applied_state);
                diff.apply(&mut out_str);
                applied_state = unapplied_state;
                out_str.push_str(s);
                unformatted.push_str(s);
            }
            StringPart::StartCmd(cmd) => {
                states.push(unapplied_state.clone());
                match cmd {
                    Cmd::Color(color) => unapplied_state.fg_color = color,
                }
            }
            StringPart::EndCmd => {
                if let Some(prev) = states.pop() {
                    unapplied_state = prev;
                } else {
                    panic!("Mismatched closing '}}'");
                }
            }
        }
    }
    if !states.is_empty() {
        panic!("Not all opening '<' were closed!");
    }
    (out_str, unformatted)
}
