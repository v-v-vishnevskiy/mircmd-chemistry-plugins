// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the Apache 2.0 License

use super::qc::{self, QcGeometry};
use shared_lib::types::Node;

pub fn test(file_path: &str) -> Result<bool, String> {
    qc::file_has_signature(file_path, &["A Quantum Leap Into The Future Of Chemistry"])
}

pub fn parse(content: &str, file_name: &str) -> Result<Node, String> {
    let mut geometry = QcGeometry::default();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.contains("OPTIMIZATION CONVERGED") {
            geometry.is_optimization = true;
            geometry.optimized = true;
        }
        if line.contains("OPTIMIZATION FAILED") {
            geometry.is_optimization = true;
        }
        if line.contains("Standard Nuclear Orientation") {
            let scale = if line.contains("Bohr") { qc::BOHR2ANGSTROM } else { 1.0 };
            qc::skip(&mut lines, 2);
            geometry.push_frame(qc::read_table(&mut lines, |row| {
                let items: Vec<&str> = row.split_whitespace().collect();
                let (atomic_num, x, y, z) = qc::atom_from_columns(&items, 1, 2)?;
                Some((atomic_num, x * scale, y * scale, z * scale))
            }));
        }
    }

    qc::to_molecule_node(file_name, geometry)
}
