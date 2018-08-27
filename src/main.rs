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
extern crate openat;
extern crate rayon;
extern crate reqwest;
extern crate itertools;

extern crate actix_web;

#[macro_use]
extern crate diesel;

mod fs_cache;
mod git;
mod log_config;
mod nix;
mod nix_channel;
mod nix_patches;
mod nvd;
mod process;
mod process_db;
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
            SubCommand::with_name("report").about("Generate a new issue report file").args(&[
                Arg::with_name("update")
                    .short("u")
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
                Arg::with_name("pkglist")
                    .short("p")
                    .required(true)
                    .takes_value(true)
                    .help("path to the package list that should be used/generated"),
                Arg::with_name("filter")
                    .short("f")
                    .long("filter")
                    .takes_value(true)
                    .help("filter package list by string"),
            ]),
        )
        .subcommand(
            SubCommand::with_name("channel_report").about("Generates reports for all \"bumps\" in a channel").args(&[
                Arg::with_name("channel-name")
                .long("channel-name")
                .help("The name of the channel. e.g. nixos-18.03")
                .required(true)
                .takes_value(true),
                Arg::with_name("directory")
                .long("directory")
                .help("The output directory for the reports")
                .required(true)
                .takes_value(true)
            ])
        )
        .subcommand(
            SubCommand::with_name("process")
                .about("Process the output of the report command")
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
        )
        .subcommand(
            SubCommand::with_name("process-db")
            .about("Process the output of the report command and generate an sqlite database from it").args(&[
                Arg::with_name("dbfile")
                    .required(true)
                    .short("d")
                    .long("dbfile")
                    .takes_value(true)
                    .help("databae file path"),
                ]).subcommand(
                    SubCommand::with_name("import")
                    .about("Imports an report into the database")
                    .args(&[
                        Arg::with_name("source")
                            .required(true)
                            .multiple(true)
                            .takes_value(true)
                            .index(1)
                            .help("the JSON files to consume"),
                        ])
            ).subcommand(
                    SubCommand::with_name("serve")
                    .args(&[
                          Arg::with_name("listen_address")
                            .default_value("[::1]:8080")
                            .takes_value(true)
                            .help("The address & port to listen on")
                    ])
                    .about("serves the contents of the DB via HTTP."),
            )
        );
    app
}

fn main() {
    info!(crate_version!());
    let app = create_clap_app();
    let matches = &app.get_matches();

    if matches.is_present("debug") {
        log_config::configure(log::LevelFilter::Debug);
    } else {
        log_config::configure(log::LevelFilter::Info);
    }

    if let Some(matches) = matches.subcommand_matches("report") {
        let update = matches.is_present("update");
        let source_branch = matches.value_of("source").expect("Missing source arg");
        let output = matches.value_of("output").expect("Missing output arg");
        let pkglist = matches.value_of("pkglist").expect("Missing pkglist arg");

        let filter = if matches.is_present("FILTER") {
            Some(matches.value_of("filter").expect(
                "Missing FILTER parameter",
            ))
        } else {
            None
        };
        report::run_for_revision(update, output, pkglist, source_branch, filter);
    } else if let Some(matches) = matches.subcommand_matches("process") {
        let repo = matches.value_of("repo");
        let source = matches.value_of("source").expect("Missing source");

        process::run(source, repo).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("process-db") {
        let dbfile = matches.value_of("dbfile").expect("missing dbfile");
        if let Some(matches) = matches.subcommand_matches("import") {
            let source = matches.value_of("source").expect("missing source");
            process_db::import(dbfile, source).unwrap()
        } else if let Some(matches) = matches.subcommand_matches("serve") {
            let listen_address = matches.value_of("listen_address").unwrap();
            process_db::web::serve(listen_address, dbfile);
        }
    } else if let Some(matches) = matches.subcommand_matches("channel_report") {
        let output = matches.value_of("directory").expect(
            "missing directory argument",
        );
        let channel_name = matches.value_of("channel-name").expect(
            "missing channel_name argument",
        );
        report::run_for_channel(channel_name, output).unwrap();
    }
}
