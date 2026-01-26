// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the MIT License

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub name: String,
    pub r#type: String,
    pub data: Vec<u8>,
    pub children: Vec<Node>,
}

#[derive(Serialize, Deserialize)]
pub struct AtomicCoordinates {
    pub atomic_num: Vec<i32>,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct Molecule {
    pub n_atoms: i32,
    pub atomic_num: Vec<i32>,
    pub charge: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct VolumeCube {
    pub comment1: String,
    pub comment2: String,
    pub box_origin: Vec<f64>,
    pub steps_number: Vec<i32>,
    pub steps_size: Vec<Vec<f64>>,
    pub cube_data: Vec<Vec<Vec<f64>>>,
}
