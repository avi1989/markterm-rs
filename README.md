# Markterm

Markterm is a rust library to render colored markdown into the terminal. I wanted something like
[Glow](https://github.com/charmbracelet/glow) but as a rust library.

## Why build this?
I built this for 2 reasons:
1. To learn rust
2. I couldn't find a crate that did this out of box.

## Status
Markterm is currently in development. I will be working over the next few weeks to make sure that
it supports CommonMark and Github Flavored Markdown.

MarkTerm currently does not support the following
1. Tables
2. Inline Html
3. Syntax Highlighting for embedded code

## Usage
Add it to any existing rust project using cargo. You can then render any markdown
to stdout using the code below.

### Using Default Theme
```rust
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("./test.md");
    markterm::render_file_to_stdout(&path, None);
}
```

### Using a custom theme
```rust
use std::path::PathBuf;
use markterm::{TextStyle, Theme, ElementStyle};

fn main() {
    let path = std::path::PathBuf;
    let theme = Theme {
        header_1: ElementTheme::new(Some("#000"), Some("#500"), TextStyle::Bold),
        .. markterm::get_default_theme()
    };

    markterm::render_file_to_stdout(&path, Some(&theme));
}

```

## Roadmap
- Add support for all common mark elements
- Make the cli more fully featured.
- Make sure it works in all terminals.
- TTY detection.

## Credits
This project would not be possible without [markdown-rs](https://github.com/wooorm/markdown-rs).
Their ast parsing module powers the library.