#![allow(clippy::vec_init_then_push)]
#![allow(clippy::too_many_arguments)]
use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    name: String,
    foreground: Color,
    background: Color,
    main_text: Color,
    sub_text: Color,
    highlight_text: Color,
    shade1: Color,
    shade2: Color,
    shade3: Color,
    process: Color,
}

impl Theme {
    pub fn new(
        name: &str,
        foreground: Color,
        background: Color,
        main_text: Color,
        sub_text: Color,
        highlight_text: Color,
        shade1: Color,
        shade2: Color,
        shade3: Color,
        process: Color,
    ) -> Self {
        Theme {
            name: name.to_string(),
            foreground,
            background,
            main_text,
            sub_text,
            highlight_text,
            shade1,
            shade2,
            shade3,
            process,
        }
    }

    pub fn init_themes() -> Vec<Theme> {
        let mut themes = Vec::new();

        themes.push(Theme::new(
            "Nord",
            Color::Rgb(136, 192, 208),
            Color::Rgb(46, 52, 64),
            Color::Rgb(229, 233, 240),
            Color::Rgb(143, 188, 187),
            Color::Rgb(129, 161, 193),
            Color::Rgb(59, 66, 82),
            Color::Rgb(67, 76, 94),
            Color::Rgb(76, 86, 103),
            Color::Rgb(180, 142, 173),
        ));

        themes.push(Theme::new(
            "Catppuccin",
            Color::Rgb(180, 190, 254),
            Color::Rgb(30, 30, 46),
            Color::Rgb(137, 180, 250),
            Color::Rgb(203, 166, 147),
            Color::Rgb(250, 179, 135),
            Color::Rgb(46, 50, 68),
            Color::Rgb(69, 71, 90),
            Color::Rgb(88, 91, 112),
            Color::Rgb(243, 139, 168),
        ));
        themes
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_fg(&self) -> Color {
        self.foreground
    }

    pub fn get_bg(&self) -> Color {
        self.background
    }

    pub fn get_mt(&self) -> Color {
        self.main_text
    }

    pub fn get_st(&self) -> Color {
        self.sub_text
    }

    pub fn get_ht(&self) -> Color {
        self.highlight_text
    }

    pub fn get_s1(&self) -> Color {
        self.shade1
    }

    pub fn get_s2(&self) -> Color {
        self.shade2
    }

    pub fn get_s3(&self) -> Color {
        self.shade3
    }

    pub fn get_pr(&self) -> Color {
        self.process
    }
}
