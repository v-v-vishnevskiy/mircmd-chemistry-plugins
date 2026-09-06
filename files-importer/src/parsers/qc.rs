// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::Peekable;

use shared_lib::periodic_table::get_element_by_symbol;
use shared_lib::types::{AtomicCoordinates, Molecule, Node};

pub const BOHR2ANGSTROM: f64 = 0.529177210903;
pub const MAX_SIGNATURE_LINES: usize = 120;

#[derive(Clone, Default)]
pub struct CoordFrame {
    pub atomic_num: Vec<i32>,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
}

impl CoordFrame {
    pub fn is_empty(&self) -> bool {
        self.atomic_num.is_empty()
    }

    pub fn push(&mut self, atomic_num: i32, x: f64, y: f64, z: f64) {
        self.atomic_num.push(atomic_num);
        self.x.push(x);
        self.y.push(y);
        self.z.push(z);
    }

    pub fn extend(&mut self, other: CoordFrame) {
        self.atomic_num.extend(other.atomic_num);
        self.x.extend(other.x);
        self.y.extend(other.y);
        self.z.extend(other.z);
    }
}

#[derive(Default)]
pub struct QcGeometry {
    pub charge: i32,
    pub frames: Vec<CoordFrame>,
    pub is_optimization: bool,
    pub optimized: bool,
}

impl QcGeometry {
    pub fn push_frame(&mut self, frame: CoordFrame) {
        if !frame.is_empty() {
            self.frames.push(frame);
        }
    }
}

pub fn file_has_signature(path: &str, needles: &[&str]) -> Result<bool, String> {
    file_has_signature_n(path, MAX_SIGNATURE_LINES, needles)
}

pub fn file_has_signature_n(path: &str, max_lines: usize, needles: &[&str]) -> Result<bool, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    for (index, line) in BufReader::new(file).lines().enumerate() {
        if index >= max_lines {
            break;
        }
        let line = line.map_err(|e| e.to_string())?;
        if needles.iter().any(|needle| line.contains(needle)) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn skip<'a, I: Iterator<Item = &'a str>>(lines: &mut I, count: usize) {
    for _ in 0..count {
        lines.next();
    }
}

pub fn is_separator(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 2
        && trimmed
            .chars()
            .all(|c| matches!(c, '-' | '=' | '*' | '+' | '|') || c.is_whitespace())
}

pub fn bohr_to_angstrom(value: f64) -> f64 {
    value * BOHR2ANGSTROM
}

pub fn normalize_element_symbol(raw: &str) -> String {
    let letters: String = raw.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    let mut chars = letters.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut symbol = first.to_uppercase().to_string();
    symbol.extend(chars.flat_map(char::to_lowercase));
    symbol
}

pub fn atomic_number_from_token(token: &str) -> Option<i32> {
    let cleaned = token.trim_matches(|c: char| c == '*' || c == '>' || c == '(' || c == ')');
    if let Ok(number) = cleaned.parse::<i32>() {
        return Some(number);
    }
    if let Ok(number) = cleaned.parse::<f64>() {
        return Some(number.round() as i32);
    }
    let symbol = normalize_element_symbol(cleaned);
    match symbol.as_str() {
        "D" | "T" => Some(1),
        "Gh" | "Bq" | "Ghost" => Some(-1),
        _ => get_element_by_symbol(&symbol).map(|element| element.atomic_number),
    }
}

pub fn atom_from_columns(items: &[&str], z_col: usize, x_col: usize) -> Option<(i32, f64, f64, f64)> {
    if items.len() <= x_col + 2 {
        return None;
    }
    let atomic_num = atomic_number_from_token(items[z_col])?;
    let x = items[x_col].parse().ok()?;
    let y = items[x_col + 1].parse().ok()?;
    let z = items[x_col + 2].parse().ok()?;
    Some((atomic_num, x, y, z))
}

pub fn read_numbers<'a, I: Iterator<Item = &'a str>>(lines: &mut I, count: usize) -> Vec<f64> {
    let mut values = Vec::with_capacity(count);
    while values.len() < count {
        let Some(line) = lines.next() else {
            break;
        };
        for token in line.split_whitespace() {
            if let Ok(value) = token.parse::<f64>() {
                values.push(value);
                if values.len() == count {
                    break;
                }
            }
        }
    }
    values
}

pub fn prefer_frames(primary: Vec<CoordFrame>, fallback: Vec<CoordFrame>) -> Vec<CoordFrame> {
    if primary.is_empty() { fallback } else { primary }
}

pub fn to_molecule_node(file_name: &str, geometry: QcGeometry) -> Result<Node, String> {
    if geometry.frames.is_empty() {
        return Err("No atomic coordinates found".to_string());
    }
    let last = geometry.frames.last().unwrap();
    let mut node = molecule_node(file_name, last, geometry.charge)?;
    node.children.push(coords_node(representative_name(&geometry), last)?);
    if geometry.frames.len() > 1 {
        node.children.push(group_node(&geometry)?);
    }
    Ok(node)
}

fn representative_name(geometry: &QcGeometry) -> &'static str {
    if geometry.optimized {
        "Optimized XYZ"
    } else if geometry.is_optimization {
        "Unoptimized final XYZ"
    } else {
        "Final coordinates"
    }
}

fn molecule_node(file_name: &str, frame: &CoordFrame, charge: i32) -> Result<Node, String> {
    Ok(Node {
        name: file_name.to_string(),
        r#type: "mircmd:chemistry:molecule".to_string(),
        data: serde_json::to_vec(&Molecule {
            n_atoms: frame.atomic_num.len() as i32,
            atomic_num: frame.atomic_num.clone(),
            charge,
            name: file_name.to_string(),
        })
        .map_err(|e| format!("Failed to serialize molecule: {}", e))?,
        children: vec![],
    })
}

fn coords_node(name: &str, frame: &CoordFrame) -> Result<Node, String> {
    Ok(Node {
        name: name.to_string(),
        r#type: "mircmd:chemistry:atomic_coordinates".to_string(),
        data: serde_json::to_vec(&AtomicCoordinates {
            atomic_num: frame.atomic_num.clone(),
            x: frame.x.clone(),
            y: frame.y.clone(),
            z: frame.z.clone(),
        })
        .map_err(|e| format!("Failed to serialize coordinates: {}", e))?,
        children: vec![],
    })
}

fn group_node(geometry: &QcGeometry) -> Result<Node, String> {
    let name = if geometry.is_optimization {
        "Optimization"
    } else {
        "Coordinates"
    };
    let mut group = Node {
        name: name.to_string(),
        r#type: "mircmd:chemistry:atomic_coordinates_group".to_string(),
        data: vec![],
        children: vec![],
    };
    for (index, frame) in geometry.frames.iter().enumerate() {
        group.children.push(coords_node(&format!("Step {}", index + 1), frame)?);
    }
    Ok(group)
}

pub fn read_table<'a, I, F>(lines: &mut Peekable<I>, parse_row: F) -> CoordFrame
where
    I: Iterator<Item = &'a str>,
    F: Fn(&str) -> Option<(i32, f64, f64, f64)>,
{
    let mut frame = CoordFrame::default();
    while let Some(line) = lines.peek().copied() {
        if line.trim().is_empty() || is_separator(line) {
            break;
        }
        if let Some((atomic_num, x, y, z)) = parse_row(line) {
            frame.push(atomic_num, x, y, z);
        }
        lines.next();
    }
    frame
}
