use crate::protocol::{FEATURE_REPORT_SIZE, FeatureReportResponse, RazerReport};
use hidapi::{HidDevice, HidError};
use std::{thread, time::Duration};

pub(crate) fn send_report(
    device: &HidDevice,
    request: &RazerReport,
    wait_ms: u64,
) -> Result<FeatureReportResponse, HidError> {
    let feature_report = request.to_feature_report();
    device.send_feature_report(&feature_report)?;

    thread::sleep(Duration::from_millis(wait_ms));

    let mut response = [0u8; FEATURE_REPORT_SIZE];
    let received = device.get_feature_report(&mut response)?;

    #[cfg(debug_assertions)]
    {
        dbg!(received);
        dbg!(&response[..received]);
    }

    Ok(FeatureReportResponse {
        bytes: response,
        len: received,
    })
}
