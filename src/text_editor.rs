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
        let mut editor = TextEditor {
            term_size: TermSize {
                width: terminal_size.0,
                height: terminal_size.1,
            },
            open_files: HashMap::new(),
            current_file: String::from(""),
        };

        editor.open_file(config.file_name)?;

        Ok(editor)
    }

    fn open_file(&mut self, path: String) -> io::Result<()> {
        let content = std::fs::read_to_string(&path)?;
        let file = OpenFile::new(
            path.clone(),
            content.split('\n').map(String::from).collect(),
        );

        self.open_files.insert(path.clone(), file);
        self.current_file = path;

        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode()?;

        write!(
            stdout,
            "{}{}ESC to exit. Type stuff, use alt, and so on.\n\r",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )?;

        stdout.flush()?;

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
            file.render(self.term_size);
            file.set_cursor();

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
