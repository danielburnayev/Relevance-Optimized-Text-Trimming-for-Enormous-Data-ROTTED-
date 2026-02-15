use rayon::prelude::*;
use embed_anything::embeddings::embed::Embedder;
use std::collections::HashMap;
use ndarray::{Array1, Array2};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use futures::executor::block_on;
use std::fs::File;
use std::io::{Write, BufReader, BufWriter};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

pub struct RandomProjector {
  matrix: Array2<f32>,
}

impl RandomProjector {
  pub fn new(input_dim: usize, output_dim: usize) -> Self {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let data: Vec<f32> = (0..input_dim * output_dim)
      .map(|_| rng.sample(rand_distr::StandardNormal))
      .collect();
    let matrix = Array2::from_shape_vec((output_dim, input_dim), data).unwrap();
    Self { matrix }
  }

  pub fn project_to_hash(&self, vector: &Array1<f32>) -> u64 {
    let projected = self.matrix.dot(vector);
    let mut hash: u64 = 0;
    for (i, &val) in projected.iter().enumerate() {
      if val > 0.0 { hash |= 1 << i; }
    }
    hash
  }
}

pub struct MaskBaker;
impl MaskBaker {
  pub fn bake_mask(vectors: Vec<Array1<f32>>, proj_mat: &RandomProjector) -> u64 {
    let n = vectors.len() as f32;
    let dim = vectors[0].len();
    let mut centroid = Array1::<f32>::zeros(dim);
    for v in vectors { centroid += &v; }
    centroid /= n;
    proj_mat.project_to_hash(&centroid)
  }
}

struct ContextFilter {
  name: String,
  mask: u64,
  threshold: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ConversationItem {
  number: u32,
  question: String,
  answer: String,
}

// 2. Output format
// This combines the original data + our analysis results
#[derive(Serialize)]
struct OutputMatch {
  matched_category: String,
  hamming_distance: u32,
  #[serde(flatten)] // This merges "number", "question", "answer" into the top level
  original_data: ConversationItem,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let projector = RandomProjector::new(384, 64); 
  let embedder = Embedder::from_pretrained_hf(
    "sentence-transformers/all-MiniLM-L6-v2", 
    Some("main"), None, None
  )?;

  let mut filters = Vec::new();
  filters.push(create_filter("Together", vec!["meetup", "hangout", "sleepover", "go out"], &embedder, &projector));
  filters.push(create_filter("Conflict", vec!["crash", "angry", "sick", "pressure", "hurt"], &embedder, &projector));

  let input_path = "text_messages.json";
  let output_path = "filtered_output.json";

  let output_file = File::create(output_path)?;
  let mut writer = BufWriter::new(output_file);

  // START JSON ARRAY
  writer.write_all(b"[\n")?;

  let file = File::open(input_path)?;
  let reader = BufReader::new(file);
  let stream_iterator = serde_json::Deserializer::from_reader(reader).into_iter::<ConversationItem>();

  let batch_size = 512;
  println!("--- Opening JSON Stream: {} ---", input_path);

  let mut batch_items: Vec<ConversationItem> = Vec::with_capacity(batch_size);

  let mut total_processed = 0;
  let mut total_hits = 0;
  let mut is_first_write = true; // Flag to handle commas correctly

  for item_result in stream_iterator {
    if let Ok(item) = item_result {
      batch_items.push(item);

      if batch_items.len() >= batch_size {
        // Process the batch
        let hits = process_batch_structs(&batch_items, &embedder, &projector, &filters);

        // Write hits to JSON stream
        for hit in hits {
          if !is_first_write {
            writer.write_all(b",\n")?;
          }
          serde_json::to_writer(&mut writer, &hit)?;
          is_first_write = false;
          total_hits += 1;
        }

        total_processed += batch_items.len();
        println!("Processed: {} | Hits Found: {}", total_processed, total_hits);
        batch_items.clear();
      }
    }
  }

  // Flush remaining
  if !batch_items.is_empty() {
    let hits = process_batch_structs(&batch_items, &embedder, &projector, &filters);
    for hit in hits {
      if !is_first_write {
        writer.write_all(b",\n")?;
      }
      serde_json::to_writer(&mut writer, &hit)?;
      is_first_write = false;
      total_hits += 1;
    }
  }

  // CLOSE JSON ARRAY
  writer.write_all(b"\n]")?;
  writer.flush()?;

  println!("Done! Saved {} matches to '{}'", total_hits, output_path);
  Ok(())
}

fn create_filter(
  name: &str, 
  anchors: Vec<&str>, 
  embedder: &Embedder,
  projector: &RandomProjector
) -> ContextFilter {
  let embed_results = block_on(embedder.embed(&anchors, None, None)).unwrap();
  let vectors: Vec<Array1<f32>> = embed_results.iter()
    .map(|emb| Array1::from_vec(emb.to_dense().unwrap().to_vec()))
    .collect();
  let mask = MaskBaker::bake_mask(vectors, projector);
  ContextFilter {
    name: name.to_string(),
    mask,
    threshold: 12,
  }
}

fn process_batch_structs(
  batch: &[ConversationItem],
  embedder: &Embedder,
  projector: &RandomProjector,
  filters: &[ContextFilter]
) -> Vec<OutputMatch> {
  // 1. Prepare Text for Embedding
  // We construct the "Q: ... A: ..." string here for the model
  let text_refs: Vec<String> = batch.iter()
    .map(|item| format!("Q: {} A: {}", item.question, item.answer))
    .collect();

  let ref_slices: Vec<&str> = text_refs.iter().map(|s| s.as_str()).collect();

  // 2. Embed
  let embeds = block_on(embedder.embed(&ref_slices, None, None)).expect("Batch embed error");

  // 3. Filter and Map back to OutputMatch
  embeds.par_iter().enumerate().flat_map(|(i, emb_output)| {
    let floats = emb_output.to_dense().unwrap();
    let vector = Array1::from_vec(floats.to_vec());
    let hash = projector.project_to_hash(&vector);

    let mut batch_hits = Vec::new();

    for filter in filters {
      let dist = (hash ^ filter.mask).count_ones();
      if dist <= filter.threshold {
        // Return the FULL structured object
        batch_hits.push(OutputMatch {
          matched_category: filter.name.clone(),
          hamming_distance: dist,
          original_data: batch[i].clone(),
        });
      }
    }
    batch_hits
  }).collect()
}
