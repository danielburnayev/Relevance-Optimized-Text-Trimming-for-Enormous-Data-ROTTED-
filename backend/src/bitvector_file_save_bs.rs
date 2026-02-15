use rayon::prelude::*;
use dashmap::DashMap;
use embed_anything::embeddings::{embed::Embedder, local::text_embedding::ONNXModel};
use std::collections::HashMap;
use ndarray::{Array1, Array2};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use futures::executor::block_on;
use bytecheck::CheckBytes;
use rkyv::{Archive, Serialize};
use std::fs::File;
use memmap2::Mmap;
use std::io::{Write, BufWriter, Seek, SeekFrom};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct IndexEntry {
  hash: u64,
  offset: u64,
  len: u32,
}

#[derive(Archive, Serialize, CheckBytes, Debug)]
#[repr(C)]
pub struct BakedDataset {
  pub messages: Vec<String>,
  pub embeddings: Vec<Vec<f32>>,
}

pub fn bake_streaming(
  csv_path: &str, 
  index_path: &str, 
  data_path: &str, 
  embedder: &Embedder,
  projector: &RandomProjector,
) -> anyhow::Result<()> {
  let mut reader = csv::Reader::from_path(csv_path)?;
  // Use BufWriters for speed
  let mut data_writer = BufWriter::new(File::create(data_path)?);
  let mut index_writer = BufWriter::new(File::create(index_path)?);
  let mut current_offset = 0u64;
  let batch_size = 256; // Increased batch size for throughput
  let mut batch_text = Vec::with_capacity(batch_size);
  println!("Streaming and baking dataset from {}...", csv_path);

  for (count, result) in reader.records().enumerate() {
    let record = result?;
    let text = record[0].to_string();
    batch_text.push(text);

    if batch_text.len() >= batch_size {
      process_and_write_batch(
        &batch_text,
        embedder,
        projector,
        &mut index_writer,
        &mut data_writer,
        &mut current_offset
      )?;
      batch_text.clear();

      if count % 5000 == 0 {
        println!("Processed {} records...", count);
      }
    }
  }

  // Process remaining
  if !batch_text.is_empty() {
    process_and_write_batch(
      &batch_text,
      embedder,
      projector,
      &mut index_writer,
      &mut data_writer,
      &mut current_offset
    )?;
  }
  data_writer.flush()?;
  index_writer.flush()?;
  println!("Baking complete.");
  Ok(())
}

fn process_and_write_batch(
  batch: &[String],
  embedder: &Embedder,
  projector: &RandomProjector,
  index_writer: &mut BufWriter<File>,
  data_writer: &mut BufWriter<File>,
  current_offset: &mut u64,
) -> anyhow::Result<()> {
  let refs: Vec<&str> = batch.iter().map(|s| s.as_str()).collect();
  let embeds = block_on(embedder.embed(&refs, None, None)).expect("Batch embed error");

  for (i, emb_output) in embeds.iter().enumerate() {
    let floats = emb_output.to_dense().unwrap();
    let vector = Array1::from_vec(floats.to_vec());
    let hash = projector.project_to_hash(&vector);

    let text_bytes = batch[i].as_bytes();
    data_writer.write_all(text_bytes)?;

    let entry = IndexEntry {
      hash,
      offset: *current_offset,
      len: text_bytes.len() as u32,
    };

    // Unsafe cast to write struct bytes directly (fastest serialization)
    // Or use individual writes. Using std::slice::from_raw_parts for speed.
    let entry_slice = unsafe {
      std::slice::from_raw_parts(
        &entry as *const IndexEntry as *const u8,
        std::mem::size_of::<IndexEntry>()
      )
    };
    index_writer.write_all(entry_slice)?;

    *current_offset += text_bytes.len() as u64;
  }
  Ok(())
}

fn process_batch(
  batch: &[String], 
  embedder: &Embedder, 
  all_msgs: &mut Vec<String>, 
  all_embs: &mut Vec<Vec<f32>>
) {
  let refs: Vec<&str> = batch.iter().map(|s| s.as_str()).collect();
  let embeds = block_on(embedder.embed(&refs, None, None)).expect("Batch embed error");

  for (i, emb) in embeds.into_iter().enumerate() {
    all_msgs.push(batch[i].clone());
    all_embs.push(emb.to_dense().unwrap().to_vec());
  }
}

#[derive(Archive, Serialize, CheckBytes, Debug)]
#[repr(C)]
pub struct SynonymDatabase {
  pub map: HashMap<String, Vec<String>>,
}

pub struct RandomProjector {
  matrix: Array2<f32>, // Dimensions: 64 x 512
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
    // 1. Multiply: [64x512] * [512x1] = [64x1]
    let projected = self.matrix.dot(vector);

    let mut hash: u64 = 0;
    for (i, &val) in projected.iter().enumerate() {
      if val > 0.0 {
        hash |= 1 << i;
      }
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

    for v in vectors {
      centroid += &v;
    }

    centroid /= n;
    let mask = proj_mat.project_to_hash(&centroid);
    mask
  }
}

pub struct ActionExpander {
  _mmap: memmap2::Mmap,
}

impl ActionExpander {
  pub fn load_from_binary(path: &str) -> anyhow::Result<Self> {
    let file = std::fs::File::open(path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    Ok(Self { _mmap: mmap })
  }

  pub fn generate_all_possibilities(&self, k1: &str, k2: &str, action_type: &str) -> Vec<String> {
    let mut results = Vec::new();

    let archived = unsafe {
      rkyv::access_unchecked::<ArchivedSynonymDatabase>(&self._mmap[..])
    };

    println!("{}", archived.map.len());

    if let Some(syns) = archived.map.get(action_type) {
      for s in syns.into_iter() {
        results.push(format!("{} {} {}", k1, s, k2));
      }
    }
    println!("{}", results.len());
    results
  }
}

struct SemanticIndexer {
  embedder: Embedder,
  context_mask: u64, 
}

impl SemanticIndexer {
  fn new(key_1: String, key_2: String, context: String, proj_mat: &RandomProjector) -> Self {
    //let embedder = Embedder::from_pretrained_hf("openai/clip-vit-base-patch32", None, None, None).unwrap();
    let embedder = Embedder::from_pretrained_onnx("bert", Some(ONNXModel::AllMiniLML6V2), None, None, None, None).expect("Life isn't very fun right now...");
    let expander = ActionExpander::load_from_binary("thesaurus.bin").expect("Binary missing");
    let expanded_context = expander.generate_all_possibilities(&key_1, &key_2, &context);

    let ctx_refs: Vec<&str> = expanded_context.iter().map(|s| s.as_str()).collect();
    let embed_results = block_on(embedder.embed(&ctx_refs, None, None)).unwrap();

    let context_vec: Vec<Array1<f32>> = embed_results.iter()
      .map(|emb| Array1::from_vec(emb.to_dense().unwrap().to_vec()))
      .collect();

    let context_mask = MaskBaker::bake_mask(context_vec, proj_mat);

    Self { embedder, context_mask }
  }

  fn search_streaming(&self, index_path: &str, data_path: &str) -> anyhow::Result<HashMap<String, u32>> {
    let idx_file = File::open(index_path)?;
    let idx_mmap = unsafe { Mmap::map(&idx_file)? };

    let data_file = File::open(data_path)?;
    // We might not want to mmap 10GB data file if we don't have to, 
    // but Random Access reading is needed. Mmap is good for random reads.
    let data_mmap = unsafe { Mmap::map(&data_file)? };

    // Cast index mmap to slice of IndexEntry
    let entry_count = idx_mmap.len() / std::mem::size_of::<IndexEntry>();
    let entries: &[IndexEntry] = unsafe {
      std::slice::from_raw_parts(
        idx_mmap.as_ptr() as *const IndexEntry,
        entry_count
      )
    };

    let results = DashMap::new();

    // Parallel scan of the tiny index file
    entries.par_iter().for_each(|entry| {
      let dist = (entry.hash ^ self.context_mask).count_ones();

      if dist < 12 {
        // Only touch the massive data file if we have a match
        let start = entry.offset as usize;
        let end = start + entry.len as usize;

        if let Some(bytes) = data_mmap.get(start..end) {
          if let Ok(msg) = std::str::from_utf8(bytes) {
            results.insert(msg.to_string(), dist);
          }
        }
      }
    });

    Ok(results.into_iter().collect())
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let projector = RandomProjector::new(512, 64);

  println!("--- Initializing Semantic Indexer ---");
  let indexer = SemanticIndexer::new(
    "Trump".to_string(),
    "Putin".to_string(),
    "abscond".to_string(),
    &projector
  );

  let index_file = format!("{}_{}_{}_dataset.idx", "Trump", "Putin", "abscond");
  let data_file = format!("{}_{}_{}_dataset.data", "Trump", "Putin", "abscond");

  // 2. Check if baked data exists
  if !std::path::Path::new("dataset.idx").exists() {
    println!("Baking dataset (Streaming Mode)...");
    // Pass the projector so we can hash DURING baking
    bake_streaming(
      "src/reddit_wsb.csv", 
      &index_file, 
      &data_file, 
      &indexer.embedder, 
      &projector
    )?;
  }

  println!("Searching pre-baked hashes...");
  // Search now uses the pre-calculated hashes
  let matches = indexer.search_streaming(&index_file, &data_file)?;
  println!("\nScan Complete. Found {} matches.", matches.len());

  for (msg, dist) in matches.iter().take(10) {
    println!("{} : {}", msg, dist);
  }

  Ok(())
}
