use colored::{Color, Colorize};

fn colour_lines(txt: &str, colours: &[Color]) -> String {
    let lines = txt.split('\n');
    let len = colours.len();
    let lines: Vec<_> = lines
        .into_iter()
        .enumerate()
        .map(|(i, t)| t.bold().color(colours[i % len]).to_string())
        .collect();

    lines.join("\n")
}

fn rainbow_lines(txt: &str, ) -> String {
    let mut rainbow = [
        Color::TrueColor {
            r: 148,
            g: 0,
            b: 211,
        },
        Color::TrueColor { r: 75, g: 0, b: 30 },
        Color::TrueColor { r: 0, g: 0, b: 255 },
        Color::TrueColor { r: 0, g: 255, b: 0 },
        Color::TrueColor {
            r: 255,
            g: 255,
            b: 0,
        },
        Color::TrueColor {
            r: 255,
            g: 127,
            b: 0,
        },
        Color::TrueColor {
            r: 255,
            g: 127,
            b: 127,
        },
    ];
    rainbow.reverse();
    colour_lines(txt, &rainbow)
}

////////////////////////////////////////////////////////////////////////////////
pub fn get_styles() -> clap::builder::Styles {
    use clap::builder::styling::{AnsiColor::*, Effects, Styles};

    Styles::styled()
        .header(Green.on_default() | Effects::BOLD)
        .usage(Green.on_default() | Effects::BOLD)
        .literal(Blue.on_default() | Effects::BOLD)
        .placeholder(Cyan.on_default())
}
////////////////////////////////////////////////////////////////////////////////
pub fn get_banner() -> String {
    let banner =  r"
  __ _  __ _ _____ __ ___
 / _` |/ _` |_  / '_ ` _ \
| (_| | (_| |/ /| | | | | |
 \__, |\__,_/___|_| |_| |_|
 |___/ Assembler (currently only 6809)";

    rainbow_lines(banner)
}

