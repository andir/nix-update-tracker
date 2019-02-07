extern crate serde_json;
use num_cpus;
use regex::Regex;
use std;
use std::collections::HashMap;
use std::process::Command;
use std::sync::mpsc::channel;
use threadpool::ThreadPool;

const GET_PATCHES_TMPL: &str = include_str!("get_patches.nix");

pub fn get_patches(pkg: &str, source: &str) -> Result<Vec<String>, String> {
    debug!("Retrieving patches for {} from {}", pkg, source);

    lazy_static! {
        static ref PKG_NAME_EXPR: Regex =
            Regex::new(r"^[^\${}]*$").expect("Regex compliation failed");
    }

    // I don't really care about handling this gracefully right now. Just don't want any funny
    // things to happen..
    assert!(PKG_NAME_EXPR.is_match(pkg));

    let nixexpr = GET_PATCHES_TMPL.replace("__pkg_name__", pkg);

    let args = [
        "eval",
        "-I",
        &format!("nixpkgs={}", source),
        "--json",
        &nixexpr,
    ];

    debug!("Executing: nix {}", args.join(" "));

    let output = match Command::new("nix").args(&args).output() {
        Ok(o) => o,
        Err(e) => return Err(format!("Failed to execute command {:}", e)),
    };

    if !output.status.success() {
        let output: &str = std::str::from_utf8(&output.stderr.as_slice()).unwrap();
        info!("failed to execute nix eval for {}: {}", pkg, output);
        return Ok(vec![]);
    }

    let output: &str = match std::str::from_utf8(&output.stdout.as_slice()) {
        Ok(o) => o,
        Err(e) => return Err(format!("Failed to convert to utf str: {:}", e)),
    };
    // Data: [["CVE-2017-1000231.patch","CVE-2017-1000232.patch"]]

    let results: Vec<String> = match serde_json::from_str(&output) {
        Ok(o) => o,
        Err(e) => return Err(format!("Failed to deserialize data: {:}", e)),
    };

    let results = remove_nix_references(&results);

    Ok(results)
}

fn remove_nix_references(patches: &[String]) -> Vec<String> {
    let results: Vec<String> = patches
        .iter()
        .map(|patch| {
            if patch.starts_with("/nix/store/") {
                match patch.find('-') {
                    Some(n) => String::from(&patch[(n + 1)..patch.len()]),
                    None => "".to_owned(),
                }
            } else {
                patch.clone()
            }
        })
        .filter(|patch| !patch.is_empty())
        .collect();
    results
}

pub fn get_cves_from_patches(patches: &[String]) -> Vec<String> {
    let results: Vec<String> = patches
        .iter()
        .map(|patch| patch.to_uppercase())
        .filter(|patch| patch.contains("CVE"))
        .map(|patch| {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"CVE-\d{2,}-\d{2,}").unwrap();
            }
            RE.find_iter(&patch)
                .map(|m| m.as_str().to_owned())
                .collect()
        })
        .flat_map(|s: Vec<String>| s.clone())
        .collect();
    results
}

pub fn get_multiple_patches(packages: &[String], source: &str) -> HashMap<String, Vec<String>> {
    let cpus = num_cpus::get();
    let pool = ThreadPool::new(cpus);

    let (tx, rx) = channel();
    for pkg in packages.iter() {
        let pkg = pkg.clone();
        let tx = tx.clone();
        let source = source.to_owned();
        pool.execute(move || {
            let ret = (pkg.to_owned(), get_patches(&pkg, &source));
            tx.send(ret).expect("blocking send unti consumer is ready.");
        });
    }

    let mut results = HashMap::new();
    for (pkg, patches) in rx.iter().take(packages.len()) {
        let patches = patches.unwrap_or_else(|_| vec![]);
        results.insert(pkg.to_owned(), patches.clone());
    }
    results
}

#[cfg(test)]
mod test {
    use super::get_cves_from_patches;
    #[test]
    fn test_get_cves_from_patches() {
        let patches: Vec<String> = vec![
            "CVE-2017-1000231.patch".to_owned(),
            "CVE-2017-1000232.patch".to_owned(),
        ];
        let results = get_cves_from_patches(&patches);
        assert_eq!(results, vec!["CVE-2017-1000231", "CVE-2017-1000232"]);
    }

    use super::remove_nix_references;
    #[test]
    fn test_remove_nix_references() {
        let patches: Vec<String> = vec![
            "".to_owned(),
            "/nix/store/".to_owned(),
            "foo-bar.patch".to_owned(),
            "/nix/store/8xwqx6kfxv55l9zqn0vqw5fwzqzsyqqb-pcf-introduce-driver.patch".to_owned(),
            "/nix/store/gskmkklqqraan5mhqqr2yp24w6g8ax13-pcf-config-long-family-names.patch"
                .to_owned(),
            "/nix/store/sm30zs0p22ah93w60z05pm8rcv2j7zri-disable-pcf-long-family-names.patch"
                .to_owned(),
            "/nix/store/mn053qsl9lbqyiy66f842rxggkhk2b2d-enable-table-validation.patch".to_owned(),
            "/nix/store/kw079xbkhd8gy08lspachg75mdaqsx08-cve-2017-8105.patch".to_owned(),
            "/nix/store/wwjz0w9wi2bz4ii39fx0zn0cxrjx98l0-cve-2017-8287.patch".to_owned(),
            "/nix/store/px6gwpr2l0xb640nrhlbrnghprslcsmm-enable-subpixel-rendering.patch"
                .to_owned(),
        ];
        let results = remove_nix_references(&patches);
        println!("patches: {:?}", results);
        for patch in results.iter() {
            assert!(
                !patch.starts_with("/nix/store"),
                "patch {} starts with /nix/store",
                patch
            );
            // assert that there are no single character pats (due to the /nix/store/ patch)
            assert!(patch.len() > 1);
        }
        for patch in vec![
            "pcf-introduce-driver.patch".to_owned(),
            "enable-subpixel-rendering.patch".to_owned(),
            "foo-bar.patch".to_owned(),
        ]
        .iter()
        {
            assert!(results.contains(patch), "patch {} is missing", patch);
        }
    }
}
