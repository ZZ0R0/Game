#![allow(clippy::needless_return)]

use std::thread;
use std::time::{Duration, Instant};

pub struct Engine {
    tick_hz: u32,
    dt: Duration,
    state: State,
    schedule: Schedule,
}

#[derive(Default)]
pub struct State {
    pub tick: u64,
}

pub struct Schedule {
    // later: DAG with phases; for now simple ordered list
    phases: Vec<fn(&mut State)>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            phases: vec![
                phase_input,
                phase_physics,
                phase_gameplay,
                phase_net,
                phase_render_prep,
            ],
        }
    }
    pub fn run(&self, s: &mut State) {
        for sys in &self.phases {
            sys(s);
        }
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

// Placeholder systems
fn phase_input(_s: &mut State) {}
fn phase_physics(_s: &mut State) {}
fn phase_gameplay(_s: &mut State) {}
fn phase_net(_s: &mut State) {}
fn phase_render_prep(_s: &mut State) {}

impl Engine {
    pub fn new_fixed_hz(tick_hz: u32) -> Self {
        assert!(tick_hz > 0);
        let dt = Duration::from_secs_f64(1.0 / tick_hz as f64);
        Self {
            tick_hz,
            dt,
            state: State::default(),
            schedule: Schedule::new(),
        }
    }

    pub fn run_blocking(&mut self) {
        // Fixed timestep, deterministic tick ordering
        let mut next_tick = Instant::now();
        let mut last_report = Instant::now();
        let mut ticks_in_window: u32 = 0;

        loop {
            let now = Instant::now();
            if now >= next_tick {
                self.tick_once();
                ticks_in_window += 1;
                next_tick += self.dt;

                // Catch-up if late, avoid spiral of death
                if now > next_tick + self.dt {
                    next_tick = now + self.dt;
                }
            } else {
                let sleep_for = next_tick - now;
                // sleep granularity is OK for 60 Hz targets
                thread::sleep(sleep_for);
            }

            // 1 Hz telemetry
            if last_report.elapsed() >= Duration::from_secs(1) {
                eprintln!(
                    "[engine] tick={} hz={} last_sec_ticks={}",
                    self.state.tick, self.tick_hz, ticks_in_window
                );
                ticks_in_window = 0;
                last_report = Instant::now();
            }
        }
    }

    pub fn tick_once(&mut self) {
        // deterministic order
        self.schedule.run(&mut self.state);
        self.state.tick = self.state.tick.wrapping_add(1);
    }
}
