use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about, long_about = None)]
#[derive(Debug)]
struct Args {
    // Path to the file
    file_path: String,

    #[arg(short, long, default_value_t = clap::ColorChoice::Auto)]
    color: clap::ColorChoice,
}

fn main() {
    let args = Args::parse();

    let mut file_path = PathBuf::new();
    file_path.push(&args.file_path);

    if !file_path.exists() {
        println!("File not found");
        return;
    }

    let color_choice = match args.color {
        clap::ColorChoice::Always => markterm::ColorChoice::Always,
        clap::ColorChoice::Auto => markterm::ColorChoice::Auto,
        clap::ColorChoice::Never => markterm::ColorChoice::Never,
    };

    let result = markterm::render_file_to_stdout(&file_path, None, color_choice);
    match result {
        Ok(()) => (),
        Err(err) => panic!("Failed to render markdown {}", err),
    }
}
