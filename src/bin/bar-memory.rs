use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::process::exit;

const WARN_THRESHOLD: f32 = 70.0;
const ERROR_THRESHOLD: f32 = 90.0;

const WARN_FORMAT: &'static str = "<span color=\"#d79921\">"; // neutral_yellow
const ERROR_FORMAT: &'static str = "<span color=\"#cc241d\">"; // neutral_red

const FA_MICROCHIP: &'static str = "\u{f2db}";

fn main() {
    let mut total_kbytes: Option<i32> = None;
    let mut available_kbytes: Option<i32> = None;

    let was_clicked = env::var("BLOCK_BUTTON")
        .map(|string| string == "1")
        .unwrap_or(false);

    let stats_file = File::open("/proc/meminfo").unwrap_or_else(|_| exit(1));

    for line in BufReader::new(stats_file).lines().filter_map(|r| r.ok()) {
        if line.starts_with("MemTotal:") {
            total_kbytes = kilobytes_in_line(&line);
        } else if line.starts_with("MemAvailable:") {
            available_kbytes = kilobytes_in_line(&line);
        }
    }

    match (total_kbytes, available_kbytes) {
        (Some(total_kb), Some(available_kb)) => print_stats(was_clicked, total_kb, available_kb),
        (None, Some(available_kb)) => println!("{} used", available_kb),
        _ => println!("mem unknown"),
    }
}

fn kilobytes_in_line(line: &str) -> Option<i32> {
    line.chars()
        .filter(|c| c.is_numeric())
        .collect::<String>()
        .parse()
        .ok()
}

fn print_stats(was_clicked: bool, total_kb: i32, available_kb: i32) {
    let percent_available = ((available_kb as f32) * 100.0 / (total_kb as f32)).round();
    let percent_used = 100.0 - percent_available;

    let (style_start, style_end) = match percent_used {
        ERROR_THRESHOLD...100.0 => (ERROR_FORMAT, "</span>"),
        WARN_THRESHOLD...ERROR_THRESHOLD => (WARN_FORMAT, "</span>"),
        _ => ("", ""),
    };

    if was_clicked {
        // Yes, gigabytes are in powers of 1000. GiB are in powers of 1024. 1v1 me irl.
        let gibibytes_total = total_kb as f32 / 1024.0 / 1024.0;
        let gibibytes_used = (total_kb - available_kb) as f32 / 1024.0 / 1024.0;

        println!(
            "{icon} {style_start}{used:.1}/{total:.1} GiB{style_end}",
            icon = FA_MICROCHIP,
            style_start = style_start,
            used = gibibytes_used,
            total = gibibytes_total,
            style_end = style_end
        );
    } else {
        println!(
            "{icon} {style_start}{percent:3}%{style_end}",
            icon = FA_MICROCHIP,
            style_start = style_start,
            percent = percent_used,
            style_end = style_end
        );
    }
}
