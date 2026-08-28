use hidapi::HidDevice;

use crate::protocol::RazerReport;
use crate::protocol::RazerReportParser;
use crate::transport::send_report;

pub(crate) struct BatteryInfo {
    level: u8,
    pub(crate) charging: bool,
}

impl BatteryInfo {
    pub(crate) fn query(device: &HidDevice) -> Result<Self, Box<dyn std::error::Error>> {
        let transaction_id = 0x1f;

        let charging_report = RazerReport::for_charging(transaction_id);
        let response = send_report(device, &charging_report, 60)?;
        let charging_response = RazerReportParser::parse_charging_status(
            &response.bytes[..response.len],
            transaction_id,
        )?;

        let battery_report = RazerReport::for_battery_level(transaction_id);
        let response = send_report(device, &battery_report, 60)?;
        let battery_response = RazerReportParser::parse_battery_level(
            &response.bytes[..response.len],
            transaction_id,
        )?;

        Ok(BatteryInfo {
            level: battery_response,
            charging: charging_response,
        })
    }

    pub(crate) fn percentage(&self) -> f64 {
        f64::from(self.level) / 255.0 * 100.0
    }
}
