use std::fs::File;
use std::io::{self, BufRead};
use log::debug;
use std::time::Instant;
use rev_buf_reader::RevBufReader;
use std::fs;

pub fn simple_tail(path: &str, n: usize) -> io::Result<Vec<String>> {
    let start = Instant::now();
    let file = File::open(path)?;

    let buf = RevBufReader::new(file);
    let lines = buf
        .lines()
        .take(n)
        .map(|l| l.expect("Could not parse line"))
        .collect();
    
    debug!("simple_tail took: {} ms", start.elapsed().as_millis());
    Ok(lines)
}

pub fn read_to_string(path: &str) -> io::Result<String> {
    let contents = fs::read_to_string(path)?;
    Ok(contents)
}