use hidapi::{DeviceInfo, HidApi, HidDevice, HidError};
use std::ffi::CString;

#[derive(Debug)]
pub(crate) struct DeviceProfile {
    pub(crate) product_id: u16,
    pub(crate) interface_number: i32,
    pub(crate) usage_page: u16,
    pub(crate) usage: u16,
}

const DEVICE_PROFILES: &[DeviceProfile] = &[DeviceProfile {
    product_id: 0x00ab,
    interface_number: 0,
    usage_page: 0x01,
    usage: 0x02,
}];

#[derive(Debug)]
pub(crate) struct RazerDeviceInfo {
    path: CString,
    product_name: Option<String>,
}

impl RazerDeviceInfo {
    fn from(info: &DeviceInfo) -> Self {
        RazerDeviceInfo {
            path: info.path().to_owned(),
            product_name: info.product_string().map(str::to_string),
        }
    }

    pub(crate) fn open(&self, api: &HidApi) -> Result<HidDevice, HidError> {
        api.open_path(self.path.as_c_str())
    }

    pub(crate) fn product_name(&self) -> Option<&str> {
        self.product_name.as_deref()
    }
}

fn profile_by_product_id(product_id: u16) -> Option<&'static DeviceProfile> {
    DEVICE_PROFILES
        .iter()
        .find(|profile| profile.product_id == product_id)
}

pub(crate) fn get_devices_by_vendor(api: &HidApi, vendor_id: u16) -> Vec<RazerDeviceInfo> {
    let mut devices = Vec::new();
    for device in api.device_list() {
        if device.vendor_id() != vendor_id {
            continue;
        }

        let Some(profile) = profile_by_product_id(device.product_id()) else {
            continue;
        };

        if device.interface_number() == profile.interface_number
            && device.usage_page() == profile.usage_page
            && device.usage() == profile.usage
        {
            devices.push(RazerDeviceInfo::from(device))
        }
    }
    devices
}
