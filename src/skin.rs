use termimad::crossterm::style::Attribute::{Bold, Underlined};
use termimad::crossterm::style::Color::Yellow;
use termimad::{rgb, MadSkin};

pub fn get_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(Yellow);
    skin.italic.add_attr(Underlined);
    skin.italic.add_attr(Bold);
    skin.italic.set_fg(rgb(32, 157, 224));
    return skin;
}
