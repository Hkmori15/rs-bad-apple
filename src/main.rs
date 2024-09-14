use core::ascii;
use std::{env, fs, process::Command};
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use image::ImageReader;
use termion::{clear, cursor, terminal_size};

fn image_to_ascii(image_path: &str, term_width: u16, term_height: u16) -> String {
    let img = ImageReader::open(image_path)
    .unwrap()
    .decode()
    .unwrap()
    .to_luma8(); // return image in format luma8

    let width_scale = img.width() as f32 / term_width as f32;
    let height_scale = img.height() as f32 / term_height as f32;

    let scale = width_scale.max(height_scale);

    let chars = " .'`^\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$".chars().collect::<Vec<char>>();

    // String for saving ASCII arts
    let mut ascii_art = String::with_capacity((term_width * term_height) as usize);
    // Copy image for dithering
    let image = img.clone();

    // Ordered Dithering Matrix (2x2)
    let dither_matrix = [
        [1, 3],
        [4, 2],
    ];

    for j in 0..term_height {
        for i in 0..term_width {
            let px = (i as f32 * scale) as u32;
            let py = (j as f32 * scale) as u32;

            let px = px.min(img.width() - 1);
            let py = py.min(img.height() - 1);

            let old_pixel = image.get_pixel(px, py)[0] as f32;

            // Calculate dithering threshold
            let threshold = (dither_matrix[(j % 2) as usize][(i % 2) as usize] as f32) / 5.0 * 255.0;

            let new_pixel_index = if old_pixel > threshold {
                ((chars.len() - 1) as f32).round() as usize
            } else {
                0
            };

            ascii_art.push(chars[new_pixel_index]);
           
        }
        
        ascii_art.push('\n');
    }

    ascii_art
}

fn main()  {
    let frames_dir = format!("{}/frames", env!("CARGO_MANIFEST_DIR"));
    let mut frames = fs::read_dir(frames_dir).unwrap()
        .map(|res| res.unwrap().path())
        .collect::<Vec<_>>();
    
    frames.sort();
    
    let audio_path = format!("{}/bad_apple.mp3", env!("CARGO_MANIFEST_DIR"));
    // You can use other audio players just replace "cvlc" with the appropriate command
    let mut child = Command::new("cvlc")
        .arg(audio_path)
        .spawn()
        .expect("Failed to start VLC");


    for frame_path in frames {
        let frame_start = Instant::now();
        let (term_width, term_height) = terminal_size().unwrap();
        
        let ascii_frame = image_to_ascii(frame_path.to_str().unwrap(), term_width, term_height); // generate ascii art for every frame
        println!("{}{}", cursor::Goto(1, 1), clear::All);
        print!("{}", ascii_frame);
        
        let frame_duration = frame_start.elapsed();
        let target_duration = Duration::from_millis(33); // target frame duration 33ms

        // Dynamic correction for duration
        if frame_duration < target_duration {
            sleep(target_duration - frame_duration);
        } else {
            sleep(Duration::from_millis(1)); // minimal duration for not overloading CPU
        }
    }

    child.wait().expect("Failed to wait on child process");
}