use freya::{
    engine::prelude::{FilterMode, Paint, SamplingOptions, SkImage, SkRect},
    prelude::*,
};

use crate::{
    app::{Action, App},
    scene::{self, CREAM, MUTED, ORANGE},
    ui::ActionButton,
};

#[derive(PartialEq)]
pub struct TitleScreen {
    pub app: State<App>,
    pub pumpkin: State<Option<SkImage>>,
    pub result: Option<u32>,
}

impl Component for TitleScreen {
    fn render(&self) -> impl IntoElement {
        let app = self.app.read();
        let best = app.best;
        let demo = app.demo;
        let seconds = app.seconds;
        let pumpkin = self.pumpkin.read().clone();
        let height = Platform::get().current_window().root_size.read().height;
        let hero_size = (height * 0.42).clamp(130.0, 340.0);
        let background = RenderCallback::new(move |context| {
            if let RenderTarget::Background { canvas, .. } = &context.target {
                scene::backdrop(canvas, context.size.width, context.size.height, seconds);
            }
        });
        let hero = RenderCallback::new(move |context| {
            if let RenderTarget::Background { canvas, .. } = &context.target
                && let Some(image) = &pumpkin
            {
                canvas.draw_image_rect_with_sampling_options(
                    image,
                    None,
                    SkRect::from_wh(context.size.width, context.size.height),
                    SamplingOptions::from(FilterMode::Linear),
                    &Paint::default(),
                );
            }
        });
        rect()
            .width(Size::fill())
            .height(Size::fill())
            .background(background)
            .center()
            .spacing(8.0)
            .child(
                label()
                    .text("A MIDNIGHT FLIGHT")
                    .font_size(12.0)
                    .color(MUTED),
            )
            .child(
                label()
                    .text("FLAPPY JACK")
                    .font_size((height * 0.075).clamp(28.0, 58.0))
                    .color(ORANGE),
            )
            .child(
                rect()
                    .width(Size::px(hero_size))
                    .height(Size::px(hero_size))
                    .background(hero),
            )
            .maybe(self.result.is_some(), |el| {
                el.child(
                    label()
                        .text(format!(
                            "SCORE {:03}    BEST {best:03}",
                            self.result.unwrap_or(0)
                        ))
                        .font_size(22.0)
                        .color(CREAM),
                )
            })
            .maybe(self.result.is_none(), |el| {
                el.child(
                    label()
                        .text(format!("BEST {best:03}  ·  Gate +1  ·  Candy +3"))
                        .font_size(15.0)
                        .color(MUTED),
                )
            })
            .child(ActionButton {
                text: if self.result.is_some() {
                    "Fly again"
                } else {
                    "Awaken"
                },
                app: self.app,
                action: Action::Activate,
            })
            .child(
                label()
                    .text(if self.result.is_some() {
                        "Space or R to retry"
                    } else {
                        "Press Space to awaken"
                    })
                    .font_size(14.0)
                    .color(CREAM),
            )
            .child(
                label()
                    .text(if demo {
                        "DEMO  ·  Deterministic flight, no records saved"
                    } else {
                        "Space to flap  ·  P / Esc to pause  ·  Q to quit"
                    })
                    .font_size(12.0)
                    .color(MUTED),
            )
    }
}
