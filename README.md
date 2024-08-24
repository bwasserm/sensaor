# senSAOr - a sensor SAO

A Simple Add-On (SAO) for a conference badge with a modular sensor port and a reactive multicolor LED.

# Project Goals
* Design and make an SAO in time for Hackaday Supercon 2024
* Make it inexpensive enough and simple enough to make a bunch and trade with others
* Blinky, but more interesting than only blinky
* Cool design
* If writing firmware, write it in Rust
* If using a microcontroller, use Risc-V

# High Level Design

SAO is a microcontroller that reads a sensor from an input pin, and uses the measured voltage to drive the color/brightness/pattern of an LED.
Create ports for multiple different sensors to be used, with the ability to solder in different signal conditioning.
Use ID pins or a solderable resistors to tell firmware how to "read" the sensor and drive the LED in response.

Example sensor modes:
* Optional invert response pin
* 0-3.3V
* Rising edge
* Falling edge
* Variance of input
* Frequency of edges

# Resources
* [Shitty Add-On Version 2.0 Specification](https://docs.google.com/document/u/0/d/1EJqvkkLMAPsQ9VWF5A4elWoi0qMlKyr5Giw5rqRmtnM/mobilebasic?pli=1)
* [Opensource toolchain for WCH CH32V RISC-V 32bit MCU](https://github.com/cjacker/opensource-toolchain-ch32v)
* [ch32-hal](https://github.com/ch32-rs/ch32-hal)