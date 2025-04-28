use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex},
    thread::{self, JoinHandle},
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

pub struct Metronome {
    stop_flag: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Metronome {
    pub fn start(bpm: u32) -> Result<Self> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        let handle = thread::spawn(move || {
            let host = cpal::default_host();
            let device = host.default_output_device().expect("No output device");
            let config = device.default_output_config().unwrap().config();

            let sample_rate = config.sample_rate.0 as f32;
            let tick_freq = 432.0; // Hz
            let tick_duration = 0.05; // 50ms

            let state = Arc::new(Mutex::new(MetronomeState::new(bpm as f32)));

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
            ).unwrap();

            stream.play().unwrap();

            while !stop_flag_clone.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100));
            }

            drop(stream); // stops the stream
            println!("Metronome stopped.");
        });

        Ok(Self {
            stop_flag,
            handle: Some(handle),
        })
    }

    pub fn stop(self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle {
            let _ = handle.join(); // wait for the thread to cleanly exit
        }
    }
}