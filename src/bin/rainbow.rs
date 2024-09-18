#![no_main]
#![no_std]

use ws2812_spi::{self, Ws2812};


use embedded_hal_1::spi;

use smart_leds::{brightness, SmartLedsWrite, RGB8};

use embedded_hal_1::delay::DelayNs;
use panic_halt as _;

use ch32v00x_hal::{self as hal};
use ch32v00x_hal::prelude::*;

#[qingke_rt::entry]
fn main() -> ! {

    // Constrain clocking registers
    let p = ch32v0::ch32v003::Peripherals::take().unwrap();

    p.RCC.apb2pcenr.modify(|_, w| w
        .spi1en().set_bit()
        .adc1en().set_bit()
        .iopcen().set_bit()
    );
    p.RCC.apb2prstr.modify(|_, w| w
        .spi1rst().set_bit()
    );
    p.RCC.apb2prstr.modify(|_, w| w
        .spi1rst().clear_bit()
    );
    p.SPI1.ctlr1.modify(|_, w| { w
        .bidimode().set_bit()
        .bidioe().set_bit()
        .crcen().clear_bit()
        .crcnext().clear_bit()
        .dff().clear_bit()
        .rxonly().clear_bit()
        .ssm().set_bit()
        .ssi().set_bit()
        .lsbfirst().clear_bit()
        .spe().clear_bit()
        .br().variant(2_u8)  // 24MHz / 8 => 3Mhz
        .mstr().set_bit()
        .cpha().clear_bit()
        .cpol().clear_bit()
    });
    p.SPI1.ctlr1.modify(|_, w| { w
        .spe().set_bit()
    });
    let mut rcc = p.RCC.constrain();    
    let clocks = rcc.config.freeze();
    let gpioc = p.GPIOC.split(&mut rcc);

    // Get delay provider
    let mut delay = hal::delay::CycleDelay::new(&clocks);

    let _mosi = gpioc.pc6.into_alternate().set_speed(ch32v00x_hal::gpio::Speed::Mhz50);
    let _sck = gpioc.pc5.into_alternate().set_speed(ch32v00x_hal::gpio::Speed::Mhz50);
    let spi = SpiDriver::new(
        p.SPI1,
    );

    let mut ws = Ws2812::new(spi);

    const NUM_LEDS: usize = 59;
    let mut data = [RGB8::default(); NUM_LEDS];

    loop {
        for j in 0..(256 * 5) {
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
            }
            ws.write(brightness(data.iter().cloned(), 32)).unwrap();
            delay.delay_ms(5);
        }
    }
}

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

#[derive(Debug)]
struct SpiDriver{
    spi: ch32v00x_hal::pac::SPI1,
}

impl SpiDriver {
    pub fn new(spi: ch32v00x_hal::pac::SPI1) -> Self
    {
        Self {
            spi,
        }
    }
}

impl spi::ErrorType for SpiDriver {
    type Error = spi::ErrorKind;
}

impl spi::SpiBus for SpiDriver{
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        todo!()
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        while self.spi.statr.read().txe().bit_is_clear() {}
        for word in words {
            self.spi.datar.write(|w| unsafe { w.bits(*word as u32) } );
        }
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}