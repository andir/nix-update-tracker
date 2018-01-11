extern crate serde_json;
use log;
use std::process::Command;
use std::slice::SliceConcatExt;
use std;

pub fn get_patches(pkg: &str, source: &str) -> Result<Vec<String>, String> {
    debug!("Retrieving patches for {} from {}", pkg, source);

    let nixexpr = "(with import <nixpkgs> {}; (map (p: p.name)".to_owned() + &format!("{}.patches))", pkg);

    let args = ["eval", "-I",  &format!("nixpkgs={}", source), "--json", &nixexpr];

    info!("Executing: nix {}", args.join(" "));

    let output = match Command::new("nix")
        .args(&args)
        .output() {
        Ok(o) => o,
        Err(e) => return Err(format!("Failed to execute command {:}", e)),
    };

    if !output.status.success() {
        let output: &str = std::str::from_utf8(&output.stderr.as_slice()).unwrap();
        info!("failed to execute nix eval: {}",  output);
        return Ok(vec![]);
    }

    let output: &str = match std::str::from_utf8(&output.stdout.as_slice()) {
        Ok(o) => o,
        Err(e) => return Err(format!("Failed to convert to utf str: {:}", e)),
    };
    // Data: [["CVE-2017-1000231.patch","CVE-2017-1000232.patch"]]
    println!("Data: {}", output);

    let results: Vec<Vec<String>> = match serde_json::from_str(&output) {
        Ok(o) => o,
        Err(e) => return Err(format!("Failed to deserialize data: {:}", e)),
    };

    let results: Vec<String> = match results.get(0) {
        None => vec![],
        Some(o) => o.clone(),
    };
    
    Ok(results)
}

pub fn get_cves_from_patches(patches: &Vec<String>) -> Vec<String> {
    let results: Vec<String> = patches
        .iter()
        .filter(|patch| patch.to_lowercase().contains("cve"))
        .map(|patch| if patch.ends_with(".patch") {
            String::from(&patch[0..(patch.len() - 6)])
        } else {
            patch.clone()
        })
        .collect();

    results
}

#[cfg(test)]
mod test {
    use super::get_cves_from_patches;
    #[test]
    fn test_get_cves_from_patches() {
       let patches : Vec<String> = vec!["CVE-2017-1000231.patch".to_owned(),"CVE-2017-1000232.patch".to_owned()];
       let results = get_cves_from_patches(&patches);
       assert_eq!(results, vec!["CVE-2017-1000231", "CVE-2017-1000232"]);
    }
}
