use std::{
    collections::HashSet,
    rc::Rc,
    sync::mpsc::{self, Receiver, SyncSender},
    time::{Duration, Instant},
};

use freya::{
    engine::prelude::{AlphaType, ColorType, Data, ImageInfo, SkImage, images},
    prelude::*,
};

use crate::{
    app::{Action, App as GameApp, Phase},
    pumpkin,
    scene::NIGHT,
    ui::{DEATH_BURST_DURATION, GameView},
};

const PUMPKIN_SIZE: u16 = 256;

struct PumpkinWorker {
    requests: SyncSender<f32>,
    frames: Receiver<Vec<u8>>,
}

impl PumpkinWorker {
    fn new() -> Self {
        let (requests, times) = mpsc::sync_channel::<f32>(0);
        let (frames, results) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            while let Ok(seconds) = times.recv() {
                let pixels = pumpkin::render(PUMPKIN_SIZE, PUMPKIN_SIZE, seconds);
                match frames.try_send(pixels) {
                    Ok(()) | Err(mpsc::TrySendError::Full(_)) => {}
                    Err(mpsc::TrySendError::Disconnected(_)) => break,
                }
            }
        });
        Self {
            requests,
            frames: results,
        }
    }

    fn receive_image(&self) -> Option<SkImage> {
        let pixels = self.frames.try_recv().ok()?;
        let dimensions = (i32::from(PUMPKIN_SIZE), i32::from(PUMPKIN_SIZE));
        let info = ImageInfo::new(dimensions, ColorType::RGBA8888, AlphaType::Opaque, None);
        images::raster_from_data(
            &info,
            Data::new_copy(&pixels),
            usize::from(PUMPKIN_SIZE) * 4,
        )
    }
}

pub fn launch_game(demo: bool) {
    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new_app(GameWindow { demo })
                .with_title("Flappy Jack")
                .with_size(1100.0, 700.0)
                .with_min_size(700.0, 520.0)
                .with_background(NIGHT),
        ),
    );
}

struct GameWindow {
    demo: bool,
}

impl App for GameWindow {
    fn render(&self) -> impl IntoElement {
        let mut app = use_state(|| GameApp::new(self.demo));
        let mut pumpkin = use_state(|| None::<SkImage>);
        let mut held_keys = use_state(HashSet::<Code>::new);
        let worker = use_hook(|| Rc::new(PumpkinWorker::new()));
        let window = Platform::get().current_window();
        let focus_window = window.clone();
        use_side_effect(move || {
            let focused = *focus_window.is_app_focused.read();
            let size = *focus_window.root_size.read();
            let suspended = !focused || size.width <= 0.0 || size.height <= 0.0;
            if suspended {
                held_keys.write().clear();
                if app.peek().phase == Phase::Playing {
                    app.write().pause_for_focus_loss();
                }
            }
            if app.peek().suspended != suspended {
                app.write().advance(Duration::ZERO, suspended);
            }
        });
        use_future(move || {
            let worker = worker.clone();
            let window = window.clone();
            async move {
                let ticker = RenderingTicker::get();
                let mut previous = Instant::now();
                loop {
                    ticker.tick().await;
                    let now = Instant::now();
                    let elapsed = now.duration_since(previous);
                    previous = now;
                    let size = *window.root_size.peek();
                    let suspended =
                        !*window.is_app_focused.peek() || size.width <= 0.0 || size.height <= 0.0;
                    let advancing = !suspended && app.peek().phase != Phase::Paused;
                    if advancing || app.peek().suspended != suspended {
                        app.write().advance(elapsed, suspended);
                    }

                    if let Some(image) = worker.receive_image() {
                        pumpkin.set(Some(image));
                    }
                    let (render_pumpkin, seconds, quit) = {
                        let app = app.peek();
                        let render_pumpkin = app.phase == Phase::Title
                            || (app.phase == Phase::GameOver
                                && app.game.death_time >= DEATH_BURST_DURATION);
                        (render_pumpkin, app.seconds, app.quit)
                    };
                    if quit {
                        Platform::get().exit();
                        break;
                    }
                    if !suspended {
                        if render_pumpkin {
                            let _ = worker.requests.try_send(seconds);
                        }
                        window.request_redraw();
                    }
                }
            }
        });
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .background(NIGHT)
            .on_global_key_down(move |event: Event<KeyboardEventData>| {
                let Some(action) = key_action(&event) else {
                    return;
                };
                event.prevent_default();
                if held_keys.write().insert(event.code) {
                    app.write().action(action);
                    if action == Action::Quit {
                        Platform::get().exit();
                    }
                }
            })
            .on_global_key_up(move |event: Event<KeyboardEventData>| {
                held_keys.write().remove(&event.code);
            })
            .child(GameView { app, pumpkin })
    }
}

fn key_action(event: &KeyboardEventData) -> Option<Action> {
    match &event.key {
        Key::Character(character) => match character.to_ascii_lowercase().as_str() {
            " " => Some(Action::Activate),
            "r" => Some(Action::Retry),
            "p" => Some(Action::Pause),
            "q" => Some(Action::Quit),
            "c" if event.modifiers.contains(Modifiers::CONTROL) => Some(Action::Quit),
            _ => None,
        },
        Key::Named(NamedKey::Escape) => Some(Action::Pause),
        _ => None,
    }
}
