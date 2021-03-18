use std::env::current_dir;
use std::process::exit;

use clap::{App, AppSettings, Arg, SubCommand};

use talent_plan_rs::KvStore;
use talent_plan_rs::Result;

fn main() -> Result<()> {
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
    let mut store = KvStore::open(current_dir()?.as_path())?;
    match matches.subcommand() {
        ("set", Some(_matches)) => {
            store.set(_matches.value_of("KEY").expect("key miss").to_string(),
                      _matches.value_of("VALUE").expect("value miss").to_string())?;
        }
        ("get", Some(_matches)) => {
            let v = store.get(_matches.value_of("KEY").expect("key miss").to_string())
                .unwrap()
                .unwrap_or("Key not found".to_string());
            println!("{}", v);
        }
        ("rm", Some(_matches)) => {
            match store.remove(_matches.value_of("KEY").unwrap_or("key miss").to_string()) {
                Ok(()) => {}
                Err(_) => {
                    println!("Key not found");
                    exit(1);
                }
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
