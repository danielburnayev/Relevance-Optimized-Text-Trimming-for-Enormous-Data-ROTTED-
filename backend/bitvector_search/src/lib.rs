use anyhow::{Error, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, HiddenAct};
use tokenizers::{PaddingParams, Tokenizer};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, mpsc};
use std::thread;
use std::error::Error as StdError;
use rayon::prelude::*;
use csv::*;

pub struct BertEngine {
  model: BertModel,
  tokenizer: Tokenizer,
  pub device: Device,
}

impl BertEngine {
  pub fn new(model_id: &str) -> Result<Self> {
    let device = candle_core::Device::new_cuda(0).unwrap_or(candle_core::Device::Cpu);

    let config_path = "/app/model_cache/config.json"; 
    let tokenizer_path = "/app/model_cache/tokenizer.json";
    let weights_path = "/app/model_cache/model.safetensors";

    let config: Config = serde_json::from_str(&std::fs::read_to_string(config_path)?)?;
    let mut tokenizer = Tokenizer::from_file(tokenizer_path).map_err(anyhow::Error::msg)?;
    let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], candle_core::DType::F32, &device)? };
    let model = BertModel::load(vb, &config)?;

    if let Some(pp) = tokenizer.get_padding_mut() {
      pp.strategy = tokenizers::PaddingStrategy::BatchLongest;
    }

    Ok(Self { model, tokenizer, device })
  }

  pub fn embed_batch(&self, sentences: &[String]) -> Result<Tensor> {
    let tokens = self.tokenizer.encode_batch(sentences.to_vec(), true).map_err(anyhow::Error::msg)?;
    let token_ids = tokens.iter().map(|t: &tokenizers::Encoding| t.get_ids().to_vec()).collect::<Vec<_>>();
    let token_ids_tensor = Tensor::new(token_ids, &self.device)?;

    let token_type_ids = token_ids_tensor.zeros_like()?;
    let embeddings = self.model.forward(&token_ids_tensor, &token_type_ids, None)?;

    let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3()?;
    let embeddings = (embeddings.sum(1)? / (n_tokens as f64))?;

    let sum_sq = embeddings.sqr()?.sum_keepdim(1)?;
    let norm = sum_sq.sqrt()?;
    let embeddings = embeddings.broadcast_div(&norm)?;

    Ok(embeddings)
  }
}

pub struct ContextFilter {
  pub name: String,
  pub centroid: [u64; 6],
}

pub struct ContextEngine {
  embedder: Arc<BertEngine>,
}

impl ContextEngine {
  pub fn new(model_id: &str) -> Result<Self, Box<dyn StdError>> {
    let embedder = Arc::new(BertEngine::new(model_id)?);

    Ok(Self { embedder })
  }

  pub fn process_filters(&self, input_keyw_path: &str) -> Result<Vec<ContextFilter>, Box<dyn StdError>> {
    let mut filters = Vec::new();
    let mut keyw_reader = ReaderBuilder::new()
      .has_headers(true)
      .flexible(true)
      .from_path(input_keyw_path)?;

    for result in keyw_reader.records() {
      let record = match result {
        Ok(rec) => rec,
        Err(e) => {
          eprintln!("Skipping bad row: {}", e);
          continue;
        }
      };
      let cat = record.get(0).unwrap_or("default");
      let rec_to_vec: Vec<String> = record.iter().skip(1).map(|s| s.to_string()).collect();
      println!("Pushing new filter {}", cat);
      filters.push(self.create_filter(cat, rec_to_vec)?);
    }
    Ok(filters)
  }

  fn create_filter(&self, name: &str, anchors: Vec<String>) -> Result<ContextFilter> {
    println!("Embedding full-sentence queries for filter: {}...", name);

    let embeddings = self.embedder.embed_batch(&anchors)?;
    let vec_embeddings = embeddings.to_vec2::<f32>()?;

    let dim = vec_embeddings[0].len();
    let mut float_centroid = vec![0.0; dim];

    for emb in &vec_embeddings {
      for (i, &v) in emb.iter().enumerate() {
        float_centroid[i] += v;
      }
    }

    let quantized_centroid = self.quantize_single(&float_centroid);

    println!("Filter '{}' successfully quantized, pushing now...", name);

    Ok(ContextFilter { 
      name: name.to_string(), 
      centroid: quantized_centroid 
    })
  }

  pub fn process_csv(&self, 
    input_data_path: &str, 
    record_index: usize, 
    output_path: &str, 
    filters: &[ContextFilter]) -> Result<(), Box<dyn StdError>> {

    let batch_size: usize = 64;

    let (tx, rx) = mpsc::sync_channel::<Vec<String>>(100);

    let output_path_owned = output_path.to_string();
    let writer_handle = thread::spawn(move || -> Result<(), std::io::Error> {
      let output_file = File::create(output_path_owned)?;
      let mut writer = BufWriter::with_capacity(1024 * 1024, output_file);

      while let Ok(lines) = rx.recv() {
        for line in lines {
          writeln!(writer, "{}", line)?;
        }
      }
      writer.flush()?;
      Ok(())
    });

    println!("Reading entire CSV into memory to unblock the disk...");
    let mut all_texts = Vec::new();
    let mut reader = ReaderBuilder::new().has_headers(true).flexible(true).from_path(input_data_path)?;

    for result in reader.records() {
      if let Ok(record) = result {
        if let Some(text) = record.get(record_index) { 
          all_texts.push(text.to_string());
        }
      }
    }
    println!("Loaded {} rows. Blasting to all CPU cores...", all_texts.len());

    all_texts.par_chunks(batch_size).for_each(|batch| {
      if let Ok(output_lines) = self.compute_batch_strings(batch, filters) {
        if !output_lines.is_empty() {
          let _ = tx.send(output_lines); 
        }
      }
    });

    drop(tx); 
    match writer_handle.join() {
      Ok(result) => result.map_err(|e| e.into()),
      Err(_) => Err("Writer thread panicked".into()),
    }
  }

  fn compute_batch_strings(&self, 
    batch: &[String], 
    filters: &[ContextFilter]) -> Result<Vec<String>> {

    let similarity_threshold = 0.65;

    let embeddings = self.embedder.embed_batch(batch)?.to_vec2::<f32>()?;

    let output_lines: Vec<String> = embeddings.par_iter().enumerate().flat_map(|(i, row_floats)| {
      let mut hits = Vec::new();
      let mut high_score: f32 = 0.0;
      let row_bits = self.quantize_single(row_floats);

      for filter in filters {
        let dist = self.hamming_distance(&row_bits, &filter.centroid);
        let score = 1.0 - (dist as f32 / 384.0);

        if score > similarity_threshold && score > high_score {
          let clean_msg = batch[i].replace("\n", " ").replace("\"", "'");
          hits.push(format!("{} : {:.4} : \"{}\"", filter.name, score, clean_msg));
          high_score = score;
        }
      }
      hits
    }).collect();

    Ok(output_lines)
  }

  fn quantize_single(&self, embedding: &[f32]) -> [u64; 6] {
    let mut bits = [0u64; 6];
    for (i, &val) in embedding.iter().enumerate() {
      if val > 0.0 {
        bits[i / 64] |= 1 << (i % 64);
      }
    }
    bits
  }

  fn hamming_distance(&self, a: &[u64; 6], b: &[u64; 6]) -> u32 {
    let mut dist = 0;
    for i in 0..6 {
      dist += (a[i] ^ b[i]).count_ones();
    }
    dist
  }
}
