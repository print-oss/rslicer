use std::env;
use std::fs::File;
use std::io::BufReader;
use stl_io::{read_stl, IndexedMesh};
use serde_json::{json, to_string};

// g/cm³
const PLA_DENSITY: f64 = 1.24;
const ABS_DENSITY: f64 = 1.04;
const PETG_DENSITY: f64 = 1.27;
const TPU_DENSITY: f64 = 1.21;

mod api;

fn calculate_volume(mesh: &IndexedMesh) -> f64 {
    let mut volume: f64 = 0.0;
    for face in &mesh.faces {
        let v0 = mesh.vertices[face.vertices[0]];
        let v1 = mesh.vertices[face.vertices[1]];
        let v2 = mesh.vertices[face.vertices[2]];

        let v0 = [v0[0] as f64, v0[1] as f64, v0[2] as f64];
        let v1 = [v1[0] as f64, v1[1] as f64, v1[2] as f64];
        let v2 = [v2[0] as f64, v2[1] as f64, v2[2] as f64];
        
        let v321 = v2[0] * v1[1] * v0[2];
        let v231 = v1[0] * v2[1] * v0[2];
        let v312 = v2[0] * v0[1] * v1[2];
        let v132 = v0[0] * v2[1] * v1[2];
        let v213 = v1[0] * v0[1] * v2[2];
        let v123 = v0[0] * v1[1] * v2[2];

        volume += (1.0 / 6.0) * (-v321 + v231 + v312 - v132 - v213 + v123);
    }
    volume.abs()
}

fn scale_volume(original_volume: f64, desired_x: f64, desired_y: f64, desired_z: f64, mesh: &IndexedMesh) -> f64 {
    // Calculate model's current bounding box
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut min_z = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut max_z = f64::MIN;
    
    for vertex in &mesh.vertices {
        let x = vertex[0] as f64;
        let y = vertex[1] as f64;
        let z = vertex[2] as f64;
        
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        min_z = min_z.min(z);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
        max_z = max_z.max(z);
    }
    
    // Calculate current dimensions
    let current_x = max_x - min_x;
    let current_y = max_y - min_y;
    let current_z = max_z - min_z;
    
    // Calculate scaling factors
    let scale_x = desired_x / current_x;
    let scale_y = desired_y / current_y;
    let scale_z = desired_z / current_z;
    
    // Scale volume - volume scales with the cube of the scaling factor
    let volume_scale = scale_x * scale_y * scale_z;
    original_volume * volume_scale
}

fn calculate_weight(volume_mm3: f64, infill_percentage: f64, material_density: f64) -> f64 {
    // Convert volume from mm³ to cm³ (divide by 1000)
    let volume_cm3 = volume_mm3 / 1000.0;
    
    // Calculate effective volume based on infill and shell
    let shell_thickness = 0.8; // Typical 2 perimeters at 0.4mm each
    let solid_layers_factor = 0.15; // Top/bottom solid layers (approx 15% of volume)
    
    // Effective volume = shell volume + (internal volume * infill percentage)
    let shell_volume_percentage = shell_thickness / 10.0; // Rough approximation of shell as percentage
    let effective_volume = (shell_volume_percentage + solid_layers_factor) * volume_cm3 + 
                           ((1.0 - shell_volume_percentage - solid_layers_factor) * volume_cm3 * (infill_percentage / 100.0));
    
    // Weight = volume * density
    effective_volume * material_density
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Special flag to start API server
    if args.len() > 1 && args[1] == "--api" {
        return api::start_api_server().await;
    }
    
    if args.len() < 6 {
        eprintln!("Usage: cargo run <stl-file-path> <x-dim> <y-dim> <z-dim> <infill_percentage> [material]");
        eprintln!("       cargo run --api  (to start API server)");
        eprintln!("Materials: pla (default), abs, petg, tpu");
        return Ok(());
    }

    let file_path = &args[1];
    let x_dim: f64 = args[2].parse().expect("Invalid x dimension");
    let y_dim: f64 = args[3].parse().expect("Invalid y dimension");
    let z_dim: f64 = args[4].parse().expect("Invalid z dimension");
    let infill_percentage: f64 = args[5].parse().expect("Invalid infill percentage");
    
    // Default to PLA if material not specified
    let material = if args.len() > 6 { args[6].to_lowercase() } else { "pla".to_string() };
    
    let material_density = match material.as_str() {
        "abs" => ABS_DENSITY,
        "petg" => PETG_DENSITY,
        "tpu" => TPU_DENSITY,
        _ => PLA_DENSITY, // Default to PLA
    };

    if infill_percentage < 0.0 || infill_percentage > 100.0 {
        eprintln!("Infill percentage must be in the range of 0-100.");
        return Ok(());
    }

    let file = File::open(file_path).expect("Failed to open file");
    let mut reader = BufReader::new(file);
    let stl = read_stl(&mut reader).expect("Failed to read STL file");

    let original_volume = calculate_volume(&stl);
    let scaled_volume = scale_volume(original_volume, x_dim, y_dim, z_dim, &stl);
    let weight = calculate_weight(scaled_volume, infill_percentage, material_density);
    
    // Format weight to 2 decimal places and return as JSON
    let weight_formatted = format!("{:.2}", weight);
    let result = json!({ "weight_grams": weight_formatted });
    
    // Print the JSON result without pretty printing
    println!("{}", to_string(&result).expect("Failed to serialize JSON"));
    
    Ok(())
}
