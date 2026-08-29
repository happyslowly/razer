use hidapi::HidDevice;

use crate::protocol::RazerReport;
use crate::protocol::RazerReportParser;
use crate::transport::send_report;

pub(crate) struct FirmwareInfo {
    major: u8,
    minor: u8,
}

pub(crate) fn get(device: &HidDevice) -> Result<FirmwareInfo, Box<dyn std::error::Error>> {
    let transaction_id = 0x1f;

    let request = RazerReport::for_firmware(transaction_id);
    let response = send_report(device, &request, 60)?;
    let response =
        RazerReportParser::parse_firmware_info(&response.bytes[..response.len], transaction_id)?;

    Ok(FirmwareInfo {
        major: response.0,
        minor: response.1,
    })
}

impl std::fmt::Display for FirmwareInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Version: {}.{}", self.major, self.minor)
    }
}
