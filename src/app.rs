use std::{path::PathBuf, time::Duration};

use crate::{game::Game, storage};

const HZ: u128 = 120;
const TICK: u128 = 1_000_000_000;
const DT: f32 = 1.0 / HZ as f32;
const MAX_ELAPSED: Duration = Duration::from_millis(250);
const SEED: u64 = 0x4a41_434b;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum Phase {
    Title,
    Playing,
    Paused,
    GameOver,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum Action {
    Activate,
    Retry,
    Pause,
    Quit,
}

pub struct App {
    pub phase: Phase,
    pub game: Game,
    pub seconds: f32,
    pub best: u32,
    pub demo: bool,
    pub quit: bool,
    pub suspended: bool,
    accumulator: u128,
    title_ticks: u32,
    seed: u64,
    score_path: Option<PathBuf>,
}

impl App {
    pub fn new(demo: bool) -> Self {
        let score_path = if demo || cfg!(test) {
            None
        } else {
            storage::best_path()
        };
        Self::with_score_path(demo, score_path)
    }

    pub fn with_score_path(demo: bool, score_path: Option<PathBuf>) -> Self {
        let score_path = if demo { None } else { score_path };
        let best = score_path.as_deref().map(storage::load).unwrap_or(0);
        Self {
            phase: Phase::Title,
            game: Game::new(SEED),
            seconds: 0.0,
            best,
            demo,
            quit: false,
            suspended: false,
            accumulator: 0,
            title_ticks: 0,
            seed: SEED,
            score_path,
        }
    }

    pub fn advance(&mut self, elapsed: Duration, suspended: bool) {
        let changed = self.suspended != suspended;
        self.suspended = suspended;
        if self.quit || suspended || changed || self.phase == Phase::Paused {
            self.accumulator = 0;
            return;
        }
        let elapsed = elapsed.min(MAX_ELAPSED);
        self.seconds += elapsed.as_secs_f32();
        self.accumulator += elapsed.as_nanos() * HZ;
        while self.accumulator >= TICK {
            self.accumulator -= TICK;
            match self.phase {
                Phase::Title => {
                    self.title_ticks += 1;
                    if self.demo && self.title_ticks >= 600 {
                        self.start();
                    }
                }
                Phase::Playing => {
                    if self.demo {
                        self.game.demo_input();
                    }
                    self.game.tick(DT);
                    if !self.game.alive {
                        self.phase = Phase::GameOver;
                        if self.game.score > self.best {
                            self.best = self.game.score;
                            if let Some(path) = &self.score_path {
                                let _ = storage::save(path, self.best);
                            }
                        }
                    }
                }
                Phase::GameOver => {
                    self.game.tick(DT);
                    if self.demo && self.game.death_time >= 1.5 {
                        self.start();
                    }
                }
                Phase::Paused => unreachable!(),
            }
        }
    }

    pub fn action(&mut self, action: Action) {
        if action == Action::Quit {
            self.quit = true;
            return;
        }
        if self.quit || self.suspended {
            return;
        }
        match (self.phase, action) {
            (Phase::Title | Phase::GameOver, Action::Activate)
            | (Phase::GameOver, Action::Retry) => self.start(),
            (Phase::Playing, Action::Activate) => {
                self.game.flap();
                return;
            }
            (Phase::Playing, Action::Pause) => self.phase = Phase::Paused,
            (Phase::Paused, Action::Pause) => self.phase = Phase::Playing,
            _ => return,
        }
        self.accumulator = 0;
    }

    pub fn pause_for_focus_loss(&mut self) {
        if self.phase == Phase::Playing {
            self.phase = Phase::Paused;
        }
        self.accumulator = 0;
    }

    fn start(&mut self) {
        self.game = Game::new(self.seed);
        if !self.demo {
            self.seed = self.seed.wrapping_add(1);
        }
        self.phase = Phase::Playing;
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{Action, App, Phase};
    use std::time::Duration;

    #[test]
    fn starting_and_retrying_do_not_flap() {
        let mut app = App::with_score_path(false, None);
        app.action(Action::Retry);
        assert_eq!(app.phase, Phase::Title);
        app.action(Action::Activate);
        assert_eq!(app.phase, Phase::Playing);
        assert_eq!(app.game.velocity, 0.0);
        app.action(Action::Activate);
        assert!(app.game.velocity < 0.0);
        app.phase = Phase::GameOver;
        app.action(Action::Retry);
        assert_eq!(app.game.velocity, 0.0);
    }

    #[test]
    fn pause_focus_loss_and_suspension_freeze_without_catchup() {
        let mut app = App::with_score_path(false, None);
        app.action(Action::Activate);
        app.advance(Duration::from_millis(40), false);
        app.pause_for_focus_loss();
        let frozen = app.game.clone();
        app.advance(Duration::from_secs(60), true);
        app.advance(Duration::from_secs(60), false);
        assert_eq!(app.phase, Phase::Paused);
        assert_eq!(app.game, frozen);
        app.action(Action::Pause);
        app.advance(Duration::ZERO, false);
        assert_eq!(app.game, frozen);
        app.advance(Duration::from_millis(10), false);
        assert!(app.game.time > frozen.time);
    }

    #[test]
    fn accumulator_is_exact_and_stalls_are_clamped() {
        let mut fast = App::with_score_path(true, None);
        let mut slow = App::with_score_path(true, None);
        for _ in 0..1000 {
            fast.advance(Duration::from_millis(7), false);
        }
        for _ in 0..200 {
            slow.advance(Duration::from_millis(35), false);
        }
        assert_eq!(fast.game, slow.game);
        assert_eq!(fast.accumulator, slow.accumulator);
        let mut app = App::with_score_path(false, None);
        app.action(Action::Activate);
        app.advance(Duration::from_secs(60), false);
        assert!((app.game.time - 0.25).abs() < 0.00001);
    }

    #[test]
    fn demo_waits_five_seconds_and_never_opens_storage() {
        let mut app = App::with_score_path(true, Some("unused-best-score".into()));
        assert!(app.score_path.is_none());
        for _ in 0..49 {
            app.advance(Duration::from_millis(100), false);
        }
        assert_eq!(app.phase, Phase::Title);
        app.advance(Duration::from_millis(100), false);
        assert_eq!(app.phase, Phase::Playing);
        assert_eq!(app.game.velocity, 0.0);
    }

    #[test]
    fn death_preserves_the_game_and_persists_best() {
        let directory = crate::storage::tests::TestDir::new();
        let path = directory.0.join("best-score");
        let mut app = App::with_score_path(false, Some(path.clone()));
        app.action(Action::Activate);
        app.game.score = 23;
        app.game.alive = false;
        app.advance(Duration::from_millis(10), false);
        assert_eq!(app.phase, Phase::GameOver);
        assert_eq!(app.game.score, 23);
        assert_eq!(crate::storage::load(&path), 23);
        app.action(Action::Retry);
        assert_eq!(app.best, 23);
    }
}
