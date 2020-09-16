#[macro_use]
extern crate log;
extern crate env_logger;

extern crate structopt;

use structopt::StructOpt;
use std::io::{self, Write};

mod util;

#[derive(StructOpt, Debug)]
#[structopt(name = "cat", about = "concatenate files and print on the standard output")]
struct Opt {
    /// equivlaent to -vET
    #[structopt(short = "A", long = "show-all")]
    show_all: bool,

    /// number nonempty output lines, overrides -n
    #[structopt(short = "b", long = "number-nonblank")]
    number_nonempty: bool,
    
    /// equivalent to -vE
    #[structopt(short = "e")]
    enable_ve: bool,

    /// display $ at end off each line
    #[structopt(short = "E", long = "show-ends")]
    show_ends: bool,

    /// number all output lines
    #[structopt(long, short)]
    number: bool,

    /// suppress repeated empty output lines
    #[structopt(long, short)]
    squeeze_blank: bool,

    /// equivalent to -vT
    #[structopt(short = "t")]
    enable_vt: bool,

    /// display TAB characters as ^I
    #[structopt(short = "T", long = "show-tabs")]
    show_tabs: bool,

    /// (ignored)
    #[structopt(short = "u")]
    ignored: bool,

    /// use ^ and M- notation, except for LFD and TAB
    #[structopt(short = "v", long = "show-nonprinting")]
    show_nonprinting: bool,

    ///
    #[structopt(name = "FILES")]
    files: Vec<String>,
    
}

impl Opt {
    fn initialize(&mut self) {
        if self.show_all {
            self.show_nonprinting = true;
            self.show_tabs = true;
            self.show_ends = true;
        }

        if self.enable_vt {
            self.show_nonprinting = true;
            self.show_tabs = true;
        }

        if self.enable_ve {
            self.show_nonprinting = true;
            self.show_ends = true;
        }

        if self.files.len() <= 0 {
            self.files.push("-".to_string());
        }
        
    }
}

fn valid(line: String, mut blank: usize, squeeze: bool) -> (bool, usize) {
    let mut ret = true;
    if squeeze {
        if line.len() <= 0 {
            blank +=1;
        } else{
            blank = 0;
        }
        if blank > 1 {
            ret = false;
        }
    }
    return (ret, blank);
}

fn format_line (mut line: String, show_ends: bool, show_tabs: bool, number: bool, line_count: usize) -> String {
    if show_ends {
        line.push('$')
    }
    if show_tabs {
        line = line.replace("\t", "^I");
    }
    if number {
        line = format!("\t{}  {}", line_count, line);
    }
    line
}


fn main()  {
    env_logger::init();
    let mut opt = Opt::from_args();
    opt.initialize();
    info!("Working with options => {:?}", opt);
    
    let mut line_count = 0;
    let blank_line_count = 0;
    
    for file in opt.files {
        trace!("Processing file => {}", file);
        let mut f = util::file::new(file.clone());
        match f.prepare(0){
            Err(e) => {
                info!("error found while preparing file {} => {}", file, e);
                println!("cat: {}: {}", file, e);
                continue
            },
            _ => Some(0),
        };

        loop {
            let mut valid_line = false;
            let mut line = match f.read_line() {
                Some(line) => line,
                None => break,
            };
            line.pop(); // ignoring the new line feed
            let (valid_line, blank_line_count) = valid(line.clone(), blank_line_count, opt.squeeze_blank);
            if valid_line {
                line_count += 1;
                println!("{}", format_line(line.clone(), opt.show_ends, opt.show_tabs, opt.number, line_count));
            }
        }
        if !f.is_text() {
            debug!("Non text file found, trying by chunk");
            let size = 1024 * 60;
            let mut writter = io::stdout();
            match f.read_chunk(size) {
                Some(b) => writter.write_all(&b),
                _ => Ok(()),
            };
        }
        
        trace!("File processed => {}", file);
    }
}

