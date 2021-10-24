use rusb::DeviceHandle;
use rusb::{request_type, Direction, Recipient, RequestType, Result, UsbContext};
use std::time::Duration;

use structopt::StructOpt;

#[derive(StructOpt, Clone, Copy, Debug, PartialEq, Eq)]
enum Dir {
    In,
    Out,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "usb")]
enum Opt {
    ReadAnalog {
        #[structopt(short, long)]
        port: u8,
    },
    SetDirection {
        #[structopt(short, long)]
        port: u8,
        #[structopt(subcommand)]
        direction: Dir,
    },
    DigitalRead {
        #[structopt(short, long)]
        port: u8,
    },
    DigitalWrite {
        #[structopt(short, long)]
        port: u8,
        #[structopt(short, long)]
        value: bool,
    },
    EepromRead {
        #[structopt(short, long)]
        address: u16,
    },
    EepromWrite {
        #[structopt(short, long)]
        address: u16,
        #[structopt(short, long)]
        value: u8,
    },
    WriteDefaults {
        #[structopt(short, long)]
        power_on_delay: u16,
        #[structopt(short, long)]
        sequence_delay: u16,
        #[structopt(short, long)]
        config: Vec<char>,
    },
    DebugReadSof {
    },
    DebugReadOsccal {
    },
    DebugRereadEeprom {
    },
    DebugBootloader {
    },
}

fn read_port_analog<T: UsbContext>(port: u8, device: &DeviceHandle<T>) -> Result<u16> {
    let mut buf = [0; 2];
    let count = device.read_control(
        request_type(Direction::In, RequestType::Vendor, Recipient::Device),
        4,
        0,
        port as u16,
        &mut buf,
        Duration::from_secs(10),
    )?;
    assert_eq!(count, 2);
    Ok(u16::from_le_bytes(buf))
}

fn read_port_digital<T: UsbContext>(port: u8, device: &DeviceHandle<T>) -> Result<bool> {
    let mut buf = [0; 1];
    let count = device.read_control(
        request_type(Direction::In, RequestType::Vendor, Recipient::Device),
        3,
        0,
        port as u16,
        &mut buf,
        Duration::from_secs(10),
    )?;
    assert_eq!(count, 2);
    Ok(buf[0] > 0)
}

fn write_port_mode<T: UsbContext>(
    port: u8,
    direction: Direction,
    device: &DeviceHandle<T>,
) -> Result<()> {
    let mut buf = [0; 0];
    let count = device.write_control(
        request_type(Direction::Out, RequestType::Vendor, Recipient::Device),
        1,
        if direction == Direction::Out {
            0xffff
        } else {
            0
        },
        port as u16,
        &mut buf,
        Duration::from_secs(10),
    )?;
    assert_eq!(count, 0);
    Ok(())
}

fn write_port_digital<T: UsbContext>(
    port: u8,
    state: bool,
    device: &DeviceHandle<T>,
) -> Result<()> {
    let mut buf = [0; 0];
    let count = device.write_control(
        request_type(Direction::Out, RequestType::Vendor, Recipient::Device),
        2,
        if state { 0xffff } else { 0 },
        port as u16,
        &mut buf,
        Duration::from_secs(10),
    )?;
    assert_eq!(count, 0);
    Ok(())
}

fn read_eeprom<T: UsbContext>(index: u16, device: &DeviceHandle<T>) -> Result<u8> {
    let mut buf = [0; 1];
    let count = device.read_control(
        request_type(Direction::In, RequestType::Vendor, Recipient::Device),
        5,
        0,
        index,
        &mut buf,
        Duration::from_secs(10),
    )?;
    assert_eq!(count, 1);
    Ok(buf[0])
}

fn write_eeprom<T: UsbContext>(index: u16, value: u8, device: &DeviceHandle<T>) -> Result<()> {
    let mut buf = [0; 0];
    let count = device.write_control(
        request_type(Direction::Out, RequestType::Vendor, Recipient::Device),
        6,
        value as u16,
        index,
        &mut buf,
        Duration::from_secs(10),
    )?;
    assert_eq!(count, 0);
    Ok(())
}

fn debug_read_sof_count<T: UsbContext>(device: &DeviceHandle<T>) -> Result<u8> {
    let mut buf = [0; 1];
    let count = device.read_control(
        request_type(Direction::In, RequestType::Vendor, Recipient::Device),
        250,
        0,
        0,
        &mut buf,
        Duration::from_secs(10),
    )?;
    assert_eq!(count, 1);
    Ok(buf[0])
}

fn debug_read_osccal<T: UsbContext>(device: &DeviceHandle<T>) -> Result<u8> {
    let mut buf = [0; 1];
    let count = device.read_control(
        request_type(Direction::In, RequestType::Vendor, Recipient::Device),
        251,
        0,
        0,
        &mut buf,
        Duration::from_secs(10),
    )?;
    assert_eq!(count, 1);
    Ok(buf[0])
}

fn debug_reread_eeprom<T: UsbContext>(device: &DeviceHandle<T>) -> Result<u8> {
    let mut buf = [0; 1];
    let count = device.read_control(
        request_type(Direction::In, RequestType::Vendor, Recipient::Device),
        252,
        0,
        0,
        &mut buf,
        Duration::from_secs(10),
    )?;
    assert_eq!(count, 1);
    Ok(buf[0])
}

fn debug_bootloader<T: UsbContext>(device: &DeviceHandle<T>) -> Result<()> {
    let mut buf = [0; 0];
    let count = device.read_control(
        request_type(Direction::In, RequestType::Vendor, Recipient::Device),
        253,
        0,
        0,
        &mut buf,
        Duration::from_secs(10),
    )?;
    assert_eq!(count, 1);
    Ok(())
}

fn write_defaults<T: UsbContext>(
    poweron_delay: u8,
    sequence_delay: u8,
    defaults: [(Direction, bool); 4],
    device: &DeviceHandle<T>,
) -> Result<()> {
    let mut directions = 0_u8;
    let mut states = 0_u8;
    for state in (&defaults).iter().enumerate() {
        if (state.1).0 == Direction::Out {
            directions |= 1 << state.0;
            if (state.1).1 {
                states |= 1 << state.0;
            }
        }
    }
    let checksum1 = poweron_delay ^ sequence_delay ^ directions ^ states;
    let checksum2 = poweron_delay + sequence_delay + directions + states;
    write_eeprom(0, poweron_delay, &device)?;
    write_eeprom(1, sequence_delay, &device)?;
    write_eeprom(2, directions, &device)?;
    write_eeprom(3, states, &device)?;
    write_eeprom(4, checksum1, &device)?;
    write_eeprom(5, checksum2, &device)?;
    Ok(())
}

fn map_config(c: char) -> (Direction, bool) {
    match c {
        'i' => (Direction::In, false),
        'I' => (Direction::In, true),
        'o' => (Direction::Out, false),
        'O' => (Direction::Out, true),
        _ => panic!("Invalid setting"),
    }
}

fn main() {
    let opt = Opt::from_args();
    for device in rusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        if device_desc.vendor_id() == 0x16c0 && device_desc.product_id() == 0x05df {
            println!(
                "Bus {:03} Device {:03} ID {:04x}:{:04x}",
                device.bus_number(),
                device.address(),
                device_desc.vendor_id(),
                device_desc.product_id()
            );
            let device = device.open().unwrap();

            if let Opt::SetDirection { port, direction } = opt {
                write_port_mode(
                    port,
                    if direction == Dir::In {
                        Direction::In
                    } else {
                        Direction::Out
                    },
                    &device,
                )
                .unwrap();
                eprintln!("ok");
            } else if let Opt::DigitalRead { port } = opt {
                eprintln!("{}", read_port_digital(port, &device).unwrap());
            } else if let Opt::DigitalWrite { port, value } = opt {
                write_port_digital(port, value, &device).unwrap();
                eprintln!("ok");
            } else if let Opt::ReadAnalog { port } = opt {
                eprintln!("{}", read_port_analog(port, &device).unwrap());
            } else if let Opt::EepromRead { address } = opt {
                eprintln!("{}", read_eeprom(address, &device).unwrap());
            } else if let Opt::EepromWrite { address, value } = opt {
                write_eeprom(address, value, &device).unwrap();
                eprintln!("ok");
            } else if let Opt::WriteDefaults { power_on_delay, sequence_delay, ref config } = opt {
                write_defaults(
                    (power_on_delay / 250) as u8,
                    (sequence_delay / 250) as u8,
                    [
                        map_config(config[0]),
                        map_config(config[1]),
                        map_config(config[2]),
                        map_config(config[3]),
                    ],
                    &device,
                )
                .unwrap();
                eprintln!("ok");
            } else if let Opt::DebugReadSof{} = opt {
                eprintln!("sof = {}", debug_read_sof_count(&device).unwrap());
            } else if let Opt::DebugReadOsccal{} = opt {
                eprintln!("OSCCAL = {}", debug_read_osccal(&device).unwrap());
            } else if let Opt::DebugRereadEeprom{} = opt {
                eprintln!("eeprom = {}", debug_reread_eeprom(&device).unwrap());
            } else if let Opt::DebugBootloader{} = opt {
                let _ = debug_bootloader(&device);
                eprintln!("ok");
            }
        }
    }
}
