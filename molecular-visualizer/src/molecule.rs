use std::collections::HashSet;

use shared_lib::periodic_table::get_element_by_number;
use shared_lib::types::AtomicCoordinates;

use super::atom::{Atom, AtomInfo};
use super::bond::Bond;
use super::bonds;
use super::config::Config;
use super::core::char_instance_data::create_char_instance_buffer;
use super::core::instance_data::create_instance_buffer;
use super::core::{CharInstanceData, InstanceBuffer, InstanceData, Mat4, Vec3, mesh_objects};
use super::types::Color;
use super::utils::id_to_color;
use super::vertex_buffer::VertexBuffer;

pub struct Molecule {
    data: AtomicCoordinates,

    cube_vb: VertexBuffer,

    atoms: Vec<Atom>,
    bonds: Vec<Bond>,

    pub center: Vec3<f32>,

    pub radius: f32,
    pub transform: Mat4<f32>,
    pub atoms_instance_buffer: InstanceBuffer,
    pub atom_labels_instance_buffer: InstanceBuffer,
    pub atom_selections_instance_buffer: InstanceBuffer,
    pub bonds_instance_buffer: InstanceBuffer,

    highlighted_atom: usize, // atom (index starts from 1) under cursor, 0 = no atoms under cursor
    selected_atoms: HashSet<usize>,
}

impl Molecule {
    pub fn new(
        device: &wgpu::Device,
        config: &Config,
        atomic_coordinates: AtomicCoordinates,
        font_atlas: &super::core::FontAtlas,
    ) -> Result<Self, String> {
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
                (i + 1) as i32,
                atomic_coordinates.atomic_num[i],
                position,
                atom.radius,
                atom.color,
                id_to_color(i + 1),
                config.style.selected_atom.color,
                config.style.selected_atom.scale_factor,
                config.style.atom_label.label_visible,
                config.style.atom_label.symbol_visible,
                config.style.atom_label.number_visible,
            ));
        }

        let bond_thickness = config.style.bond.thickness;
        let mut bonds = Vec::new();
        let bonds_list = bonds::build(&atomic_coordinates, config.style.geom_bond_tolerance);
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

        let (
            atoms_instance_buffer,
            atom_labels_instance_buffer,
            atom_labels_instance_count,
            atom_selections_instance_buffer,
            num_selected_atoms,
        ) = Self::create_atoms_instance_buffers(&atoms, device, font_atlas, config);

        Ok(Self {
            data: atomic_coordinates,
            cube_vb: VertexBuffer::new(device, &mesh_objects::cube::create(2.0)),
            atoms_instance_buffer: InstanceBuffer::new(atoms_instance_buffer, atoms.len()),
            atom_labels_instance_buffer: InstanceBuffer::new(atom_labels_instance_buffer, atom_labels_instance_count),
            atom_selections_instance_buffer: InstanceBuffer::new(atom_selections_instance_buffer, num_selected_atoms),
            bonds_instance_buffer: InstanceBuffer::new(Self::create_bonds_instance_buffer(&bonds, device), bonds.len()),
            atoms,
            bonds,
            center,
            radius: radius.sqrt(),
            transform,
            highlighted_atom: 0,
            selected_atoms: HashSet::new(),
        })
    }

    fn create_atoms_instance_buffers(
        atoms: &Vec<Atom>,
        device: &wgpu::Device,
        font_atlas: &super::core::FontAtlas,
        config: &Config,
    ) -> (wgpu::Buffer, wgpu::Buffer, usize, wgpu::Buffer, usize) {
        let mut atoms_data: Vec<InstanceData> = Vec::new();
        let mut atom_labels_data: Vec<CharInstanceData> = Vec::new();
        let mut spheres_data: Vec<InstanceData> = Vec::new();
        for atom in atoms {
            if atom.visible {
                atoms_data.push(atom.get_instance_data(false));
                if atom.selected {
                    spheres_data.push(atom.get_instance_data(true));
                }
                if atom.label_visible && (atom.symbol_visible || atom.number_visible) {
                    let labels = atom.get_label_instance_data(
                        config.style.atom_label.color,
                        config.style.atom_label.size / 100.0, // convert to angstroms
                        config.style.atom_label.offset,
                        font_atlas,
                    );
                    for (_, data) in labels {
                        atom_labels_data.push(data);
                    }
                }
            }
        }

        let atom_labels_instance_count = atom_labels_data.len();
        let atom_labels_instance_buffer =
            create_char_instance_buffer(&atom_labels_data, device, "Atom Labels Instance Buffer");

        (
            create_instance_buffer(&atoms_data, device, "Atoms Instance Buffer"),
            atom_labels_instance_buffer,
            atom_labels_instance_count,
            create_instance_buffer(&spheres_data, device, "Spheres Instance Buffer"),
            spheres_data.len(),
        )
    }

    fn create_bonds_instance_buffer(bonds: &Vec<Bond>, device: &wgpu::Device) -> wgpu::Buffer {
        create_instance_buffer(
            &bonds
                .iter()
                .filter(|item| item.visible)
                .map(|item| item.get_instance_data())
                .collect(),
            device,
            "Bonds Instance Buffer",
        )
    }

    /// Returns (atom_info, needs_render)
    pub fn highlight_atom(
        &mut self,
        index: usize,
        device: &wgpu::Device,
        font_atlas: &super::core::FontAtlas,
        config: &Config,
    ) -> (Option<AtomInfo>, bool) {
        if index == 0 || index > self.atoms.len() {
            // No atom under cursor - clear highlight if any
            if self.highlighted_atom > 0 {
                self.atoms[self.highlighted_atom - 1].highlighted = false;
                self.highlighted_atom = 0;
                (
                    self.atoms_instance_buffer.buffer,
                    self.atom_labels_instance_buffer.buffer,
                    self.atom_labels_instance_buffer.count,
                    self.atom_selections_instance_buffer.buffer,
                    self.atom_selections_instance_buffer.count,
                ) = Self::create_atoms_instance_buffers(&self.atoms, device, font_atlas, config);
                return (None, true);
            }
            return (None, false);
        }

        // Same atom already highlighted - return info without updating buffer
        if self.highlighted_atom == index {
            let element = match get_element_by_number(self.atoms[index - 1].atomic_number) {
                Some(e) => e,
                None => return (None, false),
            };
            return (Some(AtomInfo::new(element.symbol.to_string(), index)), false);
        }

        let element = match get_element_by_number(self.atoms[index - 1].atomic_number) {
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
        (
            self.atoms_instance_buffer.buffer,
            self.atom_labels_instance_buffer.buffer,
            self.atom_labels_instance_buffer.count,
            self.atom_selections_instance_buffer.buffer,
            self.atom_selections_instance_buffer.count,
        ) = Self::create_atoms_instance_buffers(&self.atoms, device, font_atlas, config);
        (Some(AtomInfo::new(element.symbol.to_string(), index)), true)
    }

    pub fn toggle_atom_selection(
        &mut self,
        index: usize,
        device: &wgpu::Device,
        font_atlas: &super::core::FontAtlas,
        config: &Config,
    ) -> bool {
        if index == 0 || index > self.atoms.len() {
            // No atom under cursor - clear highlight if any
            return false;
        }

        if self.atoms[index - 1].selected {
            self.selected_atoms.remove(&(index - 1));
        } else {
            self.selected_atoms.insert(index - 1);
        }

        self.atoms[index - 1].toggle_selection();
        (
            self.atoms_instance_buffer.buffer,
            self.atom_labels_instance_buffer.buffer,
            self.atom_labels_instance_buffer.count,
            self.atom_selections_instance_buffer.buffer,
            self.atom_selections_instance_buffer.count,
        ) = Self::create_atoms_instance_buffers(&self.atoms, device, font_atlas, config);
        true
    }

    pub fn max_coordinate(&self) -> f32 {
        self.atoms.iter().fold(0.0_f32, |acc, atom| {
            acc.max(atom.position.x.abs())
                .max(atom.position.y.abs())
                .max(atom.position.z.abs())
        })
    }

    pub fn set_atom_labels_symbol_visible(&mut self, value: bool) {
        for atom in &mut self.atoms {
            atom.symbol_visible = value;
        }
    }

    pub fn set_atom_labels_number_visible(&mut self, value: bool) {
        for atom in &mut self.atoms {
            atom.number_visible = value;
        }
    }

    pub fn set_all_atom_labels_visible(&mut self, value: bool) {
        for atom in &mut self.atoms {
            atom.label_visible = value;
        }
    }

    pub fn set_selected_atom_labels_visible(&mut self, value: bool) {
        for index in &self.selected_atoms {
            let atom = &mut self.atoms[*index];
            atom.label_visible = value;
        }
    }

    pub fn toggle_all_atom_labels_visible(&mut self) -> bool {
        let all_visible = self.atoms.iter().all(|atom| atom.label_visible);
        let next = !all_visible;
        for atom in &mut self.atoms {
            atom.label_visible = next;
        }
        next
    }

    pub fn toggle_selected_atom_labels_visible(&mut self) {
        if self.selected_atoms.is_empty() {
            return;
        }
        let all_visible = self
            .selected_atoms
            .iter()
            .all(|&index| self.atoms[index].label_visible);
        let next = !all_visible;
        for &index in &self.selected_atoms {
            self.atoms[index].label_visible = next;
        }
    }

    pub fn update_atom_labels(&mut self, device: &wgpu::Device, font_atlas: &super::core::FontAtlas, config: &Config) {
        let (count, buffer) = Self::create_atom_labels_instance_buffers(&self.atoms, device, font_atlas, config);
        self.atom_labels_instance_buffer = InstanceBuffer::new(buffer, count);
    }

    fn create_atom_labels_instance_buffers(
        atoms: &Vec<Atom>,
        device: &wgpu::Device,
        font_atlas: &super::core::FontAtlas,
        config: &Config,
    ) -> (usize, wgpu::Buffer) {
        let mut atom_labels_data: Vec<CharInstanceData> = Vec::new();
        for atom in atoms {
            if atom.visible && atom.label_visible && (atom.symbol_visible || atom.number_visible) {
                let labels = atom.get_label_instance_data(
                    config.style.atom_label.color,
                    config.style.atom_label.size / 100.0, // convert to angstroms
                    config.style.atom_label.offset,
                    font_atlas,
                );
                for (_, data) in labels {
                    atom_labels_data.push(data);
                }
            }
        }

        (
            atom_labels_data.len(),
            create_char_instance_buffer(&atom_labels_data, device, "Atom Labels Instance Buffer"),
        )
    }

    pub fn render_atoms_and_bonds(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.cube_vb.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.cube_vb.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        // Render atoms (opaque)
        if self.atoms_instance_buffer.count > 0 {
            render_pass.set_vertex_buffer(1, self.atoms_instance_buffer.buffer.slice(..));
            render_pass.draw_indexed(
                0..self.cube_vb.num_indices,
                0,
                0..self.atoms_instance_buffer.count as u32,
            );
        }

        // Render bonds (opaque)
        if self.bonds_instance_buffer.count > 0 {
            render_pass.set_vertex_buffer(1, self.bonds_instance_buffer.buffer.slice(..));
            render_pass.draw_indexed(
                0..self.cube_vb.num_indices,
                0,
                0..self.bonds_instance_buffer.count as u32,
            );
        }
    }

    pub fn render_bounding_spheres(&self, render_pass: &mut wgpu::RenderPass) {
        // Render bounding spheres
        if self.atom_selections_instance_buffer.count > 0 {
            render_pass.set_vertex_buffer(0, self.cube_vb.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.cube_vb.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_vertex_buffer(1, self.atom_selections_instance_buffer.buffer.slice(..));
            render_pass.draw_indexed(
                0..self.cube_vb.num_indices,
                0,
                0..self.atom_selections_instance_buffer.count as u32,
            );
        }
    }

    pub fn render_labels(&self, render_pass: &mut wgpu::RenderPass, rect_vb: &VertexBuffer) {
        if self.atom_labels_instance_buffer.count > 0 {
            render_pass.set_vertex_buffer(1, self.atom_labels_instance_buffer.buffer.slice(..));
            render_pass.draw_indexed(
                0..rect_vb.num_indices,
                0,
                0..self.atom_labels_instance_buffer.count as u32,
            );
        }
    }

    pub fn render_picking_frame(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_vertex_buffer(0, self.cube_vb.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.cube_vb.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        // Render atoms only (bonds don't have picking IDs)
        render_pass.set_vertex_buffer(1, self.atoms_instance_buffer.buffer.slice(..));
        render_pass.draw_indexed(
            0..self.cube_vb.num_indices,
            0,
            0..self.atoms_instance_buffer.count as u32,
        );
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
