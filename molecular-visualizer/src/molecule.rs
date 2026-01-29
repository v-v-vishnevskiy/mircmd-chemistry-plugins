use super::config::Config;
use super::core::mesh::InstanceData;
use super::core::{Mat4, Transform, Vec3};
use shared_lib::types::AtomicCoordinates;
use wgpu::util::DeviceExt;

pub struct Molecule {
    atoms_transform: Vec<[[f32; 4]; 4]>,
    atoms_visibility: Vec<bool>,
    atoms_color: Vec<[f32; 4]>,
    pub radius: f32,
    pub transform: Mat4<f32>,
    pub instance_buffer: wgpu::Buffer,
}

impl Molecule {
    pub fn new(device: &wgpu::Device, config: &Config, atomic_coordinates: &AtomicCoordinates) -> Result<Self, String> {
        let mut atoms_transform = Vec::new();
        let mut atoms_visibility = Vec::new();
        let mut atoms_color = Vec::new();
        let mut radius: f32 = 0.0;
        let num_atoms = atomic_coordinates.atomic_num.len();

        let x = atomic_coordinates.x.iter().sum::<f64>();
        let y = atomic_coordinates.y.iter().sum::<f64>();
        let z = atomic_coordinates.z.iter().sum::<f64>();

        let center = Vec3::new(
            x as f32 / num_atoms as f32,
            y as f32 / num_atoms as f32,
            z as f32 / num_atoms as f32,
        );

        for i in 0..num_atoms {
            let atom = config.atoms.get(&atomic_coordinates.atomic_num[i]).ok_or(format!(
                "Atom not found for atomic number: {}",
                atomic_coordinates.atomic_num[i]
            ))?;
            let mut transform = Transform::new();

            let position = Vec3::new(
                atomic_coordinates.x[i] as f32,
                atomic_coordinates.y[i] as f32,
                atomic_coordinates.z[i] as f32,
            );

            radius = radius.max((position - center).length_squared() + atom.radius);

            transform.set_position(position);
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

        let mut transform = Mat4::new();
        transform.translate(-center);

        let data: Vec<InstanceData> = atoms_transform
            .iter()
            .zip(atoms_color.iter())
            .zip(atoms_visibility.iter())
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
            .collect();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Ok(Self {
            atoms_transform,
            atoms_visibility,
            atoms_color,
            radius: radius.sqrt(),
            transform: transform,
            instance_buffer,
        })
    }

    pub fn instance_count(&self) -> u32 {
        self.atoms_visibility.iter().filter(|v| **v).count() as u32
    }
}
