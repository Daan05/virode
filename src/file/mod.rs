pub mod undo;
pub use undo::{CursorPos, Snapshot, TermSize};

use std::{
    fs,
    io::{self, Stdout, Write},
    time::{Duration, Instant},
    usize,
};

use termion::raw::RawTerminal;

const GUTTER_WIDTH: usize = 6;

// rename to FileBuffer???
#[derive(Debug)]
pub struct OpenFile {
    path: String,
    lines: Vec<String>,
    line_no: usize,
    cursor: CursorPos,
    modified: bool,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    last_edit_time: Instant,
}

impl OpenFile {
    pub fn new(path: String, mut lines: Vec<String>) -> OpenFile {
        if lines.len() == 0 {
            lines.push(String::from(""));
        }

        OpenFile {
            path: path,
            lines: lines,
            line_no: 1,
            cursor: CursorPos {
                col: GUTTER_WIDTH as u16,
                row: 1,
            },
            modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_edit_time: Instant::now(),
        }
    }

    pub fn render(
        &self,
        term_size: TermSize,
        stdout: &mut RawTerminal<Stdout>,
        mode: String,
    ) -> io::Result<()> {
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

        let line_len = self.lines[self.line_no + self.cursor.row as usize - 2].len();
        write!(
            stdout,
            "{}{} | col: {}, row: {} | {} | {} ",
            termion::cursor::Goto(1, term_size.height),
            self.path,
            self.cursor.col.min((line_len + GUTTER_WIDTH) as u16) - GUTTER_WIDTH as u16 + 1,
            self.cursor.row,
            if self.modified { "Modified" } else { "Saved" },
            mode
        )
    }

    pub fn set_cursor(&self, stdout: &mut RawTerminal<Stdout>) -> io::Result<()> {
        let line_len = self.lines[self.line_no + self.cursor.row as usize - 2].len();

        write!(
            stdout,
            "{}",
            termion::cursor::Goto(
                self.cursor.col.min((line_len + GUTTER_WIDTH) as u16),
                self.cursor.row
            )
        )
    }

    pub fn handle_enter(&mut self, term_size: TermSize) {
        self.take_snapshot();
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
        self.take_snapshot();
        self.modified = true;

        let cursor = &self.cursor;
        let line_len = self.lines[self.line_no + self.cursor.row as usize - 2].len();
        self.lines[self.line_no + cursor.row as usize - 2].insert(
            self.cursor.col.min((line_len + GUTTER_WIDTH) as u16) as usize - GUTTER_WIDTH,
            c,
        );
        self.move_right(term_size, false);
    }

    pub fn save_file(&mut self) -> std::io::Result<()> {
        let contents = self.lines.join("\n");
        fs::write(&self.path, contents)?;

        self.modified = false;
        Ok(())
    }

    pub fn handle_delete(&mut self) {
        self.take_snapshot();
        self.modified = true;

        let cursor = &self.cursor;
        if cursor.col as usize == self.get_line_len() + GUTTER_WIDTH {
            if self.line_no + cursor.row as usize - 1 != self.lines.len() {
                let row_index = self.line_no + cursor.row as usize - 2;
                let combined_line = self.lines[row_index].clone() + &self.lines[row_index + 1];
                self.lines[row_index] = combined_line;
                self.lines.remove(row_index + 1);
            }
        } else {
            self.delete_char_at_cursor_pos();
        }
    }

    pub fn handle_backspace(&mut self) {
        self.take_snapshot();
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
        let line_len = self.lines[self.line_no + self.cursor.row as usize - 2].len();
        self.cursor.col = self.cursor.col.min((line_len + GUTTER_WIDTH) as u16);

        if self.cursor.col > GUTTER_WIDTH as u16 {
            self.cursor.col -= 1;
        }
    }

    pub fn move_right(&mut self, term_size: TermSize, stop_sooner: bool) {
        if term_size.width > self.cursor.col
            && self.get_line_len() + GUTTER_WIDTH > self.cursor.col as usize + stop_sooner as usize
        {
            self.cursor.col += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor.row > 1 {
            self.cursor.row -= 1;
        } else {
            self.scroll_up();
        }
    }

    pub fn move_down(&mut self, term_size: TermSize) {
        if term_size.height > self.cursor.row + 1
            && self.lines.len() > self.line_no + self.cursor.row as usize - 1
        {
            self.cursor.row += 1;
        } else {
            self.scroll_down(term_size);
        }
    }

    pub fn scroll_up(&mut self) {
        if self.line_no > 1 {
            self.line_no -= 1;
        }
    }

    pub fn scroll_down(&mut self, term_size: TermSize) {
        if self.lines.len() > self.line_no + term_size.height as usize as usize - 1 {
            self.line_no += 1;
        }
    }

    fn get_line_len(&self) -> usize {
        self.lines[self.line_no + self.cursor.row as usize - 2].len()
    }

    pub fn delete_char_at_cursor_pos(&mut self) {
        self.take_snapshot();
        self.modified = true;

        let line = &mut self.lines[self.line_no + self.cursor.row as usize - 2];
        if line.len() == 0 {
            return;
        }

        line.remove(self.cursor.col as usize - GUTTER_WIDTH);

        if self.cursor.col as usize - GUTTER_WIDTH > line.len() {
            self.cursor.col -= 1;
        }
    }

    fn take_snapshot(&mut self) {
        if self.last_edit_time.elapsed() < Duration::from_millis(500) && !self.undo_stack.is_empty()
        {
            return;
        }

        self.undo_stack.push(Snapshot {
            content: self.lines.clone(),
            cursor: self.cursor,
            timestamp: Instant::now(),
        });
        self.redo_stack.clear();
        self.last_edit_time = Instant::now();
    }

    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            self.redo_stack.push(Snapshot {
                content: self.lines.clone(),
                cursor: self.cursor,
                timestamp: Instant::now(),
            });

            self.lines = snapshot.content;
            self.cursor = snapshot.cursor;
        }
    }

    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            self.undo_stack.push(Snapshot {
                content: self.lines.clone(),
                cursor: self.cursor,
                timestamp: Instant::now(),
            });

            self.lines = snapshot.content;
            self.cursor = snapshot.cursor;
        }
    }
}
