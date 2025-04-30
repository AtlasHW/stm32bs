//! Prints "Hello, world!" on the host console using semihosting

#![no_main]
#![no_std]

use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m::delay::Delay;
use stm32f0::stm32f0x1 as stm32;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();

    let mut delay = Delay::new(cp.SYST, 48);
    let gpiob = dp.GPIOB;
	

	
	// Low bits
	


    loop {
        gpiob.odr.modify(|_, w| w.odr5().set_bit());
        delay.delay_ms(500);
        gpiob.odr.modify(|_, w| w.odr5().clear_bit());
        delay.delay_ms(500);
    }
}
