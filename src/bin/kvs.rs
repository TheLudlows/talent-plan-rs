use std::process::exit;

use clap::{App, AppSettings, Arg, SubCommand};

use talent_plan_rs::kv::KvStore;

fn main() {
    let mut store = KvStore::new();
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name("set")
                .about("Set the value of a string key to a string")
                .arg(Arg::with_name("KEY").help("A string key").required(true))
                .arg(
                    Arg::with_name("VALUE")
                        .help("The string value of the key")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get the string value of a given string key")
                .arg(Arg::with_name("KEY").help("A string key").required(true)),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Remove a given key")
                .arg(Arg::with_name("KEY").help("A string key").required(true)),
        )
        .get_matches();

    match matches.subcommand() {
        ("set", Some(_matches)) => {
            store.set(_matches.value_of("KEY").expect("key miss").to_string(),
                      _matches.value_of("VALUE").expect("value miss").to_string());
        }
        ("get", Some(_matches)) => {
            let v = store.get(_matches.value_of("KEY").expect("key miss").to_string())
                .unwrap_or("".to_string());
            println!("{}", v);
        }
        ("rm", Some(_matches)) => {
            store.remove(_matches.value_of("KEY").expect("key miss").to_string());
        }
        _ => unreachable!(),
    }
}
