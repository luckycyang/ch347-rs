use std::{cell::RefCell, usize};

pub struct Gpio_Config<'d> {
    device: &'d crate::ch347::Ch347UsbDevice,
    pub ibuf: [u8; 11],
    pub obuf: [u8; 11],
}

impl<'d> Gpio_Config<'d> {
    pub fn from_device(device: &'d crate::ch347::Ch347UsbDevice) -> Gpio_Config<'d> {
        let mut ibuf = [0; 11];
        let obuf = [
            0xCC, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        device.write_bulk(&obuf).expect("is that ch347 work???");
        device.read_bulk(&mut ibuf).expect("is that ch347 work???");
        Self { device, obuf, ibuf }
    }

    fn pin_command(&mut self, pin: u8, command: u8) {
        self.obuf[3 + pin as usize] = command;
    }

    fn pin_state(&self, pin: u8) -> u8 {
        self.ibuf[3 + pin as usize]
    }

    fn flush(&mut self) {
        self.device
            .write_bulk(&self.obuf)
            .expect("is that ch347 work???");
        self.device
            .read_bulk(&mut self.ibuf)
            .expect("is that ch347 work???");
    }
}

pub struct Flex<'d, 'a> {
    config: &'a RefCell<Gpio_Config<'d>>,
    pin: u8,
    state: bool,
}

impl<'d, 'a> Flex<'d, 'a> {
    pub fn from_config(config: &'a RefCell<Gpio_Config<'d>>, pin: u8) -> Flex<'d, 'a> {
        Flex {
            config: config,
            pin: pin,
            state: false,
        }
    }

    pub fn trans_input(&self) {
        self.config.borrow_mut().pin_command(self.pin, 0xC0);
        self.config.borrow_mut().flush();
    }

    pub fn trans_output(&self) {
        self.config.borrow_mut().pin_command(self.pin, 0xF0);
        self.config.borrow_mut().flush();
    }

    pub fn is_high(&self) -> bool {
        self.config.borrow_mut().flush();
        self.config.borrow().pin_state(self.pin) & 0x40 == 0x40
    }

    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    fn set_state(&mut self, state: bool) {
        self.state = state;
        let state = if state { 0x08 } else { 0x00 };
        self.config.borrow_mut().pin_command(self.pin, 0xF0 | state);
        self.config.borrow_mut().flush();
    }

    pub fn set_low(&mut self) {
        self.set_state(false);
    }

    pub fn set_high(&mut self) {
        self.set_state(true);
    }

    pub fn toggle(&mut self) {
        self.set_state(!self.state);
    }
}

pub struct Output<'d, 'a> {
    pin: Flex<'d, 'a>,
}

pub struct Input<'d, 'a> {
    pin: Flex<'d, 'a>,
}

impl<'d, 'a> Input<'d, 'a> {
    pub fn new(config: &'a RefCell<Gpio_Config<'d>>, pin: u8) -> Self {
        let flex = Flex::from_config(config, pin);
        flex.trans_input();
        Self { pin: flex }
    }
}

impl<'d, 'a> Output<'d, 'a> {
    pub fn new(config: &'a RefCell<Gpio_Config<'d>>, pin: u8) -> Self {
        let flex = Flex::from_config(config, pin);
        flex.trans_output();
        Self { pin: flex }
    }

    fn set_low(&mut self) {
        self.pin.set_low();
    }

    fn set_high(&mut self) {
        self.pin.set_high();
    }

    fn set_state(&mut self, state: bool) {
        self.pin.set_state(state);
    }
}

impl<'d, 'a> embedded_hal::digital::ErrorType for Output<'d, 'a> {
    type Error = core::convert::Infallible;
}

impl<'d, 'a> embedded_hal::digital::OutputPin for Output<'d, 'a> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_high();
        Ok(())
    }

    fn set_state(&mut self, state: embedded_hal::digital::PinState) -> Result<(), Self::Error> {
        match state {
            embedded_hal::digital::PinState::Low => Ok(self.set_low()),
            embedded_hal::digital::PinState::High => Ok(self.set_high()),
        }
    }
}
