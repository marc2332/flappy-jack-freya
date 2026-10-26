use freya::{engine::prelude::SkImage, prelude::*};

use crate::{
    app::{Action, App, Phase},
    scene::{self, CREAM, MUTED, NIGHT, ORANGE},
    title::TitleScreen,
};

const PAGE_PADDING: f32 = 24.0;
const SECTION_SPACING: f32 = 16.0;
const CONTROL_HEIGHT: f32 = 44.0;
const FOOTER_HEIGHT: f32 = 20.0;
pub(crate) const DEATH_BURST_DURATION: f32 = 0.35;

#[derive(PartialEq)]
pub struct GameView {
    pub app: State<App>,
    pub pumpkin: State<Option<SkImage>>,
}

impl Component for GameView {
    fn render(&self) -> impl IntoElement {
        let app = self.app.read();
        let show_title = app.phase == Phase::Title
            || (app.phase == Phase::GameOver && app.game.death_time >= DEATH_BURST_DURATION);
        let result = (app.phase == Phase::GameOver).then_some(app.game.score);

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .background(NIGHT)
            .maybe(show_title, |el| {
                el.child(TitleScreen {
                    app: self.app,
                    pumpkin: self.pumpkin,
                    result,
                })
            })
            .maybe(!show_title, |el| el.child(FlightScreen { app: self.app }))
            .maybe(app.phase == Phase::Paused, |el| {
                el.child(PauseOverlay { app: self.app })
            })
    }
}

#[derive(PartialEq)]
struct FlightScreen {
    app: State<App>,
}

impl Component for FlightScreen {
    fn render(&self) -> impl IntoElement {
        let app = self.app.read();
        let mut app_state = self.app;
        let window_size = *Platform::get().current_window().root_size.read();
        let reserved_height =
            PAGE_PADDING * 2.0 + SECTION_SPACING * 2.0 + CONTROL_HEIGHT + FOOTER_HEIGHT;
        let playfield_height = (window_size.height - reserved_height).max(0.0);

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .padding(PAGE_PADDING)
            .spacing(SECTION_SPACING)
            .child(Hud {
                score: app.game.score,
                best: app.best,
                demo: app.demo,
                on_pause: (move |()| app_state.write().action(Action::Pause)).into(),
            })
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(playfield_height))
                    .corner_radius(14.0)
                    .background(scene::flight(app.game.clone(), app.seconds))
                    .on_press(move |_| app_state.write().action(Action::Activate)),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(FOOTER_HEIGHT))
                    .center()
                    .child(
                        label()
                            .text("Space / click to flap  ·  P / Esc to pause  ·  Gate +1  ·  Candy +3")
                            .font_size(13.0)
                            .color(MUTED),
                    ),
            )
    }
}

#[derive(PartialEq)]
struct Hud {
    score: u32,
    best: u32,
    demo: bool,
    on_pause: EventHandler<()>,
}

impl Component for Hud {
    fn render(&self) -> impl IntoElement {
        rect()
            .horizontal()
            .width(Size::fill())
            .height(Size::px(CONTROL_HEIGHT))
            .cross_align(Alignment::center())
            .main_align(Alignment::space_between())
            .child(
                rect()
                    .horizontal()
                    .height(Size::px(CONTROL_HEIGHT))
                    .cross_align(Alignment::center())
                    .spacing(20.0)
                    .child(label().text("FLAPPY JACK").font_size(21.0).color(ORANGE))
                    .child(
                        label()
                            .text(format!("SCORE {:03}", self.score))
                            .font_size(21.0)
                            .color(CREAM),
                    )
                    .child(
                        label()
                            .text(format!(
                                "BEST {:03}{}",
                                self.best,
                                if self.demo { "  ·  DEMO" } else { "" }
                            ))
                            .font_size(16.0)
                            .color(MUTED),
                    ),
            )
            .child(ActionButton {
                text: "Pause",
                on_activate: self.on_pause.clone(),
            })
    }
}

#[derive(PartialEq)]
struct PauseOverlay {
    app: State<App>,
}

impl Component for PauseOverlay {
    fn render(&self) -> impl IntoElement {
        let mut app = self.app;

        rect()
            .position(Position::new_absolute().left(0.0).top(0.0))
            .width(Size::fill())
            .height(Size::fill())
            .background(Color::from_argb(210, 8, 7, 15))
            .center()
            .spacing(18.0)
            .child(label().text("PAUSED").font_size(42.0).color(CREAM))
            .child(
                label()
                    .text("Even pumpkins need a breath.")
                    .font_size(17.0)
                    .color(MUTED),
            )
            .child(ActionButton {
                text: "Resume flight",
                on_activate: (move |()| app.write().action(Action::Pause)).into(),
            })
            .child(
                label()
                    .text("P / Escape to resume  ·  Q to quit")
                    .font_size(13.0)
                    .color(MUTED),
            )
    }
}

#[derive(PartialEq)]
pub(crate) struct ActionButton {
    pub text: &'static str,
    pub on_activate: EventHandler<()>,
}

impl Component for ActionButton {
    fn render(&self) -> impl IntoElement {
        let accessibility_id = use_a11y();
        let focus = use_focus(accessibility_id);
        let mut hovered = use_state(|| false);
        let mut keyboard_held = use_state(|| false);
        let on_activate = self.on_activate.clone();

        rect()
            .width(Size::px(140.0))
            .height(Size::px(CONTROL_HEIGHT))
            .center()
            .corner_radius(10.0)
            .background(if hovered() {
                Color::from_rgb(255, 178, 91)
            } else {
                ORANGE
            })
            .a11y_id(accessibility_id)
            .a11y_role(AccessibilityRole::Button)
            .a11y_alt(self.text)
            .a11y_focusable(true)
            .maybe(focus() == Focus::Keyboard, |el| {
                el.border(Border::new().fill(CREAM).width(2.0))
            })
            .on_pointer_enter(move |_| hovered.set_if_modified(true))
            .on_pointer_leave(move |_| hovered.set_if_modified(false))
            .on_key_up(move |_| keyboard_held.set_if_modified(false))
            .on_press(move |event: Event<PressEventData>| {
                if let PressEventData::Keyboard(keyboard) = event.data() {
                    if matches!(&keyboard.key, Key::Character(character) if character == " ")
                        || *keyboard_held.peek()
                    {
                        return;
                    }
                    keyboard_held.set(true);
                }

                event.stop_propagation();
                on_activate.call(());
            })
            .child(label().text(self.text).font_size(16.0).color(NIGHT))
    }
}
