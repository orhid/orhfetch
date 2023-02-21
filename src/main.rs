use std::fmt::Write;
use systemstat::Platform;

const COLOUR: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

type StringRes = Result<String, Box<dyn std::error::Error>>;

fn format_data(key: &str, value: &str) -> String {
    format!(" {COLOUR}{key}{RESET} {value}", key = key, value = value,)
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

    // print_left_to_right(ascii_art, data_list);
    for s in data_list {
        println!("{}", s);
    }
    println!();
}
