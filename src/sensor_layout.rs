use macroquad::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct ExactRegion {
    pub index: usize,
    pub label: &'static str,
    pub triangles: Vec<[Vec2; 3]>,
}

struct MeshGroup {
    faces: Vec<Vec<usize>>,
}

struct SceneInstance {
    label: &'static str,
    group_index: usize,
    angle_deg: f32,
}

const LABELS: [&str; 33] = [
    "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8",
    "C", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "E1", "E2", "E3", "E4", "E5", "E6", "E7",
    "E8",
];

pub fn load_exact_regions() -> Option<Vec<ExactRegion>> {
    let obj_path = export_reference_obj().ok()?;
    let obj_text = fs::read_to_string(obj_path).ok()?;
    let (vertices, groups) = parse_obj(&obj_text);
    let instances = parse_scene_instances().ok()?;

    let mut regions = Vec::new();
    let mut all_points = Vec::new();

    for instance in instances {
        let group = groups.get(instance.group_index)?;
        let radians = instance.angle_deg.to_radians();
        let (sin_theta, cos_theta) = radians.sin_cos();
        let mut triangles = Vec::new();

        for face in &group.faces {
            if face.len() < 3 {
                continue;
            }
            let mut transformed = Vec::new();
            for vertex_index in face {
                let (x, z) = vertices.get(*vertex_index).copied()?;
                let rotated_x = x * cos_theta - z * sin_theta;
                let rotated_y = x * sin_theta + z * cos_theta;
                let point = vec2(rotated_x, rotated_y);
                transformed.push(point);
                all_points.push(point);
            }
            for index in 1..transformed.len() - 1 {
                triangles.push([transformed[0], transformed[index], transformed[index + 1]]);
            }
        }

        regions.push(ExactRegion {
            index: label_to_index(instance.label)?,
            label: instance.label,
            triangles,
        });
    }

    for instance in missing_instances() {
        if regions.iter().any(|region| region.label == instance.label) {
            continue;
        }

        let group = groups.get(instance.group_index)?;
        let radians = instance.angle_deg.to_radians();
        let (sin_theta, cos_theta) = radians.sin_cos();
        let mut triangles = Vec::new();

        for face in &group.faces {
            if face.len() < 3 {
                continue;
            }
            let mut transformed = Vec::new();
            for vertex_index in face {
                let (x, z) = vertices.get(*vertex_index).copied()?;
                let rotated_x = x * cos_theta - z * sin_theta;
                let rotated_y = x * sin_theta + z * cos_theta;
                let point = vec2(rotated_x, rotated_y);
                transformed.push(point);
                all_points.push(point);
            }
            for index in 1..transformed.len() - 1 {
                triangles.push([transformed[0], transformed[index], transformed[index + 1]]);
            }
        }

        regions.push(ExactRegion {
            index: label_to_index(instance.label)?,
            label: instance.label,
            triangles,
        });
    }

    if all_points.is_empty() {
        return None;
    }

    let min_x = all_points
        .iter()
        .map(|point| point.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = all_points
        .iter()
        .map(|point| point.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = all_points
        .iter()
        .map(|point| point.y)
        .fold(f32::INFINITY, f32::min);
    let max_y = all_points
        .iter()
        .map(|point| point.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let center = vec2((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
    let scale = 2.0 / ((max_x - min_x).max(max_y - min_y));

    for region in &mut regions {
        for triangle in &mut region.triangles {
            for point in triangle.iter_mut() {
                *point = (*point - center) * scale;
            }
        }
    }

    Some(regions)
}

fn export_reference_obj() -> Result<PathBuf, ()> {
    let fbx_path = PathBuf::from("reference/MajdataPlay/Assets/Models/JudgeAreas.fbx");
    if !fbx_path.exists() {
        return Err(());
    }
    let tmp_dir = std::env::temp_dir().join("lnmai-sensor-layout");
    let _ = fs::create_dir_all(&tmp_dir);
    let obj_path = tmp_dir.join("judge.obj");
    let status = Command::new("assimp")
        .arg("export")
        .arg(&fbx_path)
        .arg(&obj_path)
        .status()
        .map_err(|_| ())?;
    if !status.success() {
        return Err(());
    }
    Ok(obj_path)
}

fn parse_obj(text: &str) -> (Vec<(f32, f32)>, Vec<MeshGroup>) {
    let mut vertices = Vec::new();
    let mut groups = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("v ") {
            let mut parts = rest.split_whitespace();
            let x = parts
                .next()
                .and_then(|value| value.parse::<f32>().ok())
                .unwrap_or(0.0);
            let _y = parts.next();
            let z = parts
                .next()
                .and_then(|value| value.parse::<f32>().ok())
                .unwrap_or(0.0);
            vertices.push((x, -z));
        } else if line.starts_with("g ") {
            groups.push(MeshGroup { faces: Vec::new() });
        } else if let Some(rest) = line.strip_prefix("f ") {
            if let Some(group) = groups.last_mut() {
                let face = rest
                    .split_whitespace()
                    .filter_map(|token| token.split('/').next())
                    .filter_map(|value| value.parse::<usize>().ok())
                    .map(|value| value - 1)
                    .collect::<Vec<_>>();
                group.faces.push(face);
            }
        }
    }
    (vertices, groups)
}

fn parse_scene_instances() -> Result<Vec<SceneInstance>, ()> {
    let scene =
        fs::read_to_string("reference/MajdataPlay/Assets/Scenes/Test.unity").map_err(|_| ())?;
    let blocks = scene.split("--- !u!1 ").skip(1).collect::<Vec<_>>();
    let mut instances = Vec::new();
    let family_map = mesh_family_map();

    for block in blocks {
        let label = LABELS
            .iter()
            .find(|label| block.contains(&format!("m_Name: {label}\n")))
            .copied();
        let Some(label) = label else { continue };
        if !block.contains("MeshFilter:") {
            continue;
        }
        let mesh_file_id = extract_between(block, "m_Mesh: {fileID: ", ", guid:").ok_or(())?;
        let Some(group_index) = family_map.get(mesh_file_id).copied() else {
            continue;
        };
        let angle_text = extract_between(block, "m_LocalEulerAnglesHint: {x: ", "}").ok_or(())?;
        let parts = angle_text.split(',').collect::<Vec<_>>();
        let z_part = parts
            .get(2)
            .and_then(|value| value.split(':').nth(1))
            .unwrap_or("0")
            .trim();
        let angle_deg = z_part.parse::<f32>().unwrap_or(0.0);
        instances.push(SceneInstance {
            label: match label {
                "A2" => "A3",
                "A3" => "A4",
                "A4" => "A5",
                "A5" => "A6",
                "A6" => "A7",
                "A7" => "A8",
                "A8" => "A1",
                "B2" => "B3",
                "B3" => "B4",
                "B4" => "B5",
                "B5" => "B6",
                "B6" => "B7",
                "B7" => "B8",
                "B8" => "B1",
                other => other,
            },
            group_index,
            angle_deg,
        });
    }

    if !instances.iter().any(|instance| instance.label == "C1") {
        instances.push(SceneInstance {
            label: "C1",
            group_index: 0,
            angle_deg: 0.0,
        });
    }

    Ok(instances)
}

fn mesh_family_map() -> HashMap<&'static str, usize> {
    HashMap::from([
        ("8939266724133516769", 0),
        ("-7746497135072881621", 1),
        ("281315202980364329", 2),
        ("6157117880141131119", 3),
        ("-4909277789374002350", 4),
    ])
}

fn missing_instances() -> Vec<SceneInstance> {
    vec![
        SceneInstance {
            label: "A2",
            group_index: 3,
            angle_deg: -45.0,
        },
        SceneInstance {
            label: "B2",
            group_index: 4,
            angle_deg: -45.0,
        },
        SceneInstance {
            label: "C2",
            group_index: 0,
            angle_deg: 180.0,
        },
        SceneInstance {
            label: "D1",
            group_index: 2,
            angle_deg: -90.0,
        },
        SceneInstance {
            label: "E1",
            group_index: 1,
            angle_deg: -90.0,
        },
    ]
}

fn extract_between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start_index = text.find(start)? + start.len();
    let rest = &text[start_index..];
    let end_index = rest.find(end)?;
    Some(&rest[..end_index])
}

fn label_to_index(label: &str) -> Option<usize> {
    if matches!(label, "C" | "C1" | "C2") {
        return LABELS.iter().position(|candidate| *candidate == "C");
    }
    LABELS.iter().position(|candidate| *candidate == label)
}
