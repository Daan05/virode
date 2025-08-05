use std::{
    collections::HashMap,
    fs,
    io::{self, Write, stdin, stdout},
};
use termion::{event::Key, input::TermRead, raw::IntoRawMode};

use crate::arguments::ArgsConfig;

const GUTTER_WIDTH: usize = 6;

#[derive(Debug)]
struct CursorPos {
    row: u16,
    col: u16,
}

#[derive(Debug, Clone)]
struct TermSize {
    width: u16,
    height: u16,
}

// rename to FileBuffer???
#[derive(Debug)]
struct OpenFile {
    path: String,
    lines: Vec<String>,
    line_no: usize,
    cursor: CursorPos,
    modified: bool,
}

impl OpenFile {
    fn render(&self, term_size: TermSize) {
        for idx in
            self.line_no..(self.line_no + term_size.height as usize - 1).min(self.lines.len() + 1)
        {
            println!(
                "\r{:>4}|{:.len$}",
                idx,
                self.lines[idx - 1],
                len = term_size.width as usize - GUTTER_WIDTH
            );
        }
    }
}

#[derive(Debug)]
pub struct TextEditor {
    term_size: TermSize,
    open_files: HashMap<String, OpenFile>,
    current_file: Option<String>,
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
            current_file: None,
        };

        if let Some(filename) = config.filename {
            editor.open_file(filename)?;
        };

        Ok(editor)
    }

    fn open_file(&mut self, path: String) -> io::Result<()> {
        let content = std::fs::read_to_string(&path)?;
        let mut file = OpenFile {
            path: path.clone(),
            lines: content.split('\n').map(String::from).collect(),
            line_no: 1,
            cursor: CursorPos {
                row: 1,
                col: GUTTER_WIDTH as u16,
            },
            modified: false,
        };

        if file.lines.len() == 0 {
            file.lines.push(String::from(""));
        }

        self.open_files.insert(path.clone(), file);
        self.current_file = Some(path);

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
            let term_size = self.term_size.clone();
            let filekey = self.current_file.clone();

            let file = match filekey {
                Some(key) => match self.open_files.get_mut(&key) {
                    Some(f) => f,
                    None => break,
                },
                None => break,
            };

            // clear terminal
            write!(
                stdout,
                "{}{}",
                termion::clear::All,
                termion::cursor::Goto(1, 1)
            )?;

            // render file
            file.render(term_size.clone());

            // render status bar
            write!(
                stdout,
                "{}{} | col: {}, row: {} | {}",
                termion::cursor::Goto(1, self.term_size.height),
                file.path,
                file.cursor.col - GUTTER_WIDTH as u16 + 1,
                file.cursor.row,
                if file.modified { "Modified" } else { "Saved" }
            )?;

            // move cursor to correct position
            write!(
                stdout,
                "{}",
                termion::cursor::Goto(file.cursor.col, file.cursor.row)
            )?;

            stdout.flush()?;

            // Handle input
            let key = loop {
                if let Some(result) = keys.next() {
                    break result?;
                }
            };

            match key {
                Key::Esc => break,
                Key::Char('\n') => Self::handle_enter(file, term_size),
                Key::Char(c) => Self::handle_char_input(file, term_size, c),
                Key::Ctrl('s') => Self::save_file(file)?,
                Key::Delete => Self::handle_delete(file),
                Key::Backspace => Self::handle_backspace(file),
                Key::Left => Self::move_left(file),
                Key::Right => Self::move_right(file, term_size),
                Key::Up => Self::move_up(file),
                Key::Down => Self::move_down(file, term_size),
                _ => (),
            }

            stdout.flush()?;
        }

        write!(
            stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )?;

        Ok(())
    }

    fn handle_enter(file: &mut OpenFile, term_size: TermSize) {
        file.modified = true;

        let current_line = &mut file.lines[file.line_no + file.cursor.row as usize - 2];
        let remainder = current_line.split_off(file.cursor.col as usize - GUTTER_WIDTH);

        file.lines
            .insert(file.line_no + file.cursor.row as usize - 1, remainder);

        if term_size.height > file.cursor.row + 1
            && file.lines.len() > file.line_no + file.cursor.row as usize - 1
        {
            file.cursor.row += 1;
            file.cursor.col = GUTTER_WIDTH as u16;
        } else {
            Self::scroll_down(file, term_size);
            file.cursor.col = GUTTER_WIDTH as u16;
        }
    }

    fn handle_char_input(file: &mut OpenFile, term_size: TermSize, c: char) {
        file.modified = true;

        let cursor = &file.cursor;
        file.lines[file.line_no + cursor.row as usize - 2]
            .insert(cursor.col as usize - GUTTER_WIDTH, c);
        Self::move_right(file, term_size);
    }

    fn save_file(file: &mut OpenFile) -> std::io::Result<()> {
        let contents = file.lines.join("\n");
        fs::write(&file.path, contents)?;

        file.modified = false;
        Ok(())
    }

    fn handle_delete(file: &mut OpenFile) {
        file.modified = true;

        let cursor = &file.cursor;
        if cursor.col as usize
            == file.lines[file.line_no + cursor.row as usize - 2].len() + GUTTER_WIDTH
        {
            if file.line_no + cursor.row as usize - 1 != file.lines.len() {
                let row_index = file.line_no + cursor.row as usize - 2;
                let combined_line = file.lines[row_index].clone() + &file.lines[row_index + 1];
                file.lines[row_index] = combined_line;
                file.lines.remove(row_index + 1);
            }
        } else {
            file.lines[file.line_no + cursor.row as usize - 2]
                .remove(cursor.col as usize - GUTTER_WIDTH);
        }
    }

    fn handle_backspace(file: &mut OpenFile) {
        file.modified = true;

        let cursor = &file.cursor;
        if cursor.col == GUTTER_WIDTH as u16 {
            if cursor.row != 1 {
                let row_index = file.line_no + cursor.row as usize - 3;
                let new_cursor_x = file.lines[row_index].len() + GUTTER_WIDTH;
                let combined_line = file.lines[row_index].clone() + &file.lines[row_index + 1];
                file.lines[row_index] = combined_line;
                file.lines.remove(row_index + 1);

                file.cursor.col = new_cursor_x as u16;
                file.cursor.row -= 1;
            }
        } else {
            Self::move_left(file);
            Self::handle_delete(file);
        }
    }

    fn move_left(file: &mut OpenFile) {
        if file.cursor.col > GUTTER_WIDTH as u16 {
            file.cursor.col -= 1;
        }
    }

    fn move_right(file: &mut OpenFile, term_size: TermSize) {
        let cursor = &file.cursor;
        let line_len = file.lines[file.line_no + cursor.row as usize - 2].len();

        if term_size.width > cursor.col && line_len + GUTTER_WIDTH > cursor.col as usize {
            file.cursor.col += 1;
        }
    }

    fn move_up(file: &mut OpenFile) {
        if file.cursor.row > 1 {
            file.cursor.row -= 1;

            let cursor = &file.cursor;
            let line_len = file.lines[file.line_no + cursor.row as usize - 2].len();

            if line_len + GUTTER_WIDTH < cursor.col as usize {
                file.cursor.col = (line_len + GUTTER_WIDTH) as u16;
            }
        } else {
            Self::scroll_up(file);
        }
    }

    fn move_down(file: &mut OpenFile, term_size: TermSize) {
        if term_size.height > file.cursor.row + 1
            && file.lines.len() > file.line_no + file.cursor.row as usize - 1
        {
            file.cursor.row += 1;

            let line_len = file.lines[file.line_no + file.cursor.row as usize - 2].len();
            if line_len + GUTTER_WIDTH < file.cursor.col as usize {
                file.cursor.col = (line_len + GUTTER_WIDTH) as u16;
            }
        } else {
            Self::scroll_down(file, term_size);
        }
    }

    fn scroll_up(file: &mut OpenFile) {
        if file.line_no > 1 {
            file.line_no -= 1;
        }
    }

    fn scroll_down(file: &mut OpenFile, _term_size: TermSize) {
        if file.lines.len() > file.line_no + file.cursor.row as usize - 1 {
            file.line_no += 1;
        }
    }
}
