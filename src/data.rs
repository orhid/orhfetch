use std::fmt::Write;
use systemstat::Platform;

const COLOUR: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

type StringResult = Result<String, Box<dyn std::error::Error>>;

/* # pretty formatting */

fn format_data(key: &str, value: &str) -> String {
    format!(" {COLOUR}{key}{RESET} {value}", key = key, value = value)
}

fn format_uptime(time: std::time::Duration) -> StringResult {
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

/* # retrieving information */

/* ## hostname */

pub fn get_hostname() -> StringResult {
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

/* ## operating system */

fn read_mac_release() -> StringResult {
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
            ("13", _) => "Ventura",
            _ => "",
        }
    ))
}

fn read_lsb_release() -> StringResult {
    Ok(std::str::from_utf8(
        &std::process::Command::new("lsb_release")
            .arg("-sd")
            .output()?
            .stdout,
    )?
    .to_string())
}

fn read_os_release() -> StringResult {
    Ok(std::fs::read_to_string("/etc/os_release")?
        .split('\n')
        .find(|s| s.starts_with("PRETTY_NAME"))
        .ok_or_else(|| simple_error::simple_error!("unrecognised linux distro"))?
        .strip_prefix("PRETTY_NAME=")
        .ok_or_else(|| simple_error::simple_error!("unrecognised linux distro"))?
        .replace('"', ""))
}

pub fn get_os() -> StringResult {
    match nix::sys::utsname::uname().sysname() {
        "Darwin" => Ok(format_data("", &read_mac_release()?)),
        "Linux" => Ok(format_data(
            "",
            &read_lsb_release().or_else(|_| read_os_release())?,
        )),
        _ => simple_error::bail!("unrecognised os"),
    }
}

/* ## shell */

pub fn get_shell() -> StringResult {
    Ok(format_data(
        "",
        std::env::var("SHELL")?
            .strip_prefix("/bin/")
            .ok_or_else(|| simple_error::simple_error!("unrecognised linux distro"))?,
    ))
}

/* ## uptime */

pub fn get_uptime() -> StringResult {
    format_uptime(systemstat::System::new().uptime()?)
}

/* ## terminal colours */

pub fn get_colours() -> (String, String) {
    (
        (30..38)
            .map(|i| format!("\x1b[{i}m⬣"))
            .collect::<Vec<String>>()
            .join(" "),
        format!(
            " {}",
            (90..98)
                .map(|i| format!("\x1b[{i}m⬣"))
                .collect::<Vec<String>>()
                .join(" ")
        ),
    )
}
