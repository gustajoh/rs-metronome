use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::{Duration, Instant},
};

struct MetronomeState {
    next_tick_time: Instant,
    interval: Duration,
    playing_tick: bool,
    tick_sample_pos: usize,
}

impl MetronomeState {
    fn new(bpm: f32) -> Self {
        let interval = Duration::from_secs_f32(60.0 / bpm);
        Self {
            next_tick_time: Instant::now() + interval,
            interval,
            playing_tick: false,
            tick_sample_pos: 0,
        }
    }

    fn check_and_trigger_tick(&mut self, now: Instant) {
        if now >= self.next_tick_time {
            self.playing_tick = true;
            self.tick_sample_pos = 0;
            self.next_tick_time += self.interval;
        }
    }
}

fn main() -> Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let config = device.default_output_config()?.config();

    let sample_rate = config.sample_rate.0 as f32;
    let tick_freq = 432.0; // Hz
    let tick_duration = 0.05; // seconds (50ms)

    let state = Arc::new(Mutex::new(MetronomeState::new(82.0)));

    let stream = device.build_output_stream(
        &config,
        {
            let state = state.clone();
            move |data: &mut [f32], _| {
                let now = Instant::now();
                let mut metro = state.lock().unwrap();

                metro.check_and_trigger_tick(now);

                for sample in data.iter_mut() {
                    if metro.playing_tick {
                        let t = metro.tick_sample_pos as f32 / sample_rate;
                        *sample = (2.0 * std::f32::consts::PI * tick_freq * t).sin() * 0.5;
                        metro.tick_sample_pos += 1;

                        if (metro.tick_sample_pos as f32) / sample_rate > tick_duration {
                            metro.playing_tick = false;
                        }
                    } else {
                        *sample = 0.0;
                    }
                }
            }
        },
        |err| eprintln!("Stream error: {:?}", err),
        None,
    )?;

    stream.play()?;

    // Keep main thread alive
    loop {
        sleep(Duration::from_secs(1));
    }
}
