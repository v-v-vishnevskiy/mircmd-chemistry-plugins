// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

use super::qc::{self, CoordFrame, QcGeometry};
use shared_lib::types::Node;

pub fn test(file_path: &str) -> Result<bool, String> {
    qc::file_has_signature(file_path, &["GAMESS VERSION", "Firefly (PC GAMESS)"])
}

pub fn parse(content: &str, file_name: &str) -> Result<Node, String> {
    let mut geometry = QcGeometry::default();
    let mut angs = Vec::new();
    let mut bohr = Vec::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.contains("CHARGE OF MOLECULE") {
            if let Some(value) = line.split_whitespace().last().and_then(|item| item.parse().ok()) {
                geometry.charge = value;
            }
        }
        if line.contains("EQUILIBRIUM GEOMETRY LOCATED") {
            geometry.is_optimization = true;
            geometry.optimized = true;
        }
        if line.contains("GEOMETRY SEARCH IS NOT CONVERGED") {
            geometry.is_optimization = true;
        }
        if line.contains("COORDINATES OF ALL ATOMS ARE") {
            qc::skip(&mut lines, 2);
            let scale = if line.contains("BOHR") { qc::BOHR2ANGSTROM } else { 1.0 };
            angs.push(read_gamess_atoms(&mut lines, scale));
        } else if line.contains("ATOMIC COORDINATES") && !line.contains("OF ALL ATOMS") {
            qc::skip(&mut lines, 2);
            bohr.push(read_gamess_atoms(&mut lines, qc::BOHR2ANGSTROM));
        }
    }

    geometry.frames = qc::prefer_frames(angs, bohr);
    geometry.frames.retain(|frame| !frame.is_empty());
    qc::to_molecule_node(file_name, geometry)
}

fn read_gamess_atoms<'a, I: Iterator<Item = &'a str>>(lines: &mut std::iter::Peekable<I>, scale: f64) -> CoordFrame {
    qc::read_table(lines, |line| {
        let items: Vec<&str> = line.split_whitespace().collect();
        let (atomic_num, x, y, z) = qc::atom_from_columns(&items, 1, 2)?;
        Some((atomic_num, x * scale, y * scale, z * scale))
    })
}
