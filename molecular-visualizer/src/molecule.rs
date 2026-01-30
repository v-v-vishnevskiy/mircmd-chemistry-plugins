use super::bonds;
use super::config::Config;
use super::core::mesh::InstanceData;
use super::core::{Mat4, Quaternion, Transform, Vec3};
use super::types::Color;
use shared_lib::types::AtomicCoordinates;
use wgpu::util::DeviceExt;

pub struct Molecule {
    atoms_transform: Vec<[[f32; 4]; 4]>,
    atoms_visibility: Vec<bool>,
    atoms_color: Vec<Color>,
    atoms_ray_casting: Vec<u32>,

    bonds_transform: Vec<[[f32; 4]; 4]>,
    bonds_color: Vec<Color>,
    bonds_ray_casting: Vec<u32>,

    pub radius: f32,
    pub transform: Mat4<f32>,
    pub atoms_instance_buffer: wgpu::Buffer,
    pub bonds_instance_buffer: wgpu::Buffer,
}

impl Molecule {
    pub fn new(device: &wgpu::Device, config: &Config, atomic_coordinates: &AtomicCoordinates) -> Result<Self, String> {
        let mut atoms_transform = Vec::new();
        let mut atoms_visibility = Vec::new();
        let mut atoms_color = Vec::new();
        let mut atoms_ray_casting = Vec::new();
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
            let atom = config
                .style
                .atoms
                .get(&atomic_coordinates.atomic_num[i])
                .ok_or(format!(
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
            atoms_color.push(atom.color);
            atoms_ray_casting.push(1);
        }

        let mut transform = Mat4::new();
        transform.translate(-center);

        let data: Vec<InstanceData> = atoms_transform
            .iter()
            .zip(atoms_color.iter())
            .zip(atoms_visibility.iter())
            .zip(atoms_ray_casting.iter())
            .filter_map(|(((transform, color), visible), rc_type)| {
                if *visible {
                    Some(InstanceData {
                        model_matrix: *transform,
                        color: *color,
                        ray_casting_type: *rc_type,
                    })
                } else {
                    None
                }
            })
            .collect();

        let atoms_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atoms Instance Buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bond_radius = config.style.bond.radius;
        let mut bonds_transform = Vec::new();
        let mut bonds_color = Vec::new();
        let mut bonds_ray_casting = Vec::new();
        let bonds_list = bonds::build(atomic_coordinates, config.style.geom_bond_tolerance);
        for bond in bonds_list {
            let atom_1 = config
                .style
                .atoms
                .get(&atomic_coordinates.atomic_num[bond.atom_index_1])
                .ok_or(format!(
                    "Atom not found for atomic number: {}",
                    atomic_coordinates.atomic_num[bond.atom_index_1]
                ))?;

            let atom_2 = config
                .style
                .atoms
                .get(&atomic_coordinates.atomic_num[bond.atom_index_2])
                .ok_or(format!(
                    "Atom not found for atomic number: {}",
                    atomic_coordinates.atomic_num[bond.atom_index_2]
                ))?;

            let computed_bonds = get_bonds(
                Vec3::new(
                    atomic_coordinates.x[bond.atom_index_1] as f32,
                    atomic_coordinates.y[bond.atom_index_1] as f32,
                    atomic_coordinates.z[bond.atom_index_1] as f32,
                ),
                atom_1.radius,
                atom_1.color,
                Vec3::new(
                    atomic_coordinates.x[bond.atom_index_2] as f32,
                    atomic_coordinates.y[bond.atom_index_2] as f32,
                    atomic_coordinates.z[bond.atom_index_2] as f32,
                ),
                atom_2.radius,
                atom_2.color,
            );

            for b in computed_bonds {
                let mut transform = b.0;
                transform.set_scale(Vec3::new(bond_radius, bond_radius, b.1));
                let matrix = transform.get_matrix().data;
                let matrix_4x4: [[f32; 4]; 4] = [
                    [matrix[0], matrix[1], matrix[2], matrix[3]],
                    [matrix[4], matrix[5], matrix[6], matrix[7]],
                    [matrix[8], matrix[9], matrix[10], matrix[11]],
                    [matrix[12], matrix[13], matrix[14], matrix[15]],
                ];
                bonds_transform.push(matrix_4x4);
                bonds_color.push(b.2);
                bonds_ray_casting.push(2);
            }
        }

        let data: Vec<InstanceData> = bonds_transform
            .iter()
            .zip(bonds_color.iter())
            .zip(bonds_ray_casting.iter())
            .filter_map(|((transform, color), rc_type)| {
                Some(InstanceData {
                    model_matrix: *transform,
                    color: *color,
                    ray_casting_type: *rc_type,
                })
            })
            .collect();

        let bonds_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bonds Instance Buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Ok(Self {
            atoms_transform,
            atoms_visibility,
            atoms_color,
            atoms_ray_casting,
            bonds_transform,
            bonds_color,
            bonds_ray_casting,
            radius: radius.sqrt(),
            transform: transform,
            atoms_instance_buffer,
            bonds_instance_buffer,
        })
    }

    pub fn atoms_instance_count(&self) -> u32 {
        self.atoms_transform.len() as u32
    }

    pub fn bonds_instance_count(&self) -> u32 {
        self.bonds_transform.len() as u32
    }
}

fn get_bonds(
    pos_1: Vec3<f32>,
    radius_1: f32,
    color_1: Color,
    pos_2: Vec3<f32>,
    radius_2: f32,
    color_2: Color,
) -> Vec<(Transform, f32, Color)> {
    let direction = (pos_2 - pos_1).normalized();
    let length = (pos_2 - pos_1).length();
    let mid_length = (length - radius_1 - radius_2) / 2.0;

    // position, direction, length, radius, color
    let mut bonds = Vec::new();

    if mid_length > 0.0 {
        let length_1 = radius_1 + mid_length;
        let length_2 = radius_2 + mid_length;
        bonds.push((pos_1, direction, length_1, color_1));
        bonds.push((pos_1 + direction * length_1, direction, length_2, color_2));
    }

    let mut result = Vec::new();
    for bond in bonds {
        let mut transform = Transform::new();
        let (pos, direction, l, color) = bond;
        let length = l / 2.0;

        let position = pos + direction * length;
        let rotation = Quaternion::rotation_to(Vec3::new(0.0, 0.0, 1.0), direction);

        transform.set_position(position);
        transform.set_rotation(rotation);
        result.push((transform, length, color));
    }
    result
}
