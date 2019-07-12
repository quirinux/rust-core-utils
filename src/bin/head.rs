#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
mod util;

use std::io::{self, Write};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "head", about = "output the first part of files")]
struct Opt {
    /// print the first K bytes of each file; with the leading '-', print all but the last K bytes of each file
    #[structopt(short = "c", long = "bytes", default_value="0")]
    bytes: usize,

    /// print the first K lines instead of the first 10; with the leading '-', print all but the last K lines of each file
    #[structopt(short = "n", long = "lines", default_value="10")]
    lines: usize,
    
    /// never print headers giving file names
    #[structopt(long, short)]
    quiet: bool,

    /// same as --quiet
    #[structopt(long)]
    silent: bool,

    /// always print headers giving file names
    #[structopt(long, short)]
    verbose: bool,

    ///
    #[structopt(name = "FILES")]
    files: Vec<String>,

}

impl Opt {
    fn initialize(&mut self) {
        self.silent = !self.verbose;
        self.quiet = self.silent;
        if self.files.len() <= 0 {
            self.files.push("-".to_string());
        }
    }
}

fn readprint_chunk(file: &mut util::file::FileDetail, buffer_size: usize) {
    let b = file.read_chunk(buffer_size);
    if b.is_some(){
        let mut writter = io::stdout();
        writter.write_all(&b.unwrap());
    }        
}

fn main() {
    let mut line_number = 0;
    let mut opt = Opt::from_args();
    opt.initialize();
    env_logger::init();
    
    for file in opt.files {
        let mut f = util::file::new(file.clone());
        if opt.verbose {
            println!("==> {} <==", file);
        }
        
        match f.prepare(){
            Err(e) => {
                info!("error found while preparing file {} => {}", file, e);
                println!("head: cannot open '{}' for reading: {}", file, e);
                continue
            },
            _ => Some(0),
        };
        if opt.bytes > 0 {
            readprint_chunk(&mut f, opt.bytes);
        } else {
            loop {
                match f.read_line() {
                    Some(line) => print!("{}", line),
                    None => break,
                }
                if !f.is_text() { break; }
                line_number += 1;
                if line_number > opt.lines { break; }
            }
            if !f.is_text() {
                let size = 1024 * 60;
                readprint_chunk(&mut f, size);
            }
        }
    }
}
