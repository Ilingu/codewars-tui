use tui::style::Color;

use rand::Rng;

/// generate a random integer between a and b included
pub fn rand_int(a: isize, b: isize) -> isize {
    let mut rng = rand::thread_rng();
    return rng.gen_range(a..=b);
}

pub fn gen_rand_colors() -> Color {
    Color::Rgb(
        rand_int(0, 255) as u8,
        rand_int(0, 255) as u8,
        rand_int(0, 255) as u8,
    )
}
