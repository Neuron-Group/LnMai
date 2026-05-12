use serde::{Deserialize, Serialize};
use std::path::Path;

pub const BUTTON_ZONE_COUNT: usize = 8;
pub const SENSOR_AREA_COUNT: usize = 33;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FrameInput {
    pub button_held: Vec<bool>,
    pub sensor_held: Vec<bool>,
    pub delta_sec: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimedInputEvent {
    pub at_sec: f32,
    pub index: usize,
    pub is_down: bool,
    pub kind: InputKind,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputKind {
    Button,
    Sensor,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TimedInputBatch {
    pub current_sec: f32,
    pub events: Vec<TimedInputEvent>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum JudgeEventKind {
    Tap,
    Hold,
    Slide,
    Touch,
    Break,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum JudgeGrade {
    Miss,
    LateGood,
    LateGreat3rd,
    LateGreat2nd,
    LateGreat,
    LatePerfect3rd,
    LatePerfect2nd,
    Perfect,
    FastPerfect2nd,
    FastPerfect3rd,
    FastGreat,
    FastGreat2nd,
    FastGreat3rd,
    FastGood,
    TooFast,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JudgeEvent {
    pub kind: JudgeEventKind,
    pub grade: JudgeGrade,
    pub diff_ms: f32,
    pub sensor_pos: usize,
    pub note_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AudioCommand {
    PlayJudgeSfx {
        kind: JudgeEventKind,
        grade: JudgeGrade,
        at_sec: f32,
        note_index: usize,
    },
    PlaySlideCue {
        note_index: usize,
        track_index: usize,
        at_sec: f32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RenderCommand {
    ShowJudgeResult {
        kind: JudgeEventKind,
        grade: JudgeGrade,
        diff_ms: f32,
        note_index: usize,
    },
    UpdateSlideProgress {
        note_index: usize,
        remaining: usize,
    },
    UpdateSlideTrackProgress {
        note_index: usize,
        track_index: usize,
        remaining: usize,
    },
    HideAllSlideBars {
        note_index: usize,
    },
    HideSlideBars {
        note_index: usize,
        end_index: usize,
    },
    HideSlideTrackBars {
        note_index: usize,
        track_index: usize,
        end_index: usize,
    },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HostFrame {
    pub judge_events: Vec<JudgeEvent>,
    pub audio_commands: Vec<AudioCommand>,
    pub render_commands: Vec<RenderCommand>,
}

pub trait CoreBridge {
    fn load_chart(&mut self, chart_path: &Path) -> Result<(), String>;
    fn step(&mut self, input: FrameInput) -> Result<HostFrame, String>;
}

#[derive(Default)]
pub struct MockBridge {
    pub chart_path: Option<String>,
    pub current_sec: f32,
}

impl CoreBridge for MockBridge {
    fn load_chart(&mut self, chart_path: &Path) -> Result<(), String> {
        self.chart_path = Some(chart_path.display().to_string());
        self.current_sec = 0.0;
        Ok(())
    }

    fn step(&mut self, input: FrameInput) -> Result<HostFrame, String> {
        self.current_sec += input.delta_sec.max(0.0);
        Ok(HostFrame::default())
    }
}
