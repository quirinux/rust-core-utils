
use std::io::{BufRead, BufReader, self};
use std::fs::{File, metadata};


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
        let md = match metadata(self.path.clone()) {
            Ok(m) => m,
            Err(e) => {
                warn!("couldn't determine metadata for => {}", self.path);
                return Err(e);
            },
        };
        debug!("Is {} a dir? {}", self.path, md.is_dir());
        Ok(md.is_dir())
    }
    
    fn open_buffer(&mut self) -> Result<(), io::Error> {
        if self.path != "-" {
            match self.is_dir() {
                Ok(f) => {
                    if f {
                        let error_msg = format!("Is a directory");
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, error_msg));
                    }
                },
                Err(e) => {
                    warn!("could'nt say if id_dir => {}", e);
                    return Err(e);
                },
            }
        }
        self.bufread = match self.path.as_ref() {
            "-" => {
                info!("opening stdin");
                Box::new(BufReader::new(io::stdin()))
            },
            _ => {
                let opened_file = match File::open(self.path.clone()) {
                    Ok(f) => f,
                    Err(e) => {
                        warn!("error found when opening file => {}", e);
                        return Err(e);
                    },
                };
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
            //match n {
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
                debug!("read chunk of size => {}", n);
                if n == 0 {
                    trace!("empty chunk found, returning None");
                    return None;
                }                
            },
            Err(e) => {
                debug!("read chunk error => {}", e);
                return None;
            },
        };
        Some(buffer)
    }

    pub fn is_text(&self) -> bool {
        self.is_text
    }
}
