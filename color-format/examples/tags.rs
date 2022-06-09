use color_format::*;

fn main() {
    cprintln!("#r<Red>, #g<green>, #b<blue> or #rgb(100,150,200)<a specific color>");
    // attention, spaces between the values of the rgb() arguments are not allowed right now
    cprintln!("#bg:red<Background color with 'bg:'> or with #_green<an underscore '_' before the color>!");
    cprintln!("Bright color variations: #bright-blue<bright blue> or #bright-b<with color shorthand> or #b!<with !>");
    cprintln!("#bold<Bold>, also #s<bold>");
    cprintln!("#faint<The inverse of bold>, also #f<with shorthand>");
    cprintln!("#italic<Italic> or alternatively #i<this>");
    cprintln!("Show that something is #underline<very> #u<very> important");
    cprintln!("If you want text to #blink<blink>");
    cprintln!("#_rgb(0,0,150)<#g<You can #reverse<reverse> the foreground and background color>>,");
    cprintln!("conceal your password: #conceal<password1234>");
    cprintln!("or correct #strike<spellnig> spelling errors");
}