use nusb::transfer::RequestBuffer;
use smol::Timer;
use smol::block_on;
use smol::future::FutureExt;
use std::io;
use std::sync::OnceLock;
use std::time::Duration;

use nusb::{DeviceInfo, Interface};

use crate::format_u8_array;
use crate::hal;
use crate::hal::Peripherals;

/// device info of ch347
const CH34X_VID_PID: [(u16, u16); 3] = [(0x1A86, 0x55DE), (0x1A86, 0x55DD), (0x1A86, 0x55E8)];

static CH347: OnceLock<Interface> = OnceLock::new();

#[derive(Debug)]
pub enum Error {
    UsbNoFound,
    Denied,
    Tx,
    Rx,
}

pub fn is_ch34x_device(device: &DeviceInfo) -> bool {
    CH34X_VID_PID.contains(&(device.vendor_id(), device.product_id()))
}

/// get pepherial tree
pub fn init() -> Result<Peripherals, Error> {
    let device = nusb::list_devices()
        .map_err(|_| Error::UsbNoFound)?
        .filter(is_ch34x_device)
        .next()
        .ok_or(Error::UsbNoFound)?;

    let device_handle = device.open().map_err(|_| Error::Denied)?;

    let interface = device_handle
        .claim_interface(4)
        .map_err(|_| Error::Denied)?;

    CH347.get_or_init(|| interface);
    Ok(hal::Peripherals::take())
}

pub fn write(buf: &[u8]) -> Result<(), Error> {
    if let Some(device) = CH347.get() {
        device
            .write_bulk(0x06, buf, Duration::from_millis(500))
            .map_err(|_| Error::Tx)?;
        log::info!("usb write: {}", format_u8_array(buf));

        Ok(())
    } else {
        log::info!("Can't get ch347 device");
        Err(Error::Tx)
    }
}

pub fn read(buf: &mut [u8]) -> Result<usize, Error> {
    if let Some(device) = CH347.get() {
        let rev = device
            .read_bulk(0x86, buf, Duration::from_millis(500))
            .map_err(|_| Error::Rx)?;
        log::info!("usb read: {}", format_u8_array(&buf[..rev]));

        Ok(rev)
    } else {
        log::info!("Can't get ch347 device");
        Err(Error::Rx)
    }
}

/// Copy from probe-rs
pub trait InterfaceExt {
    fn read_bulk(&self, endpoint: u8, buf: &mut [u8], timeout: Duration) -> io::Result<usize>;
    fn write_bulk(&self, endpoint: u8, buf: &[u8], timeout: Duration) -> io::Result<usize>;
}

impl InterfaceExt for Interface {
    fn write_bulk(&self, endpoint: u8, buf: &[u8], timeout: Duration) -> io::Result<usize> {
        let fut = async {
            let comp = self.bulk_out(endpoint, buf.to_vec()).await;
            comp.status.map_err(io::Error::other)?;

            let n = comp.data.actual_length();
            Ok(n)
        };

        block_on(fut.or(async {
            Timer::after(timeout).await;
            Err(std::io::ErrorKind::TimedOut.into())
        }))
    }

    fn read_bulk(&self, endpoint: u8, buf: &mut [u8], timeout: Duration) -> io::Result<usize> {
        let fut = async {
            let comp = self.bulk_in(endpoint, RequestBuffer::new(buf.len())).await;
            comp.status.map_err(io::Error::other)?;

            let n = comp.data.len();
            buf[..n].copy_from_slice(&comp.data);
            Ok(n)
        };

        block_on(fut.or(async {
            Timer::after(timeout).await;
            Err(std::io::ErrorKind::TimedOut.into())
        }))
    }
}
