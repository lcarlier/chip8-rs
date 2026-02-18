use sfml::graphics::{Color, Image, IntRect, RenderTarget, RenderWindow, Sprite, Texture, Transformable};
use sfml::window::{Style};
mod chip8;

fn main() {
    println!("Starting CHIP-8 Emulator");
    println!("Loading ROM file");
    let bytes = std::fs::read("1-chip8-logo.ch8").unwrap();
    let mut system = chip8::System::new();
    system.load_rom(bytes);
    println!("ROM file loaded successfully!");

    let pixel_scale = 16;
    let width = 128;
    let height = 64;
    let mut window = RenderWindow::new(
        (width * pixel_scale, height * pixel_scale),
        "Chip-8 emulator output",
        Style::CLOSE,
        &Default::default(),
    ).expect("Failed to create window");
    window.set_framerate_limit(60);

    // Create image from raw pixel data
    let pixel_count = (width * height) as usize;
    let pixels = vec![255u8; pixel_count]; // White pixels (RGBA)
    let mut image = Image::new().expect("Failed to create image");
    unsafe {
        image.recreate_from_pixels(width, height, &pixels);
    }

    while window.is_open() {
        match  system.step() {
            None => {},
            Some(output) => {
                for (row, value) in output.pixels.iter().enumerate() {
                    for (col, pixel) in value.iter().enumerate() {
                        if *pixel {
                            let _ = image.set_pixel((output.x + col as u8) as u32, (output.y + row as u8) as u32, Color::BLACK);
                        }
                    }
                }
            }
        }


        // Create texture and sprite for rendering
        let area = IntRect::new(0, 0, width as i32, height as i32);
        let texture = Texture::from_image(&image, area).expect("Failed to create texture");
        let mut sprite = Sprite::with_texture(&texture);

        sprite.set_scale((pixel_scale as f32, pixel_scale as f32));

        window.clear(Color::WHITE);
        window.draw(&sprite);
        window.display();
    }

}
