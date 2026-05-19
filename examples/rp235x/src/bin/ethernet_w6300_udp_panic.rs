//! This is an attempt to trigger a panic in embassy-net when a closure that can only be called once
//! is called twice.
//!
//! Example written for the [`WIZnet W6300-EVB-Pico2`](https://wiznet.io/products/evaluation-boards/w6300-evb-pico2) board.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_net::StackResources;
use embassy_net::StaticConfigV4;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net_wiznet::chip::W6300;
use embassy_net_wiznet::*;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::spi::Spi;
use embassy_rp::spi::{Async, Config as SpiConfig};
use embassy_rp::{bind_interrupts, dma};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use smoltcp::phy::ChecksumCapabilities;
use smoltcp::wire::{UdpPacket, UdpRepr};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>;
});

#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<
        'static,
        W6300,
        ExclusiveDevice<Spi<'static, PIO0, 0, Async>, Output<'static>, Delay>,
        Input<'static>,
        Output<'static>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}

async fn trigger_panic_bug(stack: Stack<'_>, config: StaticConfigV4) -> ! {
    let mut rx_meta = [PacketMetadata::EMPTY; 2];
    let mut rx_buffer = [0; 128];
    let mut tx_meta = [PacketMetadata::EMPTY; 2];
    let mut tx_buffer = [0; 1024];

    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);

    const UDP_PAYLOAD_SIZE: usize = 129;
    const SRC_PORT: u16 = 9400;
    const DST_PORT: u16 = 9401;
    let dst_address = config.gateway.unwrap();
    let local_addr = config.address.address();

    socket.bind(SRC_PORT).unwrap();
    let udp_repr = UdpRepr {
        src_port: SRC_PORT,
        dst_port: DST_PORT,
    };

    defmt::info!("Attempting to trigger panic");
    loop {
        let _ = socket
            .send_to_with(
                udp_repr.header_len() + UDP_PAYLOAD_SIZE,
                (dst_address, DST_PORT),
                |send_buf| {
                    let header_len = udp_repr.header_len();
                    let mut udp_packet = UdpPacket::new_unchecked(send_buf);
                    udp_repr.emit(
                        &mut udp_packet,
                        &local_addr.into(),
                        &dst_address.into(),
                        UDP_PAYLOAD_SIZE,
                        |buf| buf.fill(0),
                        &ChecksumCapabilities::ignored(),
                    );
                    (header_len + UDP_PAYLOAD_SIZE, ())
                },
            )
            .await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut rng = RoscRng;
    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = 15_000_000;

    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let (miso, mosi, clk) = (p.PIN_19, p.PIN_18, p.PIN_17);

    let spi = Spi::new(&mut common, sm0, clk, mosi, miso, p.DMA_CH0, p.DMA_CH1, Irqs, spi_cfg);

    let cs = Output::new(p.PIN_16, Level::High);
    let w6300_int = Input::new(p.PIN_15, Pull::Up);
    let w6300_reset = Output::new(p.PIN_22, Level::High);

    let mac_addr = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00];

    static STATE: StaticCell<State<8, 8>> = StaticCell::new();
    let state = STATE.init(State::<8, 8>::new());

    let (device, runner) = embassy_net_wiznet::new(
        mac_addr,
        state,
        ExclusiveDevice::new(spi, cs, Delay),
        w6300_int,
        w6300_reset,
    )
    .await
    .unwrap();
    spawner.spawn(unwrap!(ethernet_task(runner)));

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        device,
        embassy_net::Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::new()),
        seed,
    );

    // Launch network task
    spawner.spawn(unwrap!(net_task(runner)));

    info!("Waiting for DHCP...");
    stack.wait_config_up().await;
    let config = stack.config_v4().unwrap();
    let local_addr = config.address.address();
    info!("IP address: {:?}", local_addr);

    trigger_panic_bug(stack, config).await;
}
