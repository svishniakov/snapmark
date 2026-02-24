use ab_glyph::FontArc;
use anyhow::{anyhow, Context, Result};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

use crate::annotation::{Annotation, AnnotationKind, Point};

pub fn flatten(
    image: &DynamicImage,
    annotations: &[Annotation],
    scale: f32,
) -> Result<DynamicImage> {
    let mut pixmap = Pixmap::new(image.width(), image.height())
        .ok_or_else(|| anyhow!("cannot allocate pixmap"))?;

    copy_image_to_pixmap(image, &mut pixmap)?;

    for annotation in annotations {
        draw_annotation_shape(&mut pixmap, annotation, scale)?;
    }

    let mut output = RgbaImage::from_raw(image.width(), image.height(), pixmap.data().to_vec())
        .ok_or_else(|| anyhow!("cannot construct output image"))?;

    draw_text_annotations(&mut output, annotations, scale);

    Ok(DynamicImage::ImageRgba8(output))
}

pub fn encode_png(image: &DynamicImage) -> Result<Vec<u8>> {
    let mut buffer = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, ImageFormat::Png)
        .context("cannot encode PNG")?;
    Ok(buffer.into_inner())
}

fn copy_image_to_pixmap(image: &DynamicImage, pixmap: &mut Pixmap) -> Result<()> {
    let rgba = image.to_rgba8();
    let data = pixmap.data_mut();
    if data.len() != rgba.len() {
        return Err(anyhow!("source image and pixmap size mismatch"));
    }
    data.copy_from_slice(rgba.as_raw());
    Ok(())
}

fn draw_annotation_shape(pixmap: &mut Pixmap, annotation: &Annotation, scale: f32) -> Result<()> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(
        annotation.color[0],
        annotation.color[1],
        annotation.color[2],
        annotation.color[3],
    );
    paint.anti_alias = true;

    let stroke = Stroke {
        width: annotation.stroke_width.px() * scale,
        ..Default::default()
    };

    match &annotation.kind {
        AnnotationKind::Arrow { from, to } | AnnotationKind::ArrowWithText { from, to, .. } => {
            stroke_line(pixmap, *from, *to, &paint, &stroke, scale)?;
            fill_arrow_head(pixmap, *from, *to, &paint, scale)?;
        }
        AnnotationKind::Rectangle { rect } => {
            let rect = rect.normalize();
            let tiny_rect = Rect::from_ltrb(
                rect.min.x * scale,
                rect.min.y * scale,
                rect.max.x * scale,
                rect.max.y * scale,
            )
            .ok_or_else(|| anyhow!("invalid rectangle"))?;
            let path = PathBuilder::from_rect(tiny_rect);
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
        AnnotationKind::Ellipse { rect } => {
            let rect = rect.normalize().to_rect();
            let center = rect.center();
            let rx = (rect.width() * 0.5 * scale).max(1.0);
            let ry = (rect.height() * 0.5 * scale).max(1.0);
            let mut pb = PathBuilder::new();
            pb.push_circle(0.0, 0.0, 1.0);
            let path = pb
                .finish()
                .ok_or_else(|| anyhow!("cannot build ellipse path"))?;
            let transform =
                Transform::from_scale(rx, ry).post_translate(center.x * scale, center.y * scale);
            pixmap.stroke_path(&path, &paint, &stroke, transform, None);
        }
        AnnotationKind::Text { .. } => {
            // Text is rendered in a separate pass via imageproc.
        }
    }

    Ok(())
}

fn stroke_line(
    pixmap: &mut Pixmap,
    from: Point,
    to: Point,
    paint: &Paint,
    stroke: &Stroke,
    scale: f32,
) -> Result<()> {
    let mut pb = PathBuilder::new();
    pb.move_to(from.x * scale, from.y * scale);
    pb.line_to(to.x * scale, to.y * scale);
    let path = pb.finish().ok_or_else(|| anyhow!("cannot build line"))?;
    pixmap.stroke_path(&path, paint, stroke, Transform::identity(), None);
    Ok(())
}

fn fill_arrow_head(
    pixmap: &mut Pixmap,
    from: Point,
    to: Point,
    paint: &Paint,
    scale: f32,
) -> Result<()> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let length = (dx * dx + dy * dy).sqrt().max(1.0);
    let ux = dx / length;
    let uy = dy / length;
    let head_len = 14.0 * scale;
    let head_w = 9.0 * scale;

    let tip_x = to.x * scale;
    let tip_y = to.y * scale;
    let base_x = tip_x - ux * head_len;
    let base_y = tip_y - uy * head_len;
    let left_x = base_x + -uy * head_w;
    let left_y = base_y + ux * head_w;
    let right_x = base_x - -uy * head_w;
    let right_y = base_y - ux * head_w;

    let mut pb = PathBuilder::new();
    pb.move_to(tip_x, tip_y);
    pb.line_to(left_x, left_y);
    pb.line_to(right_x, right_y);
    pb.close();
    let path = pb
        .finish()
        .ok_or_else(|| anyhow!("cannot build arrow head path"))?;
    pixmap.fill_path(&path, paint, FillRule::Winding, Transform::identity(), None);
    Ok(())
}

fn draw_text_annotations(image: &mut RgbaImage, annotations: &[Annotation], scale: f32) {
    let Some(font) = load_system_font() else {
        return;
    };

    for annotation in annotations {
        match &annotation.kind {
            AnnotationKind::Text { pos, content, size } => {
                draw_text_mut(
                    image,
                    Rgba(annotation.color),
                    (pos.x * scale) as i32,
                    (pos.y * scale) as i32,
                    size.points() * scale,
                    &font,
                    content,
                );
            }
            AnnotationKind::ArrowWithText {
                from,
                to,
                text,
                size,
            } => {
                let anchor = arrow_text_anchor(*from, *to);
                draw_text_mut(
                    image,
                    Rgba(annotation.color),
                    (anchor.x * scale) as i32,
                    (anchor.y * scale) as i32,
                    size.points() * scale,
                    &font,
                    text,
                );
            }
            _ => {}
        }
    }
}

fn arrow_text_anchor(from: Point, to: Point) -> Point {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let ux = dx / len;
    let uy = dy / len;
    let perp_a_x = -uy;
    let perp_a_y = ux;
    let perp_b_x = uy;
    let perp_b_y = -ux;
    let (perp_x, perp_y) = if perp_a_y < perp_b_y {
        (perp_a_x, perp_a_y)
    } else {
        (perp_b_x, perp_b_y)
    };
    Point {
        x: from.x + perp_x * 12.0 + 6.0,
        y: from.y + perp_y * 12.0 - 2.0,
    }
}

fn load_system_font() -> Option<FontArc> {
    let candidates = [
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/System/Library/Fonts/SFNS.ttf",
        "/System/Library/Fonts/Supplemental/Helvetica.ttf",
    ];

    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(font) = FontArc::try_from_vec(bytes) {
                return Some(font);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, RgbaImage};

    use super::flatten;
    use crate::annotation::{Annotation, AnnotationKind, Point, RectData, StrokeWidth};

    #[test]
    fn flatten_keeps_image_size() {
        let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            320,
            200,
            image::Rgba([255, 255, 255, 255]),
        ));
        let annotations = vec![Annotation {
            id: 1,
            kind: AnnotationKind::Rectangle {
                rect: RectData {
                    min: Point { x: 8.0, y: 8.0 },
                    max: Point { x: 120.0, y: 80.0 },
                },
            },
            color: [229, 62, 62, 255],
            stroke_width: StrokeWidth::Medium,
        }];

        let result = flatten(&image, &annotations, 1.0).expect("flatten should succeed");
        assert_eq!(result.width(), 320);
        assert_eq!(result.height(), 200);
    }
}
