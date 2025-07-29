use crate::gpio::hal::Pin;
use embassy_hal_internal::{Peripheral, PeripheralRef, into_ref};

mod hal {
    static mut GPIO_COMMANDS: [u8; 11] = [
        0xCC, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    use crate::ch347::{read, write};

    use super::types::*;

    pub trait Pin {
        /// Which pin is
        fn pin(&self) -> u8;

        fn set_output(&self, level: PinState) {
            // unsafe 大法好呀，为什么不用Mutx+RefCell呢
            unsafe {
                GPIO_COMMANDS[3 + self.pin() as usize] = 0xF0
                    | match level {
                        PinState::Low => 0x00,
                        PinState::High => 0x08,
                    };
                let ptr = &raw const GPIO_COMMANDS;
                let buf = &*ptr as &[u8];
                write(buf).unwrap();
            }
            // consume read buffer
            let mut buf = [0; 11];
            read(&mut buf).unwrap();
        }

        fn set_input(&self) {
            unsafe {
                GPIO_COMMANDS[3 + self.pin() as usize] = 0xC0;
                let ptr = &raw const GPIO_COMMANDS;
                let buf = &*ptr as &[u8];
                write(buf).unwrap();
            }
            let mut buf = [0; 11];
            read(&mut buf).unwrap();
        }

        fn read(&self) -> PinState {
            unsafe {
                // update
                let ptr = &raw const GPIO_COMMANDS;
                let buf = &*ptr as &[u8];
                write(buf).unwrap();
            }
            let mut buf = [0; 11];
            read(&mut buf).unwrap();
            match buf[3 + self.pin() as usize] {
                0x40 => PinState::High,
                0x00 => PinState::Low,
                _ => panic!("Is than ch347 work???"),
            }
        }
    }
}

pub trait DegradePin: Peripheral<P = Self> + Into<AnyPin> + hal::Pin + Sized + 'static {
    fn degrade(self) -> AnyPin {
        AnyPin { pin: self.pin() }
    }
}

pub mod types {
    #[derive(Debug, PartialEq, PartialOrd)]
    pub enum PinState {
        Low,
        High,
    }
}

pub struct AnyPin {
    pin: u8,
}

embassy_hal_internal::impl_peripheral!(AnyPin);
impl hal::Pin for AnyPin {
    fn pin(&self) -> u8 {
        self.pin
    }
}

impl DegradePin for AnyPin {
    fn degrade(self) -> AnyPin {
        self
    }
}

pub struct Flex<'d> {
    pub(crate) pin: PeripheralRef<'d, AnyPin>,
}

impl<'d> Flex<'d> {
    pub fn new(pin: impl Peripheral<P = impl DegradePin> + 'd) -> Self {
        into_ref!(pin);
        Self {
            pin: pin.map_into(),
        }
    }

    pub fn set_output(&self, level: super::gpio::types::PinState) {
        self.pin.set_output(level);
    }

    pub fn set_input(&self) {
        self.pin.set_input();
    }

    pub fn read(&self) -> crate::gpio::types::PinState {
        self.pin.read()
    }

    pub fn write(&self, level: types::PinState) {
        self.pin.set_output(level);
    }
}

pub struct Output<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> Output<'d> {
    pub fn new(pin: impl Peripheral<P = impl DegradePin> + 'd) -> Self {
        let pin = Flex::new(pin);
        pin.set_output(crate::gpio::types::PinState::Low);
        Self { pin }
    }

    /// 没有开漏输出， 所以没有读方法
    pub fn write(&self, level: types::PinState) {
        self.pin.write(level);
    }
}

pub struct Input<'d> {
    pub(crate) pin: Flex<'d>,
}

impl<'d> Input<'d> {
    pub fn new(pin: impl Peripheral<P = impl DegradePin> + 'd) -> Self {
        let pin = Flex::new(pin);
        pin.set_input();
        Self { pin }
    }

    pub fn read(&self) -> types::PinState {
        self.pin.read()
    }
}

mod embedded_hal_v100_impl {
    use crate::gpio::{self, types};
    use embedded_hal::digital::*;

    impl<'d> ErrorType for gpio::Output<'d> {
        type Error = core::convert::Infallible;
    }

    impl<'d> OutputPin for gpio::Output<'d> {
        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.write(types::PinState::High);
            Ok(())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.write(types::PinState::Low);
            Ok(())
        }
    }

    impl<'d> ErrorType for gpio::Input<'d> {
        type Error = core::convert::Infallible;
    }
    impl<'d> InputPin for gpio::Input<'d> {
        fn is_high(&mut self) -> Result<bool, Self::Error> {
            Ok(self.read() == types::PinState::High)
        }

        fn is_low(&mut self) -> Result<bool, Self::Error> {
            Ok(self.read() == types::PinState::Low)
        }
    }

    impl<'d> ErrorType for gpio::Flex<'d> {
        type Error = core::convert::Infallible;
    }
    impl<'d> OutputPin for gpio::Flex<'d> {
        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.write(types::PinState::High);
            Ok(())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.write(types::PinState::Low);
            Ok(())
        }
    }
    impl<'d> InputPin for gpio::Flex<'d> {
        fn is_high(&mut self) -> Result<bool, Self::Error> {
            Ok(self.read() == types::PinState::High)
        }

        fn is_low(&mut self) -> Result<bool, Self::Error> {
            Ok(self.read() == types::PinState::Low)
        }
    }
}

mod embedded_hal_v027_impl {
    use crate::gpio::{self, types};
    use embedded_hal_027::digital::v2::*;

    impl<'d> OutputPin for gpio::Output<'d> {
        type Error = core::convert::Infallible;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.write(types::PinState::High);
            Ok(())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.write(types::PinState::Low);
            Ok(())
        }
    }

    impl<'d> InputPin for gpio::Input<'d> {
        type Error = core::convert::Infallible;
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == types::PinState::High)
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == types::PinState::Low)
        }
    }

    impl<'d> OutputPin for gpio::Flex<'d> {
        type Error = core::convert::Infallible;
        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.write(types::PinState::High);
            Ok(())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.write(types::PinState::Low);
            Ok(())
        }
    }
    impl<'d> InputPin for gpio::Flex<'d> {
        type Error = core::convert::Infallible;
        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == types::PinState::High)
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.read() == types::PinState::Low)
        }
    }
}

use crate::hal::peripherals::*;
macro_rules! gpio_pin_def {
    ($pin_name: ident, $pin_index: expr) => {
        impl hal::Pin for $pin_name {
            fn pin(&self) -> u8 {
                $pin_index
            }
        }

        impl DegradePin for $pin_name {
            fn degrade(self) -> AnyPin {
                AnyPin { pin: self.pin() }
            }
        }

        impl From<$pin_name> for AnyPin {
            fn from(value: $pin_name) -> Self {
                value.degrade()
            }
        }
    };
}

gpio_pin_def!(IO0, 0);
gpio_pin_def!(IO1, 1);
gpio_pin_def!(IO2, 2);
gpio_pin_def!(IO3, 3);
gpio_pin_def!(IO4, 4);
gpio_pin_def!(IO5, 5);
gpio_pin_def!(IO6, 6);
gpio_pin_def!(IO7, 7);
