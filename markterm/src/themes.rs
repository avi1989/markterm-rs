/// A module to assist with setting colors.
pub mod color;

use color::Color;

/// Indicates whether the text is bold, underlined, italics or strikethrough
#[derive(PartialEq, Default)]
pub enum TextStyle {
    /// Indicates normal text.
    #[default]
    Normal,

    /// Indicates **bold text**.
    Bold,

    /// Indicates *italicised text*.
    Italics,

    /// Indicates <ins>underlined text</ins>.
    Underlined,

    /// Indicates ~~Strikethrough text~~.
    Strikethrough,
}

impl TextStyle {
    /// Gets the console style key for the text style without escape characters.
    pub fn style_key(&self) -> &str {
        match self {
            TextStyle::Normal => "",
            TextStyle::Bold => "1",
            TextStyle::Italics => "3",
            TextStyle::Underlined => "4",
            TextStyle::Strikethrough => "9",
        }
    }
}

/// Properties required to theme the element.
pub struct ElementTheme {
    /// Foreground color. i.e text color
    pub fg: Option<Color>,

    /// Background color
    pub bg: Option<Color>,

    /// Indicates the text style.
    pub style: TextStyle,
}

/// A top level struct that contains all the elements and their styles.
pub struct Theme {
    /// The theme for header 1 elements.
    ///
    /// Header elements start with #)
    pub header_1: ElementTheme,

    /// The theme for all header 2, 3 and 4
    pub header_x: ElementTheme,

    /// The theme for code blocks.
    /// Code blocks are elements that are surrounded by ``
    pub code_block: ElementTheme,

    /// The theme for indentations
    /// Indent Elements start with >
    pub indents: ElementTheme,

    /// The theme for links.
    /// Links are surrounded by < >
    pub link: ElementTheme,

    /// The theme for lists
    pub list: ElementTheme,

    /// The theme for bold text.
    pub strong: ElementTheme,

    /// The theme for emphasis.
    pub emphasis: ElementTheme,

    /// The theme for strikethroughs
    pub delete: ElementTheme,
}

const T_ESC: &str = "\u{1b}";
const T_FG: &str = "38";
const T_BG: &str = "48";

impl ElementTheme {
    /// Creates a new instance of ElementTheme.
    ///
    /// Example
    /// ```rust
    /// use markterm::{ElementTheme,TextStyle};
    /// let a = ElementTheme::new(Some("#000"), Some("#FFF"), TextStyle::Normal);
    /// ```
    pub fn new(fg: Option<&str>, bg: Option<&str>, style: TextStyle) -> Self {
        let bg_color = bg.map(Color::new);
        let fg_color = fg.map(Color::new);

        Self {
            fg: fg_color,
            bg: bg_color,
            style,
        }
    }

    /// Writes the current theme into the terminal
    /// ### Usage
    /// ```rust
    /// use std::io::Write;
    /// use markterm::{TextStyle, ElementTheme};
    ///
    /// let element_theme = ElementTheme::new(Some("#CCC"), Some("#000"), TextStyle::Normal);
    /// element_theme.write(|w| write!(w, "Hello"), &mut std::io::stdout());
    /// ```
    pub fn write<F, T>(&self, write_text: F, writer: &mut T) -> Result<(), std::io::Error>
    where
        // F: FnOnce() -> Result<(), std::io::Error>,
        F: Fn(&mut T) -> Result<(), std::io::Error>,
        T: std::io::Write,
    {
        let style_key = match self.style {
            TextStyle::Normal => "".to_string(),
            _ => format!("{};", self.style.style_key()),
        };
        match (&self.fg, &self.bg) {
            (Some(fg), Some(bg)) => {
                write!(
                    writer,
                    "{T_ESC}[{style_key}{T_BG};2;{};{T_FG};2;{}m",
                    bg.rgb(),
                    fg.rgb()
                )?;
            }
            (Some(fg), None) => {
                write!(writer, "{T_ESC}[{style_key}{T_FG};2;{}m", fg.rgb())?;
            }
            (None, Some(bg)) => {
                write!(writer, "{T_ESC}[{style_key}{T_BG};2;{}m", bg.rgb())?;
            }
            (None, None) => {
                if self.style != TextStyle::Normal {
                    write!(writer, "{T_ESC}[{}m", self.style.style_key())?;
                }
            }
        };

        write_text(writer)?;

        match self {
            ElementTheme {
                fg: None,
                bg: None,
                style: TextStyle::Normal,
            } => {
                write!(writer, "")
            }
            _ => write!(writer, "{T_ESC}[0m"),
        }
    }
}

///Gets the default dark theme
pub fn get_dark_theme() -> Theme {
    Theme {
        header_1: ElementTheme::new(None, Some("#6155FB"), TextStyle::Normal),
        header_x: ElementTheme::new(Some("#01AFFD"), None, TextStyle::Normal),
        code_block: ElementTheme::new(Some("#FF6060"), Some("#303030"), TextStyle::Normal),
        indents: ElementTheme::new(Some("#555"), None, TextStyle::Normal),
        link: ElementTheme::new(Some("#008787"), None, TextStyle::Underlined),
        list: ElementTheme::new(None, None, TextStyle::Normal),
        strong: ElementTheme::new(None, None, TextStyle::Bold),
        emphasis: ElementTheme::new(None, None, TextStyle::Italics),
        delete: ElementTheme::new(None, None, TextStyle::Strikethrough),
    }
}

///Gets the default light theme
pub fn get_light_theme() -> Theme {
    Theme {
        header_1: ElementTheme::new(Some("#FFF"), Some("#6155FB"), TextStyle::Normal),
        header_x: ElementTheme::new(Some("#01AFFD"), None, TextStyle::Normal),
        code_block: ElementTheme::new(Some("#EA3323"), Some("#E4E4E4"), TextStyle::Normal),
        indents: ElementTheme::new(None, None, TextStyle::Normal),
        link: ElementTheme::new(Some("#5CBC9A"), None, TextStyle::Underlined),
        list: ElementTheme::new(None, None, TextStyle::Normal),
        strong: ElementTheme::new(None, None, TextStyle::Bold),
        emphasis: ElementTheme::new(None, None, TextStyle::Italics),
        delete: ElementTheme::new(None, None, TextStyle::Strikethrough),
    }
}

/// Gets the default theme. The default theme is based on whether the terminal
/// has a dark background or a light background.
pub fn get_default_theme() -> Theme {
    let theme = get_terminal_theme().unwrap_or(termbg::Theme::Dark);

    match theme {
        termbg::Theme::Light => get_light_theme(),
        _ => get_dark_theme(),
    }
}

fn get_terminal_theme() -> Option<termbg::Theme> {
    let timeout = std::time::Duration::from_millis(500);

    let result = std::panic::catch_unwind(|| termbg::theme(timeout));

    match result {
        Ok(theme) => match theme {
            Ok(theme) => Some(theme),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

#[cfg(test)]
mod test {
    mod write {
        use super::super::*;
        use crate::ElementTheme;
        use colored::Colorize;
        use colored::CustomColor;
        use std::io::Write;

        macro_rules! should_equal {
            ($($name:ident: $value:expr,)*) => {
                $(
                    #[test]
                    fn $name() {
                        let (value, fg, bg, style, expected) = $value;
                        let theme = ElementTheme::new(fg, bg, style);

                        let mut writer = Vec::new();

                        theme.write(|w| write!(w, "{}", value), &mut writer).unwrap();
                        let text = std::str::from_utf8(&writer).unwrap();
                        let expected = format!("{}", expected);
                        assert_eq!(expected, text);
                    }
                )*
            }
        }

        should_equal! {
            should_write_plain_text: ("Hello", None, None, TextStyle::Normal, "Hello".normal()),
            should_write_fg: ("Hello", Some("#F52"), None, TextStyle::Normal, "Hello".custom_color(CustomColor::new(255, 85, 34))),
            should_write_bg: ("Hello", None, Some("#54FD10"), TextStyle::Normal, "Hello".on_custom_color(CustomColor::new(84, 253, 16))),
            should_write_fg_and_bg: ("Hello", Some("#F52"), Some("54FD10"), TextStyle::Normal, "Hello".custom_color(CustomColor::new(255, 85, 34)).on_custom_color(CustomColor::new(84, 253, 16))),
            should_write_bold_text: ("Hello", None, None, TextStyle::Bold, "Hello".bold()),
            should_write_italics_text: ("Hello", None, None, TextStyle::Italics, "Hello".italic()),
            should_write_underlined_text: ("Hello", None, None, TextStyle::Underlined, "Hello".underline()),
            should_write_strikethrough_text: ("Hello", None, None, TextStyle::Strikethrough, "Hello".strikethrough()),
        }
    }
}
