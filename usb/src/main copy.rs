use std::time::Duration;
use rusb::DeviceHandle;
use rusb::Result;
use rusb::UsbContext;

fn read_chip_id<T: UsbContext>(device: &DeviceHandle<T>) -> Result<u32> {
    let mut buf = [0; 4];
    let count = device.read_control(0xc0, 0x03, 0, 0, &mut buf, Duration::from_secs(10))?;
    assert_eq!(count, 4);
    Ok(u32::from_le_bytes(buf))
}

fn hold_cpu<T: UsbContext>(device: &DeviceHandle<T>) -> Result<()> {
    let mut buf = [0; 0];
    let count = device.write_control(0x40, 0x04, 0, 0, &buf, Duration::from_secs(10))?;
    assert_eq!(count, 0);
    Ok(())
}

fn unhold_cpu<T: UsbContext>(device: &DeviceHandle<T>) -> Result<()> {
    let mut buf = [0; 0];
    let count = device.write_control(0x40, 0x04, 1, 0, &buf, Duration::from_secs(10))?;
    assert_eq!(count, 0);
    Ok(())
}

fn data_read<T: UsbContext>(device: &DeviceHandle<T>, addr: u16) -> Result<u8> {
    let mut buf = [0; 1];
    let count = device.read_control(0xc0, 0x82, addr, 0, &mut buf, Duration::from_secs(10))?;
    assert_eq!(count, 1);
    Ok(buf[0])
}

fn data_write<T: UsbContext>(device: &DeviceHandle<T>, addr: u16, value: u8) -> Result<()> {
    let mut buf = [0; 0];
    let count = device.write_control(0x40, 0x81, addr, u16::from(value), &buf, Duration::from_secs(10))?;
    assert_eq!(count, 0);
    Ok(())
}

fn main() {
    for device in rusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        // println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
        //     device.bus_number(),
        //     device.address(),
        //     device_desc.vendor_id(),
        //     device_desc.product_id());

        if device_desc.vendor_id() == 0x0bc2 {
            let mut device = device.open().unwrap();
            let id = read_chip_id(&device).unwrap();
            // println!("{:x}", id);
            hold_cpu(&device);
            // data_write(&device, 0x58, 0x0c);
            data_write(&device, 0x13e, 'f' as u8);
            data_write(&device, 0x18, 'f' as u8);
            data_write(&device, 0x1be, 'f' as u8);
            for i in 0..0x800 {
                print!("{:02x}", data_read(&device, i).unwrap());
            }
            unhold_cpu(&device);
            // for i in 0..255 {
            //     data_write(&device, 0x40E0, i); //addr
            //     data_write(&device, 0x40E2, 0b00001000);
            //     data_write(&device, 0x40E3, 0b10000001); //b7+b0
            //     while data_read(&device, 0x40E3).unwrap() & 0x80 != 0 {
            //         // loop
            //     }
            //     println!("{:x} {:x}", data_read(&device, 0x40E1).unwrap(), data_read(&device, 0x40E4).unwrap());
            //     println!("--");
            // }
        }
    }
}
