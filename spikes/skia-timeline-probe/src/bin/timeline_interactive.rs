use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use skia_safe::{
    AlphaType, Color, ColorType, Contains, Font, FontMgr, FontStyle, ImageInfo, Paint, PaintStyle,
    Point, Rect, surfaces,
};
use std::time::Duration;

const HEADER: f32 = 220.0;
const RULER: f32 = 38.0;
const ROW: f32 = 42.0;

#[derive(Clone)]
struct Clip {
    id: usize,
    track: usize,
    start: f32,
    length: f32,
    label: &'static str,
    color: Color,
    selected: bool,
}

struct Drag {
    mouse_x: f32,
    starts: Vec<(usize, f32)>,
}

struct App {
    clips: Vec<Clip>,
    zoom: f32,
    scroll_x: f32,
    scroll_y: f32,
    drag: Option<Drag>,
    mouse_was_down: bool,
    font11: Font,
    font12: Font,
    font13: Font,
}

fn fill(c: &skia_safe::Canvas, rect: Rect, color: Color) {
    let mut p = Paint::default();
    p.set_color(color);
    c.draw_rect(rect, &p);
}

fn stroke(c: &skia_safe::Canvas, rect: Rect, color: Color, width: f32) {
    let mut p = Paint::default();
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(width);
    p.set_color(color);
    c.draw_rect(rect, &p);
}

fn text(c: &skia_safe::Canvas, s: &str, x: f32, y: f32, font: &Font, color: Color) {
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(color);
    c.draw_str(s, (x, y), font, &p);
}

fn waveform(c: &skia_safe::Canvas, rect: Rect, seed: f32) {
    if rect.width() < 18.0 {
        return;
    }
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_stroke_width(1.0);
    p.set_color(Color::from_argb(150, 20, 22, 23));
    let mid = rect.center_y() + 3.0;
    let n = ((rect.width() / 6.0) as usize).clamp(6, 100);
    let mut last = Point::new(rect.left + 3.0, mid);
    for i in 1..n {
        let t = i as f32 / (n - 1) as f32;
        let x = rect.left + 3.0 + t * (rect.width() - 6.0);
        let y = mid + ((i as f32 * 1.61 + seed).sin() + (i as f32 * 0.47).sin() * 0.4) * 7.0;
        let next = Point::new(x, y);
        c.draw_line(last, next, &p);
        last = next;
    }
}

impl App {
    fn new() -> Self {
        let palette = [
            Color::from_rgb(93, 143, 211),
            Color::from_rgb(69, 193, 151),
            Color::from_rgb(221, 91, 159),
            Color::from_rgb(234, 165, 43),
            Color::from_rgb(128, 139, 220),
            Color::from_rgb(75, 164, 198),
        ];
        let labels = [
            "Dialogue", "Ambience", "Title", "Kick", "Snare", "Hi Hat", "Bass", "Chords", "Lead",
            "Texture", "Camera", "Grade",
        ];
        let mut clips = Vec::new();
        let mut id = 0;
        for track in 0..12 {
            let count = if track < 3 {
                4
            } else if track < 8 {
                7
            } else {
                3
            };
            for i in 0..count {
                let start =
                    0.8 + i as f32 * (2.2 + (track % 3) as f32 * 0.35) + (track % 4) as f32 * 0.55;
                let length = if track == 5 {
                    2.0
                } else {
                    1.25 + ((i + track) % 4) as f32 * 0.55
                };
                clips.push(Clip {
                    id,
                    track,
                    start,
                    length,
                    label: labels[track],
                    color: palette[track % palette.len()],
                    selected: false,
                });
                id += 1;
            }
        }
        let face = FontMgr::default()
            .legacy_make_typeface(None, FontStyle::normal())
            .expect("system font");
        Self {
            clips,
            zoom: 72.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
            drag: None,
            mouse_was_down: false,
            font11: Font::new(face.clone(), 11.0),
            font12: Font::new(face.clone(), 12.0),
            font13: Font::new(face, 13.0),
        }
    }

    fn rect(&self, clip: &Clip) -> Rect {
        Rect::from_xywh(
            HEADER + (clip.start - self.scroll_x) * self.zoom,
            RULER + clip.track as f32 * ROW - self.scroll_y + 4.0,
            (clip.length * self.zoom - 3.0).max(2.0),
            ROW - 8.0,
        )
    }

    fn hit(&self, x: f32, y: f32) -> Option<usize> {
        self.clips
            .iter()
            .rev()
            .find(|clip| self.rect(clip).contains(Point::new(x, y)))
            .map(|clip| clip.id)
    }

    fn zoom_at(&mut self, x: f32, factor: f32) {
        let anchor_x = x.max(HEADER);
        let world = self.scroll_x + (anchor_x - HEADER) / self.zoom;
        self.zoom = (self.zoom * factor).clamp(22.0, 260.0);
        self.scroll_x = (world - (anchor_x - HEADER) / self.zoom).max(0.0);
    }

    fn begin_drag(&mut self, id: usize, mouse_x: f32, additive: bool) {
        if additive {
            if let Some(clip) = self.clips.iter_mut().find(|c| c.id == id) {
                clip.selected = !clip.selected;
            }
        } else if !self.clips.iter().any(|c| c.id == id && c.selected) {
            for clip in &mut self.clips {
                clip.selected = clip.id == id;
            }
        }
        let starts = self
            .clips
            .iter()
            .filter(|c| c.selected)
            .map(|c| (c.id, c.start))
            .collect();
        self.drag = Some(Drag { mouse_x, starts });
    }

    fn drag_to(&mut self, mouse_x: f32) {
        if let Some(drag) = &self.drag {
            let raw_delta = (mouse_x - drag.mouse_x) / self.zoom;
            let delta = (raw_delta * 4.0).round() / 4.0;
            for (id, start) in &drag.starts {
                if let Some(clip) = self.clips.iter_mut().find(|c| c.id == *id) {
                    clip.start = (start + delta).max(0.0);
                }
            }
        }
    }

    fn input(&mut self, window: &Window, width: usize, height: usize) -> bool {
        let mut dirty = false;
        let (mx, my) = window
            .get_mouse_pos(MouseMode::Discard)
            .unwrap_or((-1.0, -1.0));
        if let Some((wheel_x, wheel_y)) = window.get_scroll_wheel() {
            dirty = wheel_x != 0.0 || wheel_y != 0.0;
            let command = window.is_key_down(Key::LeftSuper)
                || window.is_key_down(Key::RightSuper)
                || window.is_key_down(Key::LeftCtrl)
                || window.is_key_down(Key::RightCtrl);
            let shift = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
            if command {
                self.zoom_at(mx, if wheel_y > 0.0 { 1.14 } else { 1.0 / 1.14 });
            } else if shift || wheel_x.abs() > wheel_y.abs() {
                self.scroll_x = (self.scroll_x - (wheel_x + wheel_y) * 1.2).max(0.0);
            } else {
                let max_y = (12.0 * ROW - (height as f32 - RULER)).max(0.0);
                self.scroll_y = (self.scroll_y - wheel_y * ROW * 1.2).clamp(0.0, max_y);
            }
        }

        if window.is_key_pressed(Key::Equal, KeyRepeat::Yes) {
            dirty = true;
            self.zoom_at((width as f32 + HEADER) * 0.5, 1.12);
        }
        if window.is_key_pressed(Key::Minus, KeyRepeat::Yes) {
            dirty = true;
            self.zoom_at((width as f32 + HEADER) * 0.5, 1.0 / 1.12);
        }
        if window.is_key_pressed(Key::Escape, KeyRepeat::No) {
            dirty = true;
            for clip in &mut self.clips {
                clip.selected = false;
            }
        }

        let down = window.get_mouse_down(MouseButton::Left);
        if down && !self.mouse_was_down && mx >= HEADER && my >= RULER {
            dirty = true;
            let shift = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
            if let Some(id) = self.hit(mx, my) {
                self.begin_drag(id, mx, shift);
            } else {
                if !shift {
                    for clip in &mut self.clips {
                        clip.selected = false;
                    }
                }
                self.drag = None;
            }
        }
        if down {
            dirty = true;
            self.drag_to(mx);
        } else if self.mouse_was_down {
            dirty = true;
            self.drag = None;
        }
        self.mouse_was_down = down;
        dirty
    }

    fn draw(&self, c: &skia_safe::Canvas, width: usize, height: usize) {
        c.clear(Color::from_rgb(51, 52, 50));
        fill(
            c,
            Rect::from_xywh(0.0, 0.0, width as f32, RULER),
            Color::from_rgb(43, 44, 42),
        );

        let first_beat = self.scroll_x.floor() as i32;
        let last_beat = (self.scroll_x + width as f32 / self.zoom).ceil() as i32 + 1;
        for beat in first_beat..=last_beat {
            let x = HEADER + (beat as f32 - self.scroll_x) * self.zoom;
            let major = beat % 4 == 0;
            let mut p = Paint::default();
            p.set_color(if major {
                Color::from_rgb(77, 78, 74)
            } else {
                Color::from_rgb(62, 63, 60)
            });
            c.draw_line(
                (x, if major { 20.0 } else { RULER }),
                (x, height as f32),
                &p,
            );
            if major {
                text(
                    c,
                    &format!("{}", beat / 4 + 1),
                    x + 5.0,
                    16.0,
                    &self.font11,
                    Color::from_rgb(183, 184, 180),
                );
            }
        }

        let names = [
            "1 Audio",
            "2 Ambience",
            "3 Titles",
            "4 Kick",
            "5 Snare",
            "6 Hi Hat",
            "7 Bass",
            "8 Chords",
            "9 Lead",
            "10 Texture",
            "11 Camera",
            "12 Grade",
        ];
        for (track, name) in names.iter().enumerate() {
            let y = RULER + track as f32 * ROW - self.scroll_y;
            if y + ROW < RULER || y > height as f32 {
                continue;
            }
            fill(
                c,
                Rect::from_xywh(0.0, y, width as f32, ROW),
                if track % 2 == 0 {
                    Color::from_argb(70, 255, 255, 255)
                } else {
                    Color::from_argb(42, 255, 255, 255)
                },
            );
            let mut p = Paint::default();
            p.set_color(Color::from_rgb(42, 43, 41));
            c.draw_line((0.0, y), (width as f32, y), &p);
            fill(
                c,
                Rect::from_xywh(0.0, y + 1.0, HEADER, ROW - 1.0),
                Color::from_rgb(66, 67, 64),
            );
            fill(
                c,
                Rect::from_xywh(0.0, y + 1.0, 5.0, ROW - 1.0),
                self.clips
                    .iter()
                    .find(|c| c.track == track)
                    .map(|c| c.color)
                    .unwrap_or(Color::GRAY),
            );
            text(
                c,
                "◉",
                14.0,
                y + 26.0,
                &self.font12,
                Color::from_rgb(188, 189, 185),
            );
            text(
                c,
                name,
                38.0,
                y + 26.0,
                &self.font13,
                Color::from_rgb(226, 227, 223),
            );
        }

        for clip in &self.clips {
            let rect = self.rect(clip);
            if rect.right < HEADER
                || rect.left > width as f32
                || rect.bottom < RULER
                || rect.top > height as f32
            {
                continue;
            }
            fill(c, rect, clip.color);
            waveform(c, rect, clip.id as f32 * 0.7);
            if rect.width() > 64.0 {
                text(
                    c,
                    clip.label,
                    rect.left + 7.0,
                    rect.top + 14.0,
                    &self.font11,
                    Color::WHITE,
                );
            }
            if clip.selected {
                stroke(c, rect, Color::from_rgb(255, 218, 82), 2.0);
            }
        }

        fill(
            c,
            Rect::from_xywh(0.0, 0.0, HEADER, RULER),
            Color::from_rgb(35, 36, 34),
        );
        text(
            c,
            "ARRANGEMENT",
            14.0,
            24.0,
            &self.font12,
            Color::from_rgb(204, 205, 201),
        );
        fill(
            c,
            Rect::from_xywh(0.0, height as f32 - 28.0, width as f32, 28.0),
            Color::from_rgb(35, 36, 34),
        );
        let selected = self.clips.iter().filter(|c| c.selected).count();
        text(
            c,
            &format!(
                "wheel: scroll   shift+wheel: horizontal   cmd/ctrl+wheel or +/-: zoom   shift+click: group select   drag: move 1/4 beat   |   zoom {:.0}%   selected {}",
                self.zoom / 72.0 * 100.0,
                selected
            ),
            12.0,
            height as f32 - 9.0,
            &self.font11,
            Color::from_rgb(190, 191, 187),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_zoom_select_and_group_drag_are_stable() {
        let mut app = App::new();
        let first = app.clips[0].clone();
        let r = app.rect(&first);
        assert_eq!(app.hit(r.center_x(), r.center_y()), Some(first.id));

        let anchor_x = 700.0;
        let world_before = app.scroll_x + (anchor_x - HEADER) / app.zoom;
        app.zoom_at(anchor_x, 1.5);
        let world_after = app.scroll_x + (anchor_x - HEADER) / app.zoom;
        assert!((world_before - world_after).abs() < 0.001);

        app.begin_drag(0, 500.0, false);
        app.begin_drag(1, 500.0, true);
        let starts: Vec<f32> = app.clips.iter().take(2).map(|c| c.start).collect();
        app.drag_to(500.0 + app.zoom * 1.5);
        assert!((app.clips[0].start - starts[0] - 1.5).abs() < 0.001);
        assert!((app.clips[1].start - starts[1] - 1.5).abs() < 0.001);
    }
}

fn main() {
    let mut window = Window::new(
        "Skia Timeline Interaction Probe",
        1400,
        820,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .expect("window");
    window.set_target_fps(60);
    let mut app = App::new();
    let mut pixels = Vec::<u32>::new();
    let mut last_size = (0, 0);
    let mut dirty = true;

    while window.is_open() && !window.is_key_down(Key::Q) {
        let (width, height) = window.get_size();
        if width == 0 || height == 0 {
            continue;
        }
        if (width, height) != last_size {
            pixels.resize(width * height, 0);
            last_size = (width, height);
            dirty = true;
        }
        dirty |= app.input(&window, width, height);
        if !dirty {
            window.update();
            continue;
        }
        let byte_len = pixels.len() * std::mem::size_of::<u32>();
        let bytes =
            unsafe { std::slice::from_raw_parts_mut(pixels.as_mut_ptr().cast::<u8>(), byte_len) };
        let info = ImageInfo::new(
            (width as i32, height as i32),
            ColorType::BGRA8888,
            AlphaType::Opaque,
            None,
        );
        let mut surface =
            surfaces::wrap_pixels(&info, bytes, Some(width * 4), None).expect("surface");
        app.draw(surface.canvas(), width, height);
        window
            .update_with_buffer(&pixels, width, height)
            .expect("present");
        dirty = false;
        std::thread::sleep(Duration::from_millis(1));
    }
}
