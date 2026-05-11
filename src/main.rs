use macroquad::prelude::*;
use std::path::PathBuf;

const BUTTON_KEYS: [KeyCode; 8] = [
    KeyCode::A,
    KeyCode::S,
    KeyCode::D,
    KeyCode::F,
    KeyCode::J,
    KeyCode::K,
    KeyCode::L,
    KeyCode::Semicolon,
];

const SENSOR_KEYS: [KeyCode; 33] = [
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

struct InputState {
    buttons: [bool; 8],
    sensors: [bool; 33],
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            buttons: [false; 8],
            sensors: [false; 33],
        }
    }
}

fn parse_chart_path() -> Option<PathBuf> {
    std::env::args().nth(1).map(PathBuf::from)
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

fn draw_video_placeholder(t: f32) {
    let width = screen_width();
    let height = screen_height();
    let pulse = (t * 1.7).sin() * 0.5 + 0.5;
    draw_rectangle(0.0, 0.0, width, height, Color::new(0.04, 0.04 + 0.08 * pulse, 0.08 + 0.1 * pulse, 1.0));
    draw_circle(width * 0.5, height * 0.5, height * 0.2, Color::new(0.2, 0.3, 0.6, 0.25));
}

fn draw_overlay(chart_path: &PathBuf, input: &InputState) {
    draw_text(&format!("Chart: {}", chart_path.display()), 24.0, 36.0, 28.0, WHITE);
    draw_text("Buttons: A S D F J K L ;", 24.0, 68.0, 24.0, LIGHTGRAY);
    draw_text("Sensors: 33-key mapping placeholder", 24.0, 96.0, 24.0, LIGHTGRAY);

    let active_buttons = input.buttons.iter().filter(|pressed| **pressed).count();
    let active_sensors = input.sensors.iter().filter(|pressed| **pressed).count();
    draw_text(
        &format!("Pressed: buttons {} / sensors {}", active_buttons, active_sensors),
        24.0,
        128.0,
        24.0,
        YELLOW,
    );
}

#[macroquad::main(window_conf)]
async fn main() {
    let Some(chart_path) = parse_chart_path() else {
        println!("usage: lnmai <chart.zip>");
        return;
    };

    loop {
        clear_background(BLACK);
        let input = read_input_state();
        draw_video_placeholder(get_time() as f32);
        draw_overlay(&chart_path, &input);
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
