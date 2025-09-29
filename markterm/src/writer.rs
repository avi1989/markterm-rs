use crate::{ElementTheme, TextStyle, Theme};
use markdown::{self, mdast};

const T_ESC: &str = "\u{1b}";

/// Writes the passed in text in markdown to the writer using the theme.
pub fn write(
    text: &str,
    theme: &Theme,
    mut writer: impl std::io::Write,
    is_writer_tty: bool,
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

    write_colored_text(&ast, theme, &mut writer, &is_writer_tty)
}

#[cfg(test)]
fn print_ast_json(ast: &mdast::Node) {
    if std::env::var("PRINT_AST").is_ok() {
        let json_result = serde_json::to_string_pretty::<mdast::Node>(ast).unwrap();
        print!("{json_result}");
    }
}

#[cfg(not(test))]
fn print_ast_json(_ast: &mdast::Node) {}

fn write_colored_text(
    node: &mdast::Node,
    theme: &Theme,
    writer: &mut impl std::io::Write,
    is_writer_tty: &bool,
) -> Result<(), std::io::Error> {
    match node {
        mdast::Node::Root(root) => write_themed_text(
            ElementType::Nodes(&root.children),
            theme,
            None,
            writer,
            is_writer_tty,
        ),
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

            write_themed_text(
                ElementType::Nodes(children),
                theme,
                None,
                writer,
                is_writer_tty,
            )?;

            if is_code_para {
                writeln!(writer)?;
            }

            Ok(())
        }
        mdast::Node::Text(text) => write_themed_text(
            ElementType::Text(&text.value),
            theme,
            None,
            writer,
            is_writer_tty,
        ),
        mdast::Node::Strong(strong) => write_themed_text(
            ElementType::Nodes(&strong.children),
            theme,
            Some(&theme.strong),
            writer,
            is_writer_tty,
        ),
        mdast::Node::Emphasis(emphasis) => write_themed_text(
            ElementType::Nodes(&emphasis.children),
            theme,
            Some(&theme.emphasis),
            writer,
            is_writer_tty,
        ),
        mdast::Node::Blockquote(block_quote) => {
            let mut write_intercept = Vec::new();
            write_themed_text(
                ElementType::Nodes(&block_quote.children),
                theme,
                None,
                &mut write_intercept,
                is_writer_tty,
            )?;
            let text = std::str::from_utf8(&write_intercept).unwrap();
            let lines = text.lines();
            for line in lines {
                writeln!(writer, "│ {line}")?
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
                is_writer_tty,
            )?;
            writeln!(writer)
        }
        mdast::Node::InlineCode(code) => {
            let code_text = format!(" {} ", &code.value)
                .replace("{{", "")
                .replace("}}", "");

            write!(writer, "")?;

            write_themed_text(
                ElementType::Text(&code_text),
                theme,
                Some(&theme.code_block),
                writer,
                is_writer_tty,
            )?;

            write!(writer, "")
        }
        mdast::Node::Delete(delete) => write_themed_text(
            ElementType::Nodes(&delete.children),
            theme,
            Some(&theme.delete),
            writer,
            is_writer_tty,
        ),
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
                is_writer_tty,
            )?;

            write!(writer, " \n\n")
        }
        mdast::Node::Image(_image) => {
            // TODO: Fill in.
            write!(writer, "")
        }
        mdast::Node::Link(link) => {
            let link_text = &link.url;
            if !is_writer_tty {
                write_themed_text(
                    ElementType::Text(link_text),
                    theme,
                    Some(&theme.link),
                    writer,
                    is_writer_tty,
                )
            } else {
                write!(writer, "{T_ESC}]8;;{link_text}{T_ESC}\\")?;

                write_themed_text(
                    ElementType::Text(link_text),
                    theme,
                    Some(&theme.link),
                    writer,
                    is_writer_tty,
                )?;
                write!(writer, "{T_ESC}]8;;{T_ESC}\\")
            }
        }
        mdast::Node::List(list) => write_themed_text(
            ElementType::Nodes(&list.children),
            theme,
            None,
            writer,
            is_writer_tty,
        ),
        mdast::Node::ListItem(list_item) => {
            write!(writer, "\n• ")?;
            write_themed_text(
                ElementType::Nodes(&list_item.children),
                theme,
                None,
                writer,
                is_writer_tty,
            )?;
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
    is_writer_tty: &bool,
) -> Result<(), std::io::Error> {
    for child in children {
        write_colored_text(child, theme, writer, is_writer_tty)?;
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
    is_writer_tty: &bool,
) -> Result<(), std::io::Error> {
    let color = color.unwrap_or(&ElementTheme {
        fg: None,
        bg: None,
        style: TextStyle::Normal,
    });

    color.write(
        |writer| match input {
            ElementType::Nodes(children) => write_raw_text(children, theme, writer, is_writer_tty),
            ElementType::Text(str) => {
                write!(writer, "{str}")
            }
            ElementType::WhitespacePaddedNode(children) => {
                write!(writer, " ")?;
                write_raw_text(children, theme, writer, is_writer_tty)?;
                write!(writer, " ")
            }
        },
        writer,
        is_writer_tty,
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
            #[cfg(test)]
            mod $name {
                use super::*;

                #[test]
                fn should_print_themed_text_if_tty() {
                    let (value, expected_if_tty, _) = $value;
                    let theme = get_default_theme();
                    let mut result = Vec::new();
                    let _ = write(value, &theme, &mut result, true);
                    let result = std::str::from_utf8(&result).unwrap();

                    println!("{:?}", result);

                    assert_eq!(result, expected_if_tty.to_string())
                }

                #[test]
                fn should_print_plain_text_if_not_tty() {
                    let (value, _, expected_if_not_tty) = $value;
                    let theme = get_default_theme();
                    let mut result = Vec::new();
                    let _ = write(value, &theme, &mut result, false);
                    let result = std::str::from_utf8(&result).unwrap();

                    println!("Result = {:?}\n Value = {:?}", result, value);

                    assert_eq!(result, expected_if_not_tty)
                 }
            }
        )*
        };
    }

    string_match! {
        normal_text: ("This is text", "This is text".normal(), "This is text"),
        strong_text: ("**This is text**", "This is text".bold(), "This is text"),
        normal_plus_strong_text: ("This is **text**", format!("This is {}", "text".bold()), "This is text"),
        emphasis_text: ("*This text is italics*", "This text is italics".italic(), "This text is italics"),
        normal_plus_emphasis_text: ("This text is *italics*", format!("This text is {}", "italics".italic()), "This text is italics"),
        blockquotes: ("> This is a blockquote", "│ This is a blockquote\n", "│ This is a blockquote\n"),
        blockquotes_with_multiple_lines: (r#"
> This is a blockquote
> This is a blockquote"#, 
    r#"│ This is a blockquote
│ This is a blockquote
"#,
            r#"│ This is a blockquote
│ This is a blockquote
"#),
        line_breaks: ("This is a  \ntest", "This is a\ntest", "This is a\ntest"), // Note the two spaces before the newline. This generates a Break Node
        strikethrough: ("~Delete~", "Delete".strikethrough(), "Delete"), // Note the two spaces before the newline. This generates a Break Node
    }

    #[test]
    fn should_handle_headers_1_in_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("# This is a test", &theme, &mut result, true);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{result:?}");

        let expected = format!(
            "\n {} \n\n",
            " This is a test ".on_custom_color(to_custom_color(theme.header_1.bg.unwrap()))
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_headers_1_if_not_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("# This is a test", &theme, &mut result, false);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{result:?}");

        let expected = "\n  This is a test  \n\n";

        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_headers_2_in_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("## This is a test", &theme, &mut result, true);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{result:?}");

        let expected = format!(
            "\n ##{} \n\n",
            " This is a test ".custom_color(to_custom_color(theme.header_x.fg.unwrap()))
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_headers_2_if_not_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("## This is a test", &theme, &mut result, false);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{result:?}");

        let expected = "\n ## This is a test  \n\n";

        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_headers_3_if_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("### This is a test", &theme, &mut result, true);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{result:?}");

        let expected = format!(
            "\n ###{} \n\n",
            " This is a test ".custom_color(to_custom_color(theme.header_x.fg.unwrap()))
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_headers_3_if_not_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("### This is a test", &theme, &mut result, false);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{result:?}");

        let expected = "\n ### This is a test  \n\n";

        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_headers_4_if_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("#### This is a test", &theme, &mut result, true);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{result:?}");

        let expected = format!(
            "\n ####{} \n\n",
            " This is a test ".custom_color(to_custom_color(theme.header_x.fg.unwrap()))
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_headers_4_if_not_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("#### This is a test", &theme, &mut result, false);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{result:?}");

        let expected = "\n #### This is a test  \n\n";

        assert_eq!(result, expected);
    }

    #[test]
    fn should_pretty_print_code_if_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("`This is a test`", &theme, &mut result, true);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{result:?}");

        let expected = format!(
            "\n{}\n",
            " This is a test "
                .custom_color(to_custom_color(theme.code_block.fg.unwrap()))
                .on_custom_color(to_custom_color(theme.code_block.bg.unwrap()))
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn should_pretty_print_code_if_not_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("`This is a test`", &theme, &mut result, false);

        let result = std::str::from_utf8(&result).unwrap();

        println!("{result:?}");

        let expected = "\n This is a test \n";

        assert_eq!(result, expected);
    }

    #[test]
    fn should_add_hyperlink_to_links_if_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("<http://google.com>", &theme, &mut result, true);
        let result = std::str::from_utf8(&result).unwrap();

        let link = "http://google.com";

        let fg_color = to_custom_color(theme.link.fg.unwrap());

        let expected = format!(
            "\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\",
            link,
            link.custom_color(fg_color).underline()
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn should_not_hyperlink_to_links_if_not_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let _ = write("<http://google.com>", &theme, &mut result, false);
        let result = std::str::from_utf8(&result).unwrap();

        let expected = "http://google.com";

        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_lists_if_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let input = r#"- List Item 1
- List Item 2"#;

        let _ = write(input, &theme, &mut result, true);
        let result = std::str::from_utf8(&result).unwrap();
        println!("{result:?}");

        let expected = r#"
• List Item 1

• List Item 2
"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn should_handle_lists_if_not_tty() {
        let theme = get_dark_theme();
        let mut result = Vec::new();
        let input = r#"- List Item 1
- List Item 2"#;

        let _ = write(input, &theme, &mut result, false);
        let result = std::str::from_utf8(&result).unwrap();
        println!("{result:?}");

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
