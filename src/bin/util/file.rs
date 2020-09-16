use std::io::{BufRead, BufReader, self, Seek};
use std::fs::{File, metadata};

use std::collections::VecDeque;

pub fn new(path: String) -> FileDetail {
    FileDetail{
        path: path, 
        is_text: true,
        bufread: Box::new(BufReader::new(File::open(".").unwrap())),
        bufpos: 0,
    }
}

pub struct FileDetail {
    path: String,
    is_text: bool,
    bufread: Box<BufRead>,
    bufpos: usize,
}

impl FileDetail {
    fn is_dir(&self) -> Result<bool, io::Error> {
        let md = metadata(self.path.clone())?;
        trace!("Is {} a dir? {}", self.path, md.is_dir());
        Ok(md.is_dir())
    }
    
    fn open_buffer(&mut self, start_pos: usize) -> Result<(), io::Error> {
        if !self.is_stdin() {
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
                let mut opened_file = File::open(self.path.clone())?;
                self.bufpos = match opened_file.seek(std::io::SeekFrom::Start(start_pos as u64)) {
                    Ok(p) => p as usize,
                    Err(e) => {
                        error!("open_buffer, not able to seek: {}", e);
                        0
                    },
                };
                Box::new(BufReader::new(opened_file))
            },
        };
        Ok(())
    }

    pub fn prepare(&mut self, start_pos: usize) -> Result<(), io::Error> {
        self.open_buffer(start_pos)
    }
    
    pub fn read_line(&mut self) -> Option<String> {
        trace!("reading by line");
        let mut _line = String::new();
        match self.bufread.read_line(&mut _line) {
            Ok(i) => {
                debug!("read_line lenght => {}", i);
                self.bufpos += i;
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
                self.bufpos += n;
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
        if !self.is_stdin() {
            for _ in 0..bytes {
                self.read_chunk(1);
            }
        }
    }

    pub fn walk_buffer_lines(&mut self, lines: usize) {
        if !self.is_stdin() {
            for _ in 0..lines {
                self.read_line();
            }
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
        (size, bunch.iter().map(|r| r.to_string()).collect())
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn len(&self) -> Result<u64, io::Error> {
        if self.is_stdin() {
            let error_msg = format!("STDIN has no lenght");
            warn!("len => {} is stdin", self.path.clone());
            return Err(std::io::Error::new(std::io::ErrorKind::Other, error_msg));
        }
        let md = metadata(self.path.clone())?;
        Ok(md.len())
    }

    pub fn is_stdin(&self) -> bool {
        self.path() == "-".to_string()
    }

    pub fn buffer_position(&self) -> usize {
        self.bufpos
    }
}
