use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web::http::Method;
use actix_cors::Cors;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

use crate::{calculate_volume, scale_volume, calculate_weight};
use crate::{PLA_DENSITY, ABS_DENSITY, PETG_DENSITY, TPU_DENSITY};

#[derive(Deserialize)]
pub struct WeightQueryParams {
    pub x_dim: f64,
    pub y_dim: f64,
    pub z_dim: f64,
    pub infill_percentage: f64,
    pub material: Option<String>,
}

#[derive(Serialize)]
pub struct WeightResponse {
    pub weight_grams: String,
}

async fn calculate_weight_from_stl(mut payload: Multipart, query: web::Query<WeightQueryParams>) -> impl Responder {
    // Create temporary file to store the uploaded STL
    let mut temp_file = match NamedTempFile::new() {
        Ok(file) => file,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Failed to create temporary file"})),
    };
    
    // Process uploaded file
    let mut file_saved = false;
    
    while let Ok(Some(mut field)) = payload.try_next().await {
        // Check if this is a file field
        if let Some(content_disposition) = field.content_disposition() {
            if content_disposition.get_filename().is_some() {
                // Save file data to the temp file
                while let Some(chunk) = field.next().await {
                    let data = match chunk {
                        Ok(data) => data,
                        Err(_) => {
                            return HttpResponse::BadRequest().json(json!({"error": "Failed to read uploaded file"}));
                        }
                    };
                    
                    if let Err(_) = temp_file.write_all(&data) {
                        return HttpResponse::InternalServerError().json(json!({"error": "Failed to write file data"}));
                    }
                    
                    file_saved = true;
                }
            }
        }
    }
    
    if !file_saved {
        return HttpResponse::BadRequest().json(json!({"error": "No STL file was uploaded"}));
    }
    
    // Get dimensions and parameters from query
    let x_dim = query.x_dim;
    let y_dim = query.y_dim;
    let z_dim = query.z_dim;
    let infill_percentage = query.infill_percentage;
    
    // Validate infill percentage
    if infill_percentage < 0.0 || infill_percentage > 100.0 {
        return HttpResponse::BadRequest().json(json!({"error": "Infill percentage must be in the range of 0-100"}));
    }
    
    // Default to PLA if material not specified
    let material = query.material.clone().unwrap_or_else(|| "pla".to_string()).to_lowercase();
    
    // Get material density
    let material_density = match material.as_str() {
        "abs" => ABS_DENSITY,
        "petg" => PETG_DENSITY,
        "tpu" => TPU_DENSITY,
        _ => PLA_DENSITY, // Default to PLA
    };
    
    // Read the STL file
    let file = match fs::File::open(temp_file.path()) {
        Ok(file) => file,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error": "Failed to open uploaded file"}));
        }
    };
    
    let mut reader = std::io::BufReader::new(file);
    let stl = match stl_io::read_stl(&mut reader) {
        Ok(stl) => stl,
        Err(_) => {
            return HttpResponse::BadRequest().json(json!({"error": "Not a valid STL file"}));
        }
    };
    
    // Calculate volume and weight
    let original_volume = calculate_volume(&stl);
    let scaled_volume = scale_volume(original_volume, x_dim, y_dim, z_dim, &stl);
    let weight = calculate_weight(scaled_volume, infill_percentage, material_density);
    
    // Format weight to 2 decimal places
    let weight_formatted = format!("{:.2}", weight);
    
    // Return JSON response using serde_json::json! macro
    HttpResponse::Ok().json(json!({
        "weight_grams": weight_formatted
    }))
}

// Handler for OPTIONS requests
async fn options_handler() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub async fn start_api_server() -> std::io::Result<()> {
    println!("Starting API server on http://127.0.0.1:8080");
    HttpServer::new(|| {
        // Configure CORS middleware
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(cors) // Apply CORS middleware
            .route("/calculate_weight", web::post().to(calculate_weight_from_stl))
            .route("/calculate_weight", web::route().method(Method::OPTIONS).to(options_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}