use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::Deserialize;
use std::f32::consts::PI;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};

struct MetronomeState {
    next_tick_time: Instant,
    interval: Duration,
    playing_tick: bool,
    tick_sample_pos: usize,
    beat_index: usize,
    beats_per_measure: usize,
    shared_beat_index: Arc<AtomicUsize>,
    tick_active: Arc<AtomicBool>,
    config: Arc<Mutex<MetronomeConfig>>,
}

#[derive(Debug, Clone)]
pub struct MetronomeConfig {
    pub bpm: f32,
    pub time_signature: TimeSignature,
    pub volume: f32,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct TimeSignature {
    pub top: u8,
    pub bottom: u8,
}

impl MetronomeState {
    fn new(
        config: Arc<Mutex<MetronomeConfig>>,
        shared_beat_index: Arc<AtomicUsize>,
        tick_active: Arc<AtomicBool>,
    ) -> Self {
        let(interval, beats_per_measure) = {
            let config_lock = config.lock().unwrap();
            let interval = Duration::from_secs_f32((60.0 / config_lock.bpm) * (4.0 / config_lock.time_signature.bottom as f32));
            let beats = config_lock.time_signature.top as usize;
            (interval, beats)
        };

        Self {
            next_tick_time: Instant::now() + interval,
            interval,
            playing_tick: false,
            tick_sample_pos: 0,
            beat_index: 0,
            beats_per_measure,
            shared_beat_index,
            tick_active,
            config
        }
    }

    fn check_and_trigger_tick(&mut self, now: Instant) {
        if now >= self.next_tick_time {
            let cfg = self.config.lock().unwrap();
            let interval = Duration::from_secs_f32((60.0 / cfg.bpm) * (4.0 / cfg.time_signature.bottom as f32));
            self.interval = interval;
            self.beats_per_measure = cfg.time_signature.top as usize;            
            
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
    pub fn start(
        config: Arc<Mutex<MetronomeConfig>>,
        app_handle: AppHandle,
    ) -> Result<(Self, Arc<AtomicUsize>, Arc<AtomicBool>, Arc<Mutex<MetronomeConfig>>)> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        let beat_index_shared = Arc::new(AtomicUsize::new(0));
        let active_tick = Arc::new(AtomicBool::new(false));

        let state = Arc::new(Mutex::new(MetronomeState::new(
    Arc::clone(&config),
            Arc::clone(&beat_index_shared),
            Arc::clone(&active_tick),
            )));

        let handle = thread::spawn(move || {
            let host = cpal::default_host();
            let device = host.default_output_device().expect("No output device");
            let config = device.default_output_config().unwrap().config();
            let app_handle = app_handle.clone();

            let sample_rate = config.sample_rate.0 as f32;
            let tick_duration = 0.05;

            // let state = Arc::new(Mutex::new(MetronomeState::new(
            //     bpm as f32,
            //     time_sig,
            //     beat_index_for_thread,
            //     active_tick_for_thread,
            // )));

            let stream = device
                .build_output_stream(
                    &config,
                    {
                        let state = state.clone();
                        move |data: &mut [f32], _| {
                            let now = Instant::now();
                            let mut metro = state.lock().unwrap();

                            metro.check_and_trigger_tick(now);

                            for sample in data.iter_mut() {
                                if metro.playing_tick {
                                    let tick_freq =
                                        if metro.beat_index == 0 { 432.0 } else { 216.0 };
                                    let t = metro.tick_sample_pos as f32 / sample_rate;

                                    let volume = {
                                        let cfg = metro.config.lock().unwrap();
                                        cfg.volume
                                    };

                                    *sample = (2.0 * PI * tick_freq * t).sin() * volume;

                                    metro.tick_sample_pos += 1;

                                    //if (metro.tick_sample_pos as f32) / sample_rate > tick_duration {
                                    if metro.tick_sample_pos >= (tick_duration * sample_rate) as usize {
                                        metro.playing_tick = false;
                                        metro
                                            .shared_beat_index
                                            .store(metro.beat_index, Ordering::Relaxed);
                                        let _ = app_handle.emit("tick", metro.beat_index);
                                        metro.beat_index =
                                            (metro.beat_index + 1) % metro.beats_per_measure;
                                        thread::spawn({
                                            let tick_flag = metro.tick_active.clone();
                                            move || {
                                                std::thread::sleep(Duration::from_secs_f32(
                                                    tick_duration,
                                                ));
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
                )
                .unwrap();

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
            config.clone(),
        ))
    }

    pub fn stop(self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle {
            let _ = handle.join(); // wait for the thread to cleanly exit
        }
    }
}

pub fn update_config(
    config: &Arc<Mutex<MetronomeConfig>>,
    bpm: f32,
    time_signature: TimeSignature,
    volume: f32,
) {
    let mut cfg = config.lock().unwrap();
    cfg.bpm = bpm;
    cfg.time_signature = time_signature;
    cfg.volume = volume;
}