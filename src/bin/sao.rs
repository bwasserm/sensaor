#![no_main]
#![no_std]

use ch32v00x_hal::i2c::{I2c, I2cConfig};
use ws2812_spi::{self, Ws2812};

use embedded_hal_02::adc::OneShot;
use embedded_hal_1::spi;

use smart_leds::{brightness, SmartLedsWrite, RGB8};

use embedded_hal_1::delay::DelayNs;
use panic_halt as _;

use ch32v00x_hal::prelude::*;
use ch32v00x_hal::{self as hal};

#[qingke_rt::entry]
fn main() -> ! {
    // Constrain clocking registers
    let p = ch32v0::ch32v003::Peripherals::take().unwrap();

    p.RCC
        .apb2pcenr
        .modify(|_, w| w.spi1en().set_bit().adc1en().set_bit().iopaen().set_bit().iopcen().set_bit().iopden().set_bit());
    p.RCC.apb2prstr.modify(|_, w| w.spi1rst().set_bit());
    p.RCC.apb2prstr.modify(|_, w| w.spi1rst().clear_bit());
    p.SPI1.ctlr1.modify(|_, w| {w
        .bidimode()
            .set_bit()
        .bidioe()
            .set_bit()
        .crcen()
            .clear_bit()
        .crcnext()
            .clear_bit()
        .dff()
            .clear_bit()
        .rxonly()
            .clear_bit()
        .ssm()
            .set_bit()
        .ssi()
            .set_bit()
        .lsbfirst()
            .clear_bit()
        .spe()
            .clear_bit()
        .br()
            .variant(2_u8) // 24MHz / 8 => 3Mhz
        .mstr()
            .set_bit()
        .cpha()
            .clear_bit()
        .cpol()
            .clear_bit()
    });
    p.SPI1.ctlr1.modify(|_, w| w.spe().set_bit());
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.config.freeze();

    // Get delay provider
    let mut delay = hal::delay::CycleDelay::new(&clocks);

    let gpioa = p.GPIOA.split(&mut rcc);
    let gpioc = p.GPIOC.split(&mut rcc);
    let gpiod = p.GPIOD.split(&mut rcc);

    let mut adc_red = gpiod.pd6.into_analog();
    let mut adc_green = gpiod.pd5.into_analog();
    let mut adc_blue = gpiod.pd4.into_analog();
    let mut adc_speed = gpioc.pc4.into_analog();
    let mut _adc_shape = gpioa.pa2.into_analog();
    let mut adc_phase = gpioa.pa1.into_analog();
    let sda = gpioc.pc1.into_alternate_open_drain();
    let scl = gpioc.pc2.into_alternate_open_drain();
    let _mosi = gpioc
        .pc6
        .into_alternate()
        .set_speed(ch32v00x_hal::gpio::Speed::Mhz50);
    let mut _id0 = gpioc.pc7.into_pull_up_input();
    let mut _id1 = gpiod.pd7.into_pull_up_input();
    let mut _gpio1 = gpioc.pc3.into_floating_input();
    let mut _gpio2 = gpioc.pc0.into_floating_input();

    let mut adc = hal::adc::Adc::new(p.ADC1, &clocks);
    let _i2c = I2c::i2c1(p.I2C1, scl, sda, I2cConfig::fast_mode(), &mut rcc, &clocks);
    let spi = SpiDriver::new(p.SPI1);
    let mut ws = Ws2812::new(spi);

    // let mut adc_val: u16 = adc.read(&mut pc4).unwrap();

    const NUM_LEDS: usize = 62;
    let mut led_data = [RGB8::default(); NUM_LEDS];

    const LEN_SINE_TABLE: usize = 360;
    let sine_table: [u32; LEN_SINE_TABLE] = [
        128, 130, 132, 134, 136, 139, 141, 143, 145, 147, 150, 152, 154, 156, 158, 160,
        163, 165, 167, 169, 171, 173, 175, 177, 179, 181, 183, 185, 187, 189, 191, 193,
        195, 197, 199, 201, 202, 204, 206, 208, 209, 211, 213, 214, 216, 218, 219, 221,
        222, 224, 225, 227, 228, 229, 231, 232, 233, 234, 236, 237, 238, 239, 240, 241,
        242, 243, 244, 245, 246, 247, 247, 248, 249, 249, 250, 251, 251, 252, 252, 253,
        253, 253, 254, 254, 254, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        254, 254, 254, 253, 253, 253, 252, 252, 251, 251, 250, 249, 249, 248, 247, 247,
        246, 245, 244, 243, 242, 241, 240, 239, 238, 237, 236, 234, 233, 232, 231, 229,
        228, 227, 225, 224, 222, 221, 219, 218, 216, 214, 213, 211, 209, 208, 206, 204,
        202, 201, 199, 197, 195, 193, 191, 189, 187, 185, 183, 181, 179, 177, 175, 173,
        171, 169, 167, 165, 163, 160, 158, 156, 154, 152, 150, 147, 145, 143, 141, 139,
        136, 134, 132, 130, 128, 125, 123, 121, 119, 116, 114, 112, 110, 108, 105, 103,
        101, 99, 97, 95, 92, 90, 88, 86, 84, 82, 80, 78, 76, 74, 72, 70,
        68, 66, 64, 62, 60, 58, 56, 54, 53, 51, 49, 47, 46, 44, 42, 41,
        39, 37, 36, 34, 33, 31, 30, 28, 27, 26, 24, 23, 22, 21, 19, 18,
        17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 8, 7, 6, 6, 5, 4,
        4, 3, 3, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 4, 5, 6,
        6, 7, 8, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 21,
        22, 23, 24, 26, 27, 28, 30, 31, 33, 34, 36, 37, 39, 41, 42, 44,
        46, 47, 49, 51, 53, 54, 56, 58, 60, 62, 64, 66, 68, 70, 72, 74,
        76, 78, 80, 82, 84, 86, 88, 90, 92, 95, 97, 99, 101, 103, 105, 108,
        110, 112, 114, 116, 119, 121, 123, 125];
    let _adc_max: u32 = adc.max_sample().into();
    let mut red_data: [u32; NUM_LEDS] = [0; NUM_LEDS];
    let mut green_data: [u32; NUM_LEDS] = [0; NUM_LEDS];
    let mut blue_data: [u32; NUM_LEDS] = [0; NUM_LEDS];
        
    let scaling: u32 = 1;
    let mut time = 0;
    loop {
        red_data.rotate_right(1);
        // red_data[0] = adc.read(&mut adc_red).unwrap();
        red_data[0] = time % 10 * 300;
        green_data.rotate_right(1);
        // green_data[0] = adc.read(&mut adc_green).unwrap();
        green_data[0] = time % 20 * 200;
        blue_data.rotate_right(1);
        // blue_data[0] = adc.read(&mut adc_blue).unwrap();
        blue_data[0] = time % 30 * 100;
        
        // let phase_delay: u32 = adc.read(&mut adc_phase).unwrap();
        let phase_delay = 0;
        // let delay_val: u32 = adc.read(&mut adc_speed).unwrap();
        let delay_val = 5;

        let t_red = time;
        let t_green = (time + phase_delay) % 360;
        let t_blue = (time + 2 * phase_delay) % 360;
        for _ in 0..LEN_SINE_TABLE {
            for led_num in 0..NUM_LEDS {
                led_data[led_num] = RGB8 { r: (sine_table[t_red as usize] * red_data[led_num] / scaling) as u8,
                                           g: (sine_table[t_green as usize] * green_data[led_num] / scaling) as u8,
                                           b: (sine_table[t_blue as usize] * blue_data[led_num] / scaling) as u8};
            }
            ws.write(brightness(led_data.iter().cloned(), 32)).unwrap();
            delay.delay_ms(delay_val);
            time = (time + 1) % LEN_SINE_TABLE as u32;
        }
    }
}

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn _wheel(mut wheel_pos: u8) -> RGB8 {
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
struct SpiDriver {
    spi: ch32v00x_hal::pac::SPI1,
}

impl SpiDriver {
    pub fn new(spi: ch32v00x_hal::pac::SPI1) -> Self {
        Self { spi }
    }
}

impl spi::ErrorType for SpiDriver {
    type Error = spi::ErrorKind;
}

impl spi::SpiBus for SpiDriver {
    fn read(&mut self, _words: &mut [u8]) -> Result<(), Self::Error> {
        todo!()
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        while self.spi.statr.read().txe().bit_is_clear() {}
        for word in words {
            self.spi.datar.write(|w| unsafe { w.bits(*word as u32) });
        }
        Ok(())
    }

    fn transfer(&mut self, _read: &mut [u8], _write: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }

    fn transfer_in_place(&mut self, _words: &mut [u8]) -> Result<(), Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}
