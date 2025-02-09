use std::{
    error::Error,
    fs,
    io::{stdout, Write},
    process::Command,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, MoveTo},
    terminal::{size as terminal_size, EnterAlternateScreen},
    ExecutableCommand, QueueableCommand,
};

use image::{imageops::FilterType, GenericImageView};

// Apply algorithm Floyd-Steinberg dithering to matrix of pixels grayscale
// each element gray[y][x] has value 0.0 -> 255.0
// Parameter levels - number level of brightness for len array
fn apply_dithering(gray: &mut Vec<Vec<f32>>, levels: usize) {
    let height = gray.len();
    let width = if height > 0 { gray[0].len() } else { 0 };
    let step = 255.0 / (levels as f32 - 1.0);

    for y in 0..height {
        for x in 0..width {
            let old_value = gray[y][x];
            let new_value = (old_value / step).round() * step;

            let error = old_value - new_value; // error is my life
            gray[y][x] = new_value;

            if x + 1 < width {
                gray[y][x + 1] += error * 7.0 / 16.0;
            }

            if y + 1 < height {
                if x > 0 {
                    gray[y + 1][x - 1] += error * 3.0 / 16.0;
                }

                gray[y + 1][x] += error * 5.0 / 16.0;

                if x + 1 < width {
                    gray[y + 1][x + 1] += error * 1.0 / 16.0;
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout();

    // You can replace "cvlc" on your audio player of choice
    let _audio_child = Command::new("cvlc").arg("bad_apple.mp3").spawn()?;

    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    let frames_dir = "frames";
    let mut frames: Vec<_> = fs::read_dir(frames_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if path.extension()?.to_str()? == "png" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    // Sort frame filenames to play in order
    frames.sort();

    if frames.is_empty() {
        eprintln!("No PNG files found in {}", frames_dir);

        return Ok(());
    }

    let frame_delay = Duration::from_millis(33); // 30 FPS

    let total_frames = frames.len();

    let ascii_chars = [" ", ".", ":", "-", "=", "+", "*", "#", "%", "@"];

    // Coefficient for string
    let height_adjust: f32 = 2.0;

    let start_time = Instant::now();

    loop {
        let elapsed = start_time.elapsed();

        // Compute which frame to display now
        let frame_index = (elapsed.as_millis() / frame_delay.as_millis()) as usize % total_frames;
        let frame_path = &frames[frame_index];

        let img = image::open(frame_path)?;

        let (term_cols, term_rows) = terminal_size()?;
        let term_width = term_cols as f32;
        let term_height = term_rows as f32 * height_adjust;

        // Calculate coefficient resizing for saving proportions
        let (img_width, img_height) = img.dimensions();
        let img_width = img_width as f32;
        let img_height = img_height as f32;

        let scale = term_width.min(term_height) / img_width.max(img_height);

        let new_width = (img_width * scale).max(1.0) as u32;
        let new_height = (img_height * scale).max(1.0) as u32;

        // Resize image to terminal size
        let resized_img = img.resize(new_width, new_height, FilterType::Lanczos3);

        let (width, height) = resized_img.dimensions();

        let mut gray_matrix: Vec<Vec<f32>> =
            (0..height).map(|_| vec![0.0; width as usize]).collect();

        // Push matrix of values brightness 0.0 -> 255.0
        for y in 0..height {
            for x in 0..width {
                let pixel = resized_img.get_pixel(x, y);
                let luma =
                    0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32;

                gray_matrix[y as usize][x as usize] = luma;
            }
        }

        apply_dithering(&mut gray_matrix, ascii_chars.len());

        let mut ascii_frame = String::with_capacity((width * height) as usize);

        for row in gray_matrix.iter() {
            for &luma in row.iter() {
                let index = (luma / 255.0 * ((ascii_chars.len() - 1) as f32)).round() as usize;
                ascii_frame.push_str(ascii_chars[index]);
            }

            ascii_frame.push('\n');
        }

        stdout.queue(MoveTo(0, 0))?;
        stdout.queue(crossterm::style::Print(&ascii_frame))?;
        stdout.flush()?;

        // Compute time to next frame for more accurate timing
        let next_frame_time = start_time + frame_delay.saturating_mul(frame_index as u32 + 1);

        if let Some(remaining) = next_frame_time.checked_duration_since(Instant::now()) {
            std::thread::sleep(remaining);
        }
    }
}
