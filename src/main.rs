extern crate threadpool;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate fs2;
extern crate md5;
extern crate num_cpus;
extern crate regex;
extern crate serde_json;
extern crate time;
#[macro_use]
extern crate error_chain;
extern crate git2;
#[macro_use]
extern crate serde_derive;
extern crate reqwest;
extern crate openat;

mod fs_cache;
mod git;
mod log_config;
mod nix;
mod nix_patches;
mod nvd;
mod process;
mod release_monitoring;
mod report;


fn create_clap_app<'a>() -> clap::App<'a, 'a> {
    use clap::{App, AppSettings, Arg, SubCommand};
    let app = App::new("rust-vuln-scanner")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(crate_version!())
        .author(crate_version!())
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("enable debug logging"),
        )
        .subcommand(
            SubCommand::with_name("run").about("Run a new scan").args(&[
                Arg::with_name("update")
                    .short("c")
                    .long("update")
                    .help("Update. Don't use cached data."),
                Arg::with_name("source")
                    .short("s")
                    .long("source")
                    .required(true)
                    .takes_value(true)
                    .help("nixpkgs source branch."),
                Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .required(true)
                    .takes_value(true)
                    .help("output file"),
                Arg::with_name("filter")
                    .short("f")
                    .long("filter")
                    .takes_value(true)
                    .help("filter package list by string"),
            ]),
        )
        .subcommand(
            SubCommand::with_name("process")
                .about("Process the output of the run command")
                .args(&[
                    Arg::with_name("repo")
                        .short("r")
                        .long("repo")
                        .takes_value(true)
                        .help("path to the checkout of the output repository"),
                    Arg::with_name("source")
                        .required(true)
                        .multiple(true)
                        .takes_value(true)
                        .index(1)
                        .help("the JSON file produced by the run command"),
                ]),
        );
    app
}

fn main() {
    info!(crate_version!());
    let app = create_clap_app();
    let matches = &app.get_matches();

    if matches.is_present("debug") {
        log_config::configure(log::LogLevelFilter::Debug);
    } else {
        log_config::configure(log::LogLevelFilter::Info);
    }

    if let Some(matches) = matches.subcommand_matches("run") {
        let update = matches.is_present("update");
        let source_branch = matches.value_of("source").expect("Missing source arg");
        let output = matches.value_of("output").expect("Missing output arg");

        let filter = if matches.is_present("FILTER") {
            Some(
                matches
                    .value_of("filter")
                    .expect("Missing FILTER parameter"),
            )
        } else {
            None
        };
        report::run(update, source_branch, output, filter);
    } else if let Some(matches) = matches.subcommand_matches("process") {
        let repo = matches.value_of("repo");
        let source = matches.value_of("source").expect("Missing source");

        process::run(source, repo).unwrap();
    }
}

