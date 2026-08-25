#![feature(asm_experimental_arch)]
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::clock::CpuClock;
use esp_hal::main;

static mut UART_FIFO_REG: *mut u32 = 0x3FF40000 as *mut u32;
static mut GPIO_PIN_ENABLE_REG: *mut u32 = 0x3FF44024 as *mut u32;
static mut GPIO_W1TS_OUT_REG: *mut u32 = 0x3FF44008 as *mut u32;
static mut GPIO_W1TC_OUT_REG: *mut u32 = 0x3FF4400C as *mut u32;
static mut GPIO_ENABLE_W1TC_REG: *mut u32 = 0x3FF44028 as *mut u32;
static mut GPIO_IN_REG: *mut u32 = 0x3FF4403C as *mut u32;
static mut LAST_STATUS: u32 = 1;

fn led_glow() {
    unsafe {
        core::ptr::write_volatile(GPIO_PIN_ENABLE_REG, 1 << 5);
        core::ptr::write_volatile(GPIO_W1TS_OUT_REG, 1 << 5);
    }
}

fn gpio_input_mode() {
    unsafe {
        core::ptr::write_volatile(GPIO_ENABLE_W1TC_REG, 1 << 18);
    }
}

fn led_low() {
    unsafe {
        core::ptr::write_volatile(GPIO_W1TC_OUT_REG, 1 << 5);
    }
}

fn magnet_sensor_read() {
    unsafe {
        let magnet_sensor_readings = core::ptr::read_volatile(GPIO_IN_REG);
        let data = (magnet_sensor_readings >> 18) & 1;

        if data == 0 && LAST_STATUS == 1 {
            core::ptr::write_volatile(UART_FIFO_REG, 0x41);
            led_glow();
        } else if data == 1 {
            led_low();
        }
        LAST_STATUS = data;
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let _ = peripherals.GPIO6;
    let _ = peripherals.GPIO7;
    let _ = peripherals.GPIO8;
    let _ = peripherals.GPIO9;
    let _ = peripherals.GPIO10;
    let _ = peripherals.GPIO11;
    let _ = peripherals.GPIO16;
    let _ = peripherals.GPIO20;

    gpio_input_mode();
    led_low();

    unsafe { LAST_STATUS = 1 };

    loop {
        magnet_sensor_read();
    }
}
