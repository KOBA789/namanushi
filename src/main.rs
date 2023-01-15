#![no_main]
#![no_std]

mod uac;

use defmt_rtt as _;
use hal::{
    clocks::init_clocks_and_plls, usb::UsbBus,
    watchdog::Watchdog
};
use panic_probe as _;
use rp_pico::{self as bsp, hal, hal::pac};

use usb_device::{class_prelude::UsbBusAllocator};

use usb_device::prelude::*;

use crate::uac::UsbAudio;

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let bus_allocator = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut usb_audio = UsbAudio::build(&bus_allocator).unwrap();

    let mut usb_dev = UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x6666, 0x0789))
        .max_packet_size_0(64)
        .manufacturer("KOBA789")
        .product("namunushi")
        .serial_number("789")
        .build();

    defmt::println!("start");
    let mut mixed = [0i16; 128];
    let mut bytes1 = [0u8; 256];
    let mut bytes2 = [0u8; 256];
    let mut is_s1_filled = false;
    let mut is_s2_filled = false;
    loop {
        if usb_dev.poll(&mut [&mut usb_audio]) {
            if let Ok(_len) = usb_audio.output1.ep.read(&mut bytes1) {
                is_s1_filled = true;
            }
            if let Ok(_len) = usb_audio.output2.ep.read(&mut bytes2) {
                is_s2_filled = true;
            }
            if is_s1_filled && is_s2_filled {
                is_s1_filled = false;
                is_s2_filled = false;
                let buf1: &[i16] = unsafe {
                    let ptr = &bytes1 as *const [u8] as *const u8 as *const i16;
                    core::slice::from_raw_parts(ptr, 96)
                };
                let buf2 = unsafe {
                    let ptr = &bytes2 as *const [u8] as *const u8 as *const i16;
                    core::slice::from_raw_parts(ptr, 96)
                };
                for (m, (s1, s2)) in mixed.iter_mut().zip(buf1.iter().zip(buf2.iter())) {
                    *m = s1.saturating_add(*s2);
                }
                let out = unsafe {
                    let ptr = &mixed as *const [i16] as *const i16 as *const u8;
                    core::slice::from_raw_parts(ptr, 192)
                };
                usb_audio.input.ep.write(out).ok();
            }
        }
    }
}
