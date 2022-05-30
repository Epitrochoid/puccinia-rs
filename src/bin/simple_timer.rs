#![deny(unsafe_code)]
#![no_main]
#![no_std]

use puccinia as _;

#[rtic::app(device = stm32f4xx_hal::pac, dispatchers = [USART1])]
mod app {
    use defmt::println;
    use stm32f4xx_hal::{
        gpio::{Output, PushPull, PC14, PD13},
        timer::MonoTimerUs,
        pac,
        prelude::*,
    };
    
    #[monotonic(binds = TIM2, default = true)]
    type MicrosecMono = MonoTimerUs<pac::TIM2>;
    
    #[shared]
    struct Shared {
        led: PD13<Output<PushPull>>,
        relay: PC14<Output<PushPull>>,
    }
    
    #[local]
    struct Local {}
    
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

        let gpiod = ctx.device.GPIOD.split();
        let mut led = gpiod.pd13.into_push_pull_output();
        
        let gpioc = ctx.device.GPIOC.split();
        let mut relay = gpioc.pc14.into_push_pull_output();
        
        let mono = ctx.device.TIM2.monotonic(&clocks);
        relay_on::spawn().ok();
        
        led.set_high();
        relay.set_high();

        println!("Starting up...");
        
        (Shared { led, relay }, Local {}, init::Monotonics(mono))
    }
   
    #[task(shared = [led, relay])]
    fn relay_on(ctx: relay_on::Context) {
        let relay_on::SharedResources {
            led,
            relay,
        } = ctx.shared;

        (relay, led).lock(|relay, led| {
            relay.set_high();
            led.set_high();
        });

        relay_off::spawn_after(10.secs()).ok();
    }
    
    #[task(shared = [led, relay])]
    fn relay_off(ctx: relay_off::Context) {
        let relay_off::SharedResources {
            led,
            relay,
        } = ctx.shared;

        (relay, led).lock(|relay, led| {
            relay.set_low();
            led.set_low();
        });

        relay_on::spawn_after(590.secs()).ok();
    }
}
