#![no_main]
#![no_std]

use core::panic::PanicInfo;
use embedded_hal::PwmPin;
use rtt_target::{rprintln, rtt_init_print};


use cortex_m;
use cortex_m_rt::entry;
use stm32f4xx_hal::{delay::Delay, prelude::*, pwm, stm32};



// straight from stm32f4xx-hal examples
#[entry]
fn main() -> ! {
    rtt_init_print!();

    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Set up the system clock.
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    let gpioa = dp.GPIOA.split();
    let channels = (
        gpioa.pa10.into_alternate_af1(),
        gpioa.pa11.into_alternate_af1(),
    );
    let mut delay = Delay::new(cp.SYST, clocks);

    let pwm = pwm::tim1(dp.TIM1, channels, clocks, 500u32.hz());
    let (mut ch1, mut ch2) = pwm;
    
    ch2.set_duty(0);
    ch2.enable();
    
    let max_duty = ch1.get_max_duty();
    rprintln!("{}", max_duty);

    ch1.set_duty(max_duty);
    ch1.enable();

    loop {
        delay.delay_ms(100u16);

        ch1.set_duty(max_duty / 16);
        delay.delay_ms(500u16);

        ch1.set_duty(max_duty / 8);
        delay.delay_ms(200u16);
        
        ch1.set_duty(max_duty / 4);
        delay.delay_ms(200u16);

        ch1.set_duty(max_duty / 2);
        delay.delay_ms(200u16);
        
        ch1.set_duty(max_duty);
        delay.delay_ms(500u16);

        ch1.set_duty(max_duty / 2);    
        delay.delay_ms(100u16);

        ch1.set_duty(max_duty / 4);
        delay.delay_ms(100u16);

        ch1.set_duty(max_duty / 8);
    }

}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {} // You might need a compiler fence in here.
}