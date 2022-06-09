use color_format::*;



#[test]
fn escapes() {
    let x = 3;
    // '#' doesn't have to be escaped inside format strings
    // '<' doesn't ever have to be escaped but can be for symmetry
    assert_eq!(
        cformat!("#> ## < #< #r<this is red>, aligned: '{:#4}'", x),
        "> # < < \u{1b}[31mthis is red\u{1b}[0m, aligned: '   3'"
    );
    assert_eq!(
        cformat!("{{ #r<___> .{:#3}.", "c"),
        "{ \u{1b}[31m___\u{1b}[0m .c  ."
    );
}

#[test]
fn recursive_tags() {
    assert_eq!(
        cformat!("uncolored, #r<red#g;u<green and underlined>,red again>, uncolored"),
        "uncolored, \u{1b}[31mred\u{1b}[32m\u{1b}[4mgreen and underlined\u{1b}[31m\u{1b}[24m,red again\u{1b}[0m, uncolored"
    );
    assert_eq!(
        cformat!("#r<r #g<'{:#3}'> r>", "ab"),
        "\u{1b}[31mr \u{1b}[32m'ab '\u{1b}[31m r\u{1b}[0m"
    );
}