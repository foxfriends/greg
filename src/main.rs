use csv::{ReaderBuilder, StringRecord};
use pancurses::{
    endwin, getmouse, initscr, mousemask, noecho, resize_term, start_color, Input, Window,
};
use std::borrow::Cow;
use std::fs::File;

mod args;
mod mode;

use args::Args;
use mode::Mode;

#[derive(Default)]
struct State {
    mode: Mode,
    command: String,
}

#[paw::main]
fn main(args: Args) -> std::io::Result<()> {
    let src = File::open(args.file)?;
    let src_records: Vec<StringRecord> = ReaderBuilder::new()
        .delimiter(args.separator)
        .has_headers(false) // we re-implement headers manually
        .flexible(true)
        .trim(args.trim)
        .terminator(args.terminator)
        .quote(args.quote)
        .escape(args.quote_escape)
        .double_quote(!args.ignore_double_quote)
        .quoting(!args.ignore_quotes)
        .comment(args.comment)
        .from_reader(src)
        .into_records()
        .collect::<csv::Result<_>>()?;
    let mut data: Vec<Vec<Cow<str>>> = src_records
        .iter()
        .map(|record| record.iter().map(Cow::from).collect())
        .collect();
    let field_count = data.iter().map(|record| record.len()).max().unwrap_or(1);
    for record in data.iter_mut() {
        record.resize(field_count, Cow::from(""));
    }

    let window = initscr();
    window.keypad(true);
    mousemask(pancurses::ALL_MOUSE_EVENTS, std::ptr::null_mut());
    noecho();
    start_color();

    let mut state = State::default();
    loop {
        match window.getch() {
            Some(Input::KeyResize) => {
                resize_term(0, 0);
            }
            Some(Input::KeyMouse) => {
                let _mouse_event = getmouse().expect("unexpected mouse error");
            }
            Some(input) if state.mode == Mode::Insert => insert_mode(&mut state, &window, input),
            Some(input) if state.mode == Mode::Command => {
                if command_mode(&mut state, &window, input) {
                    match std::mem::take(&mut state.command).as_ref() {
                        "quit" => break,
                        cmd => {
                            set_status(&window, format!("unknown command '{}'", cmd));
                            state.mode = Mode::Normal;
                        }
                    }
                }
            }
            Some(input) if state.mode == Mode::Search => {
                if command_mode(&mut state, &window, input) {
                    // commit search
                    state.mode = Mode::Normal;
                    state.command = String::new(); // TODO: implement search
                } else {
                    // soft highlight
                }
            }
            Some(input) => normal_mode(&mut state, &window, input),
            None => unreachable!(),
        }
    }

    endwin();

    Ok(())
}

fn normal_mode(state: &mut State, window: &Window, input: Input) {
    match input {
        Input::Character('i') => state.mode = Mode::Insert,
        Input::Character(':') => state.mode = Mode::Command,
        Input::Character('?') => state.mode = Mode::Search,
        _ => set_status(window, format!("received {:?}", input)),
    }
}

fn insert_mode(state: &mut State, window: &Window, input: Input) {}

fn command_mode(state: &mut State, window: &Window, input: Input) -> bool {
    true
}

fn set_status<T: AsRef<str>>(window: &Window, status: T) {
    let y = window.get_max_y();
    window.mvaddstr(y - 1, 1, status);
    window.clrtoeol();
}
