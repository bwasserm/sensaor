#![no_std]
#![no_main]

use embedded_hal_02::adc::OneShot;
use embedded_hal_1::delay::DelayNs;
use panic_halt as _;

use ch32v00x_hal::{self as hal};
use ch32v00x_hal::prelude::*;

#[qingke_rt::entry]
fn main() -> ! {
    // hal::debug::SDIPrint::enable();

    // println!("Hello world from ch32v003!");
    // To ensure safe access to peripherals, all types are !Copy singletons. The
    // PAC makes us pass these marker types around to access the registers
    let p = ch32v0::ch32v003::Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let clocks = rcc.config.freeze();

    let gpiod = p.GPIOD.split(&mut rcc);
    let gpioc = p.GPIOC.split(&mut rcc);

    let mut led1 = gpiod.pd6.into_open_drain_output();
    // let mut led2 = gpioa.pa2.into_open_drain_output();
    let mut led2 = gpiod.pd4.into_open_drain_output();

    let mut pc4 = gpioc.pc4.into_analog();
    
    // let a2 = 2;
    
    let mut delay = hal::delay::CycleDelay::new(&clocks);
    let mut adc = hal::adc::Adc::new(p.ADC1, &clocks);
    
    let mut adc_val: u16 = adc.read(&mut pc4).unwrap();

    loop {
        led1.toggle();
        
        let mut counter = 0;
        while counter <= adc_val {
            adc_val = adc.read(&mut pc4).unwrap();
            led2.toggle();
            delay.delay_ms(100);
            counter += 10;
        }
    }
}
