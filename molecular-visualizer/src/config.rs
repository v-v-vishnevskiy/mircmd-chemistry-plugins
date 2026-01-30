use super::types::Color;
use std::collections::HashMap;

pub struct Atom {
    pub radius: f32,
    pub color: Color,
}

enum BondColorMode {
    OwnColor,
    AtomColor,
}

pub struct Bond {
    pub radius: f32,
    pub color_mode: BondColorMode,
    pub color: Color,
}

pub struct Style {
    pub background_color: Color,
    pub atoms: HashMap<i32, Atom>,
    pub bond: Bond,
    pub geom_bond_tolerance: f64,
}

impl Style {
    pub fn new() -> Self {
        let mut atoms = HashMap::new();

        atoms.insert(
            -2,
            Atom {
                radius: 0.25,
                color: Color::new(0.73, 0.58, 0.31, 1.0),
            },
        );
        atoms.insert(
            -1,
            Atom {
                radius: 0.15,
                color: Color::new(0.0, 0.98, 1.0, 1.0),
            },
        );
        atoms.insert(
            1,
            Atom {
                radius: 0.17,
                color: Color::new(1.0, 1.0, 1.0, 1.0),
            },
        );
        atoms.insert(
            2,
            Atom {
                radius: 0.18,
                color: Color::new(0.85, 1.0, 1.0, 1.0),
            },
        );
        atoms.insert(
            3,
            Atom {
                radius: 0.2,
                color: Color::new(0.8, 0.5, 1.0, 1.0),
            },
        );
        atoms.insert(
            4,
            Atom {
                radius: 0.22,
                color: Color::new(0.76, 1.0, 0.0, 1.0),
            },
        );
        atoms.insert(
            5,
            Atom {
                radius: 0.24,
                color: Color::new(1.0, 0.71, 0.71, 1.0),
            },
        );
        atoms.insert(
            6,
            Atom {
                radius: 0.26,
                color: Color::new(0.56, 0.56, 0.56, 1.0),
            },
        );
        atoms.insert(
            7,
            Atom {
                radius: 0.28,
                color: Color::new(0.19, 0.31, 0.97, 1.0),
            },
        );
        atoms.insert(
            8,
            Atom {
                radius: 0.3,
                color: Color::new(1.0, 0.05, 0.05, 1.0),
            },
        );
        atoms.insert(
            9,
            Atom {
                radius: 0.32,
                color: Color::new(0.56, 0.88, 0.31, 1.0),
            },
        );
        atoms.insert(
            10,
            Atom {
                radius: 0.34,
                color: Color::new(0.7, 0.89, 0.96, 1.0),
            },
        );
        atoms.insert(
            11,
            Atom {
                radius: 0.3,
                color: Color::new(0.67, 0.36, 0.95, 1.0),
            },
        );
        atoms.insert(
            12,
            Atom {
                radius: 0.32,
                color: Color::new(0.54, 1.0, 0.0, 1.0),
            },
        );
        atoms.insert(
            13,
            Atom {
                radius: 0.34,
                color: Color::new(0.75, 0.65, 0.65, 1.0),
            },
        );
        atoms.insert(
            14,
            Atom {
                radius: 0.36,
                color: Color::new(0.94, 0.78, 0.63, 1.0),
            },
        );
        atoms.insert(
            15,
            Atom {
                radius: 0.38,
                color: Color::new(1.0, 0.5, 0.0, 1.0),
            },
        );
        atoms.insert(
            16,
            Atom {
                radius: 0.4,
                color: Color::new(1.0, 1.0, 0.19, 1.0),
            },
        );
        atoms.insert(
            17,
            Atom {
                radius: 0.42,
                color: Color::new(0.12, 0.94, 0.12, 1.0),
            },
        );
        atoms.insert(
            18,
            Atom {
                radius: 0.44,
                color: Color::new(0.5, 0.82, 0.89, 1.0),
            },
        );
        atoms.insert(
            19,
            Atom {
                radius: 0.4,
                color: Color::new(0.56, 0.25, 0.83, 1.0),
            },
        );
        atoms.insert(
            20,
            Atom {
                radius: 0.41,
                color: Color::new(0.24, 1.0, 0.0, 1.0),
            },
        );
        atoms.insert(
            21,
            Atom {
                radius: 0.42,
                color: Color::new(0.9, 0.9, 0.90, 1.0),
            },
        );
        atoms.insert(
            22,
            Atom {
                radius: 0.43,
                color: Color::new(0.75, 0.76, 0.78, 1.0),
            },
        );
        atoms.insert(
            23,
            Atom {
                radius: 0.44,
                color: Color::new(0.65, 0.65, 0.67, 1.0),
            },
        );
        atoms.insert(
            24,
            Atom {
                radius: 0.45,
                color: Color::new(0.54, 0.6, 0.78, 1.0),
            },
        );
        atoms.insert(
            25,
            Atom {
                radius: 0.46,
                color: Color::new(0.61, 0.48, 0.78, 1.0),
            },
        );
        atoms.insert(
            26,
            Atom {
                radius: 0.47,
                color: Color::new(0.88, 0.4, 0.20, 1.0),
            },
        );
        atoms.insert(
            27,
            Atom {
                radius: 0.48,
                color: Color::new(0.94, 0.56, 0.63, 1.0),
            },
        );
        atoms.insert(
            28,
            Atom {
                radius: 0.49,
                color: Color::new(0.31, 0.82, 0.31, 1.0),
            },
        );
        atoms.insert(
            29,
            Atom {
                radius: 0.5,
                color: Color::new(0.78, 0.5, 0.20, 1.0),
            },
        );
        atoms.insert(
            30,
            Atom {
                radius: 0.51,
                color: Color::new(0.49, 0.5, 0.69, 1.0),
            },
        );
        atoms.insert(
            31,
            Atom {
                radius: 0.52,
                color: Color::new(0.76, 0.56, 0.56, 1.0),
            },
        );
        atoms.insert(
            32,
            Atom {
                radius: 0.53,
                color: Color::new(0.4, 0.56, 0.56, 1.0),
            },
        );
        atoms.insert(
            33,
            Atom {
                radius: 0.54,
                color: Color::new(0.74, 0.5, 0.89, 1.0),
            },
        );
        atoms.insert(
            34,
            Atom {
                radius: 0.55,
                color: Color::new(1.0, 0.63, 0.0, 1.0),
            },
        );
        atoms.insert(
            35,
            Atom {
                radius: 0.56,
                color: Color::new(0.65, 0.16, 0.16, 1.0),
            },
        );
        atoms.insert(
            36,
            Atom {
                radius: 0.57,
                color: Color::new(0.36, 0.72, 0.82, 1.0),
            },
        );
        atoms.insert(
            37,
            Atom {
                radius: 0.5,
                color: Color::new(0.44, 0.18, 0.69, 1.0),
            },
        );
        atoms.insert(
            38,
            Atom {
                radius: 0.51,
                color: Color::new(0.0, 1.0, 0.0, 1.0),
            },
        );
        atoms.insert(
            39,
            Atom {
                radius: 0.52,
                color: Color::new(0.58, 1.0, 1.0, 1.0),
            },
        );
        atoms.insert(
            40,
            Atom {
                radius: 0.53,
                color: Color::new(0.58, 0.88, 0.88, 1.0),
            },
        );
        atoms.insert(
            41,
            Atom {
                radius: 0.54,
                color: Color::new(0.45, 0.76, 0.79, 1.0),
            },
        );
        atoms.insert(
            42,
            Atom {
                radius: 0.55,
                color: Color::new(0.33, 0.71, 0.71, 1.0),
            },
        );
        atoms.insert(
            43,
            Atom {
                radius: 0.56,
                color: Color::new(0.23, 0.62, 0.62, 1.0),
            },
        );
        atoms.insert(
            44,
            Atom {
                radius: 0.57,
                color: Color::new(0.14, 0.56, 0.56, 1.0),
            },
        );
        atoms.insert(
            45,
            Atom {
                radius: 0.58,
                color: Color::new(0.04, 0.49, 0.55, 1.0),
            },
        );
        atoms.insert(
            46,
            Atom {
                radius: 0.59,
                color: Color::new(0.0, 0.41, 0.52, 1.0),
            },
        );
        atoms.insert(
            47,
            Atom {
                radius: 0.6,
                color: Color::new(0.75, 0.75, 0.75, 1.0),
            },
        );
        atoms.insert(
            48,
            Atom {
                radius: 0.61,
                color: Color::new(1.0, 0.85, 0.56, 1.0),
            },
        );
        atoms.insert(
            49,
            Atom {
                radius: 0.62,
                color: Color::new(0.65, 0.46, 0.45, 1.0),
            },
        );
        atoms.insert(
            50,
            Atom {
                radius: 0.63,
                color: Color::new(0.4, 0.5, 0.50, 1.0),
            },
        );
        atoms.insert(
            51,
            Atom {
                radius: 0.64,
                color: Color::new(0.62, 0.39, 0.71, 1.0),
            },
        );
        atoms.insert(
            52,
            Atom {
                radius: 0.65,
                color: Color::new(0.83, 0.48, 0.0, 1.0),
            },
        );
        atoms.insert(
            53,
            Atom {
                radius: 0.66,
                color: Color::new(0.58, 0.0, 0.58, 1.0),
            },
        );
        atoms.insert(
            54,
            Atom {
                radius: 0.67,
                color: Color::new(0.26, 0.62, 0.69, 1.0),
            },
        );
        atoms.insert(
            55,
            Atom {
                radius: 0.6,
                color: Color::new(0.34, 0.09, 0.56, 1.0),
            },
        );
        atoms.insert(
            56,
            Atom {
                radius: 0.61,
                color: Color::new(0.0, 0.79, 0.0, 1.0),
            },
        );
        atoms.insert(
            57,
            Atom {
                radius: 0.62,
                color: Color::new(0.44, 0.83, 1.0, 1.0),
            },
        );
        atoms.insert(
            58,
            Atom {
                radius: 0.62,
                color: Color::new(1.0, 1.0, 0.78, 1.0),
            },
        );
        atoms.insert(
            59,
            Atom {
                radius: 0.62,
                color: Color::new(0.85, 1.0, 0.78, 1.0),
            },
        );
        atoms.insert(
            60,
            Atom {
                radius: 0.62,
                color: Color::new(0.78, 1.0, 0.78, 1.0),
            },
        );
        atoms.insert(
            61,
            Atom {
                radius: 0.62,
                color: Color::new(0.64, 1.0, 0.78, 1.0),
            },
        );
        atoms.insert(
            62,
            Atom {
                radius: 0.62,
                color: Color::new(0.56, 1.0, 0.78, 1.0),
            },
        );
        atoms.insert(
            63,
            Atom {
                radius: 0.62,
                color: Color::new(0.38, 1.0, 0.78, 1.0),
            },
        );
        atoms.insert(
            64,
            Atom {
                radius: 0.62,
                color: Color::new(0.27, 1.0, 0.78, 1.0),
            },
        );
        atoms.insert(
            65,
            Atom {
                radius: 0.62,
                color: Color::new(0.19, 1.0, 0.78, 1.0),
            },
        );
        atoms.insert(
            66,
            Atom {
                radius: 0.62,
                color: Color::new(0.12, 1.0, 0.78, 1.0),
            },
        );
        atoms.insert(
            67,
            Atom {
                radius: 0.62,
                color: Color::new(0.0, 1.0, 0.61, 1.0),
            },
        );
        atoms.insert(
            68,
            Atom {
                radius: 0.62,
                color: Color::new(0.0, 0.9, 0.46, 1.0),
            },
        );
        atoms.insert(
            69,
            Atom {
                radius: 0.62,
                color: Color::new(0.0, 0.83, 0.32, 1.0),
            },
        );
        atoms.insert(
            70,
            Atom {
                radius: 0.62,
                color: Color::new(0.0, 0.75, 0.22, 1.0),
            },
        );
        atoms.insert(
            71,
            Atom {
                radius: 0.62,
                color: Color::new(0.0, 0.67, 0.14, 1.0),
            },
        );
        atoms.insert(
            72,
            Atom {
                radius: 0.63,
                color: Color::new(0.3, 0.76, 1.0, 1.0),
            },
        );
        atoms.insert(
            73,
            Atom {
                radius: 0.64,
                color: Color::new(0.3, 0.65, 1.0, 1.0),
            },
        );
        atoms.insert(
            74,
            Atom {
                radius: 0.65,
                color: Color::new(0.13, 0.58, 0.84, 1.0),
            },
        );
        atoms.insert(
            75,
            Atom {
                radius: 0.66,
                color: Color::new(0.15, 0.49, 0.67, 1.0),
            },
        );
        atoms.insert(
            76,
            Atom {
                radius: 0.67,
                color: Color::new(0.15, 0.4, 0.59, 1.0),
            },
        );
        atoms.insert(
            77,
            Atom {
                radius: 0.68,
                color: Color::new(0.09, 0.33, 0.53, 1.0),
            },
        );
        atoms.insert(
            78,
            Atom {
                radius: 0.69,
                color: Color::new(0.82, 0.82, 0.88, 1.0),
            },
        );
        atoms.insert(
            79,
            Atom {
                radius: 0.7,
                color: Color::new(1.0, 0.82, 0.14, 1.0),
            },
        );
        atoms.insert(
            80,
            Atom {
                radius: 0.71,
                color: Color::new(0.72, 0.72, 0.82, 1.0),
            },
        );
        atoms.insert(
            81,
            Atom {
                radius: 0.72,
                color: Color::new(0.65, 0.33, 0.30, 1.0),
            },
        );
        atoms.insert(
            82,
            Atom {
                radius: 0.73,
                color: Color::new(0.34, 0.35, 0.38, 1.0),
            },
        );
        atoms.insert(
            83,
            Atom {
                radius: 0.74,
                color: Color::new(0.62, 0.31, 0.71, 1.0),
            },
        );
        atoms.insert(
            84,
            Atom {
                radius: 0.75,
                color: Color::new(0.67, 0.36, 0.0, 1.0),
            },
        );
        atoms.insert(
            85,
            Atom {
                radius: 0.76,
                color: Color::new(0.46, 0.31, 0.27, 1.0),
            },
        );
        atoms.insert(
            86,
            Atom {
                radius: 0.77,
                color: Color::new(0.26, 0.51, 0.59, 1.0),
            },
        );
        atoms.insert(
            87,
            Atom {
                radius: 0.7,
                color: Color::new(0.26, 0.0, 0.40, 1.0),
            },
        );
        atoms.insert(
            88,
            Atom {
                radius: 0.71,
                color: Color::new(0.0, 0.49, 0.0, 1.0),
            },
        );
        atoms.insert(
            89,
            Atom {
                radius: 0.72,
                color: Color::new(0.44, 0.67, 0.98, 1.0),
            },
        );
        atoms.insert(
            90,
            Atom {
                radius: 0.72,
                color: Color::new(0.0, 0.73, 1.0, 1.0),
            },
        );
        atoms.insert(
            91,
            Atom {
                radius: 0.72,
                color: Color::new(0.0, 0.63, 1.0, 1.0),
            },
        );
        atoms.insert(
            92,
            Atom {
                radius: 0.72,
                color: Color::new(0.0, 0.56, 1.0, 1.0),
            },
        );
        atoms.insert(
            93,
            Atom {
                radius: 0.72,
                color: Color::new(0.0, 0.5, 1.0, 1.0),
            },
        );
        atoms.insert(
            94,
            Atom {
                radius: 0.72,
                color: Color::new(0.0, 0.42, 1.0, 1.0),
            },
        );
        atoms.insert(
            95,
            Atom {
                radius: 0.72,
                color: Color::new(0.33, 0.36, 0.95, 1.0),
            },
        );
        atoms.insert(
            96,
            Atom {
                radius: 0.72,
                color: Color::new(0.47, 0.36, 0.89, 1.0),
            },
        );
        atoms.insert(
            97,
            Atom {
                radius: 0.72,
                color: Color::new(0.54, 0.31, 0.89, 1.0),
            },
        );
        atoms.insert(
            98,
            Atom {
                radius: 0.72,
                color: Color::new(0.63, 0.21, 0.83, 1.0),
            },
        );
        atoms.insert(
            99,
            Atom {
                radius: 0.72,
                color: Color::new(0.7, 0.12, 0.83, 1.0),
            },
        );
        atoms.insert(
            100,
            Atom {
                radius: 0.72,
                color: Color::new(0.7, 0.12, 0.73, 1.0),
            },
        );
        atoms.insert(
            101,
            Atom {
                radius: 0.72,
                color: Color::new(0.7, 0.05, 0.65, 1.0),
            },
        );
        atoms.insert(
            102,
            Atom {
                radius: 0.72,
                color: Color::new(0.74, 0.05, 0.53, 1.0),
            },
        );
        atoms.insert(
            103,
            Atom {
                radius: 0.72,
                color: Color::new(0.78, 0.0, 0.40, 1.0),
            },
        );
        atoms.insert(
            104,
            Atom {
                radius: 0.73,
                color: Color::new(0.8, 0.0, 0.35, 1.0),
            },
        );
        atoms.insert(
            105,
            Atom {
                radius: 0.74,
                color: Color::new(0.82, 0.0, 0.31, 1.0),
            },
        );
        atoms.insert(
            106,
            Atom {
                radius: 0.75,
                color: Color::new(0.85, 0.0, 0.27, 1.0),
            },
        );
        atoms.insert(
            107,
            Atom {
                radius: 0.76,
                color: Color::new(0.88, 0.0, 0.22, 1.0),
            },
        );
        atoms.insert(
            108,
            Atom {
                radius: 0.77,
                color: Color::new(0.9, 0.0, 0.18, 1.0),
            },
        );
        atoms.insert(
            109,
            Atom {
                radius: 0.78,
                color: Color::new(0.92, 0.0, 0.15, 1.0),
            },
        );
        atoms.insert(
            110,
            Atom {
                radius: 0.79,
                color: Color::new(0.94, 0.0, 0.14, 1.0),
            },
        );
        atoms.insert(
            111,
            Atom {
                radius: 0.8,
                color: Color::new(0.94, 0.0, 0.14, 1.0),
            },
        );
        atoms.insert(
            112,
            Atom {
                radius: 0.81,
                color: Color::new(0.94, 0.0, 0.14, 1.0),
            },
        );
        atoms.insert(
            113,
            Atom {
                radius: 0.82,
                color: Color::new(0.94, 0.0, 0.14, 1.0),
            },
        );
        atoms.insert(
            114,
            Atom {
                radius: 0.83,
                color: Color::new(0.94, 0.0, 0.14, 1.0),
            },
        );
        atoms.insert(
            115,
            Atom {
                radius: 0.84,
                color: Color::new(0.94, 0.0, 0.14, 1.0),
            },
        );
        atoms.insert(
            116,
            Atom {
                radius: 0.85,
                color: Color::new(0.94, 0.0, 0.14, 1.0),
            },
        );
        atoms.insert(
            117,
            Atom {
                radius: 0.86,
                color: Color::new(0.94, 0.0, 0.14, 1.0),
            },
        );
        atoms.insert(
            118,
            Atom {
                radius: 0.87,
                color: Color::new(0.94, 0.0, 0.14, 1.0),
            },
        );

        Self {
            background_color: Color::new(0.133, 0.133, 0.133, 1.0),
            atoms,
            bond: Bond {
                radius: 0.1,
                color_mode: BondColorMode::AtomColor,
                color: Color::new(0.5, 0.5, 0.5, 1.0),
            },
            geom_bond_tolerance: 0.15,
        }
    }
}

pub struct Config {
    pub style: Style,
}

impl Config {
    pub fn new() -> Self {
        Self { style: Style::new() }
    }
}
