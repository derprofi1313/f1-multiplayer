use std::sync::{Arc, LazyLock};

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

/// The banner's red, a shade under the accent so white type stays crisp on it.
const RED: u32 = 0xc8161f;
const RED_DEEP: u32 = 0x7d0b12;

fn stripes() -> Arc<Image> {
    static STRIPES: LazyLock<Arc<Image>> = LazyLock::new(|| {
        Arc::new(Image::from_bytes(
            ImageFormat::Png,
            include_bytes!("../assets/hero-stripes.png").to_vec(),
        ))
    });
    STRIPES.clone()
}

/// The F1 Multiplayer banner at the top of the News page, shorter in `compact` windows.
pub(crate) fn banner(compact: bool) -> Stateful<Div> {
    let white = |alpha: f32| hsla(0., 0., 1., alpha);
    let chip = |text: &'static str| {
        div()
            .px(px(8.))
            .py(px(3.))
            .rounded(px(4.))
            .bg(white(0.16))
            .text_size(px(11.))
            .font_weight(FontWeight::SEMIBOLD)
            .child(text)
    };
    h_flex()
        .id("hero-banner")
        .relative()
        .flex_shrink_0()
        .w_full()
        .min_h(px(if compact { 0. } else { 132. }))
        .px(px(if compact { 20. } else { 26. }))
        .py(px(if compact { 14. } else { 22. }))
        .gap(px(18.))
        .items_center()
        .rounded(px(10.))
        .overflow_hidden()
        .bg(linear_gradient(
            100.,
            linear_color_stop(rgb(RED), 0.),
            linear_color_stop(rgb(RED_DEEP), 1.),
        ))
        .child(
            img(stripes())
                .absolute()
                .top_0()
                .right_0()
                .h_full()
                .w(px(if compact { 360. } else { 520. })),
        )
        .child(crate::pages::logo(if compact { 48. } else { 76. }))
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap(px(4.))
                .text_color(white(1.))
                .child(
                    div()
                        .text_size(px(11.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(white(0.78))
                        .child("TWO PLAYERS \u{00b7} ONE CAREER \u{00b7} YOUR OWN SERVER"),
                )
                .child(
                    div()
                        .text_size(px(if compact { 18. } else { 24. }))
                        .font_weight(FontWeight::BOLD)
                        .child("F1 Multiplayer 2016\u{2013}2026"),
                )
                .when(!compact, |d| {
                    d.child(
                        h_flex()
                            .gap(px(6.))
                            .pt(px(4.))
                            .flex_wrap()
                            .child(chip("Six-digit server code"))
                            .child(chip("Two-player career"))
                            .child(chip("Classic seasons 2016\u{2013}2024"))
                            .child(chip("No EA servers")),
                    )
                }),
        )
}
