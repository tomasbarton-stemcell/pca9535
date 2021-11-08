extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use hal::digital::blocking::{InputPin, IoPin, OutputPin};
use hal::digital::PinState;

use super::expander::Expander;
use super::GPIOBank;
use super::Register;
use super::Polarity;

/// Single input device pin implementing [`InputPin`] and [`IoPin`] trait.
///
/// The [`ExpanderInputPin`] instance can be used with other pieces of software using [`hal`].
///
/// # Multithreading
/// The pins are not thread safe by default. This needs to be implemented by the user.
pub struct ExpanderInputPin<Ex>
where
    Ex: Expander,
{
    expander: Rc<RefCell<Ex>>,
    bank: GPIOBank,
    pin: u8,
}

/// Single output device pin implementing [`OutputPin`] and [`IoPin`] trait.
///
/// The [`ExpanderInputPin`] instance can be used with other pieces of software using [`hal`].
///
/// # Multithreading
/// The pins are not thread safe by default. This needs to be implemented by the user.
pub struct ExpanderOutputPin<Ex>
where
    Ex: Expander,
{
    expander: Rc<RefCell<Ex>>,
    bank: GPIOBank,
    pin: u8,
}

impl<Ex: Expander> ExpanderInputPin<Ex> {
    /// Create a new input pin
    ///
    /// # Panics
    /// The function will panic if the provided pin is not in the allowed range of 0-7
    pub fn new(expander:& Rc<RefCell<Ex>>, bank: GPIOBank, pin: u8) -> Result<Self, <Ex as Expander>::Error> {
        assert!(pin < 8);

        let register = match bank {
            GPIOBank::Bank0 => Register::ConfigurationPort0,
            GPIOBank::Bank1 => Register::ConfigurationPort1,
        };

        {
            let mut expander = expander.borrow_mut();
            let mut reg_val: u8 = 0x00;

            expander.read_byte(register, &mut reg_val)?;

            expander.write_byte(register, reg_val | (0x01 << pin))?;
        }

        Ok(Self {
            expander: Rc::clone(expander),
            bank,
            pin,
        })
    }

    /// Sets the polarity of the input pin. The input pins have normal polarity by default on device startup.
    /// 
    /// If the polarity is [`Polarity::Normal`] a logic `high` voltage level on the input is detected as `high` in the software.
    /// 
    /// If the polarity is [`Polarity::Inverse`] a logic `high` voltage level on the input is detected as `low` by the software.
    pub fn set_polarity(&mut self, polarity: Polarity) -> Result<(), <Ex as Expander>::Error>{
        let register = match self.bank {
            GPIOBank::Bank0 => Register::PolarityInversionPort0,
            GPIOBank::Bank1 => Register::PolarityInversionPort1,
        };

        let mut expander = self.expander.borrow_mut();
        let mut reg_val: u8 = 0x00;

        expander.read_byte(register, &mut reg_val)?;

        if let Polarity::Normal = polarity {
            expander.write_byte(register, reg_val & !(0x01 << self.pin))?;
        }else {
            expander.write_byte(register, reg_val | (0x01 << self.pin))?;
        }

        Ok(())
    }
}

impl<Ex: Expander> ExpanderOutputPin<Ex> {
    /// Create a new output pin
    ///
    /// # Panics
    /// The function will panic if the provided pin is not in the allowed range of 0-7
    pub fn new(expander: & Rc<RefCell<Ex>>, bank: GPIOBank, pin: u8, state: PinState) -> Result<Self, <Ex as Expander>::Error> {
        assert!(pin < 8);

        let cp_register = match bank {
            GPIOBank::Bank0 => Register::ConfigurationPort0,
            GPIOBank::Bank1 => Register::ConfigurationPort1,
        };

        let op_register = match bank {
            GPIOBank::Bank0 => Register::OutputPort0,
            GPIOBank::Bank1 => Register::OutputPort1,
        };

        {
            let mut expander = expander.borrow_mut();
            let mut reg_val: u8 = 0x00;

            expander.read_byte(op_register, &mut reg_val)?;

            if let PinState::High = state {
                expander.write_byte(op_register, reg_val | (0x01 << pin))?;
            } else {
                expander.write_byte(op_register, reg_val & !(0x01 << pin))?;
            }

            expander.read_byte(cp_register, &mut reg_val)?;

            expander.write_byte(cp_register, reg_val & !(0x01 << pin))?;
        }

        Ok(Self {
            expander: Rc::clone(expander),
            bank,
            pin,
        })
    }
}

impl<Ex: Expander> InputPin for ExpanderInputPin<Ex> {
    type Error = Ex::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        let register = match self.bank {
            GPIOBank::Bank0 => Register::InputPort0,
            GPIOBank::Bank1 => Register::InputPort1,
        };

        let mut reg_val: u8 = 0x00;

        self.expander
            .borrow_mut()
            .read_byte(register, &mut reg_val)?;

        match (reg_val >> self.pin) & 1 {
            1 => Ok(true),
            _ => Ok(false),
        }
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        let register = match self.bank {
            GPIOBank::Bank0 => Register::InputPort0,
            GPIOBank::Bank1 => Register::InputPort1,
        };

        let mut reg_val: u8 = 0x00;

        self.expander
            .borrow_mut()
            .read_byte(register, &mut reg_val)?;

        match (reg_val >> self.pin) & 1 {
            1 => Ok(false),
            _ => Ok(true),
        }
    }
}

impl<Ex: Expander> IoPin<ExpanderInputPin<Ex>, ExpanderOutputPin<Ex>> for ExpanderInputPin<Ex> {
    type Error = Ex::Error;

    fn into_input_pin(self) -> Result<ExpanderInputPin<Ex>, Self::Error> {
        Ok(self)
    }

    fn into_output_pin(self, state: PinState) -> Result<ExpanderOutputPin<Ex>, Self::Error> {
        let cp_register = match self.bank {
            GPIOBank::Bank0 => Register::ConfigurationPort0,
            GPIOBank::Bank1 => Register::ConfigurationPort1,
        };

        let op_register = match self.bank {
            GPIOBank::Bank0 => Register::OutputPort0,
            GPIOBank::Bank1 => Register::OutputPort1,
        };

        {
            let mut expander = self.expander.borrow_mut();
            let mut reg_val: u8 = 0x00;

            expander.read_byte(op_register, &mut reg_val)?;

            if let PinState::High = state {
                expander.write_byte(op_register, reg_val | (0x01 << self.pin))?;
            } else {
                expander.write_byte(op_register, reg_val & !(0x01 << self.pin))?;
            }

            expander.read_byte(cp_register, &mut reg_val)?;

            expander.write_byte(cp_register, reg_val & !(0x01 << self.pin))?;
        }

        Ok(ExpanderOutputPin {
            expander: self.expander,
            bank: self.bank,
            pin: self.pin,
        })
    }
}

impl<Ex: Expander> OutputPin for ExpanderOutputPin<Ex> {
    type Error = Ex::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        let register = match self.bank {
            GPIOBank::Bank0 => Register::OutputPort0,
            GPIOBank::Bank1 => Register::OutputPort1,
        };

        let mut expander = self.expander.borrow_mut();
        let mut reg_val: u8 = 0x00;

        expander.read_byte(register, &mut reg_val)?;

        expander.write_byte(register, reg_val & !(0x01 << self.pin))
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let register = match self.bank {
            GPIOBank::Bank0 => Register::InputPort0,
            GPIOBank::Bank1 => Register::InputPort1,
        };

        let mut expander = self.expander.borrow_mut();
        let mut reg_val: u8 = 0x00;

        expander.read_byte(register, &mut reg_val)?;

        expander.write_byte(register, reg_val | (0x01 << self.pin))
    }
}

impl<Ex: Expander> IoPin<ExpanderInputPin<Ex>, ExpanderOutputPin<Ex>> for ExpanderOutputPin<Ex> {
    type Error = Ex::Error;

    fn into_input_pin(self) -> Result<ExpanderInputPin<Ex>, Self::Error> {
        let register = match self.bank {
            GPIOBank::Bank0 => Register::ConfigurationPort0,
            GPIOBank::Bank1 => Register::ConfigurationPort1,
        };

        {
            let mut expander = self.expander.borrow_mut();
            let mut reg_val: u8 = 0x00;

            expander.read_byte(register, &mut reg_val)?;

            expander.write_byte(register, reg_val | (0x01 << self.pin))?;
        }

        Ok(ExpanderInputPin {
            expander: self.expander,
            bank: self.bank,
            pin: self.pin,
        })
    }

    fn into_output_pin(self, state: PinState) -> Result<ExpanderOutputPin<Ex>, Self::Error> {
        let register = match self.bank {
            GPIOBank::Bank0 => Register::OutputPort0,
            GPIOBank::Bank1 => Register::OutputPort1,
        };

        {
            let mut expander = self.expander.borrow_mut();
            let mut reg_val: u8 = 0x00;

            expander.read_byte(register, &mut reg_val)?;

            if let PinState::High = state {
                expander.write_byte(register, reg_val | (0x01 << self.pin))?;
            } else {
                expander.write_byte(register, reg_val & !(0x01 << self.pin))?;
            }
        }

        Ok(self)
    }
}
