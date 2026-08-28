mod battery;
mod device;
mod protocol;
mod transport;

use hidapi::HidApi;

use crate::battery::BatteryInfo;

const VENDOR_ID: u16 = 0x1532;

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new()?;
    let razer_devices = device::get_devices_by_vendor(&api, VENDOR_ID);
    if razer_devices.is_empty() {
        eprintln!("No Razer Mouse found");
        return Err("no supported Razer mouse found".into());
    }
    for info in razer_devices {
        #[cfg(debug_assertions)]
        dbg!(&info);

        let device = info.open(&api)?;

        #[cfg(debug_assertions)]
        dbg!("successfully open device");

        let battery_info = BatteryInfo::query(&device)?;

        if let Some(product_name) = info.product_name() {
            println!("{product_name}");
        }

        println!(
            "Battery: {:.2}% {}",
            battery_info.percentage(),
            if battery_info.charging {
                "(charging)"
            } else {
                ""
            }
        )
    }
    Ok(())
}
