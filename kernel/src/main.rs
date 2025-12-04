#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

#[macro_use]
pub mod terminal;
mod mem;

use core::panic::PanicInfo;
use alloc::vec;
use bootloader_api::{entry_point, BootInfo, config::{BootloaderConfig, Mapping}};
use terminal::Color;
pub(crate) use core::fmt::Write;

fn main() {
    println!("Hello, Sailor!");
    let x = vec![1, 2, 3];
    println!("{:?}", x);
}

static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(entry, config = &BOOTLOADER_CONFIG);

fn entry(info: &'static mut BootInfo) -> ! {
    terminal::init(info.framebuffer.as_mut().unwrap(), Color::CYAN);

    main();

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!(color = Color::RED, "KERNEL PANIC: {}", info.message());

    loop {
        x86_64::instructions::hlt();
    }
}
