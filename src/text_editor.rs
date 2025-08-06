use std::{
    collections::HashMap,
    io::{self, Write, stdin, stdout},
};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};

use crate::arguments::ArgsConfig;
use crate::open_file::{OpenFile, TermSize};

#[derive(Debug)]
pub struct TextEditor {
    term_size: TermSize,
    open_files: HashMap<String, OpenFile>,
    current_file: String,
}

impl TextEditor {
    pub fn new(config: ArgsConfig) -> io::Result<Self> {
        let terminal_size = termion::terminal_size()?;
        let mut open_files = HashMap::new();

        TextEditor::open_file(config.file_name.clone(), &mut open_files)?;

        Ok(TextEditor {
            term_size: TermSize {
                width: terminal_size.0,
                height: terminal_size.1,
            },
            open_files: open_files,
            current_file: config.file_name,
        })
    }

    fn open_file(path: String, open_files: &mut HashMap<String, OpenFile>) -> io::Result<()> {
        let content = std::fs::read_to_string(&path)?;
        let file = OpenFile::new(
            path.clone(),
            content.split('\n').map(String::from).collect(),
        );

        open_files.insert(path, file);

        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode()?;
        let mut keys = stdin.keys();

        // run loop
        loop {
            let file = self.open_files.get_mut(&self.current_file).unwrap();

            // clear terminal
            write!(
                stdout,
                "{}{}",
                termion::clear::All,
                termion::cursor::Goto(1, 1)
            )?;

            // render file
            file.render(self.term_size, &mut stdout)?;
            file.set_cursor(&mut stdout)?;
            stdout.flush()?;

            // Handle input
            let key = loop {
                if let Some(result) = keys.next() {
                    break result?;
                }
            };

            match key {
                Key::Esc => break,
                Key::Char('\n') => file.handle_enter(self.term_size),
                Key::Char(c) => file.handle_char_input(self.term_size, c),
                Key::Ctrl('s') => file.save_file()?,
                Key::Delete => file.handle_delete(),
                Key::Backspace => file.handle_backspace(),
                Key::Left => file.move_left(),
                Key::Right => file.move_right(self.term_size),
                Key::Up => file.move_up(),
                Key::Down => file.move_down(self.term_size),
                _ => (),
            }

            stdout.flush()?;
        }

        // clear terminal
        write!(
            stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )?;

        Ok(())
    }
}
