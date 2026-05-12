use lnmai_ffi::{CoreBridge, FrameInput, HostFrame, MockBridge, BUTTON_ZONE_COUNT, SENSOR_AREA_COUNT};
use macroquad::prelude::*;
use std::path::{Path, PathBuf};

mod sensor_layout;

const BUTTON_KEYS: [KeyCode; BUTTON_ZONE_COUNT] = [
    KeyCode::A,
    KeyCode::S,
    KeyCode::D,
    KeyCode::F,
    KeyCode::J,
    KeyCode::K,
    KeyCode::L,
    KeyCode::Semicolon,
];

const SENSOR_KEYS: [KeyCode; SENSOR_AREA_COUNT] = [
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Key5,
    KeyCode::Key6,
    KeyCode::Key7,
    KeyCode::Key8,
    KeyCode::Key9,
    KeyCode::Key0,
    KeyCode::Q,
    KeyCode::W,
    KeyCode::E,
    KeyCode::R,
    KeyCode::T,
    KeyCode::Y,
    KeyCode::U,
    KeyCode::I,
    KeyCode::O,
    KeyCode::P,
    KeyCode::Z,
    KeyCode::X,
    KeyCode::C,
    KeyCode::V,
    KeyCode::B,
    KeyCode::N,
    KeyCode::M,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Space,
    KeyCode::LeftControl,
];

const SENSOR_LABELS: [&str; SENSOR_AREA_COUNT] = [
    "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8",
    "B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8",
    "C",
    "D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8",
    "E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8",
];

#[derive(Clone, Copy)]
struct InputState {
    buttons: [bool; BUTTON_ZONE_COUNT],
    sensors: [bool; SENSOR_AREA_COUNT],
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            buttons: [false; BUTTON_ZONE_COUNT],
            sensors: [false; SENSOR_AREA_COUNT],
        }
    }
}

struct HostApp {
    bridge: MockBridge,
    chart_path: PathBuf,
    last_frame_time: f64,
    last_result: HostFrame,
}

impl HostApp {
    fn new(chart_path: PathBuf) -> Self {
        Self {
            bridge: MockBridge::default(),
            chart_path,
            last_frame_time: get_time(),
            last_result: HostFrame::default(),
        }
    }

    fn load(&mut self) {
        let _ = self.bridge.load_chart(Path::new(&self.chart_path));
    }

    fn step(&mut self, input: InputState) {
        let now = get_time();
        let delta_sec = (now - self.last_frame_time).max(0.0) as f32;
        self.last_frame_time = now;
        let frame_input = FrameInput {
            button_held: input.buttons.to_vec(),
            sensor_held: input.sensors.to_vec(),
            delta_sec,
        };
        self.last_result = self.bridge.step(frame_input).unwrap_or_default();
    }
}

#[derive(Clone)]
struct SensorRegion {
    index: usize,
    label: &'static str,
    polygons: Vec<Vec<Vec2>>,
}

fn parse_args() -> Mode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("test") => Mode::SensorTest,
        Some(path) => Mode::Play(PathBuf::from(path)),
        None => Mode::Usage,
    }
}

enum Mode {
    Usage,
    Play(PathBuf),
    SensorTest,
}

fn read_input_state() -> InputState {
    let mut state = InputState::default();
    for (index, key) in BUTTON_KEYS.iter().enumerate() {
        state.buttons[index] = is_key_down(*key);
    }
    for (index, key) in SENSOR_KEYS.iter().enumerate() {
        state.sensors[index] = is_key_down(*key);
    }
    state
}

fn draw_backdrop() {
    clear_background(WHITE);
}

fn unit_to_screen(point: Vec2, center: Vec2, scale: f32) -> Vec2 {
    vec2(center.x + point.x * scale, center.y + point.y * scale)
}

fn polar(radius: f32, degrees: f32) -> Vec2 {
    let radians = degrees.to_radians();
    vec2(radius * radians.cos(), -radius * radians.sin())
}

fn fan_polygon(points: &[Vec2], fill: Color, border: Color) {
    if points.len() < 3 {
        return;
    }
    let centroid = points.iter().copied().fold(Vec2::ZERO, |accumulator, point| accumulator + point) / points.len() as f32;
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        draw_triangle(centroid, a, b, fill);
        draw_line(a.x, a.y, b.x, b.y, 3.0, border);
    }
}

fn arc_sector(center: Vec2, inner: f32, outer: f32, start_deg: f32, end_deg: f32) -> Vec<Vec2> {
    let start_outer = unit_to_screen(polar(outer, start_deg), center, 1.0);
    let end_outer = unit_to_screen(polar(outer, end_deg), center, 1.0);
    let end_inner = unit_to_screen(polar(inner, end_deg), center, 1.0);
    let start_inner = unit_to_screen(polar(inner, start_deg), center, 1.0);
    vec![start_outer, end_outer, end_inner, start_inner]
}

fn diamond(center: Vec2, radius_x: f32, radius_y: f32) -> Vec<Vec2> {
    vec![
        vec2(center.x, center.y - radius_y),
        vec2(center.x + radius_x, center.y),
        vec2(center.x, center.y + radius_y),
        vec2(center.x - radius_x, center.y),
    ]
}

fn build_sensor_regions(center: Vec2, scale: f32) -> Vec<SensorRegion> {
    let mut regions = Vec::new();
    let outer_angles = [45.0, 0.0, -45.0, -90.0, -135.0, 180.0, 135.0, 90.0];
    let labels = ["A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8"];
    for (index, label) in labels.iter().enumerate() {
        let angle = outer_angles[index];
        let start = angle + 22.5;
        let end = angle - 22.5;
        let shape = arc_sector(center, 0.66 * scale, 1.00 * scale, start, end);
        regions.push(SensorRegion { index, label, polygons: vec![shape] });
    }

    let labels = ["B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8"];
    let centers = [45.0, 0.0, -45.0, -90.0, -135.0, 180.0, 135.0, 90.0];
    for (offset, label) in labels.iter().enumerate() {
        let angle = centers[offset];
        let polygon = arc_sector(center, 0.30 * scale, 0.60 * scale, angle + 18.0, angle - 18.0);
        regions.push(SensorRegion { index: 8 + offset, label, polygons: vec![polygon] });
    }

    let c_left = vec![
        vec2(center.x - 0.13 * scale, center.y - 0.23 * scale),
        vec2(center.x - 0.02 * scale, center.y - 0.23 * scale),
        vec2(center.x - 0.02 * scale, center.y + 0.23 * scale),
        vec2(center.x - 0.13 * scale, center.y + 0.23 * scale),
    ];
    let c_right = vec![
        vec2(center.x + 0.02 * scale, center.y - 0.23 * scale),
        vec2(center.x + 0.13 * scale, center.y - 0.23 * scale),
        vec2(center.x + 0.13 * scale, center.y + 0.23 * scale),
        vec2(center.x + 0.02 * scale, center.y + 0.23 * scale),
    ];
    regions.push(SensorRegion {
        index: 16,
        label: "C",
        polygons: vec![c_left, c_right],
    });

    let labels = ["D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8"];
    for (offset, label) in labels.iter().enumerate() {
        let angle = 90.0 - (offset as f32) * 45.0;
        let polygon = arc_sector(center, 0.72 * scale, 0.98 * scale, angle + 14.0, angle - 14.0);
        regions.push(SensorRegion { index: 17 + offset, label, polygons: vec![polygon] });
    }

    let labels = ["E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8"];
    let positions = [
        vec2(0.0, -0.48),
        vec2(0.48, -0.38),
        vec2(0.62, 0.0),
        vec2(0.48, 0.38),
        vec2(0.0, 0.48),
        vec2(-0.48, 0.38),
        vec2(-0.62, 0.0),
        vec2(-0.48, -0.38),
    ];
    for (offset, label) in labels.iter().enumerate() {
        let rect_center = vec2(center.x + positions[offset].x * scale, center.y + positions[offset].y * scale);
        regions.push(SensorRegion {
            index: 25 + offset,
            label,
            polygons: vec![diamond(rect_center, 0.14 * scale, 0.14 * scale)],
        });
    }

    regions
}

fn draw_sensor_regions(regions: &[SensorRegion], pressed: &[bool]) {
    for region in regions {
        let active = pressed.get(region.index).copied().unwrap_or(false);
        let fill = if active {
            Color::new(0.70, 0.20, 0.20, 1.0)
        } else {
            Color::new(0.0, 0.0, 0.0, 1.0)
        };
        let border = if active {
            Color::new(0.95, 0.55, 0.55, 1.0)
        } else {
            Color::new(0.10, 0.10, 0.10, 1.0)
        };
        for polygon in &region.polygons {
            fan_polygon(polygon, fill, border);
        }

        let label_center = region.polygons[0].iter().copied().fold(Vec2::ZERO, |accumulator, point| accumulator + point)
            / region.polygons[0].len() as f32;
        draw_text(region.label, label_center.x - 18.0, label_center.y + 8.0, 24.0, WHITE);
    }
}

fn draw_test_overlay(input: &InputState) {
    draw_text("LnMai sensor test", 24.0, 36.0, 28.0, BLACK);
    draw_text("Press mapped keys to highlight sensor areas", 24.0, 64.0, 22.0, BLACK);
    draw_text("Ctrl-C to exit", 24.0, 90.0, 22.0, BLACK);

    let active = input.sensors.iter().filter(|pressed| **pressed).count();
    draw_text(&format!("Active sensors: {}", active), 24.0, 120.0, 22.0, BLACK);

    let active_keys = SENSOR_LABELS
        .iter()
        .zip(input.sensors.iter())
        .filter_map(|(label, pressed)| if *pressed { Some(*label) } else { None })
        .collect::<Vec<_>>()
        .join(", ");
    draw_text(&format!("Pressed: {}", if active_keys.is_empty() { "-".to_string() } else { active_keys }), 24.0, 148.0, 22.0, BLACK);
}

fn draw_play_overlay(chart_path: &Path, input: &InputState, result: &HostFrame) {
    draw_text(&format!("Chart: {}", chart_path.display()), 24.0, 36.0, 28.0, BLACK);
    draw_text("Buttons: A S D F J K L ;", 24.0, 68.0, 24.0, BLACK);
    draw_text("Sensors: 33-key mapping placeholder", 24.0, 96.0, 24.0, BLACK);

    let active_buttons = input.buttons.iter().filter(|pressed| **pressed).count();
    let active_sensors = input.sensors.iter().filter(|pressed| **pressed).count();
    draw_text(
        &format!("Pressed: buttons {} / sensors {}", active_buttons, active_sensors),
        24.0,
        128.0,
        24.0,
        BLACK,
    );

    draw_text(
        &format!("Commands: judge={} audio={} render={}", result.judge_events.len(), result.audio_commands.len(), result.render_commands.len()),
        24.0,
        160.0,
        24.0,
        BLACK,
    );
}

#[macroquad::main(window_conf)]
async fn main() {
    match parse_args() {
        Mode::Usage => {
            println!("usage: lnmai test");
            println!("   or: lnmai <chart.zip>");
        }
        Mode::SensorTest => {
            run_sensor_test().await;
        }
        Mode::Play(chart_path) => {
            run_play(chart_path).await;
        }
    }
}

async fn run_sensor_test() {
    let exact_regions = sensor_layout::load_exact_regions();

    loop {
        draw_backdrop();
        let input = read_input_state();
        if let Some(regions) = &exact_regions {
            draw_exact_sensor_regions(regions, &input.sensors);
        } else {
            let center = vec2(screen_width() * 0.5, screen_height() * 0.54);
            let scale = screen_height().min(screen_width()) * 0.42;
            let regions = build_sensor_regions(center, scale);
            draw_sensor_regions(&regions, &input.sensors);
        }
        draw_test_overlay(&input);
        next_frame().await;
    }
}

fn draw_exact_sensor_regions(regions: &[sensor_layout::ExactRegion], pressed: &[bool]) {
    let center = vec2(screen_width() * 0.5, screen_height() * 0.54);
    let scale = screen_height().min(screen_width()) * 0.40;
    for region in regions {
        let active = pressed.get(region.index).copied().unwrap_or(false);
        let fill = if active {
            Color::new(0.70, 0.20, 0.20, 1.0)
        } else {
            Color::new(0.0, 0.0, 0.0, 1.0)
        };
        let border = if active {
            Color::new(0.95, 0.55, 0.55, 1.0)
        } else {
            Color::new(0.10, 0.10, 0.10, 1.0)
        };
        let mut point_count = 0.0;
        let mut label_center = Vec2::ZERO;
        for triangle in &region.triangles {
            let a = vec2(center.x + triangle[0].x * scale, center.y + triangle[0].y * scale);
            let b = vec2(center.x + triangle[1].x * scale, center.y + triangle[1].y * scale);
            let c = vec2(center.x + triangle[2].x * scale, center.y + triangle[2].y * scale);
            label_center += a + b + c;
            point_count += 3.0;
            draw_triangle(a, b, c, fill);
            draw_line(a.x, a.y, b.x, b.y, 2.5, border);
            draw_line(b.x, b.y, c.x, c.y, 2.5, border);
            draw_line(c.x, c.y, a.x, a.y, 2.5, border);
        }
        if point_count > 0.0 {
            label_center /= point_count;
            draw_text(region.label, label_center.x - 18.0, label_center.y + 8.0, 24.0, WHITE);
        }
    }
}

async fn run_play(chart_path: PathBuf) {
    let mut app = HostApp::new(chart_path.clone());
    app.load();

    loop {
        draw_backdrop();
        let input = read_input_state();
        app.step(input);
        draw_play_overlay(&chart_path, &input, &app.last_result);
        next_frame().await;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "LnMai".to_string(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        fullscreen: false,
        sample_count: 1,
        window_resizable: true,
        ..Default::default()
    }
}
