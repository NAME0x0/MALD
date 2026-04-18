/// Terminal panel state for PTY + ANSI grid.
pub struct TerminalState {
    pub cols: u16,
    pub rows: u16,
    pub grid: Vec<Vec<Cell>>,
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub scrollback: Vec<Vec<Cell>>,
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub ch: char,
    pub fg: CellColor,
    pub bg: CellColor,
    pub bold: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum CellColor {
    Default,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: CellColor::Default,
            bg: CellColor::Default,
            bold: false,
        }
    }
}

impl TerminalState {
    pub fn new(cols: u16, rows: u16) -> Self {
        let grid = vec![vec![Cell::default(); cols as usize]; rows as usize];
        Self {
            cols,
            rows,
            grid,
            cursor_row: 0,
            cursor_col: 0,
            scrollback: Vec::new(),
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        self.grid = vec![vec![Cell::default(); cols as usize]; rows as usize];
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    /// Write a character at the current cursor position.
    pub fn put_char(&mut self, ch: char) {
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            self.grid[self.cursor_row as usize][self.cursor_col as usize].ch = ch;
            self.cursor_col += 1;
            if self.cursor_col >= self.cols {
                self.cursor_col = 0;
                self.cursor_row += 1;
                if self.cursor_row >= self.rows {
                    self.scroll_up();
                    self.cursor_row = self.rows - 1;
                }
            }
        }
    }

    pub fn newline(&mut self) {
        self.cursor_col = 0;
        self.cursor_row += 1;
        if self.cursor_row >= self.rows {
            self.scroll_up();
            self.cursor_row = self.rows - 1;
        }
    }

    fn scroll_up(&mut self) {
        if !self.grid.is_empty() {
            let top = self.grid.remove(0);
            self.scrollback.push(top);
            self.grid
                .push(vec![Cell::default(); self.cols as usize]);
        }
    }
}
