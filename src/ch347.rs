use std::{fmt::Display, time::Duration, usize};

use nusb::DeviceInfo;

use crate::usb_util::InterfaceExt;

const CH34X_VID_PID: [(u16, u16); 3] = [(0x1A86, 0x55DE), (0x1A86, 0x55DD), (0x1A86, 0x55E8)];

#[derive(Debug)]
pub enum Ch347Error {
    DeviceNotFound,
    OpenDeviceError,
}

impl std::error::Error for Ch347Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl Display for Ch347Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ch347Error::DeviceNotFound => {
                write!(f, "Can't found ch347 device")
            }
            Ch347Error::OpenDeviceError => {
                write!(f, "Can't open usb device")
            }
        }
    }
}

pub(crate) fn is_ch34x_device(device: &DeviceInfo) -> bool {
    CH34X_VID_PID.contains(&(device.vendor_id(), device.product_id()))
}

pub struct Ch347UsbDevice {
    device: nusb::Interface,
    epin: u8,
    epout: u8,
}

impl Ch347UsbDevice {
    pub fn new() -> Result<Self, Ch347Error> {
        let device = nusb::list_devices()
            .map_err(|_| Ch347Error::DeviceNotFound)?
            .filter(is_ch34x_device)
            .next()
            .ok_or(Ch347Error::DeviceNotFound)?;

        let device_handle = device.open().map_err(|_| Ch347Error::OpenDeviceError)?;

        let interface = device_handle
            .claim_interface(4)
            .map_err(|_| Ch347Error::DeviceNotFound)?;

        Ok(Self {
            device: interface,
            epin: 0x86,
            epout: 0x06,
        })
    }

    pub fn write_bulk(&self, buf: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        let bytes = self
            .device
            .write_bulk(self.epout, buf, Duration::from_millis(500))?;
        Ok(bytes)
    }

    pub fn read_bulk(&self, buf: &mut [u8]) -> Result<usize, Box<dyn std::error::Error>> {
        let bytes = self
            .device
            .read_bulk(self.epin, buf, Duration::from_millis(500))?;

        Ok(bytes)
    }
}
