extern crate anyhow;
extern crate embedded_hal;
use anyhow::Result;
use embedded_hal::blocking::{delay, i2c, spi};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use slog::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Error<SpiError, PinError> {
    Spi(SpiError),
    Pin(PinError),
}

pub struct EPDDriverConfig {
    pub poll_rate_ms: u32,
}

impl Default for EPDDriverConfig {
    fn default() -> Self {
        Self { poll_rate_ms: 5 }
    }
}

// Needs seperate bus since there are two SPI devices that need to be operated in conjuntion
pub struct EPDDriver<SPI, CS, BUSY, RESET, Delay> {
    config: EPDDriverConfig,
    spi: SPI,
    cs_m: CS,
    cs_s: CS,
    busy: BUSY,
    reset: RESET,
    delay: Delay,
}

impl<CS, SPI, BUSY, RESET, Delay> EPDDriver<CS, SPI, BUSY, RESET, Delay>
where
    CS: OutputPin<Error = PinError>,
    SPI: SpiBus,
    BUSY: InputPin<Error = PinError>,
    RESET: OutputPin<Error = PinError>,
    Delay: delay::DelayMs<u32>,
{
    pub fn new(
        config: EPDDriverConfig,
        spi: SPI,
        cs_m_pin: CS,
        cs_s_pin: CS,
        busy_pin: BUSY,
        reset_pin: RESET,
        delay: Delay,
    ) -> Result<Self> {
        Ok(Self {
            config,
            spi,
            cs_m,
            cs_s,
            busy,
            reset,
            delay,
        })
    }

    fn cs_all_low(&mut self) {
        self.cs_m.set_low();
        self.cs_s.set_low();
    }

    fn cs_all_high(&mut self) {
        self.cs_m.set_high();
        self.cs_s.set_high();
    }

    pub fn power_on(self: &mut Self) -> Result<()> {
        // Power on the display
        debug!("Write PON");
        self.cs_all_low()?;
        self.spi.write(&[0x04])?;
        self.cs_all_high()?;
        self.wait_for_busy();

        self.delay.delay_ms(50);

        debug!("Write DRF");
        self.cs_all_low();
        self.spi.write(&[0x12, 0x00])?;
        self.cs_all_high();
        self.wait_for_busy();

        debug!("Write POF");
        self.cs_all_low();
        self.spi.write(&[0x02, 0x00])?;
        self.cs_all_high();

        info!("Power on complete");
        Ok(())
    }

    pub fn init(self: &mut Self) -> Result<()> {
        // SPI INIT

        self.reset();
        self.wait_for_busy();

        self.cs_m.set_low();
        self.spi
            .write([0x74, 0xC0, 0x1C, 0x1C, 0xCC, 0xCC, 0xCC, 0x15, 0x15, 0x55])?;
        self.cs_all_high();

        self.cs_all_low();
        self.spi
            .write(&[0xF0, 0x49, 0x55, 0x13, 0x5D, 0x05, 0x10])?;
        self.cs_all_high();

        self.cs_all_low();
        self.spi.write(&[0x00, 0xDF, 0x69])?;
        self.cs_all_high();

        self.cs_all_low();
        self.spi.write(&[0x50, 0xF7])?;
        self.cs_all_high();

        self.cs_all_low();
        self.spi.write(&[0x00, 0xF7])?;
        self.cs_all_high();

        self.cs_all_low();
        self.spi.write(&[0x60, 0x03, 0x03])?;
        self.cs_all_high();

        self.cs_all_low();
        self.spi.write(&[0x86, 0x10])?;
        self.cs_all_high();

        self.cs_all_low();
        self.spi.write(&[0xE3, 0x22])?;
        self.cs_all_high();

        self.cs_all_low();
        self.spi.write(&[0xE0, 0x01])?;
        self.cs_all_high();

        self.cs_all_low();
        self.spi.write(&[0x61, 0x04, 0xB0, 0x03, 0x20])?;
        self.cs_all_high();

        self.cs_m.set_low();
        self.spi
            .write(&[0x01, 0x0F, 0x00, 0x28, 0x2C, 0x28, 0x38])?;
        self.cs_all_high();

        self.cs_m.set_low();
        self.spi.write(&[0xB6, 0x07])?;
        self.cs_all_high();

        self.cs_m.set_low();
        self.spi.write(&[0x06, 0xE8, 0x28])?;
        self.cs_all_high();

        self.cs_m.set_low();
        self.spi.write(&[0xB7, 0x01])?;
        self.cs_all_high();

        self.cs_m.set_low();
        self.spi.write(&[0x05, 0xE8, 0x28])?;
        self.cs_all_high();

        self.cs_m.set_low();
        self.spi.write(&[0xB0, 0x01])?;
        self.cs_all_high();

        self.cs_m.set_low();
        self.spi.write(&[0xB1, 0x02])?;
        self.cs_all_high();

        info!("Init complete");
        Ok(())
    }

    /// Performs a hardware reset of the EPD display using the reset pin.
    ///
    /// This function toggles the reset pin in a specific sequence with timing delays:
    /// 1. Sets reset high (3ms)
    /// 2. Sets reset low (3ms)
    /// 3. Sets reset high (3ms)
    /// 4. Sets reset low
    pub fn reset(self: &mut Self) {
        self.reset.set_high();
        self.delay.delay_ms(3);
        self.reset.set_low();
        self.delay.delay_ms(3);
        self.reset.set_high();
        self.delay.delay_ms(3);
        self.reset.set_low();
        info!("Reset complete");
    }

    pub fn wait_for_busy(&mut self, poll_rate: hal::DelayMs<u32>) {
        // Wait for the display to become ready
        while self.busy_pin.is_low() {
            self.delay.delay_ms(self.config.poll_rate_ms);
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.cs_m.set_low();
        // WRITE HALF 1
        self.spi.write(data)?;
        self.cs_all_high();

        // WRITE HALF 2
        self.cs_s.set_low();
        self.spi.write(data)?;
        self.cs_all_high();

        Ok(())
    }

    pub fn sleep(&mut self) -> Result<()> {
        self.cs_all_low();
        self.spi.write(&[0x07, 0xA5])?;
        self.cs_all_high();
        info!("Putting display to sleep");

        self.delay.delay_ms(2000);

        //EXIT??

        Ok(())
    }
}
