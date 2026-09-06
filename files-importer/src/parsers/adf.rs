// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

use super::qc::{self, CoordFrame, QcGeometry};
use shared_lib::types::Node;

pub fn test(file_path: &str) -> Result<bool, String> {
    qc::file_has_signature(file_path, &["Amsterdam Density Functional"])
}

pub fn parse(content: &str, file_name: &str) -> Result<Node, String> {
    let mut geometry = QcGeometry::default();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.contains("Geometry Convergence") {
            geometry.is_optimization = true;
        }
        if line.contains("CONVERGED") && geometry.is_optimization {
            geometry.optimized = true;
        }
        if line.trim() == "ATOMS" {
            qc::skip(&mut lines, 3);
            geometry.push_frame(read_adf_atoms(&mut lines, false));
        }
        if line.contains("Coordinates (Cartesian)") {
            qc::skip(&mut lines, 5);
            geometry.push_frame(read_adf_atoms(&mut lines, true));
        }
    }

    qc::to_molecule_node(file_name, geometry)
}

fn read_adf_atoms<'a, I: Iterator<Item = &'a str>>(lines: &mut std::iter::Peekable<I>, cartesian: bool) -> CoordFrame {
    qc::read_table(lines, move |line| {
        let items: Vec<&str> = line.split_whitespace().collect();
        if cartesian {
            if items.len() >= 8 {
                return qc::atom_from_columns(&items, 1, 5);
            }
            return qc::atom_from_columns(&items, 1, 2);
        }
        qc::atom_from_columns(&items, 1, 2)
    })
}
