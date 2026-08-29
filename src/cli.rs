use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Read battery level and charging status
    Battery,
    /// Read firmware version
    Firmware,
    /// Read or set current X and Y DPI
    Dpi {
        /// Horizontal DPI; also used for Y when omitted
        x: Option<u16>,

        /// Vertical DPI
        y: Option<u16>,
    },
}

/// Query information from supported Razer devices.
#[derive(Debug, Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}
