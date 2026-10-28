use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Running,
    Paused,
    Ready,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Countdown {
    duration: Duration,
    remaining: Duration,
    state: TimerState,
}

impl Countdown {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            remaining: duration,
            state: if duration.is_zero() {
                TimerState::Finished
            } else {
                TimerState::Running
            },
        }
    }

    pub fn is_finished(&self) -> bool {
        self.remaining.is_zero()
    }

    pub fn remaining(&self) -> Duration {
        self.remaining
    }

    pub fn state(&self) -> TimerState {
        self.state
    }

    pub fn stop(&mut self) -> Result<(), &'static str> {
        if self.state != TimerState::Running {
            return Err("Timer is not running.");
        }
        self.state = TimerState::Paused;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), &'static str> {
        if !matches!(self.state, TimerState::Paused | TimerState::Ready) {
            return Err("Only a paused or ready timer can continue.");
        }
        self.state = TimerState::Running;
        Ok(())
    }

    pub fn restart(&mut self, duration: Option<Duration>) {
        if let Some(duration) = duration {
            self.duration = duration;
        }
        self.remaining = self.duration;
        self.state = if self.remaining.is_zero() {
            TimerState::Finished
        } else {
            TimerState::Running
        };
    }

    pub fn set_duration(&mut self, duration: Duration) -> Result<(), &'static str> {
        if self.state == TimerState::Running {
            return Err("Stop the timer before setting its duration.");
        }
        self.duration = duration;
        self.remaining = duration;
        self.state = if duration.is_zero() {
            TimerState::Finished
        } else {
            TimerState::Ready
        };
        Ok(())
    }

    pub fn tick(&mut self, elapsed: Duration) {
        if self.state == TimerState::Running {
            self.remaining = self.remaining.saturating_sub(elapsed);
            if self.is_finished() {
                self.state = TimerState::Finished;
            }
        }
    }

    pub fn display_text(&self) -> String {
        let seconds = self.remaining.as_secs() + u64::from(self.remaining.subsec_nanos() > 0);
        format!(
            "{:02}:{:02}:{:02}",
            seconds / 3600,
            (seconds % 3600) / 60,
            seconds % 60
        )
    }
}

pub fn parse_duration(input: &str) -> Option<Duration> {
    let mut rest = input;
    let mut total = 0u64;
    let mut found = false;

    for (suffix, multiplier) in [("h", 3600u64), ("m", 60), ("s", 1)] {
        if let Some(index) = rest.find(suffix) {
            let digits = &rest[..index];
            if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            let value = digits.parse::<u64>().ok()?;
            total = total.checked_add(value.checked_mul(multiplier)?)?;
            rest = &rest[index + suffix.len()..];
            found = true;
        }
    }

    if !found || !rest.is_empty() || total == 0 {
        return None;
    }

    Some(Duration::from_secs(total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_duration_components() {
        assert_eq!(parse_duration("1h30m10s"), Some(Duration::from_secs(5410)));
        assert_eq!(parse_duration("45s"), Some(Duration::from_secs(45)));
        assert_eq!(parse_duration("2m"), Some(Duration::from_secs(120)));
    }

    #[test]
    fn rejects_invalid_or_zero_durations() {
        assert_eq!(parse_duration("0s"), None);
        assert_eq!(parse_duration("1x"), None);
        assert_eq!(parse_duration(""), None);
    }

    #[test]
    fn keeps_fractional_seconds_visible_until_finished() {
        let mut countdown = Countdown::new(Duration::from_secs(2));
        countdown.tick(Duration::from_millis(1));
        assert_eq!(countdown.display_text(), "00:00:02");
        countdown.tick(Duration::from_millis(999));
        assert_eq!(countdown.display_text(), "00:00:01");
    }

    #[test]
    fn rejects_overflows_and_invalid_component_order() {
        for input in [
            "18446744073709551616s",
            "18446744073709551615h",
            "1s1m",
            "1m1m",
            "+1s",
            "1m 2s",
        ] {
            assert_eq!(parse_duration(input), None, "{input}");
        }
    }

    #[test]
    fn tick_saturates_and_display_changes_when_finished() {
        let mut countdown = Countdown::new(Duration::from_secs(2));
        countdown.tick(Duration::from_secs(1));
        assert_eq!(countdown.display_text(), "00:00:01");
        countdown.tick(Duration::from_secs(3));
        assert!(countdown.is_finished());
        assert_eq!(countdown.display_text(), "00:00:00");
        countdown.tick(Duration::from_secs(5));
        assert_eq!(countdown.display_text(), "00:00:00");
        assert_eq!(countdown.state(), TimerState::Finished);
    }
}
