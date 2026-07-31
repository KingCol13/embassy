//! Trait for different types of SPI and blanket implementations

mod spi;

use embedded_hal::spi::ErrorType;

/// Number of lines used by SPI
#[derive(Debug)]
pub enum SpiType {
    /// Regular, full-duplex SPI with 2 data lines
    Single,
    /// Dual SPI - half-duplex using 2 bi-directional data lines
    Dual,
    /// Quad SPI - half-duplex using 4 bi-directional data lines
    Quad,
}

/// Wiznet SPI read transaction
#[derive(Debug, PartialEq, Eq)]
pub struct WiznetSpiRead<'a> {
    /// Write on a single SPI line. Can be set to empty slice if not required.
    pub write_single: &'a [u8],
    /// Write using full width of SPI.
    pub write: &'a [u8],
    /// Read using full width of SPI.
    pub read_data: &'a mut [u8],
}

/// Wiznet SPI write transaction
#[derive(Debug, PartialEq, Eq)]
pub struct WiznetSpiWrite<'a> {
    /// Write on a single SPI line. Can be set to empty slice if not required.
    pub write_single: &'a [u8],
    /// Write using full width of SPI.
    pub write: &'a [u8],
    /// Read using full width of SPI.
    pub write_data: &'a [u8],
}

/// Interface for communicating with Wiznet chip with various types of SPI
pub trait WiznetSpiBus<Word: Copy + 'static = u8>: ErrorType {
    /// Type of SPI implemented by the type
    const SPI_TYPE: SpiType;

    /// Perform a read transaction against the device.
    ///
    /// - Locks the bus
    /// - Asserts the CS (Chip Select) pin.
    /// - Performs the WiznetSpiRead operations.
    /// - [Flushes](SpiBus::flush) the bus.
    /// - Deasserts the CS pin.
    /// - Unlocks the bus.
    async fn read<'a>(
        &mut self,
        transaction: WiznetSpiRead<'a>,
    ) -> Result<(), Self::Error>;

    /// Perform a write against the device.
    ///
    /// - Locks the bus
    /// - Asserts the CS (Chip Select) pin.
    /// - Performs the WiznetSpiWrite operations.
    /// - [Flushes](SpiBus::flush) the bus.
    /// - Deasserts the CS pin.
    /// - Unlocks the bus.
    async fn write<'a>(
        &mut self,
        transaction: WiznetSpiWrite<'a>,
    ) -> Result<(), Self::Error>;
}
