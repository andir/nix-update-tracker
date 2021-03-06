extern crate brotli;
extern crate reqwest;
use std::fs::metadata;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::Path;

pub fn write_cache(filename: &str, contents: &[u8]) -> Result<(), String> {
    let mut file = match File::create(filename) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!("Failed to write cache: {:?}", e));
        }
    };

    if filename.ends_with(".br") {
        // brotli compression requested
        match brotli::CompressorWriter::new(&mut file, 4096, 9, 22).write_all(&contents) {
            Ok(_n) => (),
            Err(e) => {
                return Err(format!("failed to write compressed content: {:?}", e));
            }
        }
    } else {
        match file.write_all(&contents) {
            Ok(_n) => (),
            Err(e) => {
                return Err(format!("failed to write content: {:?}", e));
            }
        }
    };

    Ok(())
}

pub fn read_cache(filename: &str) -> Option<Vec<u8>> {
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            info!("File {} not found: {:?}", filename, e);
            return None;
        }
    };
    let mut contents: Vec<u8> = Vec::new();
    if filename.ends_with(".br") {
        let mut r = BufReader::new(brotli::Decompressor::new(&mut file, 4096));
        r.read_to_end(&mut contents).unwrap();
    } else {
        BufReader::new(file).read_to_end(&mut contents).unwrap();
    }

    info!("Read {} from local cache.", filename);

    Some(contents)
}

fn remote_newer_then_local(filename: &str, url: &str) -> bool {
    info!("Checking if the remote version is newer...");
    let local_ts = {
        match metadata(filename) {
            Ok(metadata) => match metadata.modified() {
                Ok(time) => time,
                Err(e) => {
                    error!(
                        "Failed to read modified metadata of file {}: {}",
                        filename, e
                    );
                    return false;
                }
            },
            Err(e) => {
                error!("Failed to read metadata of file {}: {}", filename, e);
                return false;
            }
        }
    };

    let client = reqwest::Client::new();
    let response = match client.head(url).send() {
        Ok(r) => r,
        Err(e) => {
            error!(
                "Failed to send HTTP HEAD request. Assuming local version is new enough: {}",
                e
            );
            return false;
        }
    };

    debug!("response: {:?}", response);
    let res = match response.status() {
        reqwest::StatusCode::OK => {
            let headers = response.headers();
            match headers
                .get("Last-Modified")
                .map(|last_modified|
                     last_modified.to_str()
                    .map(|last_modified| httpdate::parse_http_date(last_modified))
                )
            {
                Some(Ok(Ok(last_modified))) => {
                    debug!(
                        "last_modified {:?} > local_ts {:?}: {}",
                        last_modified,
                        local_ts,
                        last_modified > local_ts
                    );
                    last_modified > local_ts
                }
                _ => true,
            }
        }
        _ => {
            info!(
                "Status code for HEAD from {} was {}. Assuming local is newer.",
                url,
                response.status()
            );
            false
        }
    };
    if res {
        info!("remote version is newer");
    } else {
        info!("local version is newer");
    }
    res
}

pub fn http_get_cached(key: &str, url: &str, update: bool) -> Result<Vec<u8>, String> {
    let should_download_file = |key: &str, url: &str| -> bool {
        if Path::new(key).exists() {
            remote_newer_then_local(key, url)
        } else {
            true
        }
    };

    if !update || !should_download_file(key, url) {
        if let Some(o) = read_cache(key) {
            return Ok(o);
        } else {
            info!("Filling local cache with {}.", url);
        }
    }

    let mut response: reqwest::Response = match reqwest::get(url) {
        Ok(o) => o,
        Err(e) => return Err(format!("http request failed; {:?}", e)),
    };

    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("HTTP status code: {:?}", response.status()));
    }

    let mut data: Vec<u8> = Vec::new();
    if let Err(e) = response.read_to_end(&mut data) {
        return Err(format!("Failed to read response: {:?}", e));
    }

    if let Err(e) = write_cache(key, &data) {
        error!("failed to write to cache: {:?}", e);
    }
    Ok(data)
}
