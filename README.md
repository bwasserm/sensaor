# senSAOr - a sensor SAO

A Simple Add-On (SAO) for a conference badge with a modular sensor port and a reactive multicolor LED.

# Project Goals
* Design and make an SAO in time for Hackaday Supercon 2024
* Make it inexpensive enough and simple enough to make a bunch and trade with others
* Blinky, but more interesting than only blinky
* Cool design
* If writing firmware, write it in Rust
* If using a microcontroller, use RISC-V

# High Level Design

SAO is a microcontroller that reads a sensor from an input pin, and uses the measured voltage to drive the color/brightness/pattern of an LED.
Create ports for multiple different sensors to be used, with the ability to solder in different signal conditioning.
Use ID pins or a solderable resistors to tell firmware how to "read" the sensor and drive the LED in response.

# Expected Behaviors

The inputs are read to set the color of the bottom LED. Every cycle, the colors are clocked one LED up the board, and potentially out to an attached strand of addition WS2812s attached at the top. All inputs are read in [0V, 3.3V] as low to high.

## Red/Green/Blue
These inputs are read to set the base intensity of the LEDs accordingly.

## Speed
This sets the delay between update cycles from fast (low) to slow (high). Full low has no added delay between cycles, and full high has about 1s between cycles.

## Shape
This is the mixture ration between setting the output intensity of the colors based directly on the input (low) vs also mixing in a sine wave (high) to the brightness (off to the intensity set by the color input).

## Phase
When mixing in the sine wave, this delays the mix of the sine wave applied to green and blue relative to red. Low is the same phase of sine wave is applied to all three colors, and high is about 120 degrees of delay to green and 240 degrees of delay to blue.

## ID pins
Currently unused

## I2C
Currently unused. Set as high impedance inputs.

## GPIO\[12\]
Currently unused. Set as high impedance inputs.

## More WS2812
Connect more Neopixels here to continue clocking out colors beyond the three on the SAO.

## SAOAO
Hopefully "standard" SAOAO port to connect an SAOAO.

# Resources
* [Shitty Add-On Version 2.0 Specification](https://docs.google.com/document/u/0/d/1EJqvkkLMAPsQ9VWF5A4elWoi0qMlKyr5Giw5rqRmtnM/mobilebasic?pli=1)
* [Opensource toolchain for WCH CH32V RISC-V 32bit MCU](https://github.com/cjacker/opensource-toolchain-ch32v)
* [ch32-hal](https://github.com/ch32-rs/ch32-hal)
* [Rust on the CH32V003](https://noxim.xyz/blog/rust-ch32v003/)
* [CH32V00X-hal](https://github.com/ch32-rs/ch32v00x-hal)
* [wlink application](https://github.com/ch32-rs/wlink)
* [wlink with CH32V003](https://github.com/ch32-rs/wlink/blob/main/docs/CH32V003.md)
* [CH32V003F4P6-R0 image](https://github.com/openwch/ch32v003/blob/main/SCHPCB/CH32V003F4P6-R0-1v1/image/board1.jpg)
* [CH32V003F4P6-R0 schematic](https://github.com/openwch/ch32v003/blob/main/SCHPCB/CH32V003F4P6-R0-1v1/SCH_PCB/CH32V003F4P6-R0-1v1.pdf)
* [ch32v003.svd](https://github.com/ch32-rs/ch32-rs/blob/main/svd/fixed/ch32v003.svd)
* [CH32V003 Reference Manual](https://www.wch-ic.com/downloads/file/358.html)
* [CH32V003 Processor Manual](https://www.wch-ic.com/downloads/file/369.html)
* [CH32V003 Datasheet](https://www.wch-ic.com/downloads/file/359.html)
* [RISC-V Spec](https://riscv.org/wp-content/uploads/2019/12/riscv-spec-20191213.pdf)
* [SAOAO](https://hackaday.io/project/198060-yo-dawg-sao-introducing-saoao)


# Implementation blog

## Getting rust to build

Getting rust to compile was mostly following the [Rust on the CH32V003](https://noxim.xyz/blog/rust-ch32v003/) blog, with some off-roading. LLVM had already
picked up the support for `rv32ec` (the `e` extension is the core `i` architecture, but with 16 registers instead of 32 for smaller silicon), but rustc didn't know
the riscv32ec-unknown-none-elf target. Eventually I figured out I didn't need to make the hack to rustc either, if I provided a target .json file to build against.
However, having a local copy of llvm and the binary tools was useful, such as for llvm-objdump.

Eventually I found that much of the work on the blog was "done" in the [CH32V00X-hal](https://github.com/ch32-rs/ch32v00x-hal) project, so I cloned that next to this project.
To avoid copying files, I hacked this to link to it. .cargo/config.toml points the target to `ch32v00x-hal/riscv32ec-unknown-none-elf.json`. The target json I copied from the noxim
blog is included for good measure.

I also found that the `-Tmemory.x` and `-Tlink.x` arguments got redundant with the CH32V00X-hal repo and build.rs (adding that repo as a Cargo dep changes things), so it may
require some experimentation.

Note on `memory.x`:
Looking at the memory map, it appears that program flash is a 16KB block starting at address `0x0800 0000`. However, ch32v00x-hal and the noxim blog both define
`FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 16K` at address `0x0000 0000`, which is the `Aliased to Flash or system memory depending on software configuration` block of a much
larger size. This block defaults to flash, and the program counter (`pc`) will default to address `0x0000 0000` when the chip boots. So even though it seems unintuitive, set the `FLASH`
address in `memory.x` to be 0x0. 

I went down an exploratory path using `ch32v00x-hal/link-qkv2.x` instead of finding `link.x`, which required modifying `INCLUDE device.x` to become `INCLUDE svd2rs/device.x`,
after also `cargo install svd2rust`, and running it on [cj32v003.svd](https://github.com/ch32-rs/ch32-rs/blob/main/svd/fixed/ch32v003.svd) to generate `device.x`. However, after
adding in [CH32V00X-hal](https://github.com/ch32-rs/ch32v00x-hal), I found I didn't need this.


## Getting blinky to run

I'm playing with the CH32V003F4P6-R0 dev board, pictured [here](https://github.com/openwch/ch32v003/blob/main/SCHPCB/CH32V003F4P6-R0-1v1/image/board1.jpg), and referencing
the [schematic](https://github.com/openwch/ch32v003/blob/main/SCHPCB/CH32V003F4P6-R0-1v1/SCH_PCB/CH32V003F4P6-R0-1v1.pdf) as well was super handy. Especially because I tried using
PA1 as an arbitrary GPIO pin choice, but it is already tied to the oscillator.

I used the [wlink](https://github.com/ch32-rs/wlink) application with the WLinkE . It turned out to be surprisingly easy to run and use to dump registers and flash. It's not even
necessary to convert the elf file produced by cargo/rustc into an Intel Hex file or a bin first.

```
cargo install wlink
```

* Programming command: `sudo ~/.cargo/bin/wlink -v --chip ch32v003 flash target/riscv32ec-unknown-none-elf/debug/sensaor`
* Get registers command: `sudo ~/.cargo/bin/wlink -v --chip ch32v003 regs`
* Dump the beginning of program flash: `sudo ~/.cargo/bin/wlink -v --chip ch32v003 dump 0x00000000 400`

The sudo is becuase I'm too lazy to figure out which group to add myself to to get permissions to talk to the programmer.

`wlink` also has a serial terminal functionality that works with the WLinkE debugger I'm using. However, my WLinkE isn't showing up as a serial device (`/dev/ttyASM0` or similar),
so I can't connect to that. I'm guessing it's a udev rule I need to update, since `lsusb` indicates multiple modem/serial interfaces on the USB device. Probably an Arch (BTW) config
issue on my end.

Using the LEDs on the dev board, it's important to set the GPIO pins into open drain mode, not push-pull. With the extra resistors from push-pull, not enough current flows to turn
on the LEDs (since they already have resistors installed inline).

### WLink on Windows

On my Windows 10/11 machines, I was getting `Error: USB error: Entity not found` errors when trying to run `wlink status`. This is because the wlink doesn't activate the correct drivers by default.

* See this [github issue](https://github.com/ch32-rs/wlink/issues/5) for an example
* Install [zadig](https://zadig.akeo.ie/)
* Select `WCH-Link (Interface 0)` and install the driver `WinUSB`.

## Reading the ADC to modify the blink

First, I moved the binaries into src/bin/ so I could build multiple at once, and therefore make things easier to have multiple apps in work.

The greatest challenge reading the ADC was finding the right part of the HAL to use. Eventually I found `ch32v00x-hal/src/adc.rs` `OneShot::read()`, but due to not knowing rust's import system well enough, spent way too long getting it to build. Through help, the answer is adding `embedded-hal-02` as a dependency, and `use embedded_hal_02::adc::OneShot;`. While I don't have a great analog source yet, the blink rate does apear to change when I touch the input pin, so I think it might be working.

## Driving the Neopixels

I wanted to use a Neopixel (WS2811/WS2812) library to drive them, but those rely on a timer or SPI implementation for the chip. The hal I'm using for the CH32V003 doesn't implement SPI. After getting annoyed with the traits required to get the existing timer library to work, I figured implementing enough of SPI to write would be easier. With enough time with the manual, and some C examples of setting up SPI on this chip to compare which bits are set in the control registers, I got my neopixel string to change colors.


# Programming tips

Including any `println!()` calls without having the debugger connected and watching the serial port will cause the application to block upon a print.

With serial console:

```
cargo b --release; wlink -v --chip ch32v003 flash .\target\riscv32ec-unknown-none-elf\release\sao --enable-sdi-print --watch-serial
```

Without serial console:

```
cargo b --release; wlink -v --chip ch32v003 flash .\target\riscv32ec-unknown-none-elf\release\sao
```

Note that using `println!()` when sdi print is not enabled or the debugger is not watching serial will cause the program to lock up at the first `println!()`.