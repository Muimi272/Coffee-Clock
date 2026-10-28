use std::time::Duration;

use crate::model::{Countdown, TimerState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Work,
    Break,
    Rest,
}

impl Phase {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "work" => Some(Self::Work),
            "break" => Some(Self::Break),
            "rest" => Some(Self::Rest),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Work => "Work",
            Self::Break => "Break",
            Self::Rest => "Rest",
        }
    }
}

#[derive(Debug)]
pub struct Pomodoro {
    timer: Countdown,
    phase: Phase,
    completed_work: u8,
    worktime: Duration,
    breaktime: Duration,
    resttime: Duration,
}

impl Default for Pomodoro {
    fn default() -> Self {
        Self {
            timer: Countdown::new(Duration::from_secs(25 * 60)),
            phase: Phase::Work,
            completed_work: 0,
            worktime: Duration::from_secs(25 * 60),
            breaktime: Duration::from_secs(5 * 60),
            resttime: Duration::from_secs(15 * 60),
        }
    }
}

impl Pomodoro {
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn display_text(&self) -> String {
        self.timer.display_text()
    }

    pub fn stop(&mut self) -> Result<(), &'static str> {
        self.timer.stop()
    }
    pub fn resume(&mut self) -> Result<(), &'static str> {
        self.timer.resume()
    }

    pub fn set_duration(&mut self, phase: Phase, duration: Duration) -> Result<(), &'static str> {
        if duration.is_zero() {
            return Err("Phase duration must be positive.");
        }
        match phase {
            Phase::Work => self.worktime = duration,
            Phase::Break => self.breaktime = duration,
            Phase::Rest => self.resttime = duration,
        }
        Ok(())
    }

    fn duration(&self, phase: Phase) -> Duration {
        match phase {
            Phase::Work => self.worktime,
            Phase::Break => self.breaktime,
            Phase::Rest => self.resttime,
        }
    }

    pub fn restart(&mut self, phase: Phase) {
        self.completed_work = 0;
        self.enter(phase);
    }

    fn enter(&mut self, phase: Phase) {
        self.phase = phase;
        self.timer.restart(Some(self.duration(phase)));
    }

    pub fn tick(&mut self, mut elapsed: Duration) {
        if self.timer.state() != TimerState::Running {
            return;
        }
        while elapsed >= self.timer.remaining() {
            elapsed -= self.timer.remaining();
            let next = match self.phase {
                Phase::Work => {
                    self.completed_work += 1;
                    if self.completed_work == 4 {
                        Phase::Rest
                    } else {
                        Phase::Break
                    }
                }
                Phase::Break => Phase::Work,
                Phase::Rest => {
                    self.completed_work = 0;
                    Phase::Work
                }
            };
            self.enter(next);
            // Skip whole identical cycles after returning to their starting boundary.
            if self.phase == Phase::Work
                && self.completed_work == 0
                && let Some(cycle) = self
                    .worktime
                    .checked_mul(4)
                    .and_then(|work| {
                        self.breaktime
                            .checked_mul(3)
                            .and_then(|short| work.checked_add(short))
                    })
                    .and_then(|cycle| cycle.checked_add(self.resttime))
            {
                let remainder = elapsed.as_nanos() % cycle.as_nanos();
                elapsed = Duration::new(
                    (remainder / 1_000_000_000) as u64,
                    (remainder % 1_000_000_000) as u32,
                );
            }
        }
        self.timer.tick(elapsed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_cycle_through_four_work_periods_and_long_rest() {
        let mut pomo = Pomodoro::default();
        for completed in 1..=4 {
            assert_eq!(pomo.phase(), Phase::Work);
            assert_eq!(pomo.display_text(), "00:25:00");
            pomo.tick(Duration::from_secs(1500));
            assert_eq!(pomo.completed_work, completed);
            assert_eq!(
                pomo.phase(),
                if completed == 4 {
                    Phase::Rest
                } else {
                    Phase::Break
                }
            );
            assert_eq!(
                pomo.display_text(),
                if completed == 4 {
                    "00:15:00"
                } else {
                    "00:05:00"
                }
            );
            pomo.tick(Duration::from_secs(if completed == 4 { 900 } else { 300 }));
        }
        assert_eq!(pomo.phase(), Phase::Work);
        assert_eq!(pomo.completed_work, 0);
    }

    #[test]
    fn settings_are_deferred_for_each_current_phase_and_restarts_reset_count() {
        let mut pomo = Pomodoro::default();
        for phase in [Phase::Work, Phase::Break, Phase::Rest] {
            pomo.restart(phase);
            let before = pomo.display_text();
            pomo.set_duration(phase, Duration::from_secs(7)).unwrap();
            assert_eq!(pomo.display_text(), before);
            pomo.completed_work = 3;
            pomo.restart(phase);
            assert_eq!(pomo.display_text(), "00:00:07");
            assert_eq!(pomo.completed_work, 0);
        }
        assert!(pomo.set_duration(Phase::Work, Duration::ZERO).is_err());
    }

    #[test]
    fn pause_and_delayed_ticks_preserve_time_across_transitions() {
        let mut pomo = Pomodoro::default();
        pomo.stop().unwrap();
        pomo.tick(Duration::from_secs(5000));
        assert_eq!(pomo.display_text(), "00:25:00");
        pomo.resume().unwrap();
        pomo.tick(Duration::from_millis(1500500));
        assert_eq!(pomo.phase(), Phase::Break);
        assert_eq!(pomo.timer.remaining(), Duration::from_millis(299500));
        pomo.tick(Duration::from_millis(299500));
        assert_eq!(pomo.phase(), Phase::Work);
        assert_eq!(pomo.display_text(), "00:25:00");
        pomo.restart(Phase::Work);
        pomo.tick(Duration::from_secs(7800 * 100 + 1501));
        assert_eq!(pomo.phase(), Phase::Break);
        assert_eq!(pomo.display_text(), "00:04:59");
        assert_eq!(pomo.completed_work, 1);
    }
}
