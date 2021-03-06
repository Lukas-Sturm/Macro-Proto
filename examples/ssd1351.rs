#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use stm32f4xx_hal::stm32;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::spi::Spi;
use stm32f4xx_hal::delay::Delay;

use ssd1351::builder::Builder;
use ssd1351::mode::{GraphicsMode};
use ssd1351::prelude::*;

use embedded_graphics::fonts::{Font12x16, Font6x6, Text};
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::*;
use embedded_graphics::style::TextStyle;

use embedded_graphics::{
    egcircle, egrectangle, egtext, fonts::Font6x8,
    pixelcolor::Rgb565, prelude::*, primitive_style, text_style,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let p = stm32::Peripherals::take().unwrap();

    let rcc = p.RCC.constrain();

    // TRY the other clock configuration
    let clocks= rcc
        .cfgr
        .use_hse(25.mhz())
        .sysclk(96.mhz())
        .pclk1(24.mhz())
        .freeze();
    
    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();

    let cp = cortex_m::Peripherals::take().unwrap();
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut rst = gpioa.pa9.into_push_pull_output();
    let dc = gpioa.pa8.into_push_pull_output();

    let sck = gpiob.pb13.into_alternate_af5();
    let miso = gpiob.pb14.into_alternate_af5();
    let mosi = gpiob.pb15.into_alternate_af5();

    let spi = Spi::spi2(
        p.SPI2,
        (sck, miso, mosi),
        SSD1351_SPI_MODE,
        500.hz(),
        clocks
    );
    
    let mut display: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();
    display.reset(&mut rst, &mut delay).unwrap();
    display.init().unwrap();

    rprintln!("Setup Done!");

    // let i: u16 = 0xFFFF;
    // // display.set_rotation(DisplayRotation::Rotate270).unwrap();
    // Text::new("Macro Proto", Point::zero())
    //     .into_styled(TextStyle::new(Font6x6, RawU16::new(i).into()))
        
    //     .draw(&mut display)
    //     .unwrap();

    fn build_thing(text: &'static str) -> impl Iterator<Item = Pixel<Rgb565>> {
        egrectangle!(top_left = (0, 0), bottom_right = (40, 40))
            .into_iter()
            .chain(&egcircle!(
                center = (20, 20),
                radius = 8,
                style = primitive_style!(fill_color = Rgb565::RED)
            ))
            .chain(&egtext!(
                text = text,
                top_left = (20, 16),
                style = text_style!(font = Font6x8, text_color = Rgb565::GREEN)
            ))
    }
        
    build_thing("Hello Rust!").draw(&mut display).unwrap();

    // loop {
    //     display.draw(Font12x16::render_str("Wavey! - superlongo stringer").with_stroke(Some(i.into())).into_iter());
    //     // display.clear();
    //     delay.delay_ms(32_u16);
    //     i+=1;
    //     if i == u16::max_value() {
    //         i = 0;
    //     }
    // }

    loop {}
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {} // You might need a compiler fence in here.
}