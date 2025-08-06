use std::{
    fs,
    io::{self, Stdout, Write},
};

use termion::raw::RawTerminal;

const GUTTER_WIDTH: usize = 6;

#[derive(Debug, Clone, Copy)]
pub struct CursorPos {
    row: u16,
    col: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct TermSize {
    pub width: u16,
    pub height: u16,
}

// rename to FileBuffer???
#[derive(Debug)]
pub struct OpenFile {
    path: String,
    lines: Vec<String>,
    line_no: usize,
    cursor: CursorPos,
    modified: bool,
}

impl OpenFile {
    pub fn new(path: String, lines: Vec<String>) -> OpenFile {
        let mut file = OpenFile {
            path: path,
            lines: lines,
            line_no: 1,
            cursor: CursorPos {
                col: GUTTER_WIDTH as u16,
                row: 1,
            },
            modified: false,
        };

        if file.lines.len() == 0 {
            file.lines.push(String::from(""));
        }

        file
    }

    pub fn render(&self, term_size: TermSize, stdout: &mut RawTerminal<Stdout>) -> io::Result<()> {
        for idx in
            self.line_no..(self.line_no + term_size.height as usize - 1).min(self.lines.len() + 1)
        {
            write!(
                stdout,
                "{:>4}|{:.len$}\r\n",
                idx,
                self.lines[idx - 1],
                len = term_size.width as usize - GUTTER_WIDTH
            )?;
        }

        write!(
            stdout,
            "{}{} | col: {}, row: {} | {}",
            termion::cursor::Goto(1, term_size.height),
            self.path,
            self.cursor.col - GUTTER_WIDTH as u16 + 1,
            self.cursor.row,
            if self.modified { "Modified" } else { "Saved" }
        )
    }

    pub fn set_cursor(&self, stdout: &mut RawTerminal<Stdout>) -> io::Result<()> {
        write!(
            stdout,
            "{}",
            termion::cursor::Goto(self.cursor.col, self.cursor.row)
        )
    }

    pub fn handle_enter(&mut self, term_size: TermSize) {
        self.modified = true;

        let current_line = &mut self.lines[self.line_no + self.cursor.row as usize - 2];
        let remainder = current_line.split_off(self.cursor.col as usize - GUTTER_WIDTH);

        self.lines
            .insert(self.line_no + self.cursor.row as usize - 1, remainder);

        if term_size.height > self.cursor.row + 1
            && self.lines.len() > self.line_no + self.cursor.row as usize - 1
        {
            self.cursor.row += 1;
            self.cursor.col = GUTTER_WIDTH as u16;
        } else {
            self.scroll_down(term_size);
            self.cursor.col = GUTTER_WIDTH as u16;
        }
    }

    pub fn handle_char_input(&mut self, term_size: TermSize, c: char) {
        self.modified = true;

        let cursor = &self.cursor;
        self.lines[self.line_no + cursor.row as usize - 2]
            .insert(cursor.col as usize - GUTTER_WIDTH, c);
        Self::move_right(self, term_size);
    }

    pub fn save_file(&mut self) -> std::io::Result<()> {
        let contents = self.lines.join("\n");
        fs::write(&self.path, contents)?;

        self.modified = false;
        Ok(())
    }

    pub fn handle_delete(&mut self) {
        self.modified = true;

        let cursor = &self.cursor;
        if cursor.col as usize
            == self.lines[self.line_no + cursor.row as usize - 2].len() + GUTTER_WIDTH
        {
            if self.line_no + cursor.row as usize - 1 != self.lines.len() {
                let row_index = self.line_no + cursor.row as usize - 2;
                let combined_line = self.lines[row_index].clone() + &self.lines[row_index + 1];
                self.lines[row_index] = combined_line;
                self.lines.remove(row_index + 1);
            }
        } else {
            self.lines[self.line_no + cursor.row as usize - 2]
                .remove(cursor.col as usize - GUTTER_WIDTH);
        }
    }

    pub fn handle_backspace(&mut self) {
        self.modified = true;

        let cursor = &self.cursor;
        if cursor.col == GUTTER_WIDTH as u16 {
            if cursor.row != 1 {
                let row_index = self.line_no + cursor.row as usize - 3;
                let new_cursor_x = self.lines[row_index].len() + GUTTER_WIDTH;
                let combined_line = self.lines[row_index].clone() + &self.lines[row_index + 1];
                self.lines[row_index] = combined_line;
                self.lines.remove(row_index + 1);

                self.cursor.col = new_cursor_x as u16;
                self.cursor.row -= 1;
            }
        } else {
            self.move_left();
            self.handle_delete();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor.col > GUTTER_WIDTH as u16 {
            self.cursor.col -= 1;
        }
    }

    pub fn move_right(&mut self, term_size: TermSize) {
        let cursor = &self.cursor;
        let line_len = self.lines[self.line_no + cursor.row as usize - 2].len();

        if term_size.width > cursor.col && line_len + GUTTER_WIDTH > cursor.col as usize {
            self.cursor.col += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor.row > 1 {
            self.cursor.row -= 1;

            let cursor = &self.cursor;
            let line_len = self.lines[self.line_no + cursor.row as usize - 2].len();

            if line_len + GUTTER_WIDTH < cursor.col as usize {
                self.cursor.col = (line_len + GUTTER_WIDTH) as u16;
            }
        } else {
            self.scroll_up();
        }
    }

    pub fn move_down(&mut self, term_size: TermSize) {
        if term_size.height > self.cursor.row + 1
            && self.lines.len() > self.line_no + self.cursor.row as usize - 1
        {
            self.cursor.row += 1;

            let line_len = self.lines[self.line_no + self.cursor.row as usize - 2].len();
            if line_len + GUTTER_WIDTH < self.cursor.col as usize {
                self.cursor.col = (line_len + GUTTER_WIDTH) as u16;
            }
        } else {
            self.scroll_down(term_size);
        }
    }

    pub fn scroll_up(&mut self) {
        if self.line_no > 1 {
            self.line_no -= 1;
        }
    }

    pub fn scroll_down(&mut self, _term_size: TermSize) {
        if self.lines.len() > self.line_no + self.cursor.row as usize - 1 {
            self.line_no += 1;
        }
    }
}
