extern crate structopt;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "echo", about = "Display a line of text")]
struct Opt {
    /// do not output the trailing newline
    #[structopt(short = "n")]
    disable_new_line: bool,

    /// enable interpretation of backslash escape
    #[structopt(short = "e")]
    enable_escape: bool,
    
    /// disable interpretation of backslash escape (default)
    #[structopt(short = "E")]
    disable_escape: bool,

    ///
    #[structopt(name = "STRING")]
    strings: Vec<String>,
    
}

fn apply_escapes(line: String) -> String {
    line
        .replace("\\t", "\t")
        .replace("\\n", "\n")
        .replace("\\a", "\x07")
        .replace("\\b", "\x08")
        .replace("\\e", "\x1b")
        .replace("\\f", "\x0c")
        .replace("\\r", "\r")
        .replace("\\v", "\x0b")
        .replace("\\", "\\")
        .split("\\c").next().unwrap().to_string()
}

fn main() {
    let opt = Opt::from_args();
    let mut output_line = opt.strings.join(" ");

    if !opt.disable_escape {
        output_line = apply_escapes(output_line);
    }

    print!("{}", output_line);
    if !opt.disable_new_line {
        print!("\n")
    }
}

