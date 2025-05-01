use anyhow::Result;
use atomic_float::AtomicF32;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use crate::gui::TimeSignature;

struct MetronomeState {
    next_tick_time: Instant,
    interval: Duration,
    playing_tick: bool,
    tick_sample_pos: usize,
    beat_index: usize,
    beats_per_measure: usize,
    shared_beat_index: Arc<AtomicUsize>,
    tick_active: Arc<AtomicBool>,
}

impl MetronomeState {
    fn new(bpm: f32, time_sig: TimeSignature, shared_beat_index: Arc<AtomicUsize>, tick_active: Arc<AtomicBool>) -> Self {
        let interval = Duration::from_secs_f32((60.0 / bpm) * (4.0/time_sig.bottom as f32));
        Self {
            next_tick_time: Instant::now() + interval,
            interval,
            playing_tick: false,
            tick_sample_pos: 0,
            beat_index: 0,
            beats_per_measure: time_sig.top as usize,
            shared_beat_index,
            tick_active,
        }
    }

    fn check_and_trigger_tick(&mut self, now: Instant) {
        if now >= self.next_tick_time {
            self.playing_tick = true;
            self.tick_active.store(true, Ordering::Relaxed);
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
    pub fn start(bpm: u32, time_sig: TimeSignature, volume: Arc<AtomicF32>) -> Result<(Self, Arc<AtomicUsize>, Arc<AtomicBool>)> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();
    
        let beat_index_shared = Arc::new(AtomicUsize::new(0));
        let active_tick = Arc::new(AtomicBool::new(false));
    
        let beat_index_for_thread = beat_index_shared.clone();
        let active_tick_for_thread = active_tick.clone();
    
        let handle = thread::spawn(move || {
            let host = cpal::default_host();
            let device = host.default_output_device().expect("No output device");
            let config = device.default_output_config().unwrap().config();
    
            let sample_rate = config.sample_rate.0 as f32;
            let tick_duration = 0.05;
    
            let state = Arc::new(Mutex::new(MetronomeState::new(
                bpm as f32,
                time_sig,
                beat_index_for_thread,
                active_tick_for_thread,
            )));
    
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
                                let tick_freq = if metro.beat_index == 0 { 432.0 } else { 216.0 };
                                let t = metro.tick_sample_pos as f32 / sample_rate;
                                *sample = (2.0 * std::f32::consts::PI * tick_freq * t).sin() * volume.load(Ordering::Relaxed);
                                metro.tick_sample_pos += 1;
    
                                if (metro.tick_sample_pos as f32) / sample_rate > tick_duration {
                                    metro.playing_tick = false;
                                    metro.shared_beat_index.store(metro.beat_index, Ordering::Relaxed);
                                    metro.beat_index = (metro.beat_index + 1) % metro.beats_per_measure;
                                    thread::spawn({
                                        let tick_flag = metro.tick_active.clone();
                                        move || {
                                            std::thread::sleep(Duration::from_secs_f32(tick_duration));
                                            tick_flag.store(false, Ordering::Relaxed);
                                        }
                                    });
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
    
            drop(stream);
            println!("Metronome stopped.");
        });
    
        Ok((
            Self {
                stop_flag,
                handle: Some(handle),
            },
            beat_index_shared,
            active_tick,
        ))
    }
    

    pub fn stop(self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle {
            let _ = handle.join(); // wait for the thread to cleanly exit
        }
    }
}