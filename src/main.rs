use std::fmt::Write;
use systemstat::Platform;

const COLOUR: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

type StringRes = Result<String, Box<dyn std::error::Error>>;

fn format_data(key: &str, value: &str) -> String {
    format!("{COLOUR}{key}{RESET} {value}", key = key, value = value,)
}

fn get_ascii_art() -> Vec<String> {
    let bytes = [
        "20", "20", "20", "20", "20", "20", "20", "20", "5f", "20", "20", "20", "20", "20", "20",
        "20", "20", "20", "0a", "20", "20", "20", "20", "5f", "20", "28", "60", "2d", "60", "29",
        "20", "5f", "20", "20", "20", "20", "20", "0a", "20", "20", "2f", "60", "20", "27", "2e",
        "5c", "20", "2f", "2e", "27", "20", "60", "5c", "20", "20", "20", "0a", "20", "20", "60",
        "60", "27", "2d", "2e", "2c", "3d", "2c", "2e", "2d", "27", "60", "60", "20", "20", "20",
        "0a", "20", "20", "20", "20", "2e", "27", "2f", "2f", "76", "5c", "5c", "27", "2e", "20",
        "20", "20", "20", "20", "0a", "20", "20", "20", "28", "5f", "2f", "5c", "20", "22", "20",
        "2f", "5c", "5f", "29", "20", "20", "20", "20", "0a", "20", "20", "20", "20", "20", "20",
        "20", "27", "2d", "27", "20", "20", "20", "20", "20", "20", "20",
    ];

    bytes
        .split(|s| *s == "0a")
        .map(|s| {
            std::str::from_utf8(
                &s.iter()
                    .map(|c| u8::from_str_radix(c, 16).expect("invalid hexadecimal string"))
                    .collect::<Vec<u8>>(),
            )
            .expect("invalid utf-8 string")
            .to_string()
        })
        .collect::<Vec<String>>()
}

fn get_hostname() -> StringRes {
    Ok(format!(
        "{COLOUR}{user}{RESET}@{COLOUR}{host}{RESET}",
        user = std::env::var("USER")?,
        host = match std::env::var("HOSTNAME") {
            Ok(name) => name,
            Err(_) =>
                match std::str::from_utf8(&std::process::Command::new("hostname").output()?.stdout)
                {
                    Ok(name) => name.to_string().replace('\n', ""),
                    Err(_) => nix::sys::utsname::uname().nodename().to_string(),
                },
        },
    ))
}

fn get_os() -> StringRes {
    fn read_mac_release() -> StringRes {
        Ok(format!(
            "{} {}",
            std::str::from_utf8(
                &std::process::Command::new("sw_vers")
                    .arg("-productName")
                    .output()?
                    .stdout,
            )?
            .replace('\n', ""),
            match std::str::from_utf8(
                &std::process::Command::new("sw_vers")
                    .arg("-productVersion")
                    .output()?
                    .stdout,
            )?
            .split_once('.')
            .ok_or_else(|| simple_error::simple_error!("unrecognised macOS version"))?
            {
                ("11", _) => "Big Sur",
                ("12", _) => "Monterey",
                _ => "",
            }
        ))
    }

    fn read_lsb_release() -> StringRes {
        Ok(std::str::from_utf8(
            &std::process::Command::new("lsb_release")
                .arg("-sd")
                .output()?
                .stdout,
        )?
        .to_string())
    }

    fn read_os_release() -> StringRes {
        Ok(std::fs::read_to_string("/etc/os_release")?
            .split('\n')
            .find(|s| s.starts_with("PRETTY_NAME"))
            .ok_or_else(|| simple_error::simple_error!("unrecognised linux distro"))?
            .strip_prefix("PRETTY_NAME=")
            .ok_or_else(|| simple_error::simple_error!("unrecognised linux distro"))?
            .replace('"', ""))
    }

    match nix::sys::utsname::uname().sysname() {
        "Darwin" => Ok(format_data("", &read_mac_release()?)),
        "Linux" => Ok(format_data(
            "",
            &read_lsb_release().or_else(|_| read_os_release())?,
        )),
        _ => simple_error::bail!("unrecognised os"),
    }
}

fn get_shell() -> StringRes {
    Ok(format_data(
        "",
        std::env::var("SHELL")?
            .strip_prefix("/bin/")
            .ok_or_else(|| simple_error::simple_error!("unrecognised linux distro"))?,
    ))
}

fn format_uptime(time: std::time::Duration) -> StringRes {
    let uptime_seconds = time.as_secs();

    let uptime_days = uptime_seconds / (24 * 60 * 60);
    let uptime_hours = (uptime_seconds % 24 * 60 * 60) / (60 * 60);
    let uptime_minutes = (uptime_seconds % (60 * 60)) / 60;

    let mut display = String::new();
    if uptime_days > 0 {
        write!(display, "{}d ", uptime_days)?;
    }
    if uptime_hours > 0 {
        write!(display, "{}h ", uptime_hours)?;
    }
    if uptime_minutes > 0 {
        write!(display, "{}m", uptime_minutes)?;
    }

    Ok(format_data("", &display))
}

fn get_colours() -> (String, String) {
    (
        (30..38)
            .map(|i| format!("\x1b[{}m⬣", i))
            .collect::<Vec<String>>()
            .join(" "),
        format!(
            " {}",
            (90..98)
                .map(|i| format!("\x1b[{}m⬣", i))
                .collect::<Vec<String>>()
                .join(" ")
        ),
    )
}

// Simple system fetch tool written in Rust.
fn main() {
    let stat = systemstat::System::new();
    let ascii_art = get_ascii_art();

    let mut data_list: Vec<String> = Vec::new();

    if let Ok(value) = get_hostname() {
        data_list.push(value);
    };

    if let Ok(value) = get_os() {
        data_list.push(value);
    };

    if let Ok(value) = get_shell() {
        data_list.push(value);
    };

    if let Ok(value) = stat.uptime() {
        if let Ok(uptime) = format_uptime(value) {
            data_list.push(uptime);
        }
    };

    let colours = get_colours();
    data_list.push(colours.0);
    data_list.push(colours.1);

    print_left_to_right(ascii_art, data_list);
}

// print two vectors of strings side to side
fn print_left_to_right(left: Vec<String>, right: Vec<String>) {
    let left_len = left.len();
    let right_len = right.len();
    let max_len = left_len.max(right_len);

    for i in 0..max_len {
        if i < left_len {
            print!("{}", left[i]);
        }
        if i < right_len {
            print!("{}", right[i]);
        }
        println!();
    }
}
