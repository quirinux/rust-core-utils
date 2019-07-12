use std::io::{BufRead, BufReader, self};
use std::fs::{File, metadata};

use std::collections::VecDeque;

pub fn new(path: String) -> FileDetail {
    FileDetail{
        path: path, 
        is_text: true,
        bufread: Box::new(BufReader::new(File::open(".").unwrap())),
    }
}

pub struct FileDetail {
    path: String,
    is_text: bool,
    bufread: Box<BufRead>,
}

impl FileDetail {
    fn is_dir(&self) -> Result<bool, io::Error> {
        let md = metadata(self.path.clone())?;
        trace!("Is {} a dir? {}", self.path, md.is_dir());
        Ok(md.is_dir())
    }
    
    fn open_buffer(&mut self) -> Result<(), io::Error> {
        if self.path != "-" {
            if self.is_dir()? {
                let error_msg = format!("Is a directory");
                warn!("open_buffer => {} is dir", self.path.clone());
                return Err(std::io::Error::new(std::io::ErrorKind::Other, error_msg));
            }
        }
        self.bufread = match self.path.as_ref() {
            "-" => {
                info!("opening stdin");
                Box::new(BufReader::new(io::stdin()))
            },
            _ => {
                let opened_file = File::open(self.path.clone())?;
                Box::new(BufReader::new(opened_file))
            },
        };
        Ok(())
    }

    pub fn prepare(&mut self) -> Result<(), io::Error> {
        self.open_buffer()
    }
    
    pub fn read_line(&mut self) -> Option<String> {
        trace!("reading by line");
        let mut _line = String::new();
        match self.bufread.read_line(&mut _line) {
            Ok(i) => {
                debug!("read_line lenght => {}", i);
                if i == 0 {
                    return None;
                }
            },
            Err(e) => {
                debug!("read_line error => {}", e);
                self.is_text = false;
                return None;
            }
        }
        trace!("returning some line");
        Some(_line)
    }

    pub fn read_chunk(&mut self, lenght: usize) -> Option<Vec<u8>> {
        debug!("reading by chunk size => {}", lenght);
        let mut buffer = vec![0u8; lenght];
        match self.bufread.read(&mut buffer.as_mut_slice()) {
            Ok(n) => {
                trace!("read chunk of size => {}", n);
                if n == 0 {
                    trace!("empty chunk found, returning None");
                    return None;
                }                
            },
            Err(e) => {
                warn!("read chunk error => {}", e);
                return None;
            },
        };
        Some(buffer)
    }

    pub fn is_text(&self) -> bool {
        self.is_text
    }

    pub fn walk_buffer_bytes(&mut self, bytes: usize) {
        trace!("walking buffer bytes => {} - {}", bytes, self.path.clone());
        for _ in 0..bytes {
            self.read_chunk(1);
        }
    }

    pub fn walk_buffer_lines(&mut self, lines: usize) {
        for _ in 0..lines {
            self.read_line();
        }
    }

    pub fn last_bytes(&mut self, bytes: usize) -> (usize, Vec<u8>) {
        let mut bunch: VecDeque<u8> = VecDeque::with_capacity(bytes);
        let mut size = 0;
        loop {
            let b = match self.read_chunk(1) {
                Some(s) => s, 
                None => break,
            };
            for i in b {
                bunch.push_back(i);
                size += 1;
            }
            if bunch.len() > bytes {
                bunch.pop_front();
            }
        }
        //bunch.shrink_to(0);
        (size, bunch.iter().map(|r| *r).collect())
    }

    pub fn last_lines(&mut self, lines: usize) -> (usize, Vec<String>) {
        let mut bunch: VecDeque<String> = VecDeque::with_capacity(lines);
        let mut size = 0;
        loop {
            let b = match self.read_line() {
                Some(s) => s, 
                None => break,
            };
            bunch.push_back(b);
            size += 1;
            if bunch.len() > lines {
                bunch.pop_front();
            }
        }
        //bunch.shrink_to(0);
        (size, bunch.iter().map(|r| r.to_string()).collect())
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn len(&self) -> Result<u64, io::Error> {
        let md = metadata(self.path.clone())?;
        Ok(md.len())
    }
}
