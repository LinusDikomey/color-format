use proc_macro::{TokenStream, Span};
use syn::{
    parse_macro_input, LitStr, Token, punctuated::Punctuated, Expr, token::Comma, Ident,
    parse::{Parse, ParseStream, Nothing},
};
use quote::quote;

mod convert;
mod codes;

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
                let fmt_args = if !input.is_empty() {
                    input.parse::<Token![,]>()?;
                    Punctuated::parse_terminated(input)?
                } else {
                    Punctuated::new()
                };
                Some((f_punc, fmt_str, fmt_args))
            }
        } else { None };
        Ok(Self { f, fmt })
    }
}
impl<F, P> LnArgs<F, P> {
    fn convert(self, name: &str) -> TokenStream {
        if let Some((_, fmt_str, fmt_args)) = self.fmt {
            convert::colored_macro(None, fmt_str, fmt_args, name)
        } else {
            let ident = Ident::new(name, Span::call_site().into());
            quote!{ #ident!() }.into()
        }
    }
}
impl<P> LnArgs<Expr, P> {
    fn convert_with_f(self, name: &str) -> TokenStream {
        if let Some((_, fmt_str, fmt_args)) = self.fmt {
            convert::colored_macro(Some(self.f), fmt_str, fmt_args, name)
        } else {
            let ident = Ident::new(name, Span::call_site().into());
            let f = self.f;
            quote!{ #ident!(#f) }.into()
        }
    }
}

type ColorArgs = Args<Nothing, Nothing>;
type ColorLnArgs = LnArgs<Nothing, Nothing>;

type WriteArgs = Args<Expr, Comma>;
type WriteLnArgs = LnArgs<Expr, Comma>;

#[proc_macro]
pub fn cformat(item: TokenStream) -> TokenStream {
    let args: ColorArgs = parse_macro_input!(item);
    convert::colored_macro(None, args.fmt_str, args.fmt_args, "format")
}

#[proc_macro]
pub fn cprint(item: TokenStream) -> TokenStream {
    let args: ColorArgs = parse_macro_input!(item);
    convert::colored_macro(None, args.fmt_str, args.fmt_args, "print")
}

#[proc_macro]
pub fn cprintln(item: TokenStream) -> TokenStream {
    let args: ColorLnArgs = parse_macro_input!(item);
    args.convert("println")
}

#[proc_macro]
pub fn ceprint(item: TokenStream) -> TokenStream {
    let args: ColorArgs = parse_macro_input!(item);
    convert::colored_macro(None, args.fmt_str, args.fmt_args, "eprint")
}

#[proc_macro]
pub fn ceprintln(item: TokenStream) -> TokenStream {
    if item.is_empty() {
        quote!{ eprintln!() }.into()
    } else {
        let args: ColorLnArgs = parse_macro_input!(item);
        args.convert("eprintln")
    }
}

#[proc_macro]
pub fn cwrite(item: TokenStream) -> TokenStream {
    let args: WriteArgs = parse_macro_input!(item);
    convert::colored_macro(Some(args.f), args.fmt_str, args.fmt_args, "write")
}

#[proc_macro]
pub fn cwriteln(item: TokenStream) -> TokenStream {
    let args: WriteLnArgs = parse_macro_input!(item);
    args.convert_with_f("writeln")
}