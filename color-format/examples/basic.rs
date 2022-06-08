use color_format::*;


fn main() {
    let x = 2;
    // Use ##, #< and #> to escape #, < and >
    let message = cformat!("This is a #red<secret> ##message with a number attached: {}", x);
    // Formatting tags can be recursive and will be handled and optimized correctly.
    cprintln!("#blue<Here is #green<some> text>: {}", message);
}
