#![warn(missing_docs)]

//! A cross platform library to render colored markdown to the terminal.
//! The rendered markdown is colored and is themeable.

/// Modules to help theme the output
pub mod themes;
pub use themes::{get_default_theme, Color, ElementTheme, TextStyle, Theme};

/// A module to write the appropriate terminal escape sequence to color the text
mod writer;

use std::io::Read;
use std::{
    fs::File,
    io::{self},
    path::PathBuf,
};

/// Renders the contents of the passed in file to stdout.
///
/// ### Example
/// ```rust
/// let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
/// path.push("benches/sample.md");
///
/// markterm::render_file_to_stdout(&path, None);
/// ```
pub fn render_file_to_stdout(
    file_path: &PathBuf,
    theme: Option<&self::Theme>,
) -> Result<(), std::io::Error> {
    render_file(file_path, theme, &mut std::io::stdout())
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
/// markterm::render_file(&path, None, &mut dest);
/// ```
pub fn render_file(
    file_path: &PathBuf,
    theme: Option<&Theme>,
    writer: &mut impl std::io::Write,
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

    render_text(&file_contents, theme, writer)
}

/// Renders the contents of the passed in string to stdout.
///
/// ### Example
/// ```rust
/// let str = "> This is a `test`";
/// markterm::render_text_to_stdout(str, None);
/// ```
pub fn render_text_to_stdout(text: &str, theme: Option<&Theme>) -> Result<(), std::io::Error> {
    render_text(text, theme, &mut std::io::stdout())
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
/// markterm::render_text(str, None, &mut dest);
/// ```
pub fn render_text(
    text: &str,
    theme: Option<&Theme>,
    writer: &mut impl std::io::Write,
) -> Result<(), std::io::Error> {
    let theme = match theme {
        Some(x) => x,
        None => &get_default_theme(),
    };

    writer::write(text, theme, writer)
}
