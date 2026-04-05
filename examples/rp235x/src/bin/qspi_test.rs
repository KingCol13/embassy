//! This example implements a UDP server listening on port 1234 and echoing back the data.
//!
//! Example written for the [`WIZnet W6300-EVB-Pico2`](https://wiznet.io/products/evaluation-boards/w6300-evb-pico2) board.

#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::qspi::exclusive::ExclusiveDevice;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::qspi::Qspi;
use embassy_rp::spi::{Async, Config as SpiConfig};
use embassy_rp::{bind_interrupts, dma};
use embassy_time::Delay;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let rp_peripherals = embassy_rp::init(Default::default());

    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = 15_000_000;
    // spi_cfg.frequency = 65536;

    let Pio { mut common, sm0, .. } = Pio::new(rp_peripherals.PIO0, Irqs);

    let clk = rp_peripherals.PIN_17;
    let qspi_0 = rp_peripherals.PIN_6;
    let qspi_1 = rp_peripherals.PIN_7;
    let qspi_2 = rp_peripherals.PIN_8;
    let qspi_3 = rp_peripherals.PIN_9;

    let mut qspi = Qspi::new(
        &mut common,
        sm0,
        clk,
        qspi_0,
        qspi_1,
        qspi_2,
        qspi_3,
        rp_peripherals.DMA_CH0,
        rp_peripherals.DMA_CH1,
        Irqs,
        spi_cfg,
    );

    loop {
        qspi.read(&mut [0x0F]).await.unwrap();
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}
