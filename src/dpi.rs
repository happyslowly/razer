use hidapi::HidDevice;

use crate::protocol::RazerReport;
use crate::protocol::RazerReportParser;
use crate::transport::send_report;

const MIN_DPI: u16 = 100;
const MAX_DPI: u16 = 30_000;

pub(crate) struct Dpi {
    x: u16,
    y: u16,
}

pub(crate) fn get(device: &HidDevice) -> Result<Dpi, Box<dyn std::error::Error>> {
    let transaction_id = 0x1f;

    let request = RazerReport::for_get_dpi(transaction_id);
    let response = send_report(device, &request, 60)?;
    let response =
        RazerReportParser::parse_get_dpi(&response.bytes[..response.len], transaction_id)?;

    Ok(Dpi {
        x: response.0,
        y: response.1,
    })
}

pub(crate) fn set(device: &HidDevice, x: u16, y: u16) -> Result<Dpi, Box<dyn std::error::Error>> {
    if !is_valid(x, y) {
        return Err(format!("DPI must be between {MIN_DPI} and {MAX_DPI}").into());
    }

    let transaction_id = 0x1f;

    let request = RazerReport::for_set_dpi(transaction_id, x, y);
    let response = send_report(device, &request, 60)?;
    let response =
        RazerReportParser::parse_set_dpi(&response.bytes[..response.len], transaction_id)?;

    Ok(Dpi {
        x: response.0,
        y: response.1,
    })
}

fn is_valid(x: u16, y: u16) -> bool {
    let range = MIN_DPI..=MAX_DPI;
    range.contains(&x) && range.contains(&y)
}

impl std::fmt::Display for Dpi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DPI: X = {}, Y = {}", self.x, self.y)
    }
}
