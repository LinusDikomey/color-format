use std::fmt::Display;

use color_format::*;

fn main() {
    println!("Formatted: {:?}", cformat!("This text has colors like #green<green>, #red<red> and #blue<blue>!"));
    cprint!("#blue<He>#red<llo> ");
    cprintln!("#green<World>");
    ceprint!("StdError ... ");
    ceprintln!("#red<line>");
    println!("{}", CustomDisplay(5));
}

struct CustomDisplay(u32);
impl Display for CustomDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        cwrite!(f, "#green<Custom> display ... ")?;
        cwriteln!(f, "with value #blue<{}>", self.0)
    }
}