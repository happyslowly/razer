mod battery;
mod cli;
mod device;
mod dpi;
mod firmware;
mod protocol;
mod transport;

use crate::cli::{Cli, Commands};
use clap::Parser;
use hidapi::HidApi;

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();

    match run(cli.command) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new()?;
    let razer_devices = device::get_devices(&api)?;
    if razer_devices.is_empty() {
        return Err("no supported Razer mouse found".into());
    }

    for device in razer_devices {
        if let Some(name) = &device.product_name {
            println!("{name}")
        }

        match command {
            Commands::Battery => println!("{}", device.battery_info()?),
            Commands::Firmware => println!("{}", device.firmware_info()?),
            Commands::Dpi { x, y } => {
                let dpi = match (x, y) {
                    (Some(x), Some(y)) => device.set_dpi(x, y),
                    (Some(x), None) => device.set_dpi(x, x),
                    (None, Some(_)) => unreachable!("Y cannot be provided without X"),
                    (None, None) => device.get_dpi(),
                }?;
                println!("{dpi}")
            }
        }
    }
    Ok(())
}
