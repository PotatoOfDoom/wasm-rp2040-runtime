#![no_main]

fn wheel(mut wheel_pos: u8) -> (u8, u8, u8) {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3);
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3);
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0)
}

#[no_mangle]
pub unsafe extern "C" fn calc_color(wheel_pos: i32) -> i32 {
    let (r, g, b) = wheel(wheel_pos as u8);
    ((r as u32) << 16 | (g as u32) << 8 | b as u32) as i32
}
