#![no_std]
#![no_main]

mod protocol;
use protocol::*;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::*;
use embassy_stm32::{Config, bind_interrupts, can, can::filter::*, flash, rcc, uid};
//use embassy_time::Timer;
use embedded_can::Id;
use {defmt_rtt as _, panic_probe as _};

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
    let this_node = match uid::uid_hex() {
        "001E005F3333510132313831" => CanDevices::Nuc1,
        "004500243333510132313831" => CanDevices::Nuc2,
        _ => CanDevices::UNKNOWN,
    };
    let target_id_mask = 0x1F << 21;
    let filter_all = ExtendedFilter {
        filter: FilterType::BitMask {
            filter: 0x01 << 21,
            mask: target_id_mask,
        },
        action: can::filter::Action::StoreInFifo0,
    };
    let filter_this_node = ExtendedFilter {
        filter: FilterType::BitMask {
            filter: (this_node as u32) << 21,
            mask: target_id_mask,
        },
        action: can::filter::Action::StoreInFifo0,
    };

    let mut can =
        can::CanConfigurator::new(peripherals.FDCAN1, peripherals.PA11, peripherals.PA12, Irqs);
    can.properties()
        .set_extended_filter(ExtendedFilterSlot::_0, filter_all);
    can.properties()
        .set_extended_filter(ExtendedFilterSlot::_1, filter_this_node);
    can.set_bitrate(1_000_000);
    can.set_fd_data_bitrate(5_000_000, true);
    let mut can = can.into_internal_loopback_mode();
    //let mut can = can.into_normal_mode();
    let (mut tx, mut rx, _props) = can.split();
    info!("CAN up and running, waiting for messages");
    let mut flash_address: u32 = 0;
    let mut flash = flash::Flash::new_blocking(peripherals.FLASH).into_blocking_regions();
    let mut f = flash.bank1_region;
    loop {
        match rx.read_fd().await {
            Ok(message) => {
                let (rx_frame, ts) = message.parts();

                if let Id::Extended(id) = rx_frame.id() {
                    let raw_id = id.as_raw();
                    info!("Rx with id: 0x{:x}", raw_id);
                    let can_msg = parse_can_id(raw_id);
                    if can_msg.target == 0x07 {
                        /* this is a command for us! */
                        let data = rx_frame.data();
                        let command = BootloaderCommand::try_from(can_msg.command).unwrap();
                        match command {
                            BootloaderCommand::Ping => {
                                info!("Command Ping{:X} recieved", BootloaderCommand::Ping as u16);
                            }
                            BootloaderCommand::Erase => {
                                unwrap!(f.blocking_erase(0x8000, 0x80000));
                                info!("Erase Complete");
                            }
                            BootloaderCommand::Write => {
                                let data = rx_frame.data();
                                let offset = flash_address - 0x0800_0000;

                                if data.len() % 16 != 0 {
                                    error!("Data length {} is not 16-byte aligned!", data.len());
                                } else if flash_address >= 0x0800_8000 {
                                    unwrap!(f.blocking_write(offset, data));
                                    info!("Wrote {} bytes to {:x}", data.len(), flash_address);
                                    flash_address += data.len() as u32;
                                }
                            }
                            BootloaderCommand::Jump => {}
                            BootloaderCommand::SetAddress => {
                                flash_address =
                                    u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
                            }
                            _ => warn!("Unknown command 0x{:x}", raw_id),
                        }
                    }
                }
            }
            Err(e) => error!("CAN read error: {}", e),
        }
    }
}
