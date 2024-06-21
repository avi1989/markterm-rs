use crate::themes::{Color, ElementTheme, Theme};
use colored::{ColoredString, Colorize};
use markdown::{self, mdast};

pub fn to_colored_string(text: &str, theme: &Theme) -> ColoredString {
    let parse_options = markdown::ParseOptions::gfm();
    let ast_result = match markdown::to_mdast(text, &parse_options) {
        Ok(ast) => ast,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let json_result = serde_json::to_string_pretty::<mdast::Node>(&ast_result).unwrap();
    println!("{}", json_result);

    let colored_text = colorize_node(ast_result, theme);

    return colored_text;

    // return json_result.unwrap();
}

fn colorize_node(node: mdast::Node, theme: &Theme) -> ColoredString {
    return match node {
        mdast::Node::Root(root) => get_text(root.children, theme).normal(),
        mdast::Node::Paragraph(para) => get_text(para.children, theme).normal(),
        mdast::Node::Text(block) => block.value.normal(),
        mdast::Node::Strong(strong) => get_text(strong.children, theme).bold(),
        mdast::Node::Emphasis(emphasis) => get_text(emphasis.children, theme).italic(),
        mdast::Node::BlockQuote(block_quote) => format!("│ {}", get_text(block_quote.children, theme)).normal(),
        mdast::Node::Break(_) => "\n".normal(),
        mdast::Node::Code(code) => apply_theme(&code.value, &theme.code_block),
        mdast::Node::InlineCode(code) => apply_theme(&code.value, &theme.code_block),
        mdast::Node::Delete(delete) => get_text(delete.children, theme).strikethrough(),
        _ => String::new().normal(),
    };
}

fn get_text(children: Vec<mdast::Node>, theme: &Theme) -> String {
    let mut result = String::new();
    for child in children {
        let node_text = colorize_node(child, theme);
        result = format!("{}{}", result, node_text);
    }

    return result;
}

pub fn apply_theme(text: &str, color: &ElementTheme) -> ColoredString {
    let mut colored_text = colored::ColoredString::from(text);

    if color.fg.is_some() {
        let fg_color = color.fg.as_ref().unwrap().to_custom_color();
        colored_text = colored_text.custom_color(fg_color);
    }

    if color.bg.is_some() {
        let bg_color = color.bg.as_ref().unwrap().to_custom_color();
        colored_text = colored_text.on_custom_color(bg_color);
    }

    return colored_text;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::themes::get_default_theme;
    use colored::Colorize;

    macro_rules! string_match {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (value, expected) = $value;
                let theme = get_default_theme();
                let result = to_colored_string(value, &theme);

                println!("{}", result);

                assert_eq!(result.to_string(), expected.to_string())
            }
        )*
        };
    }

    string_match! {
        should_print_text_normally: ("This is text", "This is text"),
        should_handle_strong_text: ("**This is text**", "This is text".bold()),
        should_handle_strong_text_in_middle: ("This is **text**", format!("This is {}", "text".bold())),
        should_handle_emphasis: ("*This text is italics*", "This text is italics".italic()),
        should_handle_emphasis_in_middle: ("This text is *italics*", format!("This text is {}", "italics".italic())),
        should_handle_blockquotes: ("> This is a blockquote", "│ This is a blockquote"),
        should_handle_break: ("This is a  \ntest", "This is a\ntest"), // Note the two spaces before the newline. This generates a Break Node
        should_handle_striketrough: ("~Delete~", "Delete".strikethrough()), // Note the two spaces before the newline. This generates a Break Node
    }

    // #[test]
    // fn should_handle_definition_lists() {
    //     let theme = get_default_theme();
    //     let result = to_colored_string("[a]: b", &theme);

    //     assert_eq!(result.to_string(), "[a]: b")
    // }

    #[test]
    fn should_pretty_print_code() {
        let theme = get_default_theme();
        let result = to_colored_string("`This is a test`", &theme);

        println!("{}", result);

        let expected = "This is a test"
            .custom_color(theme.code_block.fg.unwrap().to_custom_color())
            .on_custom_color(theme.code_block.bg.unwrap().to_custom_color());

        assert_eq!(result.to_string(), expected.to_string())
    }

    // #[test]
    // fn should_handle_emphasis() {
    //     let md = "*This is text*";
    //     let result = to_colored_string(md).to_string();

    //     assert_eq!("This is \u{1b}[1mtext\u{1b}[0m", result);
    // }

    //     #[test]
    //     fn should_parse_header() {
    //         let result = to_colored_string(
    //             r#"
    // > This is a `block quote`
    // "#,
    //         );

    //         let cwd = std::env::current_dir()
    //             .unwrap()
    //             .to_str()
    //             .unwrap()
    //             .to_string();

    //         let mut file = match File::create(format!("{}/temp.json", cwd)) {
    //             Ok(f) => f,
    //             _ => {
    //                 panic!("HELP!")
    //             }
    //         };

    //         let output = format!("{}", result);
    //         file.write_all(output.as_bytes());

    //         assert_eq!("# Hello", cwd);
    //     }
}
