use freya::{engine::prelude::SkImage, prelude::*};

use crate::{
    app::{Action, App, Phase},
    scene::{self, CREAM, MUTED, NIGHT, ORANGE},
    title::TitleScreen,
};

#[derive(PartialEq)]
pub struct GameView {
    pub app: State<App>,
    pub pumpkin: State<Option<SkImage>>,
}

impl Component for GameView {
    fn render(&self) -> impl IntoElement {
        let app = self.app.read();
        let phase = app.phase;
        let result = (phase == Phase::GameOver).then_some(app.game.score);
        let show_title =
            phase == Phase::Title || (phase == Phase::GameOver && app.game.death_time >= 0.35);
        let mut app_state = self.app;
        let window_size = *Platform::get().current_window().root_size.read();
        let playfield_height = (window_size.height - 144.0).max(0.0);
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .background(NIGHT)
            .maybe(show_title, |el| {
                el.child(TitleScreen { app: self.app, pumpkin: self.pumpkin, result })
            })
            .maybe(!show_title, |el| {
                el.child(
                    rect()
                        .width(Size::fill())
                        .height(Size::fill())
                        .padding(24.0)
                        .spacing(16.0)
                        .child(
                            rect()
                                .horizontal()
                                .width(Size::fill())
                                .height(Size::px(44.0))
                                .cross_align(Alignment::center())
                                .main_align(Alignment::space_between())
                                .child(
                                    rect()
                                        .horizontal()
                                        .height(Size::px(44.0))
                                        .cross_align(Alignment::center())
                                        .spacing(20.0)
                                        .child(label().text("FLAPPY JACK").font_size(21.0).color(ORANGE))
                                        .child(label().text(format!("SCORE {:03}", app.game.score)).font_size(21.0).color(CREAM))
                                        .child(label().text(format!("BEST {:03}{}", app.best, if app.demo { "  ·  DEMO" } else { "" })).font_size(16.0).color(MUTED)),
                                )
                                .child(ActionButton { text: "Pause", app: self.app, action: Action::Pause }),
                        )
                        .child(
                            rect()
                                .width(Size::fill())
                                .height(Size::px(playfield_height))
                                .corner_radius(14.0)
                                .background(scene::flight(app.game.clone(), app.seconds))
                                .on_press(move |_| app_state.write().action(Action::Activate)),
                        )
                        .child(
                            rect().width(Size::fill()).height(Size::px(20.0)).center().child(
                                label().text("Space / click to flap  ·  P / Esc to pause  ·  Gate +1  ·  Candy +3").font_size(13.0).color(MUTED),
                            ),
                        ),
                )
            })
            .maybe(phase == Phase::Paused, |el| {
                el.child(
                    rect()
                        .position(Position::new_absolute().left(0.0).top(0.0))
                        .width(Size::fill())
                        .height(Size::fill())
                        .background(Color::from_argb(210, 8, 7, 15))
                        .center()
                        .spacing(18.0)
                        .child(label().text("PAUSED").font_size(42.0).color(CREAM))
                        .child(label().text("Even pumpkins need a breath.").font_size(17.0).color(MUTED))
                        .child(ActionButton { text: "Resume flight", app: self.app, action: Action::Pause })
                        .child(label().text("P / Escape to resume  ·  Q to quit").font_size(13.0).color(MUTED)),
                )
            })
    }
}

#[derive(PartialEq)]
pub struct ActionButton {
    pub text: &'static str,
    pub app: State<App>,
    pub action: Action,
}

impl Component for ActionButton {
    fn render(&self) -> impl IntoElement {
        let accessibility_id = use_a11y();
        let focus = use_focus(accessibility_id);
        let mut hovered = use_state(|| false);
        let mut keyboard_held = use_state(|| false);
        let mut app = self.app;
        let action = self.action;
        rect()
            .width(Size::px(140.0))
            .height(Size::px(44.0))
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
            .on_pointer_enter(move |_| hovered.set(true))
            .on_pointer_leave(move |_| hovered.set(false))
            .on_key_up(move |_| keyboard_held.set(false))
            .on_press(move |event: Event<PressEventData>| {
                if let PressEventData::Keyboard(keyboard) = event.data() {
                    if keyboard.key == Key::Character(" ".into()) || *keyboard_held.peek() {
                        return;
                    }
                    keyboard_held.set(true);
                }
                event.stop_propagation();
                app.write().action(action);
            })
            .child(label().text(self.text).font_size(16.0).color(NIGHT))
    }
}
