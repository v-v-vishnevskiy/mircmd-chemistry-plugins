// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

use super::qc::{self, CoordFrame, QcGeometry};
use shared_lib::types::Node;

pub fn test(file_path: &str) -> Result<bool, String> {
    qc::file_has_signature(file_path, &["x  T  B", "x T B", "xtb version", "* xtb version"])
}

pub fn parse(content: &str, file_name: &str) -> Result<Node, String> {
    let mut geometry = QcGeometry::default();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.to_ascii_lowercase().contains("charge") && geometry.charge == 0 {
            parse_xtb_charge(&mut geometry, line);
        }
        if line.contains("GEOMETRY OPTIMIZATION CONVERGED") {
            geometry.is_optimization = true;
            geometry.optimized = true;
        }
        if line.contains("FAILED TO CONVERGE GEOMETRY") {
            geometry.is_optimization = true;
        }
        if line.contains("final structure:") {
            skip_xtb_header(&mut lines);
            geometry.push_frame(read_xtb_xyz(&mut lines));
        }
    }

    qc::to_molecule_node(file_name, geometry)
}

fn parse_xtb_charge(geometry: &mut QcGeometry, line: &str) {
    let items: Vec<&str> = line.split_whitespace().collect();
    if let Some(index) = items.iter().position(|item| item.eq_ignore_ascii_case("charge")) {
        if let Some(value) = items.get(index + 1).and_then(|item| item.parse().ok()) {
            geometry.charge = value;
        }
    }
}

fn skip_xtb_header<'a, I: Iterator<Item = &'a str>>(lines: &mut std::iter::Peekable<I>) {
    while let Some(line) = lines.peek() {
        if line.contains("====") || line.trim().is_empty() {
            lines.next();
            continue;
        }
        break;
    }
}

fn read_xtb_xyz<'a, I: Iterator<Item = &'a str>>(lines: &mut std::iter::Peekable<I>) -> CoordFrame {
    let Some(count_line) = lines.next() else {
        return CoordFrame::default();
    };
    if let Ok(count) = count_line.trim().parse::<usize>() {
        lines.next();
        let mut frame = CoordFrame::default();
        for _ in 0..count {
            let Some(line) = lines.next() else { break };
            let items: Vec<&str> = line.split_whitespace().collect();
            if let Some((atomic_num, x, y, z)) = qc::atom_from_columns(&items, 0, 1) {
                frame.push(atomic_num, x, y, z);
            }
        }
        return frame;
    }
    let mut frame = CoordFrame::default();
    if let Some((atomic_num, x, y, z)) = parse_mol_or_xyz(count_line) {
        frame.push(atomic_num, x, y, z);
    }
    frame.extend(qc::read_table(lines, parse_mol_or_xyz));
    frame
}

fn parse_mol_or_xyz(line: &str) -> Option<(i32, f64, f64, f64)> {
    let items: Vec<&str> = line.split_whitespace().collect();
    if items.len() >= 4 {
        if let Some(atom) = qc::atom_from_columns(&items, 0, 1) {
            return Some(atom);
        }
        return qc::atom_from_columns(&items, 3, 0);
    }
    None
}
