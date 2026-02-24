use egui::{Color32, Pos2, Rect, Vec2};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

pub type AnnotationId = u64;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Tool {
    Select,
    Arrow,
    ArrowWithText,
    Text,
    Rectangle,
    Ellipse,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum StrokeWidth {
    Thin,
    Medium,
    Thick,
}

impl StrokeWidth {
    pub fn px(self) -> f32 {
        match self {
            Self::Thin => 1.5,
            Self::Medium => 3.0,
            Self::Thick => 5.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextSize(u8);

impl TextSize {
    pub const MIN: u8 = 8;
    pub const MAX: u8 = 32;
    pub const S: Self = Self(14);
    pub const M: Self = Self(18);
    pub const L: Self = Self(24);

    pub fn from_points(points: u8) -> Self {
        Self(points.clamp(Self::MIN, Self::MAX))
    }

    pub fn as_u8(self) -> u8 {
        self.0
    }

    pub fn points(self) -> f32 {
        self.0 as f32
    }
}

impl Serialize for TextSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.0)
    }
}

impl<'de> Deserialize<'de> for TextSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TextSizeVisitor;

        impl<'de> Visitor<'de> for TextSizeVisitor {
            type Value = TextSize;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("font size as number 8..32 or legacy values S/M/L")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(TextSize::from_points(value.min(TextSize::MAX as u64) as u8))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let numeric = value.max(TextSize::MIN as i64).min(TextSize::MAX as i64) as u8;
                Ok(TextSize::from_points(numeric))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "S" | "s" => Ok(TextSize::S),
                    "M" | "m" => Ok(TextSize::M),
                    "L" | "l" => Ok(TextSize::L),
                    other => {
                        let parsed = other.parse::<u8>().map_err(|_| {
                            E::custom(format!(
                                "invalid text size '{other}', expected S/M/L or number"
                            ))
                        })?;
                        Ok(TextSize::from_points(parsed))
                    }
                }
            }
        }

        deserializer.deserialize_any(TextSizeVisitor)
    }
}

fn default_text_size() -> TextSize {
    TextSize::M
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn to_pos2(self) -> Pos2 {
        Pos2::new(self.x, self.y)
    }

    pub fn from_pos2(value: Pos2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }

    pub fn delta(self, other: Point) -> Vec2 {
        Vec2::new(other.x - self.x, other.y - self.y)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct RectData {
    pub min: Point,
    pub max: Point,
}

impl RectData {
    pub fn normalize(self) -> Self {
        let min_x = self.min.x.min(self.max.x);
        let min_y = self.min.y.min(self.max.y);
        let max_x = self.min.x.max(self.max.x);
        let max_y = self.min.y.max(self.max.y);
        Self {
            min: Point { x: min_x, y: min_y },
            max: Point { x: max_x, y: max_y },
        }
    }

    pub fn to_rect(self) -> Rect {
        let norm = self.normalize();
        Rect::from_min_max(norm.min.to_pos2(), norm.max.to_pos2())
    }

    pub fn from_rect(value: Rect) -> Self {
        Self {
            min: Point::from_pos2(value.min),
            max: Point::from_pos2(value.max),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Handle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    ArrowFrom,
    ArrowTo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Annotation {
    pub id: AnnotationId,
    pub kind: AnnotationKind,
    pub color: [u8; 4],
    pub stroke_width: StrokeWidth,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnnotationKind {
    Arrow {
        from: Point,
        to: Point,
    },
    ArrowWithText {
        from: Point,
        to: Point,
        text: String,
        #[serde(default = "default_text_size")]
        size: TextSize,
    },
    Text {
        pos: Point,
        content: String,
        size: TextSize,
    },
    Rectangle {
        rect: RectData,
    },
    Ellipse {
        rect: RectData,
    },
}

impl Annotation {
    pub fn color32(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(self.color[0], self.color[1], self.color[2], self.color[3])
    }

    pub fn bounds(&self) -> Rect {
        match &self.kind {
            AnnotationKind::Arrow { from, to } => {
                Rect::from_two_pos(from.to_pos2(), to.to_pos2()).expand(8.0)
            }
            AnnotationKind::ArrowWithText {
                from,
                to,
                text,
                size,
            } => {
                let arrow = Rect::from_two_pos(from.to_pos2(), to.to_pos2()).expand(8.0);
                let text_w = (text.chars().count().max(1) as f32 * size.points() * 0.55).max(16.0);
                let text_h = size.points() * 1.3;
                let text_anchor = Point::new(from.x + 8.0, from.y - size.points() * 0.8);
                let text_rect =
                    Rect::from_min_size(text_anchor.to_pos2(), Vec2::new(text_w, text_h))
                        .expand(4.0);
                arrow.union(text_rect)
            }
            AnnotationKind::Text { pos, content, size } => {
                // Conservative estimate for hit-testing and selection boxes.
                let width = (content.chars().count().max(1) as f32 * size.points() * 0.6).max(20.0);
                let height = size.points() * 1.4;
                Rect::from_min_size(pos.to_pos2(), Vec2::new(width, height))
            }
            AnnotationKind::Rectangle { rect } | AnnotationKind::Ellipse { rect } => {
                rect.to_rect().expand(4.0)
            }
        }
    }

    pub fn contains(&self, point: Point, tolerance: f32) -> bool {
        let p = point.to_pos2();
        match &self.kind {
            AnnotationKind::Arrow { from, to } | AnnotationKind::ArrowWithText { from, to, .. } => {
                distance_to_segment(p, from.to_pos2(), to.to_pos2())
                    <= tolerance + self.stroke_width.px()
            }
            AnnotationKind::Text { .. } => self.bounds().expand(tolerance).contains(p),
            AnnotationKind::Rectangle { rect } => {
                let r = rect.to_rect();
                let expanded = r.expand(tolerance + self.stroke_width.px());
                if !expanded.contains(p) {
                    return false;
                }
                let inner = r.shrink((self.stroke_width.px() + tolerance).max(1.0));
                !inner.contains(p)
            }
            AnnotationKind::Ellipse { rect } => {
                let r = rect.to_rect();
                let center = r.center();
                let radii = r.size() * 0.5;
                if radii.x <= 0.1 || radii.y <= 0.1 {
                    return false;
                }
                let nx = (p.x - center.x) / radii.x;
                let ny = (p.y - center.y) / radii.y;
                let d = nx * nx + ny * ny;
                let ring = (self.stroke_width.px() + tolerance) / radii.x.min(radii.y).max(1.0);
                (1.0 - ring).powi(2) <= d && d <= (1.0 + ring).powi(2)
            }
        }
    }

    pub fn move_by(&mut self, delta: Vec2) {
        let move_point = |p: &mut Point| {
            p.x += delta.x;
            p.y += delta.y;
        };
        match &mut self.kind {
            AnnotationKind::Arrow { from, to } | AnnotationKind::ArrowWithText { from, to, .. } => {
                move_point(from);
                move_point(to);
            }
            AnnotationKind::Text { pos, .. } => move_point(pos),
            AnnotationKind::Rectangle { rect } | AnnotationKind::Ellipse { rect } => {
                move_point(&mut rect.min);
                move_point(&mut rect.max);
            }
        }
    }

    pub fn handles(&self) -> Vec<(Handle, Point)> {
        match &self.kind {
            AnnotationKind::Arrow { from, to } | AnnotationKind::ArrowWithText { from, to, .. } => {
                vec![(Handle::ArrowFrom, *from), (Handle::ArrowTo, *to)]
            }
            AnnotationKind::Text { .. } => vec![],
            AnnotationKind::Rectangle { rect } | AnnotationKind::Ellipse { rect } => {
                let r = rect.to_rect();
                let c = r.center();
                vec![
                    (Handle::TopLeft, Point::from_pos2(r.left_top())),
                    (Handle::Top, Point::new(c.x, r.top())),
                    (Handle::TopRight, Point::from_pos2(r.right_top())),
                    (Handle::Right, Point::new(r.right(), c.y)),
                    (Handle::BottomRight, Point::from_pos2(r.right_bottom())),
                    (Handle::Bottom, Point::new(c.x, r.bottom())),
                    (Handle::BottomLeft, Point::from_pos2(r.left_bottom())),
                    (Handle::Left, Point::new(r.left(), c.y)),
                ]
            }
        }
    }

    pub fn resize_from_handle(&mut self, handle: Handle, to: Point, keep_square: bool) {
        match &mut self.kind {
            AnnotationKind::Arrow { from, to: target }
            | AnnotationKind::ArrowWithText {
                from, to: target, ..
            } => match handle {
                Handle::ArrowFrom => *from = to,
                Handle::ArrowTo => *target = to,
                _ => {}
            },
            AnnotationKind::Rectangle { rect } | AnnotationKind::Ellipse { rect } => {
                let mut r = rect.to_rect();
                match handle {
                    Handle::TopLeft => r.min = to.to_pos2(),
                    Handle::Top => r.min.y = to.y,
                    Handle::TopRight => {
                        r.min.y = to.y;
                        r.max.x = to.x;
                    }
                    Handle::Right => r.max.x = to.x,
                    Handle::BottomRight => r.max = to.to_pos2(),
                    Handle::Bottom => r.max.y = to.y,
                    Handle::BottomLeft => {
                        r.min.x = to.x;
                        r.max.y = to.y;
                    }
                    Handle::Left => r.min.x = to.x,
                    _ => {}
                }

                if keep_square {
                    let mut size = r.size();
                    let side = size.x.abs().max(size.y.abs());
                    size.x = side * size.x.signum().max(1.0);
                    size.y = side * size.y.signum().max(1.0);
                    r.max = r.min + size;
                }

                *rect = RectData::from_rect(r).normalize();
            }
            AnnotationKind::Text { .. } => {}
        }
    }
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

fn distance_to_segment(point: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let ap = point - a;
    let ab_len_sq = ab.length_sq();
    if ab_len_sq <= f32::EPSILON {
        return ap.length();
    }
    let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let projection = a + ab * t;
    (point - projection).length()
}

#[cfg(test)]
mod tests {
    use super::{Annotation, AnnotationKind, Point, RectData, StrokeWidth, TextSize};

    #[test]
    fn move_rectangle_changes_bounds() {
        let mut annotation = Annotation {
            id: 1,
            kind: AnnotationKind::Rectangle {
                rect: RectData {
                    min: Point::new(10.0, 10.0),
                    max: Point::new(20.0, 20.0),
                },
            },
            color: [0, 0, 0, 255],
            stroke_width: StrokeWidth::Medium,
        };

        annotation.move_by(egui::Vec2::new(5.0, -3.0));
        let bounds = annotation.bounds();
        assert_eq!(bounds.min.x, 11.0);
        assert_eq!(bounds.min.y, 3.0);
    }

    #[test]
    fn hit_test_arrow_line() {
        let annotation = Annotation {
            id: 1,
            kind: AnnotationKind::Arrow {
                from: Point::new(0.0, 0.0),
                to: Point::new(100.0, 0.0),
            },
            color: [0, 0, 0, 255],
            stroke_width: StrokeWidth::Medium,
        };

        assert!(annotation.contains(Point::new(50.0, 1.0), 2.0));
        assert!(!annotation.contains(Point::new(50.0, 20.0), 2.0));
    }

    #[test]
    fn text_size_deserializes_legacy_and_numeric() {
        let legacy: TextSize = serde_json::from_str("\"M\"").expect("legacy text size");
        assert_eq!(legacy, TextSize::M);

        let numeric: TextSize = serde_json::from_str("10").expect("numeric text size");
        assert_eq!(numeric.as_u8(), 10);

        let clamped: TextSize = serde_json::from_str("100").expect("clamped text size");
        assert_eq!(clamped.as_u8(), TextSize::MAX);
    }
}
