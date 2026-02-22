use bitvector_search::ContextEngine;
use std::path::Path;
use std::io::Write;
use std::sync::Arc;
use std::fs::{File, read_dir};
use axum::{
  extract::{Multipart, State, DefaultBodyLimit},
  routing::post,
  Router,
  response::IntoResponse,
  http::{StatusCode, header},
};

pub struct AppState {
  pub engine: ContextEngine,
}

#[tokio::main]
async fn main() {
  let engine = ContextEngine::new("sentence-transformers/all-MiniLM-L6-v2").expect("Failed to initialize ContextEngine");
  let shared_state = Arc::new(AppState { engine });

  let app = Router::new()
    .route("/process", post(handle_upload))
    .layer(DefaultBodyLimit::disable())
    .with_state(shared_state);

  let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
  let addr: String = format!("0.0.0.0:{}", port).parse().unwrap();

  println!("Server listening on {}", addr);
  let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
  axum::serve(listener, app).await.unwrap();
}

async fn handle_upload(State(state): State<Arc<AppState>>, 
  mut multipart: Multipart) -> Result<impl IntoResponse, (StatusCode, String)> {
  let temp_zip_path = "/tmp/upload.zip";
  let temp_input = "/tmp/input.csv";
  let temp_keywrd = "/tmp/keywords.csv";
  let temp_output = "/tmp/output.csv";
  let mut target_index: usize = 0;
  let mut zip_received = false;

  while let Some(field) = multipart.next_field().await.map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))? {
    let filename = field.file_name().unwrap_or("unknown_0.zip").to_string();
    if let Some(stem) = Path::new(&filename).file_stem() {
      let extract_dir = "/tmp/extracted_data";
      if let Some(s) = stem.to_str() {
        if let Some(last_char) = s.chars().last() {
          if let Some(digit) = last_char.to_digit(10) {
            target_index = digit as usize;
            println!("Detected Index from filename '{}': {}", filename, target_index);
          }
        }
      }
    }

    let data = field.bytes().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut file = File::create(temp_zip_path).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    file.write_all(&data).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    zip_received = true;
  }

  if !zip_received {
    return Err((StatusCode::BAD_REQUEST, "No zip file uploaded".to_string()));
  }

  let state_clone = state.clone();
  let result = tokio::task::spawn_blocking(move || {
    let extract_dir = "/tmp/extracted_data";
    let _ = std::fs::create_dir_all(extract_dir);

    let file = File::open(temp_zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    archive.extract_unwrapped_root_dir(extract_dir, zip::read::root_dir_common_filter).map_err(|e| format!("Extraction failed: {}", e))?;

    let mut found_keywords = false;
    let mut found_data = false;
    
    println!("Extracting zip to disk...");
    find_csvs_on_disk(Path::new(extract_dir), temp_keywrd, temp_input, &mut found_keywords, &mut found_data)?;

    if !found_keywords || !found_data {
      return Err("Zipfile must contain two CSV files: one with 'key' in the name and one data file.".to_string());
    }

    println!("Processing started for index: {}", target_index);
    let filters = state_clone.engine.process_filters(temp_keywrd).map_err(|e| e.to_string())?;
    state_clone.engine.process_csv(temp_input, target_index, temp_output, &filters).map_err(|e| e.to_string())?;

    Ok(())
  }).await.unwrap();

  match result {
    Ok(_) => {
      let csv_content = std::fs::read(temp_output)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

      let mut zip_buffer = Vec::new();
      {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_buffer));
        let options = zip::write::SimpleFileOptions::default()
          .compression_method(zip::CompressionMethod::Deflated)
          .unix_permissions(0o755);

        zip.start_file("results.csv", options)
          .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        zip.write_all(&csv_content)
          .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        zip.finish()
          .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }

      let headers = [
        (header::CONTENT_TYPE, "application/zip"),
        (header::CONTENT_DISPOSITION, "attachment; filename=\"results.zip\""),
      ];

      Ok((headers, zip_buffer))
    },
    Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
  }
}

fn find_csvs_on_disk(dir: &Path, temp_keywrd: &str, temp_input: &str, found_keywords: &mut bool, found_data: &mut bool) -> Result<(), String> {
  println!("Entry path: {}", dir.display());
  if dir.is_dir() {
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
      let entry = entry.map_err(|e| e.to_string())?;
      let path = entry.path();
      println!("Found directory: {}", path.display());
      
      if path.is_dir() {
        println!("Moving to sub-directory: {}", path.display());
        find_csvs_on_disk(&path, temp_keywrd, temp_input, found_keywords, found_data)?;
      } else {
        println!("Found non-directory: {}", path.display());
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();

        if name.contains("__macosx") || name.contains(".ds_store") { continue; }

        if name.contains("key") && name.ends_with(".csv") {
          std::fs::copy(&path, temp_keywrd).map_err(|e| e.to_string())?;
          println!("Keyword file copied: {}", name);
          *found_keywords = true;
        } else if name.ends_with(".csv") {
          std::fs::copy(&path, temp_input).map_err(|e| e.to_string())?;
          println!("Data file copied: {}", name);
          *found_data = true;
        }
      }
    }
  }
  Ok(())
}
