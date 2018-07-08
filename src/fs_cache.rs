extern crate reqwest;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

pub fn write_cache(filename: &str, contents: &Vec<u8>) -> Result<(), String> {
    let mut file = match File::create(filename) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!("Failed to write cache: {:?}", e));
        }
    };

    file.write_all(&contents).unwrap();
    Ok(())
}

pub fn read_cache(filename: &str) -> Option<Vec<u8>> {
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            info!("File {} not found: {:?}", filename, e);
            return None;
        }
    };

    let mut buf_reader = BufReader::new(file);
    let mut contents: Vec<u8> = Vec::new();

    buf_reader.read_to_end(&mut contents).unwrap();

    info!("Read {} from local cache.", filename);

    Some(contents)
}

pub fn http_get_cached(key: &str, url: &str, update: bool) -> Result<Vec<u8>, String> {
    if !update {
        match read_cache(key) {
            Some(o) => return Ok(o),
            None => (),
        }
    }
    let mut response: reqwest::Response = match reqwest::get(url) {
        Ok(o) => o,
        Err(e) => return Err(format!("http request failed; {:?}", e)),
    };

    if response.status() != reqwest::StatusCode::Ok {
        return Err(format!("HTTP status code: {:?}", response.status()));
    }

    let mut data: Vec<u8> = Vec::new();
    match response.read_to_end(&mut data) {
        Err(e) => return Err(format!("Failed to read response: {:?}", e)),
        _ => (),
    }

    if let Err(e) = write_cache(key, &data) {
        error!("failed to write to cache: {:?}", e);
    }
    Ok(data)
}
