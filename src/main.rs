use std::env;

mod arguments;
mod text_editor;
mod open_file;

use arguments::ArgsConfig;
use text_editor::TextEditor;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = ArgsConfig::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });

    let mut editor = TextEditor::new(config).unwrap_or_else(|err| {
        eprintln!("Problem starting the text editor: {}", err);
        std::process::exit(1);
    });

    editor.run().unwrap_or_else(|err| {
        eprintln!("Problem while running the text editor: {}", err);
        std::process::exit(1);
    });
}
