use termimad::crossterm::style::Color::*;
use termimad::*;

/// Terminmad method
pub fn render_markdown(text: &str) {
    let mut skin = MadSkin::default();

    skin.set_headers_fg(Yellow);
    skin.bold.set_fg(Cyan);
    skin.italic.set_fg(Magenta);
    skin.inline_code.set_fgbg(Green, AnsiValue(236));
    skin.code_block.set_fgbg(White, AnsiValue(235));
    skin.table.align = Alignment::Left;

    // Create area
    let mut area = Area::full_screen();
    area.pad_for_max_width(100);

    let formatted_text = skin.area_text(text, &area);
    print!("{}", formatted_text);
}
