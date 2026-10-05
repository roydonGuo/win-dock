pub fn should_restore(original: u32, applied: u32, current: u32) -> bool {
    original != applied && current == applied
}
#[derive(Default)]
pub struct WinTap {
    last_up: Option<u64>,
    down: bool,
    chord: bool,
}
impl WinTap {
    pub fn key(&mut self, win: bool, down: bool, time: u64, injected: bool) -> bool {
        if injected {
            return false;
        }
        if !win {
            if down {
                self.chord = true;
                self.last_up = None;
            }
            return false;
        }
        if down {
            if !self.down {
                self.chord = false;
            }
            self.down = true;
            return false;
        }
        if !self.down {
            return false;
        }
        self.down = false;
        if self.chord {
            self.last_up = None;
            return false;
        }
        let toggle = self
            .last_up
            .is_some_and(|last| time.saturating_sub(last) <= 300);
        self.last_up = if toggle { None } else { Some(time) };
        toggle
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restoration_only_when_our_change_is_still_current() {
        assert!(should_restore(0, 1, 1));
        assert!(!should_restore(0, 1, 0));
        assert!(!should_restore(1, 1, 1));
    }
    #[test]
    fn double_win_requires_two_complete_taps() {
        let mut t = WinTap::default();
        assert!(!t.key(true, true, 0, false));
        assert!(!t.key(true, true, 10, false));
        assert!(!t.key(true, false, 60, false));
        assert!(!t.key(true, true, 100, false));
        assert!(t.key(true, false, 160, false));
    }
    #[test]
    fn win_chords_injected_events_and_slow_taps_do_not_toggle() {
        let mut t = WinTap::default();
        t.key(true, true, 0, false);
        t.key(false, true, 10, false);
        t.key(true, false, 20, false);
        t.key(true, true, 30, false);
        assert!(!t.key(true, false, 40, false));
        t.key(true, true, 500, false);
        assert!(!t.key(true, false, 550, false));
        assert!(!t.key(true, true, 600, true));
        assert!(!t.key(true, false, 610, true));
    }
}
