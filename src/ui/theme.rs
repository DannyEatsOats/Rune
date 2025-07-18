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
            Color::Rgb(136, 192, 208), //ForeGround
            Color::Rgb(46, 52, 64),    //BackGround
            Color::Rgb(229, 233, 240), //MainText
            Color::Rgb(143, 188, 187), //SubText
            Color::Rgb(129, 161, 193), //Highlight
            Color::Rgb(59, 66, 82),    //Shade1
            Color::Rgb(67, 76, 94),    //Shade2
            Color::Rgb(76, 86, 103),   //Shade3
            Color::Rgb(180, 142, 173), //Process
        ));

        themes.push(Theme::new(
            "Catppuccin",
            Color::Rgb(180, 190, 254), //ForeGround
            Color::Rgb(30, 30, 46),    //BackGround
            Color::Rgb(137, 180, 250), //MainText
            Color::Rgb(203, 166, 147), //SubText
            Color::Rgb(250, 179, 135), //Highlight
            Color::Rgb(46, 50, 68),    //Shade1
            Color::Rgb(69, 71, 90),    //Shade2
            Color::Rgb(88, 91, 112),   //Shade3
            Color::Rgb(243, 139, 168), //Process
        ));

        themes.push(Theme::new(
            "Rosé-Pine",
            Color::Rgb(234, 154, 151), //Highlight
            Color::Rgb(53, 33, 54),    //BackGround
            Color::Rgb(224, 222, 244), //MainText
            Color::Rgb(196, 167, 231), //SubText
            Color::Rgb(235, 111, 146), //Process
            Color::Rgb(110, 106, 134), //Shade1
            Color::Rgb(68, 65, 90),    //Shade2
            Color::Rgb(88, 91, 112),   //Shade3
            Color::Rgb(156, 207, 216), //ForeGround
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
