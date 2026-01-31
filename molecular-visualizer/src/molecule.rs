use std::collections::HashSet;

use shared_lib::periodic_table::get_element_by_number;
use shared_lib::types::AtomicCoordinates;
use wgpu::util::DeviceExt;

use super::atom::{Atom, AtomInfo};
use super::bond::Bond;
use super::bonds;
use super::config::Config;
use super::core::mesh::InstanceData;
use super::core::{Mat4, Vec3};
use super::types::Color;
use super::utils::id_to_color;

pub struct Molecule {
    atoms: Vec<Atom>,
    bonds: Vec<Bond>,

    pub radius: f32,
    pub transform: Mat4<f32>,
    pub atoms_instance_buffer: wgpu::Buffer,
    pub bonds_instance_buffer: wgpu::Buffer,

    highlighted_atom: usize, // atom (index starts from 1) under cursor, 0 = no atoms under cursor
    selected_atoms: HashSet<usize>,
}

impl Molecule {
    pub fn new(device: &wgpu::Device, config: &Config, atomic_coordinates: &AtomicCoordinates) -> Result<Self, String> {
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

        let mut transform = Mat4::new();
        transform.translate(-center);

        let mut atoms = Vec::new();
        let atoms_style = &config.style.atoms;
        for i in 0..num_atoms {
            let atom = atoms_style.get(&atomic_coordinates.atomic_num[i]).ok_or(format!(
                "Atom not found for atomic number: {}",
                atomic_coordinates.atomic_num[i]
            ))?;

            let position = Vec3::new(
                atomic_coordinates.x[i] as f32,
                atomic_coordinates.y[i] as f32,
                atomic_coordinates.z[i] as f32,
            );

            radius = radius.max((position - center).length_squared() + atom.radius);

            atoms.push(Atom::new(
                atomic_coordinates.atomic_num[i],
                position,
                atom.radius,
                atom.color,
                id_to_color(i + 1),
            ));
        }

        let bond_thickness = config.style.bond.thickness;
        let mut bonds = Vec::new();
        let bonds_list = bonds::build(atomic_coordinates, config.style.geom_bond_tolerance);
        for bond in bonds_list {
            let atom_1 = &atoms[bond.atom_index_1];
            let atom_2 = &atoms[bond.atom_index_2];

            let computed_bonds = get_bonds(
                atom_1.position,
                atom_1.radius,
                atom_1.color,
                atom_2.position,
                atom_2.radius,
                atom_2.color,
            );

            for b in computed_bonds {
                bonds.push(Bond::new(b.0, b.1, bond_thickness, b.2, b.3));
            }
        }

        Ok(Self {
            atoms_instance_buffer: Self::get_atoms_instance_buffer(&atoms, device),
            bonds_instance_buffer: Self::get_bonds_instance_buffer(&bonds, device),
            atoms,
            bonds,
            radius: radius.sqrt(),
            transform: transform,
            highlighted_atom: 0,
            selected_atoms: HashSet::new(),
        })
    }

    fn get_atoms_instance_buffer(data: &Vec<Atom>, device: &wgpu::Device) -> wgpu::Buffer {
        let data: Vec<InstanceData> = data
            .iter()
            .filter_map(|item| {
                if item.visible {
                    Some(item.get_instance_data())
                } else {
                    None
                }
            })
            .collect();

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn get_bonds_instance_buffer(data: &Vec<Bond>, device: &wgpu::Device) -> wgpu::Buffer {
        let data: Vec<InstanceData> = data
            .iter()
            .filter_map(|item| {
                if item.visible {
                    Some(item.get_instance_data())
                } else {
                    None
                }
            })
            .collect();

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    pub fn atoms_instance_count(&self) -> u32 {
        self.atoms.len() as u32
    }

    pub fn bonds_instance_count(&self) -> u32 {
        self.bonds.len() as u32
    }

    /// Returns (atom_info, needs_render)
    pub fn highlight_atom(&mut self, index: usize, device: &wgpu::Device) -> (Option<AtomInfo>, bool) {
        if index == 0 || index > self.atoms.len() {
            // No atom under cursor - clear highlight if any
            if self.highlighted_atom > 0 {
                self.atoms[self.highlighted_atom - 1].highlighted = false;
                self.highlighted_atom = 0;
                self.atoms_instance_buffer = Self::get_atoms_instance_buffer(&self.atoms, device);
                return (None, true);
            }
            return (None, false);
        }

        // Same atom already highlighted - return info without updating buffer
        if self.highlighted_atom == index {
            let element = match get_element_by_number(self.atoms[index - 1].number) {
                Some(e) => e,
                None => return (None, false),
            };
            return (Some(AtomInfo::new(element.symbol.to_string(), index)), false);
        }

        let element = match get_element_by_number(self.atoms[index - 1].number) {
            Some(e) => e,
            None => return (None, false),
        };

        // Reset previous highlighted atom
        if self.highlighted_atom > 0 {
            self.atoms[self.highlighted_atom - 1].highlighted = false;
        }

        // Set new highlighted atom
        self.atoms[index - 1].highlighted = true;
        self.highlighted_atom = index;
        self.atoms_instance_buffer = Self::get_atoms_instance_buffer(&self.atoms, device);
        (Some(AtomInfo::new(element.symbol.to_string(), index)), true)
    }
}

fn get_bonds(
    pos_1: Vec3<f32>,
    radius_1: f32,
    color_1: Color,
    pos_2: Vec3<f32>,
    radius_2: f32,
    color_2: Color,
) -> Vec<(Vec3<f32>, Vec3<f32>, f32, Color)> {
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
        let (pos, direction, l, color) = bond;
        let length = l / 2.0;

        result.push((pos + direction * length, direction, length, color));
    }
    result
}
