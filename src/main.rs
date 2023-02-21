mod data;

use crate::data::{get_colours, get_hostname, get_os, get_shell, get_uptime};

fn main() {
    for datum in [get_hostname(), get_os(), get_shell(), get_uptime()]
        .into_iter()
        .flatten()
    {
        println!("{datum}");
    }

    let colours = get_colours();
    println!("{}\n{}", colours.0, colours.1);

    println!();
}
