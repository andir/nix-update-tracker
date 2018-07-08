extern crate reqwest;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::fs::metadata;
use std::time::Duration;
use std::time::UNIX_EPOCH;
use std::path::Path;
use time;

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

fn remote_newer_then_local(filename: &str, url: &str) -> bool {
    info!("Checking if the remote version is newer...");
    let local_ts = {
        match metadata(filename) {
            Ok(metadata) => {
                match metadata.modified() {
                    Ok(time) => time,
                    Err(e) => {
                        error!("Failed to read modified metadata of file {}: {}", filename, e);
                        return false
                    }
                }
            },
            Err(e) => {
                error!("Failed to read metadata of file {}: {}", filename, e);
                return false
            }
        }
    };

    let client = match reqwest::Client::new() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to create reqwest client. Assuming local version is new enough: {}", e);
            return false
        },
    };
    let mut request = match client.head(url) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to create HTTP HEAD request. Assuming local version is new enough: {}", e);
            return false;
        },
    };
    let res = match request.send() {
        Ok(response) => {
            info!("response: {:?}", response);
            if response.status() != reqwest::StatusCode::Ok {
                let headers = response.headers();
                headers.get::<reqwest::header::LastModified>().map(|last_modified| {
                    **last_modified > reqwest::header::HttpDate::from(local_ts)
                }).unwrap_or(true)
            } else {
                info!("Status code from for HEAD {} was {}. Assuming local is newer.", url, response.status());
                false
            }
        },
        Err(e) => {
            error!("Failed to run HEAD request. Assuming local file is newer. Remote might be unavailable right now: {}", e);
            false
        },
    };
    if res {
        info!("remote version is newer");
    } else {
        info!("local version is newer");
    }
    res
}

pub fn http_get_cached(key: &str, url: &str, update: bool) -> Result<Vec<u8>, String> {
    let download_file = |key: &str, url: &str|-> bool{
        if Path::new(key).exists() {
            remote_newer_then_local(key, url)
        } else { true }
    };

    if !update || (update && !download_file(key, url)) {
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
