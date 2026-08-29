use hidapi::{HidApi, HidDevice, HidError};

use crate::battery::{self, BatteryInfo};
use crate::dpi::{self, Dpi};
use crate::firmware::{self, FirmwareInfo};

const VENDOR_ID: u16 = 0x1532;

#[derive(Debug)]
struct DeviceProfile {
    product_id: u16,
    interface_number: i32,
    usage_page: u16,
    usage: u16,
}

const DEVICE_PROFILES: &[DeviceProfile] = &[DeviceProfile {
    product_id: 0x00ab,
    interface_number: 0,
    usage_page: 0x01,
    usage: 0x02,
}];

pub(crate) struct RazerDevice {
    device: HidDevice,
    pub(crate) product_name: Option<String>,
}

impl RazerDevice {
    pub(crate) fn battery_info(&self) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
        battery::get(&self.device)
    }

    pub(crate) fn firmware_info(&self) -> Result<FirmwareInfo, Box<dyn std::error::Error>> {
        firmware::get(&self.device)
    }

    pub(crate) fn get_dpi(&self) -> Result<Dpi, Box<dyn std::error::Error>> {
        dpi::get(&self.device)
    }

    pub(crate) fn set_dpi(&self, x: u16, y: u16) -> Result<Dpi, Box<dyn std::error::Error>> {
        dpi::set(&self.device, x, y)
    }
}

fn profile_by_product_id(product_id: u16) -> Option<&'static DeviceProfile> {
    DEVICE_PROFILES
        .iter()
        .find(|profile| profile.product_id == product_id)
}

pub(crate) fn get_devices(api: &HidApi) -> Result<Vec<RazerDevice>, HidError> {
    let mut devices = Vec::new();
    for info in api.device_list() {
        if info.vendor_id() != VENDOR_ID {
            continue;
        }

        let Some(profile) = profile_by_product_id(info.product_id()) else {
            continue;
        };

        if info.interface_number() != profile.interface_number
            || info.usage_page() != profile.usage_page
            || info.usage() != profile.usage
        {
            continue;
        }

        let device = api.open_path(info.path())?;
        devices.push(RazerDevice {
            device,
            product_name: info.product_string().map(str::to_string),
        });
    }
    Ok(devices)
}
