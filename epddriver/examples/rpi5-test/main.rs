use embedded_hal as hal;
use rppal::Delay;
use rppal::Pins;
use rppal::Spi;
use epddriver::EPDDriver;

#[cfg(feature = "rpi5")]
const EPD_SCK_PIN: u8 = 11;
const EPD_MOSI_PIN: u8 = 10;
const EPD_CS_M_PIN: u8 = 8;
const EPD_CS_S_PIN: u8 = 7;
const EPD_DC_PIN: u8 = 25;
const EPD_RST_PIN: u8 = 17;
const EPD_BUSY_PIN: u8 = 24;
const EPD_PWR_PIN: u8 = 18;

fn main() {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss)
    let cs_m =Gpio::
    let cs_s =
    let busy =
    let dc =
    let reset
    let power =

    let delay = Delay::new();

    let driver = EPDDriver::new();
    driver.power_on();
    driver.init();
    driver.write();
}
