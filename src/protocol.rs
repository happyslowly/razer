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

#[derive(Debug, Clone, Copy)]
struct CommandSpec {
    command_class: u8,
    command_id: u8,
    data_size: u8,
}

const BATTERY_LEVEL: CommandSpec = CommandSpec {
    command_class: 0x07,
    command_id: 0x80,
    data_size: 2,
};

const CHARGING_STATUS: CommandSpec = CommandSpec {
    command_class: 0x07,
    command_id: 0x84,
    data_size: 2,
};

const FIRMWARE_INFO: CommandSpec = CommandSpec {
    command_class: 0x00,
    command_id: 0x81,
    data_size: 2,
};

const GET_DPI: CommandSpec = CommandSpec {
    command_class: 0x04,
    command_id: 0x85,
    data_size: 7,
};

const SET_DPI: CommandSpec = CommandSpec {
    command_class: 0x04,
    command_id: 0x05,
    data_size: 7,
};

#[derive(Debug, Clone)]
pub(crate) struct RazerReport {
    bytes: [u8; REPORT_SIZE],
}

impl RazerReport {
    fn new(transaction_id: u8, spec: CommandSpec) -> Self {
        let mut report = Self {
            bytes: [0; REPORT_SIZE],
        };

        report.bytes[STATUS] = 0x00;
        report.bytes[TRANSACTION_ID] = transaction_id;
        report.bytes[REMAINING_PACKETS_HIGH] = 0x00;
        report.bytes[REMAINING_PACKETS_LOW] = 0x00;
        report.bytes[PROTOCOL_TYPE] = 0x00;
        report.bytes[DATA_SIZE] = spec.data_size;
        report.bytes[COMMAND_CLASS] = spec.command_class;
        report.bytes[COMMAND_ID] = spec.command_id;
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
        Self::new(transaction_id, BATTERY_LEVEL)
    }

    pub(crate) fn for_charging(transaction_id: u8) -> Self {
        Self::new(transaction_id, CHARGING_STATUS)
    }

    pub(crate) fn for_firmware(transaction_id: u8) -> Self {
        Self::new(transaction_id, FIRMWARE_INFO)
    }

    pub(crate) fn for_get_dpi(transaction_id: u8) -> Self {
        // arguments[0] remains 0x00 (NOSTORE).
        Self::new(transaction_id, GET_DPI)
    }

    pub(crate) fn for_set_dpi(transaction_id: u8, x: u16, y: u16) -> Self {
        let mut report = Self::new(transaction_id, SET_DPI);
        let [x_high, x_low] = x.to_be_bytes();
        let [y_high, y_low] = y.to_be_bytes();

        report.bytes[ARGUMENTS_START..ARGUMENTS_START + 7].copy_from_slice(&[
            0x01, // VARSTORE
            x_high, x_low, y_high, y_low, 0x00, 0x00,
        ]);

        report.update_crc();
        report
    }

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
    fn parse(
        feature_report: &[u8],
        expected_transaction_id: u8,
        spec: CommandSpec,
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

        if report[6] != spec.command_class || report[7] != spec.command_id {
            return Err(ProtocolError::UnexpectedCommand {
                class: report[6],
                id: report[7],
            });
        }

        if usize::from(spec.data_size) > ARGUMENTS_CAPACITY || report[DATA_SIZE] != spec.data_size {
            return Err(ProtocolError::InvalidDataSize(report[DATA_SIZE]));
        }

        Ok(&report[ARGUMENTS_START..ARGUMENTS_START + usize::from(spec.data_size)])
    }

    pub(crate) fn parse_battery_level(
        feature_report: &[u8],
        transaction_id: u8,
    ) -> Result<u8, ProtocolError> {
        let raw = Self::parse(feature_report, transaction_id, BATTERY_LEVEL)?;
        Ok(raw[1])
    }

    pub(crate) fn parse_charging_status(
        feature_report: &[u8],
        transaction_id: u8,
    ) -> Result<bool, ProtocolError> {
        let raw = Self::parse(feature_report, transaction_id, CHARGING_STATUS)?;
        Ok(raw[1] != 0)
    }

    pub(crate) fn parse_firmware_info(
        feature_report: &[u8],
        transaction_id: u8,
    ) -> Result<(u8, u8), ProtocolError> {
        let raw = Self::parse(feature_report, transaction_id, FIRMWARE_INFO)?;
        Ok((raw[0], raw[1]))
    }

    pub(crate) fn parse_get_dpi(
        feature_report: &[u8],
        transaction_id: u8,
    ) -> Result<(u16, u16), ProtocolError> {
        let raw = Self::parse(feature_report, transaction_id, GET_DPI)?;
        let x = u16::from_be_bytes([raw[1], raw[2]]);
        let y = u16::from_be_bytes([raw[3], raw[4]]);
        Ok((x, y))
    }

    pub(crate) fn parse_set_dpi(
        feature_report: &[u8],
        transaction_id: u8,
    ) -> Result<(u16, u16), ProtocolError> {
        let raw = Self::parse(feature_report, transaction_id, SET_DPI)?;

        // The Basilisk V3 Pro echoes the applied X and Y DPI values in the
        // SET_DPI response using the same layout as the request arguments.
        let x = u16::from_be_bytes([raw[1], raw[2]]);
        let y = u16::from_be_bytes([raw[3], raw[4]]);
        Ok((x, y))
    }
}
