use csv::{ReaderBuilder, StringRecord};
use ncurses::set_escdelay;
use pancurses::{
    endwin, getmouse, initscr, mousemask, noecho, raw, resize_term, start_color, Input, Window,
};
use std::borrow::Cow;
use std::fs::File;

mod args;
mod matrix;
mod mode;

use args::Args;
use matrix::Matrix;
use mode::Mode;

#[derive(Default)]
struct State {
    // settings state
    column_width: (usize, usize),
    headers: usize,

    // program state
    mode: Mode,
    status: String,
    command: String,
    view: [usize; 2],
    cursors: Vec<[usize; 3]>,
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
    let data: Matrix<Cow<str>> = src_records
        .iter()
        .map(|record| record.iter().map(Cow::from).collect())
        .collect();
    let window = initscr();
    window.keypad(true);
    set_escdelay(0);
    mousemask(pancurses::ALL_MOUSE_EVENTS, std::ptr::null_mut());
    raw();
    noecho();
    start_color();

    let mut state = State {
        column_width: args.column_width,
        headers: args.headers,
        ..State::default()
    };
    loop {
        render(&window, &state, &data);
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
                    // TODO: unambiguous prefix matching & suggestion
                    match std::mem::take(&mut state.command).as_ref() {
                        "quit" => break,
                        cmd => {
                            state.status = format!("unknown command '{}'", cmd);
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
    state.status.clear();
    match input {
        Input::Character('i') => state.mode = Mode::Insert,
        Input::Character(':') => {
            state.mode = Mode::Command;
        }
        Input::Character('?') => {
            state.mode = Mode::Search;
        }
        _ => state.status = format!("received {:?}", input),
    }
}

fn insert_mode(state: &mut State, window: &Window, input: Input) {
    state.status.clear();
    match input {
        Input::Character('\u{1b}') => state.mode = Mode::Normal,
        Input::Character(ch) if ch.is_ascii_graphic() => {
            // TODO: write the character
        }
        _ => {}
    }
}

fn command_mode(state: &mut State, window: &Window, input: Input) -> bool {
    match input {
        Input::Character('\u{1b}') => {
            state.mode = Mode::Normal;
            state.command.clear();
        }
        Input::Character('\n') => {
            state.mode = Mode::Normal;
            return true;
        }
        Input::Character(ch) if ch.is_ascii_graphic() => {
            state.command.push(ch);
        }
        Input::KeyBackspace => {
            state.command.pop();
        }
        _ => {}
    }
    false
}

fn set_status<T: AsRef<str>>(window: &Window, status: T) {
    let y = window.get_max_y();
    window.mvaddstr(y - 1, 0, status);
    window.clrtoeol();
}

macro_rules! add {
    ($a:expr, $b:expr) => {
        [$a[0] + $b[0], $a[1] + $b[1]]
    };
}

fn render(
    window: &Window,
    State {
        column_width,
        headers,
        view,
        mode,
        command,
        status,
        ..
    }: &State,
    data: &Matrix<Cow<str>>,
) {
    let y = if *headers == 0 {
        0
    } else {
        *headers as i32 + 1
    };
    let mut x = 2;
    let (max_y, max_x) = window.get_max_yx();
    let max_y = max_y - 2; // save space for status line

    let rows_to_show = usize::min(data.dimensions()[0] - headers, max_y as usize / 2 - headers);

    window.mv(0, 0);
    window.vline('|', (headers + rows_to_show * 2) as i32);

    let mut column = 0;
    while x < max_x && column < data.dimensions()[1] {
        if *headers > 0 {
            for i in 0..*headers {
                let header = data[&[i, view[1] + column]]
                    .chars()
                    .take(column_width.1)
                    .collect::<String>();
                window.mvaddstr(i as i32, x, header);
            }
        }

        let mut width = column_width.0;
        for i in 0..rows_to_show {
            let element = data[&add!(view, [i + 1, column])]
                .chars()
                .take(column_width.1)
                .collect::<String>();
            width = usize::max(width, element.chars().count());
            window.mvaddstr(y + i as i32 * 2, x, element);
        }
        x += width as i32 + 3;
        window.mv(0, x - 2);
        window.vline('|', (headers + rows_to_show * 2) as i32);

        column += 1;
    }
    window.mv(y - 1, 0);
    window.hline('=', x - 1);
    for i in 0..rows_to_show {
        window.mv(y + i as i32 * 2 + 1, 0);
        window.hline('-', x - 1);
    }

    match mode {
        Mode::Command => set_status(&window, format!(":{}", command)),
        Mode::Search => set_status(&window, format!("?{}", command)),
        _ => set_status(&window, status),
    }
}
