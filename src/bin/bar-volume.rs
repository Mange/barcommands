//
// Runs pactl and gathers volume information about the default sink.
//
// It uses three different commands:
//  - pactl info
//      Lists the default sink
//  - pactl list sinks
//      Lists all information about all sinks
//  - pactl subscribe
//      Notifies us when something changes
//
// The program works by getting information about the currently selected sink and printing the
// volume. Then a the subscriber command is run and the current sink is refreshed every time the
// subscriber command has any output.
//

#[macro_use]
extern crate lazy_static;

extern crate regex;

use std::io::BufReader;
use std::io::prelude::*;
use std::process::{Command, Stdio};

use regex::Regex;

const FA_VOLUME_UP: &'static str = "\u{f028}";
const FA_VOLUME_DOWN: &'static str = "\u{f027}";
const FA_VOLUME_OFF: &'static str = "\u{f026}";
const FA_BAN: &'static str = "\u{f05e}";

fn main() {
    refresh();
    subscribe();
}

#[derive(Debug)]
struct Sink {
    number: i32,
    name: String,
    description: String,
    is_muted: bool,
    volume_percent: i32,
}

impl Default for Sink {
    fn default() -> Sink {
        Sink {
            number: -1,
            name: String::default(),
            description: String::default(),
            is_muted: true,
            volume_percent: 0,
        }
    }
}

impl Sink {
    fn is_valid(&self) -> bool {
        self.number >= 0 && self.name.len() > 0
    }

    fn icon(&self) -> &'static str {
        match (self.is_muted, self.volume_percent) {
            (true, _) => FA_BAN,
            (false, 0...10) => FA_VOLUME_OFF,
            (false, 10...50) => FA_VOLUME_DOWN,
            (false, _) => FA_VOLUME_UP,
        }
    }
}

fn parse_sinks(text: &str) -> Vec<Sink> {
    const NAME_MARKER: &'static str = "\tName: ";
    const DESCRIPTION_MARKER: &'static str = "\tDescription: ";
    const MUTE_MARKER: &'static str = "\tMute: ";
    const VOLUME_MARKER: &'static str = "\tVolume: ";

    lazy_static! {
        static ref SINK_RE: Regex = Regex::new(
            "^Sink #(?P<number>\\d+)"
        ).expect("Failed to compile regexp");

        static ref PERCENT_RE: Regex = Regex::new(
            "(?P<percent>\\d+)%"
        ).expect("Failed to compile regexp");
    }

    let mut sinks = vec![];
    let mut sink = Sink::default();

    for line in text.lines() {
        if SINK_RE.is_match(&line) {
            if sink.is_valid() {
                sinks.push(sink);
            }
            sink = Sink::default();

            let captures = SINK_RE.captures(line).unwrap();
            sink.number = captures["number"].parse().unwrap();
        } else if line.starts_with(NAME_MARKER) {
            sink.name = String::from(&line[NAME_MARKER.len()..]);
        } else if line.starts_with(DESCRIPTION_MARKER) {
            sink.description = String::from(&line[DESCRIPTION_MARKER.len()..]);
        } else if line.starts_with(MUTE_MARKER) {
            sink.is_muted = line.contains("yes");
        } else if line.starts_with(VOLUME_MARKER) {
            // Assume all channels are balanced... because I always have them balanced.
            // Instead of trying to read all volumes and deal with the line wrapped format, just
            // pick the first volume percent.
            if let Some(captures) = PERCENT_RE.captures(line) {
                sink.volume_percent = captures["percent"].parse().unwrap();
            }
        }
    }

    if sink.is_valid() {
        sinks.push(sink);
    }

    sinks
}

fn get_sinks() -> Vec<Sink> {
    let output = Command::new("pactl")
        .arg("list")
        .arg("sinks")
        .env("LC_ALL", "C")
        .output()
        .expect("Could not run pactl")
        .stdout;

    let stdout = String::from_utf8(output).unwrap();
    parse_sinks(&stdout)
}

fn get_default_sink_name() -> Option<String> {
    const SINK_MARKER: &'static str = "Default Sink: ";

    let output = Command::new("pactl")
        .arg("info")
        .env("LC_ALL", "C")
        .output()
        .expect("Could not run pactl")
        .stdout;

    let stdout = String::from_utf8(output).unwrap();
    for line in stdout.lines() {
        if line.starts_with(SINK_MARKER) {
            return Some(String::from(&line[SINK_MARKER.len()..]));
        }
    }

    None
}

fn subscribe() {
    lazy_static! {
        static ref SINK_EVENT_RE: Regex = Regex::new(
            "^Event '[^']*' on sink"
        ).expect("Failed to compile regexp");
    }

    let process = Command::new("pactl")
        .arg("subscribe")
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Could not run pactl");

    if let Some(output) = process.stdout {
        for line in BufReader::new(output).lines().flat_map(|l| l.ok()) {
            if SINK_EVENT_RE.is_match(&line) {
                refresh();
            }
        }
    }
}

fn refresh() {
    let sinks = get_sinks();
    let default_sink = get_default_sink_name();

    let sink = match default_sink {
        Some(name) => sinks.iter().find(|sink| sink.name == name),
        None => sinks.first(),
    };

    match sink {
        Some(sink) => {
            println!(
                "{icon} {percent:>3}%",
                icon = sink.icon(),
                percent = sink.volume_percent
            );
        }
        None => println!("ERR"),
    }
}
