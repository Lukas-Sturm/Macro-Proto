use ssd1351::builder::Builder;
use ssd1351::interface::SpiInterface;
use ssd1351::mode::GraphicsMode;

use embedded_graphics::prelude::*;

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::spi::Transfer;
use embedded_hal::blocking::spi::Write;
use embedded_hal::blocking::delay::DelayMs;


pub struct Display<SPI, DC, RST>
where
    SPI: Transfer<u8> + Write<u8>,
    DC: OutputPin, 
    RST: OutputPin 
{
    display: GraphicsMode<SpiInterface<SPI, DC>>,
    reset: RST
}

impl<SPI, DC, RST> Display<SPI, DC, RST>
where
    SPI: Transfer<u8> + Write<u8>,
    DC: OutputPin,
    RST: OutputPin
    {
    pub fn new (spi: SPI, dc: DC, rst: RST) -> Display<SPI, DC, RST> {
        Display {
            display: Builder::new().connect_spi(spi, dc).into(),
            reset: rst
        }
    }

    pub fn init<DL> (&mut self, delay: &mut DL) -> Result<(), ()> 
    where DL: DelayMs<u8> {

        self.display.reset(&mut self.reset, delay).map_err(| _ | ())?;
        self.display.init()?;

        Ok(())
    }

    pub fn clear(&mut self) {
        self.display.clear();
    }

    pub fn get(&mut self) -> &mut GraphicsMode<SpiInterface<SPI, DC>> {
        &mut self.display
    }
}