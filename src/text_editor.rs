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

impl EditorMode {
    fn to_string(&self) -> String {
        match self {
            EditorMode::Normal => String::from("Normal"),
            EditorMode::Insert => String::from("Insert"),
            EditorMode::Visual => String::from("Visual"),
            EditorMode::VLine => String::from("VLine"),
            EditorMode::Command => String::from("Command"),
        }
    }
}

pub struct TextEditor {
    quit: bool,
    term_size: TermSize,
    open_files: HashMap<String, OpenFile>,
    current_file: String,
    mode: EditorMode,
    stdout: RawTerminal<Stdout>,
    keys: Keys<Stdin>,
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
            mode: EditorMode::Normal,
            stdout: stdout().into_raw_mode()?,
            keys: stdin().keys(),
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
        while !self.quit {
            self.clear_terminal()?;
            self.render_file()?;
            self.handle_input()?;
        }
        self.clear_terminal()
    }

    fn clear_terminal(&mut self) -> io::Result<()> {
        write!(
            self.stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
        )
    }

    fn render_file(&mut self) -> io::Result<()> {
        let file = self.open_files.get_mut(&self.current_file).unwrap();

        file.render(self.term_size, &mut self.stdout, self.mode.to_string())?;
        file.set_cursor(&mut self.stdout)?;

        self.stdout.flush()?;

        Ok(())
    }

    fn handle_input(&mut self) -> io::Result<()> {
        let key = loop {
            if let Some(result) = self.keys.next() {
                break result?;
            }
        };

        match self.mode {
            EditorMode::Normal => self.handle_input_normal_mode(key)?,
            EditorMode::Insert => self.handle_input_insert_mode(key)?,
            EditorMode::Visual => self.handle_input_visual_mode(key),
            EditorMode::VLine => self.handle_input_vline_mode(key),
            EditorMode::Command => self.handle_input_command_mode(key),
        };

        Ok(())
    }

    fn handle_input_normal_mode(&mut self, key: Key) -> io::Result<()> {
        let file = self.open_files.get_mut(&self.current_file).unwrap();

        match key {
            Key::Esc => self.quit = true,
            Key::Char('i') => self.enter_insert_mode()?,
            Key::Char('a') => {
                file.move_right(self.term_size, false);
                self.enter_insert_mode()?;
            }
            Key::Char('x') => file.delete_char_at_cursor_pos(),
            Key::Ctrl('u') => {
                for _ in 0..self.term_size.height / 2 {
                    file.scroll_up();
                }
            }
            Key::Ctrl('d') => {
                for _ in 0..self.term_size.height / 2 {
                    file.scroll_down(self.term_size);
                }
            }
            Key::Left | Key::Char('h') => file.move_left(),
            Key::Right | Key::Char('l') => file.move_right(self.term_size, true),
            Key::Up | Key::Char('k') => file.move_up(),
            Key::Down | Key::Char('j') => file.move_down(self.term_size),
            _ => (),
        };

        Ok(())
    }

    fn handle_input_insert_mode(&mut self, key: Key) -> io::Result<()> {
        let file = self.open_files.get_mut(&self.current_file).unwrap();

        match key {
            Key::Esc => self.exit_insert_mode()?,
            Key::Char('\n') => file.handle_enter(self.term_size),
            Key::Char('\t') => {
                file.handle_char_input(self.term_size, ' ');
                file.handle_char_input(self.term_size, ' ');
                file.handle_char_input(self.term_size, ' ');
                file.handle_char_input(self.term_size, ' ');
            }
            Key::Char(c) => file.handle_char_input(self.term_size, c),
            Key::Ctrl('s') => {
                file.save_file()?;
                self.mode = EditorMode::Normal;
                write!(self.stdout, "{}", termion::cursor::SteadyBlock)?;
            }
            Key::Delete => file.handle_delete(),
            Key::Backspace => file.handle_backspace(),
            Key::Left => file.move_left(),
            Key::Right => file.move_right(self.term_size, false),
            Key::Up => file.move_up(),
            Key::Down => file.move_down(self.term_size),
            _ => (),
        };

        Ok(())
    }

    fn handle_input_visual_mode(&mut self, _key: Key) {
        todo!();
    }

    fn handle_input_vline_mode(&mut self, _key: Key) {
        todo!();
    }

    fn handle_input_command_mode(&mut self, _key: Key) {
        todo!();
    }

    fn enter_insert_mode(&mut self) -> io::Result<()> {
        self.mode = EditorMode::Insert;
        write!(self.stdout, "{}", termion::cursor::SteadyBar)
    }

    fn exit_insert_mode(&mut self) -> io::Result<()> {
        let file = self.open_files.get_mut(&self.current_file).unwrap();
        file.move_left();

        self.mode = EditorMode::Normal;
        write!(self.stdout, "{}", termion::cursor::SteadyBlock)
    }
}
