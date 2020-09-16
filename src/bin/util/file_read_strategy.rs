

#[derive(Debug)]
pub enum FileReadStrategy {
    FromByte(usize),
    LastBytes(usize),
    FromLine(usize),
    LastLines(usize),
    None(String),
}

impl FileReadStrategy {
    pub fn pick(bytes: String, lines: String) -> FileReadStrategy {
        let mut from_byte: usize = 0;
        let mut last_bytes: usize = 0;
        let mut from_line: usize = 0;
        let mut last_lines: usize = 0;
        match FileReadStrategy::from_last_converter(bytes.clone()){
            Some((f, l)) => {
                from_byte = f;
                last_bytes = l;
            },
            None => {
                let error_msg = format!("{}: invalid number of bytes", bytes.clone());
                debug!("FileReadStrategy - Initialize error => {}", error_msg);
                return FileReadStrategy::None(error_msg); 
            }
        };
        match FileReadStrategy::from_last_converter(lines.clone()){
            Some((f, l)) => {
                from_line = f;
                last_lines = l;
            },
            None => {
                let error_msg = format!("{}: invalid number of lines", lines.clone());
                debug!("FaileReadStrategy - Initialize error => {}", error_msg);
                return FileReadStrategy::None(error_msg); 
            }
        };
        
        if from_byte > 0 { return FileReadStrategy::FromByte(from_byte); }
        if last_bytes > 0 { return FileReadStrategy::LastBytes(last_bytes); }
        if from_line > 0 { return FileReadStrategy::FromLine(from_line); }
        if last_lines > 0 { return FileReadStrategy::LastLines(last_lines); }
        // default
        FileReadStrategy::LastLines(10) 
    }
    
    fn from_last_converter(size: String) -> Option<(usize, usize)> {
        let mut from: usize = 0;
        let mut last: usize = 0;
        let parsed = match size.parse::<usize>() {
            Ok(p) => p,
            Err(_) => return None,
        };
        if size.starts_with("+") {
            from = parsed;
            last = 0;
        } else{
            from = 0;
            last = parsed;
        }
        Some((from, last))
    }

}
