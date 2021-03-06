mod chip8;

use chip8::Chip8;
use sdl2::{audio, event, keyboard::Keycode, pixels};
use std::{collections::HashMap, env, fs, io};

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("Usage: chip8-oxidized <file-path>");
        return Err(io::Error::new(io::ErrorKind::Other, "Other"));
    }

    let error_message = format!("Unable to open {}", args[1]);
    let file: Vec<u8> = fs::read(&args[1]).expect(error_message.as_str());
    println!("{} is {} byte long", &args[1], file.len());

    let mut chip8 = Chip8::new(file.clone());

    let sdl2_context = sdl2::init().expect("Failed to initialize SDL");
    let sdl2_audio_system = sdl2_context.audio().unwrap();
    let mut sdl2_timer_system = sdl2_context.timer().unwrap();
    let sdl2_video_system = sdl2_context.video().unwrap();

    println!(
        "SDL2 version: {}.{}.{}",
        sdl2::version::version(),
        sdl2::version::revision(),
        sdl2::version::revision_number()
    );

    let mut key_bindings = HashMap::new();
    key_bindings.insert(Keycode::Num0, 0x0);
    key_bindings.insert(Keycode::Num1, 0x1);
    key_bindings.insert(Keycode::Num2, 0x2);
    key_bindings.insert(Keycode::Num3, 0x3);
    key_bindings.insert(Keycode::Num4, 0x4);
    key_bindings.insert(Keycode::Num5, 0x5);
    key_bindings.insert(Keycode::Num6, 0x6);
    key_bindings.insert(Keycode::Num7, 0x7);
    key_bindings.insert(Keycode::Num8, 0x8);
    key_bindings.insert(Keycode::Num9, 0x9);
    key_bindings.insert(Keycode::Kp0, 0x0);
    key_bindings.insert(Keycode::Kp1, 0x1);
    key_bindings.insert(Keycode::Kp2, 0x2);
    key_bindings.insert(Keycode::Kp3, 0x3);
    key_bindings.insert(Keycode::Kp4, 0x4);
    key_bindings.insert(Keycode::Kp5, 0x5);
    key_bindings.insert(Keycode::Kp6, 0x6);
    key_bindings.insert(Keycode::Kp7, 0x7);
    key_bindings.insert(Keycode::Kp8, 0x8);
    key_bindings.insert(Keycode::Kp9, 0x9);
    key_bindings.insert(Keycode::A, 0xA);
    key_bindings.insert(Keycode::B, 0xB);
    key_bindings.insert(Keycode::C, 0xC);
    key_bindings.insert(Keycode::D, 0xD);
    key_bindings.insert(Keycode::E, 0xE);
    key_bindings.insert(Keycode::F, 0xF);

    // TODO:
    let spec = audio::AudioSpecDesired {
        channels: Some(1),
        freq: Some(44100),
        samples: None,
    };
    let audio_device = sdl2_audio_system
        .open_playback(None, &spec, |spec| {
            struct SquareWave {
                phase_inc: f32,
                phase: f32,
                volume: f32,
            };

            impl audio::AudioCallback for SquareWave {
                // Data channel
                type Channel = f32;

                fn callback(&mut self, out: &mut [f32]) {
                    // Generate square wave
                    for x in out.iter_mut() {
                        if self.phase <= 0.5 {
                            *x = self.volume;
                        } else {
                            *x = -self.volume;
                        };
                        self.phase = (self.phase + self.phase_inc) % 1.0;
                    }
                }
            }

            return SquareWave {
                phase: 0.0,
                phase_inc: 440.0 / spec.freq as f32,
                volume: 0.10,
            };
        })
        .unwrap();

    let window_width: u32 = (chip8::CHIP8_SCREEN_WIDTH as u32) * 20;
    let window_height: u32 = (chip8::CHIP8_SCREEN_HEIGHT as u32) * 20;

    let window = sdl2_video_system
        .window(["chip8-oxidized", &args[1]].join(" - ").as_str(), window_width, window_height)
        .resizable()
        .build()
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .present_vsync()
        .build()
        .unwrap();
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl2_context.event_pump().unwrap();
    let mut redraw = true;
    let mut key = 0;
    let mut time = sdl2_timer_system.ticks();
    'running: loop {
        for event in event_pump.poll_iter() {
            use event::Event::*;
            match event {
                Quit { .. } => {
                    break 'running;
                }
                KeyDown { keycode, .. } => {
                    if keycode != None {
                        let code = keycode.unwrap();
                        key = 0;
                        match key_bindings.get(&code) {
                            Some(binding) => {
                                chip8.key_pad[*binding] = true;
                                key = *binding;
                            }
                            None => {}
                        }
                    }
                }
                KeyUp { keycode, .. } => {
                    if keycode != None {
                        let code = keycode.unwrap();
                        match key_bindings.get(&code) {
                            Some(binding) => {
                                chip8.key_pad[*binding] = false;
                            }
                            None => {}
                        }
                    }
                }
                _ => {}
            }
        }

        if !chip8.run(key, &mut redraw) {
            break;
        }

        sdl2_timer_system.delay(1);
        let end = sdl2_timer_system.ticks() - time;
        if end >= 16 {
            if chip8.dt > 0 {
                chip8.dt -= 1;
            }
            if chip8.st > 0 {
                chip8.st -= 1;
                audio_device.resume();
                if chip8.st == 0 {
                    audio_device.pause();
                }
            }
            time = sdl2_timer_system.ticks();
        }

        // TODO:
        if redraw {
            canvas.clear();

            let mut texture = texture_creator
                .create_texture_streaming(
                    pixels::PixelFormatEnum::RGB24,
                    chip8::CHIP8_SCREEN_WIDTH as u32,
                    chip8::CHIP8_SCREEN_HEIGHT as u32,
                )
                .unwrap();
            let mut texture_data: [u8; chip8::NUM_PIXELS * 3] = [0; chip8::NUM_PIXELS * 3];
            for (i, pixel) in chip8.screen.iter().enumerate() {
                let mut color = 0x00;
                if *pixel == 1 {
                    color = 0xFF;
                }
                texture_data[i * 3] = color;
                texture_data[i * 3 + 1] = color;
                texture_data[i * 3 + 2] = color;
            }
            texture
                .update(
                    None,
                    &texture_data,
                    (chip8::CHIP8_SCREEN_WIDTH * 3) as usize,
                )
                .unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();

            redraw = false;
        }
    }

    return Ok(());
}
