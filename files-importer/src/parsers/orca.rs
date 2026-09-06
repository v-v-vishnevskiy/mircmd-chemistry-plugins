// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

use super::qc::{self, QcGeometry};
use shared_lib::types::Node;

pub fn test(file_path: &str) -> Result<bool, String> {
    qc::file_has_signature(file_path, &["O   R   C   A", "O  R  C  A", "O R C A"])
}

pub fn parse(content: &str, file_name: &str) -> Result<Node, String> {
    let mut geometry = QcGeometry::default();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.contains("Total Charge") {
            if let Some(value) = line.split_whitespace().last().and_then(|item| item.parse().ok()) {
                geometry.charge = value;
            }
        }
        if line.contains("THE OPTIMIZATION HAS CONVERGED")
            || line.contains("FINAL ENERGY EVALUATION AT THE STATIONARY POINT")
        {
            geometry.is_optimization = true;
            geometry.optimized = true;
        }
        if line.contains("The optimization did not converge") {
            geometry.is_optimization = true;
        }
        if line.contains("CARTESIAN COORDINATES (ANGSTROEM)") {
            qc::skip(&mut lines, 1);
            geometry.push_frame(qc::read_table(&mut lines, parse_orca_atom));
        }
    }

    qc::to_molecule_node(file_name, geometry)
}

fn parse_orca_atom(line: &str) -> Option<(i32, f64, f64, f64)> {
    let items: Vec<&str> = line.split_whitespace().collect();
    if items.first()?.ends_with('>') {
        return None;
    }
    qc::atom_from_columns(&items, 0, 1)
}
