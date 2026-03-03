#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::*;
use embassy_stm32::{Config, bind_interrupts, can, rcc};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
use embedded_can::{ExtendedId,Id};

bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<FDCAN1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(25_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    config.rcc.mux.fdcan12sel = rcc::mux::Fdcansel::HSE;

    let peripherals = embassy_stm32::init(config);

    let mut can = can::CanConfigurator::new(peripherals.FDCAN1, peripherals.PA11, peripherals.PA12, Irqs);
    // 250k bps
    can.set_bitrate(1_000_000);
    can.set_fd_data_bitrate(5_000_000,true);

    let mut can = can.into_internal_loopback_mode();
    //let mut can = can.into_normal_mode();
    let (mut tx, mut rx, _props) = can.split();
    info!("CAN up and running, waiting for messages");

    loop {
        match rx.read_fd().await {
            Ok(message) => {
                let (rx_frame,ts) = message.parts();
                
                if let Id::Extended(id)= rx_frame.id() {
                    let raw_id = id.as_raw();
                    info!("Rx with id: 0x{:x}",raw_id);
                    match raw_id {
                        0x00012345 => {
                            info!("Command 0x12345 recieved");
                        }
                        _ => warn!("Unknown command 0x{:x}",raw_id),
                    }

                }
            },
            Err(e) => error!("CAN read error: {}", e),
        }
    }
}