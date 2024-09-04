#![no_std]
#![no_main]

use hal::println;
use panic_halt as _;

use ch32v00x_hal as hal;
use ch32v00x_hal::prelude::*;
use qingke::riscv;

#[qingke_rt::entry]
fn main() -> ! {
    hal::debug::SDIPrint::enable();

    // println!("Hello world from ch32v003!");
    // To ensure safe access to peripherals, all types are !Copy singletons. The
    // PAC makes us pass these marker types around to access the registers
    let p = ch32v0::ch32v003::Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let _clocks = rcc.config.freeze();

    let gpiod = p.GPIOD.split(&mut rcc);
    // let gpioa = p.GPIOA.split(&mut rcc);

    let mut led1 = gpiod.pd6.into_open_drain_output();
    // let mut led2 = gpioa.pa2.into_open_drain_output();
    let mut led2 = gpiod.pd4.into_open_drain_output();

    loop {
        // led1.toggle();
        // led2.toggle();
        led1.toggle();
        led2.toggle();
        // println!("led toggle");

        unsafe {
            riscv::asm::delay(4800000);
        }
    }
}
