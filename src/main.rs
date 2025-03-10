#![no_std]
#![no_main]

use core::ptr::addr_of_mut;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::{block::ImageDef, gpio};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};
use embassy_time::{Duration, Ticker};
use embedded_alloc::LlffHeap as Heap;
use smart_leds::RGB8;
use tinywasm::{Module, ModuleInstance, Store};
use {defmt_rtt as _, panic_probe as _};

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::non_secure_exe();

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Blinky Example"),
    embassy_rp::binary_info::rp_program_description!(
        c"This example tests the RP Pico on board LED, connected to gpio 25"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[global_allocator]
static HEAP: Heap = Heap::empty();

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

fn wasm_color_to_real(wasm_color: i32) -> RGB8 {
    let r = (wasm_color & 0xFF) as u8;
    let g = ((wasm_color >> 8) & 0xFF) as u8;
    let b = ((wasm_color >> 16) & 0xFF) as u8;
    RGB8 { r, g, b }
}

static WASM: &[u8; 239] =
    include_bytes!("../wasm/target/wasm32-unknown-unknown/release/wasm_blink.wasm");

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024 * 20;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(addr_of_mut!(HEAP_MEM) as usize, HEAP_SIZE) }
    }

    info!("Start");
    let p = embassy_rp::init(Default::default());

    let mut led = Output::new(p.PIN_25, Level::Low);

    led.set_high();

    // This is the number of leds in the string. Helpfully, the sparkfun thing plus and adafruit
    // feather boards for the 2040 both have one built in.
    const NUM_LEDS: usize = 1;
    let mut data = [RGB8::default(); NUM_LEDS];

    // Common neopixel pins:
    // Thing plus: 8
    // Adafruit Feather: 16;  Adafruit Feather+RFM95: 4
    // let mut ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_16, &program);

    let mut store = Store::new();

    let module = Module::parse_bytes(WASM).unwrap();
    info!("Module parsed");
    let instance = ModuleInstance::instantiate(&mut store, module, None).unwrap();
    /*
    let calc_color = instance
        .exported_func::<i32, i32>(&store, "calc_color")
        .unwrap();
    */

    // Loop forever making RGB values and pushing them out to the WS2812.
    let mut ticker = Ticker::every(Duration::from_millis(10));
    loop {
        for j in 0..(256 * 5) {
            debug!("New Colors:");
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);

                /*data[i] = wasm_color_to_real(
                    calc_color
                        .call(
                            &mut store,
                            (((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as i32,
                        )
                        .unwrap(),
                );*/

                debug!("R: {} G: {} B: {}", data[i].r, data[i].g, data[i].b);
            }
            // ws2812.write(&data).await;
            led.set_high();

            ticker.next().await;
        }
    }
}
