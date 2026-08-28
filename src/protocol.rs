pub(crate) const REPORT_SIZE: usize = 90;
pub(crate) const FEATURE_REPORT_SIZE: usize = REPORT_SIZE + 1;
pub(crate) const RAZER_FEATURE_REPORT_ID: u8 = 0x00;
const STATUS: usize = 0;
const TRANSACTION_ID: usize = 1;
const REMAINING_PACKETS_HIGH: usize = 2;
const REMAINING_PACKETS_LOW: usize = 3;
const PROTOCOL_TYPE: usize = 4;
const DATA_SIZE: usize = 5;
const COMMAND_CLASS: usize = 6;
const COMMAND_ID: usize = 7;
const ARGUMENTS_START: usize = 8;
const CRC: usize = 88;
const RESERVED: usize = 89;

const ARGUMENTS_CAPACITY: usize = CRC - ARGUMENTS_START;

fn calculate_crc_by_bytes(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0, |crc, byte| crc ^ byte)
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ProtocolError {
    InvalidLength(usize),
    InvalidReportId(u8),
    InvalidCrc { expected: u8, actual: u8 },
    InvalidDataSize(u8),
    UnexpectedTransactionId(u8),
    UnexpectedCommand { class: u8, id: u8 },
    Busy,
    Failed(u8),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength(len) => {
                write!(
                    f,
                    "invalid report length: expected {FEATURE_REPORT_SIZE}, received {len}"
                )
            }
            Self::InvalidReportId(id) => {
                write!(f, "invalid report ID: {id:#04x}")
            }
            Self::InvalidCrc { expected, actual } => {
                write!(
                    f,
                    "CRC mismatch: expected {expected:#04x}, received {actual:#04x}"
                )
            }
            Self::InvalidDataSize(size) => {
                write!(f, "invalid data size: {size}")
            }
            Self::UnexpectedTransactionId(id) => {
                write!(f, "unexpected transaction ID: {id:#04x}")
            }
            Self::UnexpectedCommand { class, id } => {
                write!(f, "unexpected command: class={class:#04x}, id={id:#04x}")
            }
            Self::Busy => write!(f, "device is busy"),
            Self::Failed(status) => {
                write!(f, "device returned failure status {status:#04x}")
            }
        }
    }
}

impl std::error::Error for ProtocolError {}

#[derive(Debug, Clone)]
pub(crate) struct RazerReport {
    bytes: [u8; REPORT_SIZE],
}

impl RazerReport {
    pub(crate) fn new(
        transaction_id: u8,
        command_class: u8,
        command_id: u8,
        data_size: u8,
    ) -> Self {
        let mut report = Self {
            bytes: [0; REPORT_SIZE],
        };

        report.bytes[STATUS] = 0x00;
        report.bytes[TRANSACTION_ID] = transaction_id;
        report.bytes[REMAINING_PACKETS_HIGH] = 0x00;
        report.bytes[REMAINING_PACKETS_LOW] = 0x00;
        report.bytes[PROTOCOL_TYPE] = 0x00;
        report.bytes[DATA_SIZE] = data_size;
        report.bytes[COMMAND_CLASS] = command_class;
        report.bytes[COMMAND_ID] = command_id;
        report.bytes[RESERVED] = 0x00;

        report.update_crc();
        report
    }

    pub(crate) fn to_feature_report(&self) -> [u8; FEATURE_REPORT_SIZE] {
        let mut feature_report = [0; FEATURE_REPORT_SIZE];
        feature_report[0] = RAZER_FEATURE_REPORT_ID;
        feature_report[1..].copy_from_slice(&self.bytes);
        feature_report
    }

    pub(crate) fn for_battery_level(transaction_id: u8) -> Self {
        Self::new(
            transaction_id,
            0x07, // Battery/power command class
            0x80, // Get battery level
            0x02, // Response contains two argument bytes
        )
    }

    pub(crate) fn for_charging(transaction_id: u8) -> Self {
        Self::new(
            transaction_id,
            0x07, // Battery/power command class
            0x84, // Get charging status
            0x02, // Response contains two argument bytes
        )
    }

    /*
    pub(crate) fn for_firmware(transaction_id: u8) -> Self {
        Self::new(transaction_id, 0x00, 0x81, 0x02)
    }
    */

    fn calculate_crc(&self) -> u8 {
        calculate_crc_by_bytes(&self.bytes[2..88])
    }

    fn update_crc(&mut self) {
        self.bytes[CRC] = self.calculate_crc();
    }
}

#[derive(Debug)]
pub(crate) struct FeatureReportResponse {
    pub(crate) bytes: [u8; FEATURE_REPORT_SIZE],
    pub(crate) len: usize,
}

pub(crate) struct RazerReportParser;

impl RazerReportParser {
    pub(crate) fn parse(
        feature_report: &[u8],
        expected_transaction_id: u8,
        expected_class: u8,
        expected_id: u8,
        expected_data_size: usize,
    ) -> Result<&[u8], ProtocolError> {
        if feature_report.len() != FEATURE_REPORT_SIZE {
            return Err(ProtocolError::InvalidLength(feature_report.len()));
        }

        if feature_report[0] != RAZER_FEATURE_REPORT_ID {
            return Err(ProtocolError::InvalidReportId(feature_report[0]));
        }

        let report = &feature_report[1..];

        let expected_crc = calculate_crc_by_bytes(&report[2..88]);
        let actual_crc = report[88];
        if expected_crc != actual_crc {
            return Err(ProtocolError::InvalidCrc {
                expected: expected_crc,
                actual: actual_crc,
            });
        }

        match report[0] {
            0x01 => return Err(ProtocolError::Busy),
            0x02 => {}
            status => return Err(ProtocolError::Failed(status)),
        }

        if report[1] != expected_transaction_id {
            return Err(ProtocolError::UnexpectedTransactionId(report[1]));
        }

        if report[6] != expected_class || report[7] != expected_id {
            return Err(ProtocolError::UnexpectedCommand {
                class: report[6],
                id: report[7],
            });
        }

        if expected_data_size > ARGUMENTS_CAPACITY
            || usize::from(report[DATA_SIZE]) != expected_data_size
        {
            return Err(ProtocolError::InvalidDataSize(report[DATA_SIZE]));
        }
        if usize::from(report[DATA_SIZE]) != expected_data_size {
            return Err(ProtocolError::InvalidDataSize(report[DATA_SIZE]));
        }

        Ok(&report[ARGUMENTS_START..ARGUMENTS_START + expected_data_size])
    }

    pub(crate) fn parse_battery_level(
        feature_report: &[u8],
        transaction_id: u8,
    ) -> Result<u8, ProtocolError> {
        let raw = Self::parse(feature_report, transaction_id, 0x07, 0x80, 2)?;
        Ok(raw[1])
    }

    pub(crate) fn parse_charging_status(
        feature_report: &[u8],
        transaction_id: u8,
    ) -> Result<bool, ProtocolError> {
        let raw = Self::parse(feature_report, transaction_id, 0x07, 0x84, 2)?;
        Ok(raw[1] != 0)
    }
}
