use super::{Matrix, Mode};
use std::borrow::Cow;

#[derive(Default, Eq, PartialEq, Debug)]
pub struct Cursor {
    pub row: usize,
    pub column: usize,
    pub position: usize,
    pub pinned: bool,
}

impl Cursor {
    pub fn new(row: usize, column: usize) -> Self {
        Self {
            row,
            column,
            ..Self::default()
        }
    }
}

#[derive(Default)]
pub struct State<'d> {
    // settings
    pub column_width: (usize, usize), // (min, max)
    pub headers: usize,

    // program
    pub mode: Mode,
    pub status: String,
    pub command: String,
    pub view: [usize; 2],     // [y, x]
    pub cursors: Vec<Cursor>, // [y, x, char]

    // data
    // TODO: this is a very inefficient undo-stack representation, particularly for large data.
    //       will need to improve this
    pub undo_stack: Vec<Matrix<Cow<'d, str>>>,
    pub data: Matrix<Cow<'d, str>>,
}

impl State<'_> {
    pub fn move_view(&mut self, dy: i32, dx: i32) {
        self.view[0] = i32::max(
            self.headers as i32,
            i32::min(
                self.data.dimensions()[0].saturating_sub(1) as i32,
                self.view[0] as i32 + dy,
            ),
        ) as usize;
        self.view[1] = i32::max(
            0,
            i32::min(
                self.data.dimensions()[1].saturating_sub(1) as i32,
                self.view[1] as i32 + dx,
            ),
        ) as usize;
    }

    pub fn move_cursor(&mut self, dy: i32, dx: i32) {
        for cursor in self.cursors.iter_mut().filter(|cursor| !cursor.pinned) {
            cursor.row = i32::max(
                self.headers as i32,
                i32::min(
                    self.data.dimensions()[0].saturating_sub(1) as i32,
                    cursor.row as i32 + dy,
                ),
            ) as usize;
            cursor.column = i32::max(
                0,
                i32::min(
                    self.data.dimensions()[1].saturating_sub(1) as i32,
                    cursor.column as i32 + dx,
                ),
            ) as usize;
        }
    }

    pub fn goto_line(&mut self, line: usize) {
        self.cursors.clear();
        self.cursors.push(Cursor::new(line, 0));
    }
}
