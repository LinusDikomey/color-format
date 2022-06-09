use crate::{codes::{Color, self, Code, add_ansi_code}, parse::{StringParser, StringPart, Cmd}};
use proc_macro::Span;
use quote::quote;
use syn::{Ident, Expr, LitStr, punctuated::Punctuated, Token};


pub(crate) fn colored_macro(f: Option<Expr>, fmt: LitStr, args: Punctuated<Expr, Token![,]>, emitted_macro: &str)
-> proc_macro::TokenStream {
    let (fmt_str, unformatted_str) = colored_fmt_string(&fmt.value());
    let macro_ident = Ident::new(emitted_macro, Span::call_site().into());
    let fmt_args = args.iter();
    let fmt_args2 = args.iter();
    let f = f.map_or_else(|| quote!{}, |f| quote!{ #f, });
    #[cfg(feature = "runtime_color")]
    {
        quote! {
            if ::color_format::config::CONFIG.colorize() {
                #macro_ident!(#f #fmt_str, #(#fmt_args),*)
            } else {
                #macro_ident!(#f #unformatted_str, #(#fmt_args2),*)
            }
        }.into()
    }
    #[cfg(not(feature = "runtime_color"))]
    { quote! { #macro_ident!(#f #fmt_str, #(#fmt_args),*) }.into() }
}

macro_rules! diff {
    ($($member: ident: $t: ty),*) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        struct State {
            $( $member: $t ),*
        }
        impl State {
            fn diff(&self, other: &Self) -> StateDiff {
                StateDiff {
                    $(
                        $member: (self.$member != other.$member).then(|| self.$member)
                    ),*
                } 
            }
        }
        #[derive(Debug)]
        struct StateDiff {
            $( $member: Option<$t> ),*
        }
        impl StateDiff {
            fn diff_count(&self) -> usize {
                let mut c = 0;
                $( if self.$member.is_some() { c += 1; } )*
                c
            }
        }
    };
}
diff! {
    fg_color: Color,
    bg_color: Color,
    boldness: Boldness,
    italic: bool,
    underline: bool,
    blink: bool,
    reverse: bool,
    conceal: bool,
    strike: bool
}
impl State {
    pub fn is_default(&self) -> bool {
        *self == State::default()
    }
}
impl StateDiff {
    fn apply(&self, s: &mut String) {
        macro_rules! toggle {
            ($member: ident, $t: ident, $f: ident) => {
                if let Some(v) = self.$member {
                    add_ansi_code(s, [if v { Code::$t } else { Code::$f } as u8]);
                }
            };
        }
        if let Some(fg_color) = self.fg_color { fg_color.ansi(s, false) }
        if let Some(bg_color) = self.bg_color { bg_color.ansi(s, true) }
        if let Some(boldness) = self.boldness { boldness.ansi(s) }
        toggle!(italic, Italic, NoItalic);
        toggle!(underline, Underline, NoUnderline);
        toggle!(blink, Blink, NoBlink);
        toggle!(reverse, Reverse, NoReverse);
        toggle!(conceal, Conceal, NoConceal);
        toggle!(strike, Strike, NoStrike);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Boldness { Normal, Bold, Faint }
impl Boldness {
    fn ansi(self, s: &mut String) {
        codes::add_ansi_code(s, [match self {
            Boldness::Normal => Code::NoBoldness,
            Boldness::Bold => Code::Bold,
            Boldness::Faint => Code::Faint,
        } as u8]);
    }
}
impl Default for Boldness {
    fn default() -> Self { Self::Normal }
}

/// Takes in a format string literal possibly containing color escapes like #green { ... }
/// and converts them to a string with ansi escapes. Also returns a String with all escapes just taken out.
fn colored_fmt_string(s: &str) -> (String, String) {
    let parser = StringParser::new(s);
    let mut out_str = String::new();
    let mut unformatted = String::new();
    let mut states = Vec::new();
    let mut applied_state = State::default();
    let mut state = State::default();
    for item in parser {
        match item {
            StringPart::String(s) => {
                if state.is_default() && !applied_state.is_default() {
                    add_ansi_code(&mut out_str, [Code::Reset as u8]);
                } else {
                    let diff = state.diff(&applied_state);
                    diff.apply(&mut out_str);
                }
                applied_state = state;
                out_str.push_str(s);
                unformatted.push_str(s);
            }
            StringPart::StartCmd(cmds) => {
                states.push(state);
                for cmd in cmds {
                    match cmd {
                        Cmd::Color { color, background: false } => state.fg_color = color,
                        Cmd::Color { color, background: true }  => state.bg_color = color,
                        Cmd::Bold => state.boldness = Boldness::Bold,
                        Cmd::Faint => state.boldness = Boldness::Faint,
                        Cmd::Underline => state.underline = true,
                        Cmd::Strike => state.strike = true,
                        Cmd::Reverse => state.reverse = true,
                        Cmd::Conceal => state.conceal = true,
                        Cmd::Italic => state.italic = true,
                        Cmd::Blink => state.blink = true,
                        
                    }
                }
            }
            StringPart::EndCmd => {
                if let Some(prev) = states.pop() {
                    state = prev;
                } else {
                    panic!("Mismatched closing '}}'");
                }
            }
        }
    }
    let end_diff = state.diff(&applied_state);
    if end_diff.diff_count() != 0 {
        add_ansi_code(&mut out_str, [Code::Reset as u8]);
    }
    if !states.is_empty() {
        panic!("Not all opening '<' were closed!");
    }
    assert!(state == State::default());
    (out_str, unformatted)
}
