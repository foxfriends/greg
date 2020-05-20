use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Search,
    View,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Normal => "Normal".fmt(f),
            Self::Insert => "Insert".fmt(f),
            Self::Command => "Prompt".fmt(f),
            Self::Search => "Search".fmt(f),
            Self::View => "View".fmt(f),
        }
    }
}
