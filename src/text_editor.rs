use std::{
    collections::HashMap,
    io::{self, Stdin, Stdout, Write, stdin, stdout},
};
use termion::{
    event::Key,
    input::{Keys, TermRead},
    raw::{IntoRawMode, RawTerminal},
};

use crate::arguments::ArgsConfig;
use crate::open_file::{OpenFile, TermSize};

enum EditorMode {
    Normal,
    Insert,
    Visual,
    VLine,
    Command,
}

pub struct TextEditor {
    quit: bool,
    term_size: TermSize,
    open_files: HashMap<String, OpenFile>,
    current_file: String,
    mode: EditorMode,
    stdout: RawTerminal<Stdout>,
}

impl TextEditor {
    pub fn new(config: ArgsConfig) -> io::Result<Self> {
        let terminal_size = termion::terminal_size()?;
        let mut open_files = HashMap::new();

        TextEditor::open_file(config.file_name.clone(), &mut open_files)?;

        Ok(TextEditor {
            quit: false,
            term_size: TermSize {
                width: terminal_size.0,
                height: terminal_size.1,
            },
            open_files: open_files,
            current_file: config.file_name,
            mode: EditorMode::Insert,
            stdout: stdout().into_raw_mode()?,
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
        let mut keys = stdin.keys();

        while !self.quit {
            // clear terminal
            self.clear_terminal()?;

            // render file
            self.render_file()?;

            // Handle input
            self.handle_input(&mut keys)?;
        }

        // clear terminal
        self.clear_terminal()?;

        Ok(())
    }

    fn clear_terminal(&mut self) -> io::Result<()> {
        write!(
            self.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )
    }

    fn render_file(&mut self) -> io::Result<()> {
        let file = self.open_files.get_mut(&self.current_file).unwrap();

        file.render(self.term_size, &mut self.stdout)?;
        file.set_cursor(&mut self.stdout)?;

        self.stdout.flush()?;

        Ok(())
    }

    fn handle_input(&mut self, keys: &mut Keys<Stdin>) -> io::Result<()> {
        let key = loop {
            if let Some(result) = keys.next() {
                break result?;
            }
        };

        match self.mode {
            EditorMode::Normal => (),
            EditorMode::Insert => self.handle_input_insert_mode(key)?,
            EditorMode::Visual => (),
            EditorMode::VLine => (),
            EditorMode::Command => (),
        };

        Ok(())
    }

    fn handle_input_normal_mode() {}

    fn handle_input_insert_mode(&mut self, key: Key) -> io::Result<()> {
        let file = self.open_files.get_mut(&self.current_file).unwrap();

        match key {
            Key::Esc => self.quit = true,
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
        };

        Ok(())
    }

    fn handle_input_visual_mode() {}

    fn handle_input_vline_mode() {}

    fn handle_input_command_mode() {}
}
