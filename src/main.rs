#![no_std]
#![no_main]

use core::panic::PanicInfo;
use embedded_graphics::primitives::Circle;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use stm32f4xx_hal::otg_fs::{UsbBus, USB, UsbBusType};
use stm32f4xx_hal::{prelude::*, stm32, qei::Qei, interrupt, delay::Delay};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use usb_device::bus::UsbBusAllocator;

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    style::PrimitiveStyle,
};

use stm32f4xx_hal::spi::{Mode, Phase, Polarity, Spi};
use stm32f4xx_hal::pwm;

static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
static mut USB_SERIAL: Option<usbd_serial::SerialPort<UsbBusType>> = None;
static mut USB_DEVICE: Option<UsbDevice<UsbBusType>> = None;

mod matrix;
use matrix::{Matrix, KeyState};

mod encoder;
use encoder::Encoder;

mod display;
use display::Display;

mod vibrator;
use vibrator::Vibrator;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let peripherals = stm32::Peripherals::take().unwrap();
    let cortex_peripherals = cortex_m::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();

    let clocks= rcc
        .cfgr
        .use_hse(25.mhz())
        .sysclk(96.mhz())
        .hclk(96.mhz())
        .pclk1(48.mhz())
        .pclk2(96.mhz())
        .freeze();

    let mut delay = Delay::new(cortex_peripherals.SYST, clocks);

    let gpioa = peripherals.GPIOA.split();
    let gpiob = peripherals.GPIOB.split();
    let gpioc = peripherals.GPIOC.split();
    let gpioe = peripherals.GPIOE.split();

    // Encoder
    let rotary_a = Encoder::new(
        Qei::tim2(peripherals.TIM2, (
            gpioa.pa0.into_alternate_af1(),
            gpioa.pa1.into_alternate_af1(),
        )), 
        gpioc.pc15.into_pull_up_input()
    );

    let rotary_b = Encoder::new(
        Qei::tim3(peripherals.TIM3, (
            gpiob.pb4.into_alternate_af2(),
            gpiob.pb5.into_alternate_af2(),
        )),
        gpioc.pc14.into_pull_up_input()
    );

    let rst = gpioa.pa9.into_push_pull_output();
    let dc = gpioa.pa8.into_push_pull_output();

    let sck = gpiob.pb13.into_alternate_af5();
    let miso = gpiob.pb14.into_alternate_af5(); // not used
    let mosi = gpiob.pb15.into_alternate_af5();

    let spi = Spi::spi2(
        peripherals.SPI2,
        (sck, miso, mosi),
        Mode {
            phase: Phase::CaptureOnFirstTransition,
            polarity: Polarity::IdleLow
        },
        48_000_000.hz(),
        clocks
    );

    let mut display = Display::new(spi, dc, rst);

    let channels = (
        gpioa.pa10.into_alternate_af1(),
        gpioe.pe14.into_alternate_af1(), // Channel not used
    );

    let mut vibrator = Vibrator::new(pwm::tim1(peripherals.TIM1, channels, clocks, 500u32.hz()));

    let mut matrix = Matrix::new(
        [
            gpiob.pb6.into_pull_up_input().downgrade(),
            gpiob.pb7.into_pull_up_input().downgrade(),
            gpiob.pb8.into_pull_up_input().downgrade(),
            gpiob.pb9.into_pull_up_input().downgrade(),
        ],
        [
            gpioa.pa3.into_push_pull_output().downgrade(), 
            gpioa.pa4.into_push_pull_output().downgrade(),
            gpioa.pa5.into_push_pull_output().downgrade(),
            gpioa.pa6.into_push_pull_output().downgrade()
        ]
    );

    let usb = USB {
        hclk: clocks.hclk(),
        usb_global: peripherals.OTG_FS_GLOBAL,
        usb_device: peripherals.OTG_FS_DEVICE,
        usb_pwrclk: peripherals.OTG_FS_PWRCLK,
        pin_dp: gpioa.pa12.into_alternate_af10(),
        pin_dm: gpioa.pa11.into_alternate_af10(),
    };
    
    unsafe {
        static mut USB_BUF: [u32; 32] = [0; 32];
        USB_BUS = Some(UsbBus::new(usb, &mut USB_BUF ));
        
        USB_SERIAL = Some(SerialPort::new(USB_BUS.as_ref().unwrap()));
        USB_DEVICE = Some(UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Lukas Sturm")
            .product("Macro Proto")
            .serial_number("ONE")
            .device_class(USB_CLASS_CDC)
            .build());
    }

    stm32::NVIC::unpend(stm32f4xx_hal::stm32::Interrupt::OTG_FS);
    unsafe {
        stm32::NVIC::unmask(stm32f4xx_hal::stm32::Interrupt::OTG_FS);
    };


    display.init(&mut delay).unwrap();


    loop {

        matrix.update(&mut delay);
        vibrator.update();

        cortex_m::interrupt::free(| _ | {
            for change in matrix.changes() {

                match change.new_state {
                    // KeyState::Pressed => {
                    //     serial_write(b"Pressed: ");
                    //     serial_write(&[(0x30 + change.matrix_x) as u8, b' ', (0x30 + change.matrix_y) as u8, b'\n', b'\r']);
                    // },
                    KeyState::Pressing => {
                        vibrator.enable(4);

                        if change.matrix_x == 0 && change.matrix_y == 0 {
                            serial_write(b"A: ");
                            serial_write(&format_u32(rotary_a.count()));
                            serial_write(b"\n\r");
                        } else if change.matrix_x == 1 && change.matrix_y == 0 {
                            serial_write(b"B: ");
                            serial_write(&format_u16(rotary_b.count()));
                            serial_write(b"\n\r");
                        }
                        else if change.matrix_x == 2 && change.matrix_y == 2 {
                            Circle::new(Point::new((rotary_a.count() / 4) as i32, (rotary_b.count() / 4) as i32).into(), 16)
                            .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
                            .draw(display.get()).unwrap();
                            serial_write(b"Circle \n\r");

                        } else if change.matrix_x == 2 && change.matrix_y == 0 {
                            display.clear();
                            serial_write(b"Clearing\n\r");

                        } else {
                            serial_write(b"Pressing: ");
                            serial_write(&[(0x30 + change.matrix_x) as u8, b' ', (0x30 + change.matrix_y) as u8, b'\n', b'\r']);
                        }

                    },
                    // KeyState::Released => {
                    //     serial_write(b"Released: ");
                    //     serial_write(&[(0x30 + change.matrix_x) as u8, b' ', (0x30 + change.matrix_y) as u8, b'\n', b'\r']);
                    // },
                    // KeyState::Releasing => {
                    //     serial_write(b"Releasing: ");
                    //     serial_write(&[(0x30 + change.matrix_x) as u8, b' ', (0x30 + change.matrix_y) as u8, b'\n', b'\r']);
                    // }
                    _ => ()
                }
            }
            if rotary_a.is_pressed().unwrap() {
                serial_write(b"A gedruckt \n\r");
            }
            if rotary_b.is_pressed().unwrap() {
                serial_write(b"B gedruckt \n\r");
            }
        });

        delay.delay_ms(50u16);
    }
}


// fn serial_write(serial: &mut SerialPort<UsbBus<USB>, DefaultBufferStore, DefaultBufferStore>, data: &[u8]) {
fn serial_write(data: &[u8]) {
    let serial = unsafe { USB_SERIAL.as_mut().unwrap() };

    let mut write_offset = 0;
    while write_offset < data.len() {
        match serial.write(&data[write_offset..data.len()]) {
            Ok(len) if len > 0 => {
                write_offset += len;
            }
            _ => {}
        }
    }
}

fn format_u16(number: u16) -> [u8; 5]{
    let mut res = [0u8; 5];
    let mut num = number;

    for n in (0..5).rev() {
        res[n] = 0x30 + (num % 10) as u8;
        num /= 10;
        if num == 0 { break; }
    }

    res
}

fn format_u32(number: u32) -> [u8; 10]{
    let mut res = [0u8; 10];
    let mut num = number;

    for n in (0..10).rev() {
        res[n] = 0x30 + (num % 10) as u8;
        num /= 10;
        if num == 0 { break; }
    }

    res
}


#[interrupt]
fn OTG_FS() {
    stm32::NVIC::unpend(stm32f4xx_hal::stm32::Interrupt::OTG_FS);
    usb_interrupt();
}

fn usb_interrupt() {
    let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
    let serial = unsafe { USB_SERIAL.as_mut().unwrap() };

    if !usb_dev.poll(&mut [serial]) {
        return;
    }

    let mut buf = [0u8; 64];

    match serial.read(&mut buf) {
        Ok(count) if count > 0 => {
            // Echo back in upper case
            for c in buf[0..count].iter_mut() {
                if 0x61 <= *c && *c <= 0x7a {
                    *c &= !0x20;
                }
            }

            let mut write_offset = 0;
            while write_offset < count {
                match serial.write(&buf[write_offset..count]) {
                    Ok(len) if len > 0 => {
                        write_offset += len;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {} // You might need a compiler fence in here.
}