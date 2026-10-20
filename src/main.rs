// SPDX-License-Identifier: GPL-3.0-or-later
//! `kindle-series-enabler`: shows the "Group series" switch in Settings
//! where Amazon doesn't offer it. Prints plain lines; the shell around it
//! decides where they go (ADR 0002).

use std::path::PathBuf;
use std::process::ExitCode;

use kse::appreg::{self, Appreg};
use kse::commands::{self, Change};
use kse::grouping::{self, Status};
use kse::state;

const USAGE: &str = "\
usage: kindle-series-enabler [COMMAND] [--appreg PATH] [--state DIR]
                             [--commands FILE]...

apply       (the default) add this Kindle's marketplace to the series
            grouping list, if it isn't there, so Settings shows the
            Group series switch (ADR 0001)
status      show the values involved, and the ;series command
register    add the ;series search command to each command file
unregister  remove ;series where it's ours
restore     put the list back as it was before apply's last write, if
            it still holds that write (uninstall, ADR 0003)

--appreg    the database (default /var/local/appreg.db)
--state     where apply records its write (default
            /var/local/kindle-series-enabler)
--commands  a search-command file; repeat for several (default: the
            three the firmware has used)";

struct Options {
    command: String,
    appreg: PathBuf,
    state: PathBuf,
    commands: Vec<PathBuf>,
}

fn parse() -> Result<Option<Options>, lexopt::Error> {
    use lexopt::prelude::*;
    let mut command = None;
    let mut appreg = PathBuf::from(appreg::DEFAULT_PATH);
    let mut state = PathBuf::from(state::DEFAULT_DIR);
    let mut commands = Vec::new();
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Value(v) if command.is_none() => command = Some(v.string()?),
            Long("appreg") => appreg = parser.value()?.into(),
            Long("state") => state = parser.value()?.into(),
            Long("commands") => commands.push(parser.value()?.into()),
            Long("help") | Short('h') => return Ok(None),
            _ => return Err(arg.unexpected()),
        }
    }
    if commands.is_empty() {
        commands = commands::DEFAULT_FILES.iter().map(PathBuf::from).collect();
    }
    Ok(Some(Options {
        command: command.unwrap_or_else(|| "apply".to_owned()),
        appreg,
        state,
        commands,
    }))
}

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> kse::Result<()> {
    let Some(o) = parse().map_err(|e| format!("{e}\n{USAGE}"))? else {
        println!("{USAGE}");
        return Ok(());
    };
    let open = |writable| {
        Appreg::open(&o.appreg, writable).map_err(|e| format!("{}: {e}", o.appreg.display()))
    };
    match o.command.as_str() {
        "apply" => println!("{}", grouping::apply(&open(true)?, &o.state)?),
        "restore" => println!("{}", grouping::restore(&open(true)?, &o.state)?),
        "status" => {
            println!("{}", Status::read(&open(false)?, &o.state)?);
            for (path, entry) in commands::check(&o.commands) {
                match entry {
                    Ok(entry) => println!("{}: {entry}", path.display()),
                    Err(e) => println!("{e}"),
                }
            }
        }
        "register" => report(commands::register(&o.commands)?),
        "unregister" => report(commands::unregister(&o.commands)?),
        other => return Err(format!("unknown command {other:?}\n{USAGE}").into()),
    }
    Ok(())
}

fn report(changes: Vec<(PathBuf, Change)>) {
    for (path, change) in changes {
        println!("{}: {change}", path.display());
    }
}
