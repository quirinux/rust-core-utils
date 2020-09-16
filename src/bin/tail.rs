#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
mod util;
use util::file_read_strategy::{FileReadStrategy};

use std::io::{self, Write};
use structopt::StructOpt;
use std::{thread, time};
use std::fmt;

#[derive(StructOpt, Debug)]
#[structopt(name = "tail", about = r"Print the last 10 lines of each FILE to standard output.
With more than one FILE, precede each with a header giving the file name.
With no FILE, or when FILE is -, read standard input.
")]
struct Opt {
    /// output the last K bytes; or use -c +K to output bytes starting with the Kth of each file
    #[structopt(short = "c", long = "bytes", default_value="0")]
    bytes: String,

    /// output appended data as the file grows;
    ///  an absent option argument means 'descriptor'
    ///  same as --follow=name --retry
    #[structopt(short, long,)]
    follow: bool,
               
    /// output the last K lines or use -n +K to output starting with the Kth
    #[structopt(short = "n", long = "lines", default_value="0")]
    lines: String,

    /// with --follow=name, reopen a FILE which has not
    ///  changed size after N iterations
    ///  to see if it has been unlinked or renamed
    ///  (this is the usual case of rotated log files);
    ///  with inotify, this option is rarely useful
    #[structopt(long = "max-unchanged-stats", default_value="5")]
    max_unchanged_stats: usize,
    
    /// with -f, terminate after process ID, PID dies
    #[structopt(long, default_value="0")]
    pid: usize,
        
    /// never print headers giving file names
    #[structopt(long, short)]
    quiet: bool,
    /// same as --quiet
    #[structopt(long)]
    silent: bool,

    /// keep trying to open a file if it is inaccessible
    #[structopt(long)]
    retry: bool,

    /// sleep for approximately N seconds between iterations;
    /// with inotify and --pid=P, check process P at
    /// least once every N seconds
    #[structopt(short = "s", long = "sleep-interval", default_value="1.0")]
    sleep: f64,

    /// always output headers giving file names
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
            self.follow = true;
        }
        if self.follow {
            self.retry = true;
        }
        if self.files.len() < 1 {
            self.quiet = true;
        }
    }
}

#[derive(Debug)]
struct Line {
    pub from_file: Option<String>,
    pub content: String,
}

#[derive(Debug)]
struct TailOption {
    pub read_strategy: FileReadStrategy,
    pub follow: bool,
    pub max_unchanged_stats: usize,
    pub pid: usize,
    pub quiet: bool,
    pub retry: bool,
    pub sleep: u64,
    pub verbose: bool,
    pub file: String,
    pub output_channel: crossbeam::channel::Sender<Line>,
}

impl TailOption {
    pub fn wait(&self) {
        thread::sleep(time::Duration::from_millis(self.sleep));
    }
}

macro_rules! good_togo {
    ($last_file_size:expr, $f:expr, $read_bunch:expr) => {{
        let mut ok = true;
        if !$f.is_stdin() {
            let cur_file_size = match $f.len() {
                Ok(s) => s,
                Err(e) => {
                    warn!("good_togo error found on file size => {}", e);
                    ok = false;
                    0
                },
            };
            debug!("last file size ({}) <=> current file size ({})", $last_file_size, cur_file_size);
            if cur_file_size < $last_file_size {
                debug!("File size is lower than previously, starting from zero");
                $read_bunch = 0;
            } else if cur_file_size == $last_file_size {
                debug!("File size hasn't changed, ignoring though");
                ok = false;                    
            }
            $last_file_size = cur_file_size;
        }
        ok
    }};
}

fn file_watcher(to: &mut TailOption){
    debug!("file_watcher => {:?}", to);
    let mut notify_error = true;
    let mut last_file_size: u64 = 0;

    if to.file == "-".to_string() {
        let content = "tail: warning: following standard input indefinitely is ineffective\n".to_string();
        to.output_channel.send(Line{ from_file: None, content: content });
    }

    'outter: loop {
        info!("file_watcher looping => {}", to.file.clone());
        let mut f = util::file::new(to.file.clone());
        let file_len = match f.len() {
            Ok(v) => v,
            _ => 0,
        } as usize;
        let start_pos = match to.read_strategy {
            FileReadStrategy::FromByte(v) => if file_len > v { v } else { file_len },
            FileReadStrategy::LastBytes(v) => if file_len > v { file_len - v } else { 0 },
            _ => 0,
        };
        info!("file_watcher start_pos => {}", start_pos);
        match f.prepare(start_pos){
            Err(e) => {
                if notify_error {
                    notify_error = false;
                    warn!("error found when trying to open file: {} - {}", to.file.clone(), e);
                    let content = format!("tail cannot open '{}' for reading: {}\n", to.file.clone(), e);
                    to.output_channel.send(Line{ from_file: None, content: content });
                }
                if to.retry{
                    to.wait();
                    continue
                } else {
                    break;
                }
            },
            _ => { },
        };
        info!("file read strategy => {:?}", to.read_strategy);
        let mut last_read = 0;
        match &to.read_strategy {
            FileReadStrategy::FromByte(v) => {
                let mut read_bunch = *v;
                if good_togo!(last_file_size, f, read_bunch) {
                    last_read = read_bunch;
                    loop {
                        match f.read_chunk(1) {
                            Some(b) => {
                                last_read +=1;
                                let content = String::from_utf8_lossy(&b).to_string();
                                to.output_channel.send(Line{ from_file: Some(to.file.clone()), content: content });
                            },
                            Nome => {
                                debug!("got none on read chunk");
                                to.read_strategy = FileReadStrategy::FromByte(last_read);
                                break;
                            },
                        }
                    }
                }
            },
            FileReadStrategy::LastBytes(v) => {
                let mut read_bunch = *v;
                if good_togo!(last_file_size, f, read_bunch) {
                    let (size, buffer): (usize, Vec<u8>) = f.last_bytes(read_bunch);                    
                    last_read = size;
                    to.read_strategy = FileReadStrategy::FromByte(last_read);                
                    let content = String::from_utf8_lossy(&buffer).to_string();
                    to.output_channel.send(Line{ from_file: Some(to.file.clone()), content: content });
                }                
            },
            FileReadStrategy::LastLines(v) => {
                let mut read_bunch = *v;
                if good_togo!(last_file_size, f, read_bunch) {
                    let (size, buffer): (usize, Vec<String>) = f.last_lines(read_bunch);                    
                    last_read = size;
                    to.read_strategy = FileReadStrategy::FromLine(last_read);                
                    let content = buffer.join("");
                    to.output_channel.send(Line{ from_file: Some(to.file.clone()), content: content });
                    if f.is_stdin() {
                        debug!("FromLine found NONE when reading from stdin, aborting");
                        break 'outter;
                    }
                }                
            },
            FileReadStrategy::FromLine(v) => {
                let mut read_bunch = *v;
                if good_togo!(last_file_size, f, read_bunch) {
                    f.walk_buffer_lines(read_bunch);
                    last_read = read_bunch;
                    loop {
                        match f.read_line() {
                            Some(b) => {
                                info!("FromLine read => {}", b.clone());
                                last_read +=1;
                                to.output_channel.send(Line{ from_file: Some(to.file.clone()), content: b });
                            },
                            Nome => {
                                debug!("got none on read line");
                                if f.is_stdin() {
                                    debug!("FromLine found NONE when reading from stdin, aborting");
                                    break 'outter;
                                }
                                to.read_strategy = FileReadStrategy::FromLine(last_read);
                                break;
                            },
                        }
                    }
                }
                
            },
            _ => {},
        }

        if !to.follow {
            debug!("not to follow - breaking the loop");
            break;
        };
        debug!("file_watcher, going to sleep for {}ms", to.sleep);
        to.wait();
    }
    info!("FileWatcher main loop broke, ending up the thread");
}

fn output_collector(rx: crossbeam::channel::Receiver<Line>, close: crossbeam::channel::Receiver<bool>) {
    let mut last_read = String::new();
    loop {
        for v in rx.try_iter(){
            trace!("loopping output_collector => {:?}", v);
            match v.from_file {
                Some(file) => {
                    if last_read != file {
                        println!("\n==>  {}  <==", file);
                        last_read = file;
                    }
                },
                _ => {},
            }
            print!("{}", v.content);
        }
        if close.try_iter().collect::<Vec<bool>>().len() > 0 {
            debug!("output_collector => recieve close message on close channel, quitting");
            break;
        }
    }
}

fn sleep_time(sleep: f64) -> u64 {
    (sleep * 1000.0) as u64
}

fn main() {
    let mut line_number = 0;
    let mut opt = Opt::from_args();
    opt.initialize();
    env_logger::from_env(env_logger::Env::default().default_filter_or("none")).init();
    //env_logger::init();
    
    let (s, r) = crossbeam::bounded(100);
    let (close_tx, close_rx) = crossbeam::bounded(1);

    let output_thread = thread::spawn(move || { output_collector(r, close_rx.clone()) });
    let mut file_watcher_pool = Vec::new();
        
    for file in opt.files {
        let sc = s.clone();
        let mut to = TailOption{
            read_strategy: FileReadStrategy::pick(opt.bytes.clone(), opt.lines.clone()),
            follow: opt.follow,
            max_unchanged_stats: opt.max_unchanged_stats,
            pid: opt.pid,
            quiet: opt.quiet,
            retry: opt.retry,
            sleep: sleep_time(opt.sleep),
            verbose: opt.verbose,
            file: file.clone(),
            output_channel: sc,
        };
        
        match to.read_strategy {
            FileReadStrategy::None(e) => {
                println!("tail: {}", e);
                continue;
            },
            _ => {},
        }

        debug!("{:?}", to);
        
        if opt.follow{
            file_watcher_pool.push(
                thread::spawn(move || {
                    file_watcher(&mut to);
                })
            );
        } else {
            file_watcher(&mut to);
        }
    }

    for t in file_watcher_pool {
        t.join();
    }

    close_tx.send(true);
    output_thread.join();
}
