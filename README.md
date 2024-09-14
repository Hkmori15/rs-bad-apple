## Bad Apple but it's Rust 🍎🦀

Animation with song written in Rust using ffmpeg and crates termion.

## Usage 📜:

1. Clone the repo:
   `clone https://github.com/Hkmori15/rs-bad-apple.git`
2. Create in root directory catalog/folder `frames` and use ffmpeg for convert video `bad_apple.mp4` to frames in format .png:
   `ffmpeg -i bad_apple.mp4 -vf scale=120:80 -r 30 frames/frame%04d.png`
3. For playing audio i use VLC, u can change audio player in main.rs:73:
   `let mut child = Command::new("your_audio_player")`
4. Compile and run:
   `cargo build --release` and `cargo run --release`
5. I use alacritty terminal, idk how it will work in other terminals i don't check it.
6. Enjoy!
