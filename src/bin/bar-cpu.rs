#[macro_use]
extern crate lazy_static;

extern crate regex;

use std::io::BufReader;
use std::io::prelude::*;
use std::process::{Command, Stdio, Child};

use regex::Regex;

const INTERVAL: &'static str = "3";
const FA_COG: &'static str = "\u{f013}";

const WARN_THRESHOLD: f32 = 70.0;
const ERROR_THRESHOLD: f32 = 90.0;
const HUNDRED: f32 = 100.0;

const WARN_FORMAT: &'static str = "<span color=\"#d79921\">"; // neutral_yellow
const ERROR_FORMAT: &'static str = "<span color=\"#cc241d\">"; // neutral_red

fn main() {
    print_usage(0.0);

    let process = Command::new("mpstat")
        .arg(INTERVAL)
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();

    match process {
        Ok(process) => stream_process_output(process),
        Err(error) => println!("ERROR: mpstat: {}", error),
    }
}

fn stream_process_output(mut process: Child) {
    if let Some(output) = process.stdout {
        for line in BufReader::new(output).lines() {
            let line = line.expect("Line included non-UTF-8 characters");
            process_line(&line);
        }
    } else {
        process.kill().expect("Could not kill process");
    }
}

fn process_line(line: &str) {
    lazy_static! {
        // 13:42:18     CPU    %usr   %nice    %sys %iowait    %irq   %soft  %steal  %guest  %gnice   %idle
        static ref RE: Regex = Regex::new(r#"(?x)
            all        # CPU
            (:?        # A group we don't care about
                \s+
                [\d.]+
            ){9}       # Refer to the list above; there are 9 groups between CPU and %idle
            \s+
            (?P<idle>[\d.]+)
        "#).expect("Failed to compile regexp");
    }

    if let Some(captures) = RE.captures(line) {
        let idle: f32 = captures["idle"].parse().unwrap_or(0.0);
        print_usage(100.0 - idle);
    }
}

fn print_usage(usage: f32) {
    let (style_start, style_end) = match usage {
        ERROR_THRESHOLD...HUNDRED => (ERROR_FORMAT, "</span>"),
        WARN_THRESHOLD...ERROR_THRESHOLD => (WARN_FORMAT, "</span>"),
        _ => ("", ""),
    };

    println!(
        "{icon} {style_start}{percent:>3}%{style_end}",
        icon = FA_COG,
        percent = usage.round(),
        style_start = style_start,
        style_end = style_end
    );
}
