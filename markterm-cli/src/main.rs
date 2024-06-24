use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about, long_about = None)]
#[derive(Debug)]
struct Args {
    // Path to the file
    file_path: String,
}

fn main() {
    let args = Args::parse();

    let mut file_path = PathBuf::new();
    file_path.push(&args.file_path);

    if !file_path.exists() {
        println!("File not found");
        return;
    }

    let result = markterm::render_file_to_stdout(&file_path, None);
    match result {
        Ok(()) => (),
        Err(err) => panic!("Failed to render markdown {}", err),
    }
}
