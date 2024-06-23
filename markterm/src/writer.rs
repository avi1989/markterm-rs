use crate::{ElementTheme, TextStyle, Theme};
use markdown::{self, mdast};

const T_ESC: &str = "\u{1b}";

/// Writes the passed in text in markdown to the writer using the theme.
pub fn write(
    text: &str,
    theme: &Theme,
    mut writer: impl std::io::Write,
) -> Result<(), std::io::Error> {
    let parse_options = markdown::ParseOptions::gfm();
    let ast = match markdown::to_mdast(text, &parse_options) {
        Ok(ast) => ast,
        Err(e) => {
            panic!("{}", e);
        }
    };

    if cfg!(test) {
        print_ast_json(&ast);
    }

    write_colored_text(&ast, theme, &mut writer)
}

#[cfg(test)]
fn print_ast_json(ast: &mdast::Node) {
    if std::env::var("PRINT_AST").is_ok() {
        let json_result = serde_json::to_string_pretty::<mdast::Node>(ast).unwrap();
        print!("{}", json_result);
    }
}

#[cfg(not(test))]
fn print_ast_json(_ast: &mdast::Node) {}

fn write_colored_text(
    node: &mdast::Node,
    theme: &Theme,
    writer: &mut impl std::io::Write,
) -> Result<(), std::io::Error> {
    match node {
        mdast::Node::Root(root) => {
            write_themed_text(ElementType::Nodes(&root.children), theme, None, writer)
        }
        mdast::Node::Paragraph(para) => {
            let children = &para.children;
            let mut is_code_para = false;
            if children
                .iter()
                .all(|i| matches!(i, mdast::Node::InlineCode(_)))
            {
                is_code_para = true;
            }

            if is_code_para {
                writeln!(writer)?;
            }

            write_themed_text(ElementType::Nodes(children), theme, None, writer)?;

            if is_code_para {
                writeln!(writer)?;
            }

            Ok(())
        }
        mdast::Node::Text(text) => {
            write_themed_text(ElementType::Text(&text.value), theme, None, writer)
        }
        mdast::Node::Strong(strong) => {
            return write_themed_text(
                ElementType::Nodes(&strong.children),
                theme,
                Some(&theme.strong),
                writer,
            );
        }
        mdast::Node::Emphasis(emphasis) => {
            return write_themed_text(
                ElementType::Nodes(&emphasis.children),
                theme,
                Some(&theme.emphasis),
                writer,
            );
        }
        mdast::Node::BlockQuote(block_quote) => {
            let mut write_intercept = Vec::new();
            write_themed_text(
                ElementType::Nodes(&block_quote.children),
                theme,
                None,
                &mut write_intercept,
            )?;
            let text = std::str::from_utf8(&write_intercept).unwrap();
            let lines = text.lines();
            for line in lines {
                writeln!(writer, "│ {}", line)?
            }

            Ok(())
        }
        mdast::Node::Break(_) => {
            writeln!(writer)
        }
        mdast::Node::Code(code) => {
            writeln!(writer)?;
            write_themed_text(
                ElementType::Text(&code.value),
                theme,
                Some(&theme.code_block),
                writer,
            )?;
            writeln!(writer)
        }
        mdast::Node::InlineCode(code) => {
            let code_text = format!(" {} ", &code.value);
            write!(writer, "")?;

            write_themed_text(
                ElementType::Text(&code_text),
                theme,
                Some(&theme.code_block),
                writer,
            )?;

            write!(writer, "")
        }
        mdast::Node::Delete(delete) => {
            return write_themed_text(
                ElementType::Nodes(&delete.children),
                theme,
                Some(&theme.delete),
                writer,
            );
        }
        mdast::Node::Heading(heading) => {
            // TODO: Build different styles for different depths
            write!(writer, "\n ")?;
            let header_theme = match heading.depth {
                1 => &theme.header_1,
                2 => {
                    write!(writer, "##")?;
                    &theme.header_x
                }
                3 => {
                    write!(writer, "###")?;
                    &theme.header_x
                }
                4 => {
                    write!(writer, "####")?;
                    &theme.header_x
                }
                _ => &theme.header_x,
            };

            write_themed_text(
                ElementType::WhitespacePaddedNode(&heading.children),
                theme,
                Some(header_theme),
                writer,
            )?;

            write!(writer, " \n\n")
        }
        mdast::Node::Image(_image) => {
            // TODO: Fill in.
            write!(writer, "")
        }
        mdast::Node::Link(link) => {
            let link_text = &link.url;
            write!(writer, "{T_ESC}]8;;{}{T_ESC}\\", link_text)?;

            write_themed_text(
                ElementType::Text(link_text),
                theme,
                Some(&theme.link),
                writer,
            )?;
            write!(writer, "{T_ESC}]8;;{T_ESC}\\")
        }
        mdast::Node::List(list) => {
            write_themed_text(ElementType::Nodes(&list.children), theme, None, writer)
        }
        mdast::Node::ListItem(list_item) => {
            write!(writer, "\n• ")?;
            write_themed_text(ElementType::Nodes(&list_item.children), theme, None, writer)?;
            writeln!(writer)
        }
        // mdast::Node::Table(_) => {
        //     panic!("Tables are not supported")
        // }
        // mdast::Node::TableCell(_) => {
        //     panic!("Tables are not supported")
        // }
        // mdast::Node::TableRow(_) => {
        //     panic!("Tables are not supported")
        // }
        // mdast::Node::Html(_) => {
        //     panic!("Html are not supported")
        // }
        _ => {
            write!(writer, "")
        }
    }
}

fn write_raw_text(
    children: &Vec<mdast::Node>,
    theme: &Theme,
    writer: &mut impl std::io::Write,
) -> Result<(), std::io::Error> {
    for child in children {
        write_colored_text(child, theme, writer)?;
    }

    Ok(())
}

#[derive(PartialEq)]
enum ElementType<'a> {
    Text(&'a str),
    Nodes(&'a Vec<mdast::Node>),
    WhitespacePaddedNode(&'a Vec<mdast::Node>),
}

fn write_themed_text(
    input: ElementType,
    theme: &Theme,
    color: Option<&ElementTheme>,
    writer: &mut impl std::io::Write,
) -> Result<(), std::io::Error> {
    let color = color.unwrap_or(&ElementTheme {
        fg: None,
        bg: None,
        style: TextStyle::Normal,
    });

    color.write(
        |writer| match input {
            ElementType::Nodes(children) => write_raw_text(children, theme, writer),
            ElementType::Text(str) => {
                write!(writer, "{}", str)
            }
            ElementType::WhitespacePaddedNode(children) => {
                write!(writer, " ")?;
                write_raw_text(children, theme, writer)?;
                write!(writer, " ")
            }
        },
        writer,
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::themes::get_dark_theme;
    use crate::{get_default_theme, Color};
    use colored::Colorize;

    macro_rules! string_match {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (value, expected) = $value;
                let theme = get_default_theme();
                let mut result = Vec::new();
                let _ = write(value, &theme, &mut result);
                let result = std::str::from_utf8(&result).unwrap();

                println!("{:?}", result);

                assert_eq!(result, expected.to_string())
            }
        )*
        };
    }

    string_match! {
        should_print_text_normally: ("This is text", "This is text".normal()),
        should_handle_strong_text: ("**This is text**", "This is text".bold()),
        should_handle_strong_text_in_middle: ("This is **text**", format!("This is {}", "text".bold())),
        should_handle_emphasis: ("*This text is italics*", "This text is italics".italic()),
        should_handle_emphasis_in_middle: ("This text is *italics*", format!("This text is {}", "italics".italic())),
        should_handle_blockquotes: ("> This is a blockquote", "│ This is a blockquote\n"),
        should_handle_blockquotes_2: (r#"
> This is a blockquote
> This is a blockquote"#, 
    r#"│ This is a blockquote
│ This is a blockquote
"#),
        should_handle_break: ("This is a  \ntest", "This is a\ntest"), // Note the two spaces before the newline. This generates a Break Node
        should_handle_striketrough: ("~Delete~", "Delete".strikethrough()), // Note the two spaces before the newline. This generates a Break Node
    }

    #[test]
    fn should_handle_headers_1() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("# This is a test", &theme, &mut result);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{:?}", result);

        let expected = format!(
            "\n {} \n\n",
            " This is a test ".on_custom_color(to_custom_color(theme.header_1.bg.unwrap()))
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_headers_2() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("## This is a test", &theme, &mut result);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{:?}", result);

        let expected = format!(
            "\n ##{} \n\n",
            " This is a test ".custom_color(to_custom_color(theme.header_x.fg.unwrap()))
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_headers_3() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("### This is a test", &theme, &mut result);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{:?}", result);

        let expected = format!(
            "\n ###{} \n\n",
            " This is a test ".custom_color(to_custom_color(theme.header_x.fg.unwrap()))
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_headers_4() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("#### This is a test", &theme, &mut result);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{:?}", result);

        let expected = format!(
            "\n ####{} \n\n",
            " This is a test ".custom_color(to_custom_color(theme.header_x.fg.unwrap()))
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn should_pretty_print_code() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("`This is a test`", &theme, &mut result);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{:?}", result);

        let expected = format!(
            "\n{}\n",
            " This is a test "
                .custom_color(to_custom_color(theme.code_block.fg.unwrap()))
                .on_custom_color(to_custom_color(theme.code_block.bg.unwrap()))
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn should_add_hyperlink_to_links() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("<http://google.com>", &theme, &mut result);
        let result = std::str::from_utf8(&result).unwrap();

        let link = "http://google.com";

        let fg_color = to_custom_color(theme.link.fg.unwrap());

        let expected = format!(
            "\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\",
            link,
            link.custom_color(fg_color).underline()
        );

        // let expected = "\u{1b}]8;;http://google.com\u{1b}\\\u{1b}[4;48;2;97;85;251mhttp://google.com\u{1b}[0m\u{1b}]8;;\u{1b}\\";

        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_lists() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let input = r#"- List Item 1
- List Item 2"#;

        let _ = write(input, &theme, &mut result);
        let result = std::str::from_utf8(&result).unwrap();
        println!("{:?}", result);

        let expected = r#"
• List Item 1

• List Item 2
"#;
        assert_eq!(result, expected);
    }

    fn to_custom_color(color: Color) -> colored::CustomColor {
        colored::CustomColor {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }
}
