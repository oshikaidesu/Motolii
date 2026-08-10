use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, Point, Rect, surfaces,
};

const W: i32 = 1600;
const H: i32 = 900;
const HEADER: f32 = 0.0;
const TRACK_X: f32 = 1380.0;
const RULER: f32 = 55.0;
const ROW: f32 = 40.0;

fn fill(canvas: &skia_safe::Canvas, rect: Rect, color: Color) {
    let mut p = Paint::default();
    p.set_color(color);
    canvas.draw_rect(rect, &p);
}

fn line(canvas: &skia_safe::Canvas, a: Point, b: Point, color: Color, width: f32) {
    let mut p = Paint::default();
    p.set_color(color);
    p.set_stroke_width(width);
    canvas.draw_line(a, b, &p);
}

fn text(canvas: &skia_safe::Canvas, value: &str, x: f32, y: f32, size: f32, color: Color) {
    let typeface = FontMgr::default()
        .legacy_make_typeface(None, FontStyle::normal())
        .expect("system typeface");
    let font = Font::new(typeface, size);
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(color);
    canvas.draw_str(value, (x, y), &font, &p);
}

fn waveform(canvas: &skia_safe::Canvas, rect: Rect, seed: f32, color: Color) {
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(color);
    p.set_stroke_width(1.0);
    let mid = rect.center_y();
    let points = ((rect.width() / 5.0) as usize).clamp(8, 120);
    let mut last = Point::new(rect.left + 4.0, mid);
    for i in 1..points {
        let t = i as f32 / (points - 1) as f32;
        let x = rect.left + 4.0 + t * (rect.width() - 8.0);
        let envelope = (t * std::f32::consts::PI).sin().max(0.18);
        let amp = ((i as f32 * 1.73 + seed).sin() * 0.65
            + (i as f32 * 0.51 + seed * 2.0).sin() * 0.35)
            * rect.height()
            * 0.34
            * envelope;
        let next = Point::new(x, mid + amp);
        canvas.draw_line(last, next, &p);
        last = next;
    }
}

fn clip(
    canvas: &skia_safe::Canvas,
    x: f32,
    row: usize,
    width: f32,
    label: &str,
    color: Color,
    seed: f32,
    selected: bool,
) {
    let y = RULER + row as f32 * ROW + 4.0;
    let rect = Rect::from_xywh(HEADER + x, y, width, ROW - 8.0);
    fill(canvas, rect, color);
    fill(
        canvas,
        Rect::from_xywh(rect.left, rect.top, 4.0, rect.height()),
        if selected {
            Color::from_rgb(240, 197, 80)
        } else {
            Color::from_argb(100, 255, 255, 255)
        },
    );
    waveform(canvas, rect, seed, Color::from_argb(155, 12, 15, 18));
    if width > 70.0 {
        text(
            canvas,
            label,
            rect.left + 9.0,
            rect.top + 14.0,
            12.0,
            Color::from_rgb(236, 238, 240),
        );
    }
    if selected {
        let mut p = Paint::default();
        p.set_style(PaintStyle::Stroke);
        p.set_stroke_width(2.0);
        p.set_color(Color::from_rgb(240, 197, 80));
        canvas.draw_rect(rect, &p);
    }
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "timeline-skia-probe.png".into());
    let mut surface = surfaces::raster_n32_premul((W, H)).expect("surface");
    let c = surface.canvas();
    c.clear(Color::from_rgb(49, 50, 48));

    // Ableton-like arrangement ruler.
    fill(
        c,
        Rect::from_xywh(0.0, 0.0, W as f32, RULER),
        Color::from_rgb(53, 54, 52),
    );

    for beat in 0..=32 {
        let x = HEADER + beat as f32 * 42.0;
        let major = beat % 4 == 0;
        line(
            c,
            Point::new(x, if major { 24.0 } else { 38.0 }),
            Point::new(x, H as f32),
            if major {
                Color::from_rgb(70, 71, 68)
            } else {
                Color::from_rgb(59, 60, 58)
            },
            1.0,
        );
        if major {
            text(
                c,
                &format!("{}.1", beat / 4 + 1),
                x + 5.0,
                19.0,
                11.0,
                Color::from_rgb(178, 179, 175),
            );
        }
    }

    let names = [
        "1 Audio",
        "2 City Ambience",
        "3 Titles",
        "4 Group · Pulse",
        "5 Kick 909",
        "6 Snare",
        "7 10. Hi Hat",
        "8 Bass",
        "9 Chords",
        "10 Lead",
        "11 Texture",
        "12 Camera",
        "13 Grade",
        "A Reverb",
        "Main",
    ];
    let accents = [
        Color::from_rgb(126, 166, 226),
        Color::from_rgb(78, 211, 163),
        Color::from_rgb(232, 104, 170),
        Color::from_rgb(239, 182, 50),
    ];

    for (row, name) in names.iter().enumerate() {
        let y = RULER + row as f32 * ROW;
        fill(
            c,
            Rect::from_xywh(0.0, y, TRACK_X, ROW),
            if row % 2 == 0 {
                Color::from_rgb(52, 53, 51)
            } else {
                Color::from_rgb(55, 56, 54)
            },
        );
        line(
            c,
            Point::new(0.0, y),
            Point::new(W as f32, y),
            Color::from_rgb(42, 43, 41),
            1.0,
        );
        let track_color = accents[row % accents.len()];
        fill(
            c,
            Rect::from_xywh(TRACK_X, y + 1.0, W as f32 - TRACK_X, ROW - 1.0),
            track_color,
        );
        text(
            c,
            if row == 3 { "⊖" } else { "◉" },
            TRACK_X + 9.0,
            y + 25.0,
            13.0,
            Color::from_rgb(37, 38, 36),
        );
        text(
            c,
            name,
            TRACK_X + 31.0,
            y + 25.0,
            13.0,
            Color::from_rgb(35, 36, 34),
        );
    }

    // Arrangement grid stays visible across the track lanes.
    for beat in 0..=32 {
        let x = beat as f32 * 42.0;
        let major = beat % 4 == 0;
        line(
            c,
            Point::new(x, RULER),
            Point::new(x, H as f32 - 34.0),
            if major {
                Color::from_argb(155, 70, 71, 68)
            } else {
                Color::from_argb(115, 62, 63, 60)
            },
            1.0,
        );
    }
    fill(
        c,
        Rect::from_xywh(TRACK_X, 0.0, W as f32 - TRACK_X, RULER),
        Color::from_rgb(43, 44, 42),
    );
    text(
        c,
        "TRACKS",
        TRACK_X + 16.0,
        34.0,
        12.0,
        Color::from_rgb(184, 185, 181),
    );

    // A restrained, realistic editing density: around 50 detailed clips.
    clip(
        c,
        24.0,
        0,
        310.0,
        "night_drive_take_04",
        Color::from_rgb(87, 132, 201),
        1.0,
        false,
    );
    clip(
        c,
        345.0,
        0,
        260.0,
        "city_signal",
        Color::from_rgb(87, 132, 201),
        2.0,
        false,
    );
    clip(
        c,
        630.0,
        0,
        405.0,
        "neon reflections",
        Color::from_rgb(87, 132, 201),
        3.0,
        false,
    );
    clip(
        c,
        45.0,
        1,
        535.0,
        "Baltimore ambience",
        Color::from_rgb(71, 190, 150),
        4.0,
        false,
    );
    clip(
        c,
        665.0,
        1,
        290.0,
        "rain tail",
        Color::from_rgb(71, 190, 150),
        5.0,
        false,
    );
    clip(
        c,
        175.0,
        2,
        250.0,
        "NIGHT / DRIVE",
        Color::from_rgb(214, 92, 158),
        6.0,
        true,
    );
    clip(
        c,
        440.0,
        2,
        315.0,
        "CITY SIGNAL",
        Color::from_rgb(214, 92, 158),
        7.0,
        false,
    );

    for i in 0..12 {
        clip(
            c,
            80.0 + i as f32 * 76.0,
            4,
            62.0,
            "",
            Color::from_rgb(226, 155, 36),
            i as f32,
            false,
        );
    }
    for i in 0..9 {
        clip(
            c,
            112.0 + i as f32 * 104.0,
            5,
            88.0,
            "SNARE",
            Color::from_rgb(225, 76, 160),
            i as f32 + 3.0,
            false,
        );
    }
    clip(
        c,
        80.0,
        6,
        960.0,
        "Hi Hat · 1/16",
        Color::from_rgb(124, 139, 220),
        8.0,
        false,
    );
    for i in 0..7 {
        clip(
            c,
            185.0 + i as f32 * 126.0,
            7,
            106.0,
            "BASS",
            Color::from_rgb(62, 154, 197),
            i as f32 + 4.0,
            false,
        );
    }
    clip(
        c,
        120.0,
        8,
        460.0,
        "Cmaj7 / Am9",
        Color::from_rgb(116, 91, 157),
        9.0,
        false,
    );
    clip(
        c,
        595.0,
        8,
        360.0,
        "Fmaj7",
        Color::from_rgb(116, 91, 157),
        10.0,
        false,
    );
    for i in 0..8 {
        clip(
            c,
            290.0 + i as f32 * 83.0,
            9,
            68.0,
            "",
            Color::from_rgb(77, 133, 165),
            i as f32 + 7.0,
            false,
        );
    }
    clip(
        c,
        230.0,
        10,
        270.0,
        "VHS texture",
        Color::from_rgb(117, 105, 137),
        12.0,
        false,
    );
    clip(
        c,
        515.0,
        10,
        520.0,
        "dust / bloom",
        Color::from_rgb(117, 105, 137),
        13.0,
        false,
    );

    // Group band and playhead.
    let gy = RULER + 3.0 * ROW;
    fill(
        c,
        Rect::from_xywh(HEADER, gy + 4.0, TRACK_X, ROW - 8.0),
        Color::from_argb(55, 239, 182, 50),
    );
    line(
        c,
        Point::new(HEADER + 482.0, 0.0),
        Point::new(HEADER + 482.0, H as f32),
        Color::from_rgb(244, 87, 99),
        2.0,
    );
    fill(
        c,
        Rect::from_xywh(HEADER + 476.0, 0.0, 12.0, 8.0),
        Color::from_rgb(244, 87, 99),
    );

    // Bottom status strip.
    fill(
        c,
        Rect::from_xywh(0.0, H as f32 - 34.0, W as f32, 34.0),
        Color::from_rgb(38, 39, 37),
    );
    line(
        c,
        Point::new(0.0, H as f32 - 34.0),
        Point::new(W as f32, H as f32 - 34.0),
        Color::from_rgb(62, 65, 70),
        1.0,
    );
    text(
        c,
        "Arrangement     1/4",
        18.0,
        H as f32 - 12.0,
        11.0,
        Color::from_rgb(176, 178, 173),
    );
    text(
        c,
        "48 kHz",
        W as f32 - 96.0,
        H as f32 - 12.0,
        11.0,
        Color::from_rgb(176, 178, 173),
    );

    let png = surface
        .image_snapshot()
        .encode(None, EncodedImageFormat::PNG, 100)
        .expect("png encode");
    std::fs::write(&output, png.as_bytes()).expect("write png");
    println!("{output}");
}
