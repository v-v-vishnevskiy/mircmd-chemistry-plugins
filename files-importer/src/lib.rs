// Copyright (c) 2026 Valery Vishnevskiy and Yury Vishnevskiy
// Licensed under the MIT License

#[allow(warnings)]
mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "plugin",
        generate_all,
    });

    use super::ChemistryImporter;

    export!(ChemistryImporter);
}

mod parsers;

use bindings::Guest;
use shared_lib::types;

struct ChemistryImporter;

type ParserTestFn = fn(&str) -> Result<bool, String>;
type ParserParseFn = fn(&str, &str) -> Result<types::Node, String>;

const PARSERS: &[(&str, ParserTestFn, ParserParseFn)] = &[
    ("XYZ", parsers::xyz::test, parsers::xyz::parse),
    ("Gaussian Cube", parsers::cube::test, parsers::cube::parse),
    ("UNEX", parsers::unex::test, parsers::unex::parse),
    ("Cfour", parsers::cfour::test, parsers::cfour::parse),
    ("MDL Mol V2000", parsers::mdlmol2000::test, parsers::mdlmol2000::parse),
];

impl Guest for ChemistryImporter {
    fn load(file_path: String) -> Result<Vec<u8>, String> {
        let content = std::fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        let file_name = std::path::Path::new(&file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let mut errors: Vec<String> = Vec::new();

        for (name, test_fn, parse_fn) in PARSERS {
            match test_fn(&file_path) {
                Ok(true) => match parse_fn(&content, file_name) {
                    Ok(node) => {
                        return serde_json::to_vec(&node).map_err(|e| format!("Failed to serialize result: {}", e));
                    }
                    Err(e) => {
                        errors.push(format!("{}: {}", name, e));
                    }
                },
                Ok(false) => continue,
                Err(e) => {
                    errors.push(format!("{}: {}", name, e));
                }
            }
        }

        Err(format!(
            "No suitable parser found for file '{}'. Errors: {}",
            file_name,
            errors.join("; ")
        ))
    }
}
