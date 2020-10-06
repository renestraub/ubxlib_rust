use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    SerialPortNotFound,
    SerialPortConfigFailed,
    SerialPortSendFailed,
    BaudRateDetectionFailed,
    ModemNotResponding,
    ModemNAK,
    ModemUnexpectedAckNak,
    ModemNobackup,
    ModemBackupRestoreFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::SerialPortNotFound => f.write_str("serial port not found"),
            Error::SerialPortConfigFailed => f.write_str("failed to configure serial port"),
            Error::SerialPortSendFailed => f.write_str("failed to send to serial port"),
            Error::BaudRateDetectionFailed => f.write_str("failed to detect current baudrate"),
            Error::ModemNotResponding => f.write_str("modem did not respond"),
            Error::ModemNAK => f.write_str("modem NAK received"),
            Error::ModemUnexpectedAckNak => f.write_str("unexpected ACK/NAK received"),
            Error::ModemNobackup => f.write_str("no backup present"),
            Error::ModemBackupRestoreFailed => f.write_str("restoring backup failed"),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::SerialPortNotFound => "serial port not found",
            Error::SerialPortConfigFailed => "failed to configure serial port",
            Error::SerialPortSendFailed => "failed to send to serial port",
            Error::BaudRateDetectionFailed => "failed to detect current baudrate",
            Error::ModemNotResponding => "modem did not respond",
            Error::ModemNAK => "modem NAK received",
            Error::ModemUnexpectedAckNak => "unexpected ACK/NAK received",
            Error::ModemNobackup => "no backup present",
            Error::ModemBackupRestoreFailed => "restoring backup failed",
        }
    }
}
