use shared_lib::periodic_table::get_element_by_number;
use shared_lib::types::AtomicCoordinates;

pub fn build(data: &AtomicCoordinates, geom_bond_tolerance: f64) -> Vec<(usize, usize)> {
    // Optimized implementation using Spatial Sorting (Sweep and Prune).
    // Complexity: O(N log N) sorting + O(N * k) search, where k is small.

    // 1. Pre-filtering and data preparation
    // Collect a list of tuples for each valid atom.
    // This avoids accessing lists by index inside the hot loop.
    // Structure: (x, y, z, radius, original_index)
    let mut atoms: Vec<(f64, f64, f64, f64, usize)> = Vec::new();

    // Find the global maximum radius for computing limit
    // (iterate through radius table or atoms - atoms are more reliable if table is huge)
    let mut max_radius: f64 = 0.0;

    // Iterate once for preparation
    for i in 0..data.atomic_num.len() {
        let atomic_number = data.atomic_num[i];
        if atomic_number < 1 {
            continue;
        }

        let element = match get_element_by_number(atomic_number) {
            Some(element) => element,
            None => continue,
        };

        let radius = element.covalent_radius;
        if radius > max_radius {
            max_radius = radius;
        }

        atoms.push((data.x[i], data.y[i], data.z[i], element.covalent_radius, i));
    }

    // 2. Sort by X coordinate
    // This is a key step for the Sweep-and-Prune algorithm
    atoms.sort_by(|a, b| a.0.total_cmp(&b.0));

    // 3. Main bond search loop
    let mut result = Vec::new();
    let tol_factor = 1.0 + geom_bond_tolerance;
    let n_atoms = atoms.len();

    for i in 0..n_atoms {
        let ai = atoms[i];
        let xi = ai.0;
        let yi = ai.1;
        let zi = ai.2;
        let ri = ai.3;
        let origin_i = ai.4;

        // Search limit along X axis for the current atom.
        // If a neighbor along X is farther than this value, then any other neighbor
        // in the sorted list will be farther.
        let limit = (ri + max_radius) * tol_factor;

        // Inner loop: only look forward
        for j in i + 1..n_atoms {
            let aj = atoms[j];
            let xj = aj.0;
            let yj = aj.1;
            let zj = aj.2;
            let rj = aj.3;
            let origin_j = aj.4;

            // --- 1. X-axis culling (Sweep Check) ---
            let dx = xj - xi;

            // Most important line: break the inner loop
            if dx > limit {
                break;
            }

            // --- 2. Y and Z axis culling ---
            let dy = yj - yi;
            if dy > limit || dy < -limit {
                continue;
            }

            let dz = zj - zi;
            if dz > limit || dz < -limit {
                continue;
            }

            // --- 3. Exact check (Squared Distance) ---
            let cutoff = (ri + rj) * tol_factor;
            let dist_sq = dx * dx + dy * dy + dz * dz;

            if dist_sq < cutoff * cutoff {
                // Save the result
                // Usually it's conventional to return (larger_index, smaller_index) or vice versa
                // Sort the pair for consistency
                if origin_i > origin_j {
                    result.push((origin_i, origin_j))
                } else {
                    result.push((origin_j, origin_i))
                }
            }
        }
    }

    result
}
