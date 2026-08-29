use hidapi::HidDevice;

use crate::protocol::RazerReport;
use crate::protocol::RazerReportParser;
use crate::transport::send_report;

pub(crate) struct BatteryInfo {
    level: u8,
    charging: bool,
}

pub(crate) fn get(device: &HidDevice) -> Result<BatteryInfo, Box<dyn std::error::Error>> {
    let transaction_id = 0x1f;

    let charging_request = RazerReport::for_charging(transaction_id);
    let response = send_report(device, &charging_request, 60)?;
    let charging_response =
        RazerReportParser::parse_charging_status(&response.bytes[..response.len], transaction_id)?;

    let battery_request = RazerReport::for_battery_level(transaction_id);
    let response = send_report(device, &battery_request, 60)?;
    let battery_response =
        RazerReportParser::parse_battery_level(&response.bytes[..response.len], transaction_id)?;

    Ok(BatteryInfo {
        level: battery_response,
        charging: charging_response,
    })
}

impl BatteryInfo {
    pub(crate) fn percentage(&self) -> f64 {
        f64::from(self.level) / 255.0 * 100.0
    }
}

impl std::fmt::Display for BatteryInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let charging = if self.charging { " (charging)" } else { "" };
        write!(f, "Battery: {:.0}%{charging}", self.percentage(),)
    }
}
