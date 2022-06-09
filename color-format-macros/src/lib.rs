use proc_macro::{TokenStream, Span};
use syn::{
    parse_macro_input, LitStr, Token, punctuated::Punctuated, Expr, token::Comma, Ident,
    parse::{Parse, ParseStream, Nothing},
};
use quote::quote;

mod convert;
mod codes;
mod parse;

struct Args<F, P> {
    f: F,
    _f_punc: P,
    fmt_str: LitStr,
    fmt_args: Punctuated<Expr, Token![,]>,
}
impl<F: Parse, P: Parse> Parse for Args<F, P> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let f = input.parse()?;
        let f_punc = input.parse()?;
        let fmt_str = input.parse()?;
        let fmt_args = if input.peek(Token![,]) {
            input.parse::<Token![,]>().unwrap();
            Punctuated::<Expr, Token![,]>::parse_terminated(input)?
        } else {
            Punctuated::new()
        };
        Ok(Self { f, _f_punc: f_punc, fmt_str, fmt_args })
        
    }
}
impl<F, P> Args<F, P> {
    fn convert(self, macro_name: &str) -> TokenStream {
        convert::colored_macro(None, self.fmt_str, self.fmt_args, macro_name)
    }
}
struct LnArgs<F, P> {
    f: F,
    fmt: Option<(P, LitStr, Punctuated<Expr, Token![,]>)>,
}
impl<F: Parse, P: Parse> Parse for LnArgs<F, P> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let f = input.parse()?;

        let fmt = if !input.is_empty() {
            let f_punc: P = input.parse()?;
            if input.is_empty() {
                None
            } else {
                let fmt_str = input.parse()?;
                let fmt_args = if input.is_empty() {
                    Punctuated::new()
                } else {
                    input.parse::<Token![,]>()?;
                    Punctuated::parse_terminated(input)?
                };
                Some((f_punc, fmt_str, fmt_args))
            }
        } else { None };
        Ok(Self { f, fmt })
    }
}
impl<F, P> LnArgs<F, P> {
    fn convert(self, macro_name: &str) -> TokenStream {
        if let Some((_, fmt_str, fmt_args)) = self.fmt {
            convert::colored_macro(None, fmt_str, fmt_args, macro_name)
        } else {
            let ident = Ident::new(macro_name, Span::call_site().into());
            quote!{ #ident!() }.into()
        }
    }
}

type ColorArgs = Args<Nothing, Nothing>;
type ColorLnArgs = LnArgs<Nothing, Nothing>;

type WriteArgs = Args<Expr, Comma>;
type WriteLnArgs = LnArgs<Expr, Comma>;

macro_rules! basic_macros {
    ($($args: ident => $($macro_name: ident: $name: ident),*);*) => {
        $(
            $(
                #[proc_macro]
                pub fn $macro_name(item: TokenStream) -> TokenStream {
                    parse_macro_input!(item as $args).convert(stringify!($name))
                }
            )*
        )*
    };
}
basic_macros!(
    ColorArgs => cformat: format, cprint: print, ceprint: eprint;
    ColorLnArgs => cprintln: println, ceprintln: eprintln
);

#[proc_macro]
pub fn cwrite(item: TokenStream) -> TokenStream {
    let args: WriteArgs = parse_macro_input!(item);
    convert::colored_macro(Some(args.f), args.fmt_str, args.fmt_args, "write")
}

#[proc_macro]
pub fn cwriteln(item: TokenStream) -> TokenStream {
    let args: WriteLnArgs = parse_macro_input!(item);
    let macro_name = "writeln";
    if let Some((_, fmt_str, fmt_args)) = args.fmt {
        convert::colored_macro(Some(args.f), fmt_str, fmt_args, macro_name)
    } else {
        let ident = Ident::new(macro_name, Span::call_site().into());
        let f = args.f;
        quote!{ #ident!(#f) }.into()
    }
}