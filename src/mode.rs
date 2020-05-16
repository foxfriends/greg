#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}
