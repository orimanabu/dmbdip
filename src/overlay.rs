use ab_glyph::PxScale;
use image::{Rgb, RgbImage};
use crate::fonts::{draw_text_mut, text_size};

use crate::constants::*;
use crate::fonts::Fonts;

pub(crate) fn render_help_overlay(vp_width: u32, vp_height: u32, fonts: &Fonts) -> RgbImage {
    render_help_overlay_with(vp_width, vp_height, fonts, "Keybindings", KEYBINDINGS)
}

pub(crate) fn render_help_overlay_with(
    vp_width: u32,
    vp_height: u32,
    fonts: &Fonts,
    title: &str,
    bindings: &[(&str, &str)],
) -> RgbImage {
    let mut img = RgbImage::from_pixel(vp_width, vp_height, Rgb([30, 30, 40]));
    let content_width = (vp_width - MARGIN_LEFT - MARGIN_RIGHT).min(MAX_CONTENT_WIDTH);
    let margin_left = (vp_width - content_width) / 2;
    let scale = PxScale::from(20.0);
    let title_scale = PxScale::from(28.0);
    let line_height = 32i32;
    let x = margin_left as i32;
    let mut y = (PARAGRAPH_GAP + H1_EXTRA_MARGIN) as i32;

    draw_text_mut(
        &mut img,
        Rgb([100, 160, 255]),
        x,
        y,
        title_scale,
        &fonts.bold,
        title,
    );
    y += 50;

    let indent_x = x + BLOCK_INDENT as i32;
    for &(key, desc) in bindings {
        draw_text_mut(
            &mut img,
            Rgb([230, 180, 80]),
            indent_x,
            y,
            scale,
            &fonts.bold,
            key,
        );
        draw_text_mut(
            &mut img,
            Rgb([220, 220, 220]),
            indent_x + 220,
            y,
            scale,
            &fonts.regular,
            desc,
        );
        y += line_height;
    }

    y += 20;
    draw_text_mut(
        &mut img,
        Rgb([140, 140, 160]),
        indent_x,
        y,
        scale,
        &fonts.regular,
        "Press any key to dismiss",
    );

    img
}

pub(crate) fn render_search_bar(
    query: &str,
    match_info: Option<(usize, usize)>,
    width: u32,
    fonts: &Fonts,
) -> RgbImage {
    let bar_height = 30u32;
    let scale = PxScale::from(18.0);
    let mut bar = RgbImage::from_pixel(width, bar_height, Rgb([50, 50, 65]));

    // Draw top border
    for x in 0..width {
        bar.put_pixel(x, 0, Rgb([80, 80, 100]));
    }

    let display = format!("/{}", query);
    draw_text_mut(
        &mut bar,
        Rgb([220, 220, 220]),
        10,
        5,
        scale,
        &fonts.regular,
        &display,
    );

    if let Some((current, total)) = match_info {
        let info = format!("{}/{}", current, total);
        let (tw, _) = text_size(scale, &fonts.regular, &info);
        draw_text_mut(
            &mut bar,
            Rgb([180, 180, 180]),
            (width - tw - 10) as i32,
            5,
            scale,
            &fonts.regular,
            &info,
        );
    }

    bar
}
