#![warn(missing_docs)]

//! A cross-platform library to render colored markdown to the terminal.
//! The rendered markdown is colored and is themeable.
//!
//! The module exposes 4 functions that for handling markdown
//! * [`render_file_to_stdout`][]
//!   - Renders the passed in file to stdout using the theme.
//! * [`render_file`]
//!   - Themes and renders the passed in file to the implementation of `std::io::Write`` that is passed in.
//! * [`render_text_to_stdout`]
//!   - Renders the passed in string to stdout using the theme.
//! * [`render_text`][]
//!   - Renders the passed in string to an implementation of std::io::Write that is passed in.
//!
//! ## Status
//! This project started out as a way for me to learn rust. It's gone beyond that now.
//! At this point, markterm is not compatible with inline html and tables. It also does not support multi level indentations.
//! These features are in the works
//!
//! ## Roadmap
//! There is a lot we want to do to markterm. The items we have in our immediate queue are listed
//! below.
//! - Add support for nested lists.
//! - Add support for generic colors rather than always having to use RGB.
//! - Add support for tables.
//! - Add support for inline html.
//!
//! ## Credits
//! This project would not be possible without [markdown-rs](https://github.com/wooorm/markdown-rs).
//! Their ast parsing module powers the library.

/// Modules to help theme the output
pub mod themes;

pub use themes::{color::Color, get_default_theme, ElementTheme, TextStyle, Theme};

/// A module to write the appropriate terminal escape sequence to color the text
mod writer;

use std::io::{IsTerminal, Read};
use std::{
    fs::File,
    io::{self},
    path::PathBuf,
};

/// Indicates whether the output should be colorized or not.
#[derive(Debug, PartialEq)]
pub enum ColorChoice {
    /// Enables colored output only when the output is going to a terminal or TTY.
    Auto,

    /// Enables colored output regardless of whether or not the output is going to a terminal/TTY.
    Always,

    /// Disables colored output no matter if the output is going to a terminal/TTY, or not.
    Never,
}

/// Renders the contents of the passed in file to stdout.
///
/// ### Example
/// ```rust
/// use markterm::ColorChoice;
/// let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
/// path.push("benches/sample.md");
///
/// markterm::render_file_to_stdout(&path, None, ColorChoice::Auto);
/// ```
pub fn render_file_to_stdout(
    file_path: &PathBuf,
    theme: Option<&self::Theme>,
    color_choice: ColorChoice,
) -> Result<(), std::io::Error> {
    let mut stdout = std::io::stdout().lock();
    let should_colorize = match color_choice {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => stdout.is_terminal(),
    };

    render_file(file_path, theme, &mut stdout, should_colorize)
}

/// Renders the contents of the passed in file to any implementation of std::io::Write.
///
/// ### Example
/// ```rust
/// use std::io::Write;
///
/// let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
/// path.push("benches/sample.md");
///
/// let mut dest = Vec::new();
/// markterm::render_file(&path, None, &mut dest, false);
/// ```
pub fn render_file(
    file_path: &PathBuf,
    theme: Option<&Theme>,
    writer: &mut impl std::io::Write,
    should_colorize: bool,
) -> Result<(), std::io::Error> {
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            panic!("Unable to open file: {}", e);
        }
    };

    let mut file_contents = String::new();

    let _files = io::BufReader::new(file)
        .read_to_string(&mut file_contents)
        .unwrap();

    render_text(&file_contents, theme, writer, should_colorize)
}

/// Renders the contents of the passed in string to stdout.
///
/// ### Example
/// ```rust
/// use markterm::ColorChoice;
/// let str = "> This is a `test`";
/// markterm::render_text_to_stdout(str, None, ColorChoice::Auto);
/// ```
pub fn render_text_to_stdout(
    text: &str,
    theme: Option<&Theme>,
    color_choice: ColorChoice,
) -> Result<(), std::io::Error> {
    let mut stdout = std::io::stdout().lock();

    let should_colorize = match color_choice {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => stdout.is_terminal(),
    };

    render_text(text, theme, &mut stdout, should_colorize)
}

/// Renders the contents of the passed in string to any implementation of std::io::Write.
///
/// ### Example
/// ```rust
/// use std::io::Write;
///
/// let str = "> This is a `test`";
///
/// let mut dest = Vec::new();
/// markterm::render_text(str, None, &mut dest, true);
/// ```
pub fn render_text(
    text: &str,
    theme: Option<&Theme>,
    writer: &mut impl std::io::Write,
    should_colorize: bool,
) -> Result<(), std::io::Error> {
    let default_theme = get_default_theme();
    let theme = match theme {
        Some(x) => x,
        None => &default_theme,
    };

    writer::write(text, theme, writer, should_colorize)
}
