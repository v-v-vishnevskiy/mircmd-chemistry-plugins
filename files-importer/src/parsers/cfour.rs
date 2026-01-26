// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the MIT License

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use shared_lib::types::{AtomicCoordinates, Molecule, Node};

const MAX_VALIDATION_LINES: usize = 20;
const BOHR2ANGSTROM: f64 = 0.529177210903;

const CFOUR_SIGNATURE: &str = "<<<     CCCCCC     CCCCCC   |||     CCCCCC     CCCCCC   >>>";

/// Validates if the file is in Cfour log format.
pub fn test(file_path: &str) -> Result<bool, String> {
    let path = Path::new(file_path);
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader
        .lines()
        .take(MAX_VALIDATION_LINES)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Check if any line (except the first) contains the Cfour signature
    for line in lines.iter().skip(1) {
        if line.contains(CFOUR_SIGNATURE) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Parses a Cfour log file.
pub fn parse(content: &str, file_name: &str) -> Result<Node, String> {
    let mut result = Node {
        name: file_name.to_string(),
        r#type: "mircmd:chemistry:molecule".to_string(),
        data: serde_json::to_vec(&Molecule {
            n_atoms: 0,
            atomic_num: vec![],
            charge: 0,
            name: file_name.to_string(),
        })
        .map_err(|e| format!("Failed to serialize molecule: {}", e))?,
        children: vec![],
    };

    let mut cart_set_number = 0;
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.contains("Z-matrix   Atomic            Coordinates (in bohr)") {
            cart_set_number += 1;

            // Skip header of the table (2 lines)
            for _ in 0..2 {
                lines.next();
            }

            // Read the table
            let mut atomic_num: Vec<i32> = vec![];
            let mut atom_coord_x: Vec<f64> = vec![];
            let mut atom_coord_y: Vec<f64> = vec![];
            let mut atom_coord_z: Vec<f64> = vec![];

            for block_line in lines.by_ref() {
                if block_line.contains("--") {
                    break;
                }

                let items: Vec<&str> = block_line.split_whitespace().collect();
                if items.len() >= 5 {
                    let at_num = if items[1] == "0" {
                        -1
                    } else {
                        items[1].parse::<i32>().unwrap_or(-1)
                    };

                    let x: f64 = items[2].parse::<f64>().unwrap_or(0.0) * BOHR2ANGSTROM;
                    let y: f64 = items[3].parse::<f64>().unwrap_or(0.0) * BOHR2ANGSTROM;
                    let z: f64 = items[4].parse::<f64>().unwrap_or(0.0) * BOHR2ANGSTROM;

                    atomic_num.push(at_num);
                    atom_coord_x.push(x);
                    atom_coord_y.push(y);
                    atom_coord_z.push(z);
                }
            }

            let coords = AtomicCoordinates {
                atomic_num,
                x: atom_coord_x,
                y: atom_coord_y,
                z: atom_coord_z,
            };

            let at_coord_node = Node {
                name: format!("Set#{}", cart_set_number),
                r#type: "mircmd:chemistry:atomic_coordinates".to_string(),
                data: serde_json::to_vec(&coords).map_err(|e| format!("Failed to serialize coordinates: {}", e))?,
                children: vec![],
            };

            result.children.push(at_coord_node);
        }
    }

    Ok(result)
}
