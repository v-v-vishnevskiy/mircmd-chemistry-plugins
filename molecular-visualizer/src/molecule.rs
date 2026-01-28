use super::config::Config;
use super::core::mesh::InstanceData;
use super::core::{Mat4, Transform, Vec3};
use shared_lib::types::AtomicCoordinates;

pub struct Molecule {
    atoms_transform: Vec<[[f32; 4]; 4]>,
    atoms_visibility: Vec<bool>,
    atoms_color: Vec<[f32; 4]>,
    pub center: Mat4<f32>,
}

impl Molecule {
    pub fn new(atomic_coordinates: &AtomicCoordinates, config: &Config) -> Result<Self, String> {
        let mut atoms_transform = Vec::new();
        let mut atoms_visibility = Vec::new();
        let mut atoms_color = Vec::new();
        let mut center_pos = Vec3::new(0.0, 0.0, 0.0);
        let num_atoms = atomic_coordinates.atomic_num.len();

        for i in 0..num_atoms {
            let position = Vec3::new(
                atomic_coordinates.x[i] as f32,
                atomic_coordinates.y[i] as f32,
                atomic_coordinates.z[i] as f32,
            );

            center_pos += position;

            let mut transform = Transform::new();
            transform.set_position(position);
            let atom = config.atoms.get(&atomic_coordinates.atomic_num[i]).ok_or(format!(
                "Atom not found for atomic number: {}",
                atomic_coordinates.atomic_num[i]
            ))?;
            transform.set_scale(Vec3::new(atom.radius, atom.radius, atom.radius));

            let matrix = transform.get_matrix().data;
            let matrix_4x4: [[f32; 4]; 4] = [
                [matrix[0], matrix[1], matrix[2], matrix[3]],
                [matrix[4], matrix[5], matrix[6], matrix[7]],
                [matrix[8], matrix[9], matrix[10], matrix[11]],
                [matrix[12], matrix[13], matrix[14], matrix[15]],
            ];
            atoms_transform.push(matrix_4x4);
            atoms_visibility.push(true);
            atoms_color.push([atom.color[0], atom.color[1], atom.color[2], 1.0]);
        }

        let mut center = Mat4::new();
        center.translate(-(center_pos / num_atoms as f32));

        Ok(Self {
            atoms_transform,
            atoms_visibility,
            atoms_color,
            center,
        })
    }

    pub fn instance_data(&self) -> Vec<InstanceData> {
        self.atoms_transform
            .iter()
            .zip(self.atoms_color.iter())
            .zip(self.atoms_visibility.iter())
            .filter_map(|((transform, color), visible)| {
                if *visible {
                    Some(InstanceData {
                        model_matrix: *transform,
                        color: *color,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn instance_count(&self) -> u32 {
        self.atoms_visibility.iter().filter(|v| **v).count() as u32
    }
}
