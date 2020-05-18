use csv::{ReaderBuilder, StringRecord};
use ncurses::set_escdelay;
use pancurses::{
    endwin, getmouse, initscr, mousemask, noecho, raw, resize_term, start_color, Input, Window,
    A_BOLD, A_DIM, A_STANDOUT,
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
struct State<'d> {
    // settings state
    column_width: (usize, usize), // (min, max)
    headers: usize,

    // program state
    mode: Mode,
    status: String,
    command: String,
    view: [usize; 2],         // [y, x]
    cursors: Vec<[usize; 3]>, // [y, x, char]

    // data state
    // TODO: this is a very inefficient undo-stack representation, particularly for large data.
    //       will need to improve this
    undo_stack: Vec<Matrix<Cow<'d, str>>>,
    data: Matrix<Cow<'d, str>>,
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
        data,
        view: [args.headers, 0],
        cursors: vec![[0, 0, 0]],
        ..State::default()
    };
    loop {
        render(&window, &state);
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
        Input::Character('/') => {
            state.mode = Mode::Search;
        }
        Input::Character('H') => {
            state.view[1] = state.view[1].saturating_sub(1);
        }
        Input::Character('J') => {
            state.view[0] = usize::min(
                state.view[0] + 1,
                state.data.dimensions()[0].saturating_sub(1),
            );
        }
        Input::Character('K') => {
            state.view[0] = usize::max(state.headers, state.view[0].saturating_sub(1));
        }
        Input::Character('L') => {
            state.view[1] = usize::min(
                state.view[1] + 1,
                state.data.dimensions()[1].saturating_sub(1),
            );
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

fn render(
    window: &Window,
    State {
        column_width,
        headers,
        view,
        mode,
        command,
        status,
        data,
        ..
    }: &State,
) {
    window.clear();
    let y = if *headers == 0 {
        0
    } else {
        *headers as i32 + 1
    };
    let (max_y, max_x) = window.get_max_yx();
    let max_y = max_y - 2; // save space for status line
    let rows_to_show = usize::min(data.dimensions()[0] - view[0], max_y as usize / 2 - headers);

    let bottom_position = (headers + rows_to_show * 2) as i32;

    // TODO: line numbers in a more subtle colour?
    let digits = ((rows_to_show + view[0]) as f32).log10().ceil() as usize;
    for i in 0..rows_to_show {
        let s = format!("{:>width$}", i + view[0], width = digits);
        window.mvaddstr(y + i as i32 * 2, 0, s);
    }

    let mut x = digits as i32 + 2;
    let mut vline_positions = vec![x - 1];

    let mut column = view[1];
    while x < max_x && column < data.dimensions()[1] {
        let mut width = column_width.0;
        window.attron(A_BOLD);
        if *headers > 0 {
            for i in 0..*headers {
                let header = data[&[i, column]]
                    .chars()
                    .take(column_width.1)
                    .collect::<String>();
                width = usize::max(width, header.chars().count());
                window.mvaddstr(i as i32, x, header);
            }
        }
        window.attroff(A_BOLD);

        for i in 0..rows_to_show {
            let element = data[&[view[0] + i, column]]
                .chars()
                .take(column_width.1)
                .collect::<String>();
            width = usize::max(width, element.chars().count());
            window.mvaddstr(y + i as i32 * 2, x, element);
        }
        x += width as i32 + 3;
        vline_positions.push(x - 2);

        column += 1;
    }

    for position in &vline_positions {
        for y in 0..bottom_position {
            window.mvaddstr(y, *position, "│");
        }
    }
    #[rustfmt::skip]
    crossed_hline(window, y - 1, vline_positions[0], x - 1, "╞", "═", "╪", "╡", &vline_positions);
    for i in 0..rows_to_show - 1 {
        #[rustfmt::skip]
        crossed_hline(window, y + i as i32 * 2 + 1, vline_positions[0], x - 1, "├", "─", "┼", "┤", &vline_positions);
    }
    #[rustfmt::skip]
    crossed_hline(window, y + (rows_to_show - 1) as i32 * 2 + 1, vline_positions[0], x - 1, "└", "─", "┴", "┘", &vline_positions);

    match mode {
        Mode::Command => set_status(&window, format!(":{}", command)),
        Mode::Search => set_status(&window, format!("?{}", command)),
        _ => set_status(&window, status),
    }
}

fn crossed_hline(
    window: &Window,
    y: i32,
    x: i32,
    max_x: i32,
    left: &'static str,
    middle: &'static str,
    cross: &'static str,
    right: &'static str,
    crosses: &[i32],
) {
    for ix in x..max_x {
        let l = if ix == x {
            left
        } else if ix == max_x - 1 {
            right
        } else if crosses.contains(&ix) {
            cross
        } else {
            middle
        };
        window.mvaddstr(y, ix, l);
    }
}
