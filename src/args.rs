use csv::{Terminator, Trim};
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

#[derive(Debug)]
struct DelimiterError;

impl std::error::Error for DelimiterError {}

impl Display for DelimiterError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        "The delimiter must be a single ASCII character or an accepted special symbol.".fmt(f)
    }
}

fn delimiter(s: &str) -> Result<u8, DelimiterError> {
    match s {
        "nul" => Ok(0),
        "soh" => Ok(1),
        "stx" => Ok(2),
        "etx" => Ok(3),
        "eot" => Ok(4),
        "enq" => Ok(5),
        "ack" => Ok(6),
        "bel" => Ok(7),
        "bs" => Ok(8),
        "ht" => Ok(9),
        "lf" => Ok(10),
        "vt" => Ok(11),
        "ff" => Ok(12),
        "cr" => Ok(13),
        "so" => Ok(14),
        "si" => Ok(15),
        "dle" => Ok(16),
        "dc1" => Ok(17),
        "dc2" => Ok(18),
        "dc3" => Ok(19),
        "dc4" => Ok(20),
        "nak" => Ok(21),
        "syn" => Ok(22),
        "etb" => Ok(23),
        "can" => Ok(24),
        "em" => Ok(25),
        "sub" => Ok(26),
        "esc" => Ok(27),
        "fs" => Ok(28),
        "gs" => Ok(29),
        "rs" => Ok(30),
        "us" => Ok(31),
        "sp" => Ok(32),
        "exc" => Ok(33),
        "quo" => Ok(34),
        "hsh" => Ok(35),
        "dol" => Ok(36),
        "pct" => Ok(37),
        "amp" => Ok(38),
        "squ" => Ok(39),
        "lpr" => Ok(40),
        "rpr" => Ok(41),
        "ast" => Ok(42),
        "plu" => Ok(43),
        "com" => Ok(44),
        "hyp" => Ok(45),
        "dot" => Ok(46),
        "sl" => Ok(47),
        "col" => Ok(58),
        "sem" => Ok(59),
        "lt" => Ok(60),
        "eq" => Ok(61),
        "gt" => Ok(62),
        "que" => Ok(63),
        "at" => Ok(64),
        "lbr" => Ok(91),
        "bsl" => Ok(92),
        "rbr" => Ok(93),
        "car" => Ok(94),
        "und" => Ok(95),
        "tic" => Ok(96),
        "lbc" => Ok(123),
        "pip" => Ok(124),
        "rbc" => Ok(125),
        "til" => Ok(126),
        "del" => Ok(127),
        s if s.is_ascii() && s.len() == 1 => Ok(AsRef::<[u8]>::as_ref(s)[0]),
        _ => Err(DelimiterError),
    }
}

fn terminator(s: &str) -> Result<Terminator, DelimiterError> {
    if s == "crlf" {
        Ok(Terminator::CRLF)
    } else {
        Ok(Terminator::Any(delimiter(s)?))
    }
}

fn trim(s: &str) -> Trim {
    match s {
        "h" | "headers" => Trim::Headers,
        "f" | "fields" => Trim::Fields,
        "hf" | "fh" | "both" | "all" => Trim::All,
        _ => Trim::None,
    }
}

/// A Grid based Editor named Greg. Command line editor for CSV, TSV... and more?
///
/// # ASCII Characters
///
/// A number of arguments accept a single ASCII character. To facilitate choosing un-typable characters,
/// the following special sequences are accepted:
///
/// The first 32 hard-to-type ASCII characters (0-31, respectively)
///
///     nul soh stx etx eot enq ack bel
///     bs  ht  lf  vt  ff  cr  so  si
///     dle dc1 dc2 dc3 dc4 nak syn etb
///     can em  sub esc fs  gs  rs  us
///
/// And, though these symbols may work if you type them directly, they are given special sequences for convenience:
///         !   "   #   $   %   &   '   (   )   *   +   ,   -   .   /
///     sp  exc quo hsh dol pct amp squ lpr rpr ast plu com hyp dot sl
///
///     :   ;   <   =   >   ?   @
///     col sem lt  eq  gt  que at
///
///     [   \   ]   ^   _   `
///     lbr bsl rbr car und tic
///
///     {   |   }   ~   DEL
///     lbc pip rbc til del
#[derive(Debug, structopt::StructOpt)]
#[structopt(author)]
pub struct Args {
    /// The separator between columns. Must be a single ASCII character (See --help for more info). Default: ,
    #[structopt(short, long, default_value = ",", parse(try_from_str = delimiter))]
    pub separator: u8,
    /// Terminator of a record. Must be a single ASCII character, or the special value `crlf`, which accepts
    /// any of `\r`, `\n`, or `\r\n`. Default crlf
    #[structopt(short = "r", long, default_value = "crlf", parse(try_from_str = terminator))]
    pub terminator: Terminator,
    /// Lines beginning with this comment symbol are skipped. Comments are disabled by default. Must be a single
    /// ASCII character.
    #[structopt(short, long, parse(try_from_str = delimiter))]
    pub comment: Option<u8>,
    /// The number of rows of headers. Default: 0
    #[structopt(short = "H", long, default_value = "0")]
    pub headers: u8,
    /// Whether to trim leading/trailing whitespace from `headers`, `fields`, or both (`all`). Default: none
    #[structopt(short, long, default_value = "none", parse(from_str = trim))]
    pub trim: Trim,
    /// The quote character to use when parsing. Must be a single ASCII character. Default: "
    #[structopt(short, long, default_value = "\"", parse(try_from_str = delimiter))]
    pub quote: u8,
    /// Quotes are respected when parsing fields by default. This will disable that.
    #[structopt(short, long)]
    pub ignore_quotes: bool,
    /// Two quotation marks in a row are treated as an escaped (literal) quotation mark by default. This will
    /// disable that.
    #[structopt(short = "d", long)]
    pub ignore_double_quote: bool,
    /// Escape character, which can be used to escape quotation marks. By default, quotation marks can only be
    /// escaped via the double-quoting method. Follows the same syntax as delimiters.
    #[structopt(short="e", long, parse(try_from_str = delimiter))]
    pub quote_escape: Option<u8>,
    /// The string that should be output as "True" in a boolean context. Default: Yes
    #[structopt(short = "T", long = "true", default_value = "Yes")]
    pub true_value: String,
    /// The string that should be output for "False" in a boolean context. Default: No
    #[structopt(short = "F", long = "false", default_value = "No")]
    pub false_value: String,
    /// Path to the file to edit.
    #[structopt(parse(from_os_str))]
    pub file: PathBuf,
}
