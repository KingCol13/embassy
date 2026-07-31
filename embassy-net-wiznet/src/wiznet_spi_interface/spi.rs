use embedded_hal_async::spi;

use heapless;

use crate::wiznet_spi_interface::{
    SpiType, WiznetSpiBus, WiznetSpiRead, WiznetSpiWrite,
};

const MAX_OPERATIONS: usize = 3;

impl<'a> From<WiznetSpiRead<'a>> for heapless::Vec<spi::Operation<'a, u8>, MAX_OPERATIONS> {
    fn from(value: WiznetSpiRead<'a>) -> Self {
        let mut op_vec: heapless::Vec<spi::Operation<'a, u8>, MAX_OPERATIONS> = heapless::Vec::new();
        let WiznetSpiRead {
            write_single,
            write,
            read_data,
        } = value;
        // elide write_single step if the length is 0
        if write_single.len() > 0 {
            op_vec.push(spi::Operation::Write(write_single)).unwrap();
        }
        op_vec.push(spi::Operation::Write(write)).unwrap();
        op_vec.push(spi::Operation::Read(read_data)).unwrap();
        op_vec
    }
}

impl<'a> From<WiznetSpiWrite<'a>> for heapless::Vec<spi::Operation<'a, u8>, MAX_OPERATIONS> {
    fn from(value: WiznetSpiWrite<'a>) -> Self {
        let mut op_vec: heapless::Vec<spi::Operation<'a, u8>, MAX_OPERATIONS> = heapless::Vec::new();
        let WiznetSpiWrite {
            write_single,
            write,
            write_data,
        } = value;
        // elide write_single step if the length is 0
        if write_single.len() > 0 {
            op_vec.push(spi::Operation::Write(write_single)).unwrap();
        }
        op_vec.push(spi::Operation::Write(write)).unwrap();
        op_vec.push(spi::Operation::Write(write_data)).unwrap();
        op_vec
    }
}

impl<SPI: spi::SpiDevice> WiznetSpiBus for SPI {
    const SPI_TYPE: SpiType = SpiType::Single;

    async fn read<'a>(&mut self, transaction: WiznetSpiRead<'a>) -> Result<(), Self::Error> {
        let mut ops: heapless::Vec<_, MAX_OPERATIONS> = transaction.into();
        self.transaction(&mut ops).await
    }

    async fn write<'a>(&mut self, transaction: WiznetSpiWrite<'a>) -> Result<(), Self::Error> {
        let mut ops: heapless::Vec<_, MAX_OPERATIONS> = transaction.into();
        self.transaction(&mut ops).await
    }
}
