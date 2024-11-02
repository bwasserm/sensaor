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
use ch32v00x_hal::println;
use ch32v00x_hal::{self as hal};

#[qingke_rt::entry]
fn main() -> ! {
    // Constrain clocking registers
    let p = ch32v0::ch32v003::Peripherals::take().unwrap();

    p.RCC
        .apb2pcenr
        .modify(|_, w| w
            .spi1en().set_bit()
            .adc1en().set_bit()
            .iopaen().set_bit()
            .iopcen().set_bit()
            .iopden().set_bit()
        );
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
    let mut adc_shape = gpioa.pa2.into_analog();
    let mut adc_phase = gpioa.pa1.into_analog();
    let sda = gpioc.pc1.into_floating_input();
    let scl = gpioc.pc2.into_floating_input();
    let _mosi = gpioc
        .pc6
        .into_alternate()
        .set_speed(ch32v00x_hal::gpio::Speed::Mhz50);
    let mut _id0 = gpioc.pc7.into_pull_up_input();
    let mut _id1 = gpiod.pd7.into_pull_up_input();
    let mut _gpio1 = gpioc.pc3.into_floating_input();
    let mut _gpio2 = gpioc.pc0.into_floating_input();

    let mut adc = hal::adc::Adc::new(p.ADC1, &clocks);
    // let _i2c = I2c::i2c1(p.I2C1, scl, sda, I2cConfig::fast_mode(), &mut rcc, &clocks);
    let spi = SpiDriver::new(p.SPI1);
    let mut ws = Ws2812::new(spi);

    // let mut adc_val: u16 = adc.read(&mut pc4).unwrap();

    const NUM_LEDS: usize = 62;
    let mut led_data = [RGB8 { r: 0, g: 0, b: 0}; NUM_LEDS];
    const LEN_SINE_TABLE: usize = 256;
    let sine_table: [u32; LEN_SINE_TABLE] = [
        128, 131, 134, 137, 140, 143, 146, 149, 152, 155, 158, 162, 165, 167, 170, 173,
        176, 179, 182, 185, 188, 190, 193, 196, 198, 201, 203, 206, 208, 211, 213, 215,
        218, 220, 222, 224, 226, 228, 230, 232, 234, 235, 237, 238, 240, 241, 243, 244,
        245, 246, 248, 249, 250, 250, 251, 252, 253, 253, 254, 254, 254, 255, 255, 255,
        255, 255, 255, 255, 254, 254, 254, 253, 253, 252, 251, 250, 250, 249, 248, 246,
        245, 244, 243, 241, 240, 238, 237, 235, 234, 232, 230, 228, 226, 224, 222, 220,
        218, 215, 213, 211, 208, 206, 203, 201, 198, 196, 193, 190, 188, 185, 182, 179,
        176, 173, 170, 167, 165, 162, 158, 155, 152, 149, 146, 143, 140, 137, 134, 131,
        128, 124, 121, 118, 115, 112, 109, 106, 103, 100, 97, 93, 90, 88, 85, 82,
        79, 76, 73, 70, 67, 65, 62, 59, 57, 54, 52, 49, 47, 44, 42, 40,
        37, 35, 33, 31, 29, 27, 25, 23, 21, 20, 18, 17, 15, 14, 12, 11,
        10, 9, 7, 6, 5, 5, 4, 3, 2, 2, 1, 1, 1, 0, 0, 0,
        0, 0, 0, 0, 1, 1, 1, 2, 2, 3, 4, 5, 5, 6, 7, 9,
        10, 11, 12, 14, 15, 17, 18, 20, 21, 23, 25, 27, 29, 31, 33, 35,
        37, 40, 42, 44, 47, 49, 52, 54, 57, 59, 62, 65, 67, 70, 73, 76,
        79, 82, 85, 88, 90, 93, 97, 100, 103, 106, 109, 112, 115, 118, 121, 124
        ];
    let _adc_max: u32 = adc.max_sample().into();
    // let mut red_data: [u32; NUM_LEDS] = [0; NUM_LEDS];
    // let mut green_data: [u32; NUM_LEDS] = [0; NUM_LEDS];
    // let mut blue_data: [u32; NUM_LEDS] = [0; NUM_LEDS];
        
    // let mut data = [RGB8 { r: 0, g: 0, b: 0}; NUM_LEDS];
    // loop {
        //     for j in 0..(256 * 5) {
    //         for i in 0..NUM_LEDS {
    //             data[i] = _wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
    //         }
    //         ws.write(brightness(data.iter().cloned(), 16)).unwrap();
    //         delay.delay_ms(5);
    //         println!("{} {} {}", data[0], data[1], data[2]);
    //     }
    // }
    
    // let scaling: u32 = 1;
    let mut time: u32 = 0;
    loop {
        let phase_delay_a: u32 = adc.read(&mut adc_phase).unwrap();
        let phase_delay = phase_delay_a >> 3;
        let sine_red = sine_table[time as u8 as usize];
        let sine_green = sine_table[(time + phase_delay) as u8 as usize];
        let sine_blue = sine_table[(time + 2 * phase_delay) as u8 as usize];
        let shape_val: u32 = adc.read(&mut adc_shape).unwrap();
        let dc_offset = _adc_max - shape_val;

        let red_val_a: u32 = adc.read(&mut adc_red).unwrap();
        // red_val_a = x / 1024 (x / 2^10)
        // sine_red = x / 256 (x / 2^8)
        // shape_val = x / 1024 (x / 2^10)
        // red_sine_part = x / 2^16
        let red_sine_part = (red_val_a * sine_red * shape_val) >> 12;
        // red_val_a = x / 1024 (x / 2^10)
        // dc_offset = x / 1024 (x / 2^10)
        // red_dc_part = x / 2^16
        let red_dc_part = (red_val_a * dc_offset) >> 4;
        // red_val = x / 256 (x / 2^8)
        let red_val = (red_sine_part + red_dc_part) >> 8;

        let green_val_a: u32 = adc.read(&mut adc_green).unwrap();
        let green_sine_part = (green_val_a * sine_green * shape_val) >> 12;
        let green_dc_part = (green_val_a * dc_offset) >> 4;
        let green_val = (green_sine_part + green_dc_part) >> 8;

        let blue_val_a: u32 = adc.read(&mut adc_blue).unwrap();
        let blue_sine_part = (blue_val_a * sine_blue * shape_val) >> 12;
        let blue_dc_part = (blue_val_a * dc_offset) >> 4;
        let blue_val = (blue_sine_part + blue_dc_part) >> 8;
        
        let mut delay_val: u32 = adc.read(&mut adc_speed).unwrap();
        // println!("{delay_val}");
        delay_val = (delay_val - 420) * 10;
        delay_val = delay_val.max(0).min(1000);
        let delay_ms: u32 = delay_val;

        for i in (1..NUM_LEDS).rev() {
            led_data[i] = led_data[i - 1].clone();
        }
        led_data[0] = RGB8 { r: (red_val) as u8,
                             g: (green_val) as u8,
                             b: (blue_val) as u8};
        ws.write(brightness(led_data.iter().cloned(), 32)).unwrap();
        for _ in 0..delay_ms {
            time = time.wrapping_add(1);
            delay.delay_ms(1);
        }
        // println!("{red_val_a}\t{red_val}\t{delay_val}\t{phase_delay}\t{shape_val}  {dc_offset}");
        // println!("t: {time}\nLED: {} {} {}\ndelay_ms: {delay_ms} phase: {phase_delay_a} shape: {shape_val}", led_data[0], led_data[1], led_data[2]);
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
