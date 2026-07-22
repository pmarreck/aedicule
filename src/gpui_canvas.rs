//! Replays Aedicule's immutable scene command buffer through GPUI.
//!
//! Guest code never reaches this module: it consumes a complete, host-validated
//! frame, which lets native and browser frontplanes share identical painting.

use gpui::{
    App, Bounds, Hsla, PathBuilder, SharedString, TextAlign, TextRun, Window, point, px, rgba,
};

use crate::{Affine, DrawCommand, FrameOutput, PathSegment};

#[derive(Clone, Copy)]
struct Matrix(Affine);

impl Matrix {
    const IDENTITY: Self = Self(Affine {
        m11: 1.0,
        m12: 0.0,
        m21: 0.0,
        m22: 1.0,
        tx: 0.0,
        ty: 0.0,
    });

    /// Composes parent and child affine transforms without leaking GPUI matrix
    /// types into the Aedicule ABI.
    fn then(self, next: Affine) -> Self {
        let parent = self.0;
        Self(Affine {
            m11: parent.m11 * next.m11 + parent.m21 * next.m12,
            m12: parent.m12 * next.m11 + parent.m22 * next.m12,
            m21: parent.m11 * next.m21 + parent.m21 * next.m22,
            m22: parent.m12 * next.m21 + parent.m22 * next.m22,
            tx: parent.m11 * next.tx + parent.m21 * next.ty + parent.tx,
            ty: parent.m12 * next.tx + parent.m22 * next.ty + parent.ty,
        })
    }

    /// Projects one logical plugin point into its current composed viewport.
    fn apply(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.0.m11 * x + self.0.m21 * y + self.0.tx,
            self.0.m12 * x + self.0.m22 * y + self.0.ty,
        )
    }
}

/// Adapts a validated immutable scene command buffer into GPUI paths and text;
/// no guest execution occurs while GPUI window/context borrows are live.
pub fn paint_frame(
    frame: &FrameOutput,
    bounds: Bounds<gpui::Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let scale = 1.0;
    let viewport = viewport_transform(bounds);
    let mut matrices = vec![Matrix(viewport)];

    for command in &frame.commands {
        match command {
            DrawCommand::PushTransform(transform) => {
                matrices.push(
                    matrices
                        .last()
                        .copied()
                        .unwrap_or(Matrix::IDENTITY)
                        .then(*transform),
                );
            }
            DrawCommand::PopTransform => {
                if matrices.len() > 1 {
                    matrices.pop();
                }
            }
            DrawCommand::Line {
                x1,
                y1,
                x2,
                y2,
                width,
                rgba: color,
                ..
            } => {
                let matrix = *matrices.last().unwrap();
                let (x1, y1) = matrix.apply(*x1, *y1);
                let (x2, y2) = matrix.apply(*x2, *y2);
                let mut path = PathBuilder::stroke(px(*width * scale));
                path.move_to(point(px(x1), px(y1)));
                path.line_to(point(px(x2), px(y2)));
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgba(*color));
                }
            }
            DrawCommand::Circle {
                x,
                y,
                radius,
                width,
                rgba: color,
                filled,
                ..
            } => {
                let matrix = *matrices.last().unwrap();
                let mut points = Vec::with_capacity(32);
                for index in 0..32 {
                    let angle = index as f32 * std::f32::consts::TAU / 32.0;
                    let (x, y) = matrix.apply(*x + radius * angle.cos(), *y + radius * angle.sin());
                    points.push(point(px(x), px(y)));
                }
                let mut path = if *filled {
                    PathBuilder::fill()
                } else {
                    PathBuilder::stroke(px(*width * scale))
                };
                path.add_polygon(&points, true);
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgba(*color));
                }
            }
            DrawCommand::Text {
                text,
                x,
                y,
                size,
                rgba: color,
                centered,
                ..
            } => {
                let matrix = *matrices.last().unwrap();
                let (x, y) = matrix.apply(*x, *y);
                let text = SharedString::from(text.clone());
                let run = TextRun {
                    len: text.len(),
                    font: window.text_style().font(),
                    color: Hsla::from(rgba(*color)),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let font_size = px(*size * scale);
                let line = window
                    .text_system()
                    .shape_line(text, font_size, &[run], None);
                let x = if *centered {
                    x - f32::from(line.width()) / 2.0
                } else {
                    x
                };
                let _ = line.paint(
                    point(px(x), px(y - f32::from(font_size) / 2.0)),
                    font_size,
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                );
            }
            DrawCommand::Path {
                segments,
                width,
                fill_rgba,
                stroke_rgba,
                ..
            } => {
                let matrix = *matrices.last().unwrap();
                if let Some(color) = fill_rgba {
                    paint_path(segments, matrix, PathBuilder::fill(), *color, window);
                }
                if let Some(color) = stroke_rgba {
                    paint_path(
                        segments,
                        matrix,
                        PathBuilder::stroke(px(*width * scale)),
                        *color,
                        window,
                    );
                }
            }
            DrawCommand::Sprite { destination, .. } => {
                let matrix = *matrices.last().unwrap();
                let corners = [
                    (destination.x, destination.y),
                    (destination.x + destination.width, destination.y),
                    (
                        destination.x + destination.width,
                        destination.y + destination.height,
                    ),
                    (destination.x, destination.y + destination.height),
                ];
                let points: Vec<_> = corners
                    .into_iter()
                    .map(|(x, y)| {
                        let (x, y) = matrix.apply(x, y);
                        point(px(x), px(y))
                    })
                    .collect();
                let mut path = PathBuilder::stroke(px(scale));
                path.add_polygon(&points, true);
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgba(0xff00ffff));
                }
            }
        }
    }
}

/// Maps guest coordinates directly into the complete canvas instead of
/// imposing a fixed-aspect logical viewport and letterboxing unused pixels.
pub fn viewport_transform(bounds: Bounds<gpui::Pixels>) -> Affine {
    Affine {
        m11: 1.0,
        m12: 0.0,
        m21: 0.0,
        m22: 1.0,
        tx: f32::from(bounds.origin.x),
        ty: f32::from(bounds.origin.y),
    }
}

/// Replays the generic quadratic/cubic path model through GPUI after applying
/// the current affine composition matrix.
fn paint_path(
    segments: &[PathSegment],
    matrix: Matrix,
    mut path: PathBuilder,
    color: u32,
    window: &mut Window,
) {
    for segment in segments {
        match segment {
            PathSegment::Move(point_) => {
                let (x, y) = matrix.apply(point_.x, point_.y);
                path.move_to(point(px(x), px(y)));
            }
            PathSegment::Line(point_) => {
                let (x, y) = matrix.apply(point_.x, point_.y);
                path.line_to(point(px(x), px(y)));
            }
            PathSegment::Quadratic { control, end } => {
                let (cx, cy) = matrix.apply(control.x, control.y);
                let (x, y) = matrix.apply(end.x, end.y);
                path.curve_to(point(px(x), px(y)), point(px(cx), px(cy)));
            }
            PathSegment::Cubic {
                control_1,
                control_2,
                end,
            } => {
                let (c1x, c1y) = matrix.apply(control_1.x, control_1.y);
                let (c2x, c2y) = matrix.apply(control_2.x, control_2.y);
                let (x, y) = matrix.apply(end.x, end.y);
                path.cubic_bezier_to(
                    point(px(x), px(y)),
                    point(px(c1x), px(c1y)),
                    point(px(c2x), px(c2y)),
                );
            }
            PathSegment::Close => path.close(),
        }
    }
    if let Ok(path) = path.build() {
        window.paint_path(path, rgba(color));
    }
}
