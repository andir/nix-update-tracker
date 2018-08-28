use chrono::prelude::DateTime;
use chrono::Utc;
use regex::Regex;
use reqwest;
use std::time::{Duration, UNIX_EPOCH};

error_chain!{
    foreign_links {
        ReqwestError(::reqwest::Error);
        ParseIntError(::std::num::ParseIntError);
    }
}

pub struct Channel {
    pub name: String,
    pub url: String,
}

#[derive(PartialEq, Debug)]
pub struct ChannelEntry {
    pub revision: String,
    pub commit_date: String,
    pub advance_date: String,
}

impl Channel {
    pub fn from_name(name: &str) -> Self {
        let url = format!("https://channels.nix.gsc.io/{}/history-v2", &name);
        Channel {
            name: String::from(name),
            url: url,
        }
    }

    pub fn entries(&self) -> Result<Vec<ChannelEntry>> {
        let mut response = reqwest::get(&self.url).chain_err(|| "failed to download channel")?;
        let content = response.text()?;
        let entries: Result<Vec<ChannelEntry>> = content
            .lines()
            .map(|l| ChannelEntry::from_line(l))
            .collect();
        entries
    }
}

impl ChannelEntry {
    pub fn from_line(line: &str) -> Result<Self> {
        let (revision, commit_date, advance_date) = {
            let parts: Vec<String> = line
                .splitn(3, ' ')
                .map(|s| String::from(s))
                .take(3)
                .collect();
            debug!("parts: {:?}", parts);
            (parts[0].clone(), parts[1].clone(), parts[2].clone())
        };

        let commit_ts = UNIX_EPOCH + Duration::from_secs(commit_date.parse()?);
        let advance_ts = UNIX_EPOCH + Duration::from_secs(advance_date.parse()?);

        let commit_date = DateTime::<Utc>::from(commit_ts);
        let advance_date = DateTime::<Utc>::from(advance_ts);

        lazy_static! {
            static ref RE: Regex = Regex::new("^[0-9a-f]{5,40}$").unwrap();
        }

        if !RE.is_match(&revision) {
            return Err("revision doesn't match pattern".into());
        }

        Ok(ChannelEntry {
            revision: revision,
            commit_date: commit_date.to_string(),
            advance_date: advance_date.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_v2() {
        let input = "08d245eb31a3de0ad73719372190ce84c1bf3aee 1528585739 1529013440";
        let output = super::ChannelEntry::from_line(input);
        if let Ok(o) = output {
            assert_eq!(
                o,
                super::ChannelEntry {
                    revision: "08d245eb31a3de0ad73719372190ce84c1bf3aee".to_string(),
                    commit_date: "2018-06-09 23:08:59 UTC".to_string(),
                    advance_date: "2018-06-14 21:57:20 UTC".to_string(),
                }
            );
        } else {
            assert!(false);
        }
    }
}
