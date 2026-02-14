use rayon::prelude::*;
use dashmap::DashMap;
use embed_anything::embeddings::embed::Embedder;
use std::collections::HashMap;
use ndarray::{Array1, Array2};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use futures::executor::block_on;

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

struct ActionExpander {
  thesaurus: HashMap<String, Vec<String>>,
}

impl ActionExpander {
  fn new() -> Self {
    let mut t = HashMap::new();
    t.insert("interaction".to_string(), vec!["met", "spoke", "visited", "emailed"].into_iter().map(String::from).collect());
    t.insert("transaction".to_string(), vec!["bought", "sold", "paid", "traded"].into_iter().map(String::from).collect());
    t.insert("conflict".to_string(), vec!["argued", "disagreed", "fought", "blocked"].into_iter().map(String::from).collect());
    Self { thesaurus: t }
  }

  fn generate_all_possibilities(&self, k1: &str, k2: &str, action_type: &str) -> Vec<String> {
    let mut results = Vec::new();
    if let Some(syns) = self.thesaurus.get(action_type) {
      for s in syns {
        results.push(format!("{} {} {}", k1, s, k2));
      }
    }
    results.push(format!("{} was seen with {}", k1, k2)); // Structural variation
    results
  }
}

struct SemanticIndexer {
  embedder: Embedder,
  context_mask: u64, 
}

impl SemanticIndexer {
  fn new(key_1: String, key_2: String, context: String, proj_mat: &RandomProjector) -> Self {
    let embedder = Embedder::from_pretrained_hf("openai/clip-vit-base-patch32", None, None, None).unwrap();
    let expander = ActionExpander::new();
    let expanded_context = expander.generate_all_possibilities(&key_1, &key_2, &context);
    let context_vec: Vec<Array1<f32>> = expanded_context.iter()
      .map(|sent| {
        // Remove .await here; use the synchronous batch method
        let embed_result = block_on(embedder.embed(&[sent.as_str()], None, None)).unwrap();
        let float_vec: Vec<f32> = embed_result[0].to_dense().expect("Embed error").clone();
        Array1::from_vec(float_vec)
      })
    .collect();
    let context_mask = MaskBaker::bake_mask(context_vec, proj_mat); 
    Self { embedder, context_mask }
  }

  fn process_category(&self, category: &str, messages: Vec<String>, proj_mat: &RandomProjector) {
    let results = DashMap::new();
    messages.into_par_iter().for_each(|msg| {
      let embed_result = block_on(self.embedder.embed(&[msg.as_str()], None, None)).unwrap();
      let float_vec: Vec<f32> = embed_result[0].to_dense().expect("Embed error").clone();
      let vector = Array1::from_vec(float_vec);

      let semantic_hash = proj_mat.project_to_hash(&vector);
      let dist = (semantic_hash ^ self.context_mask).count_ones();

      if dist < 10 {
        results.insert(msg, "Detected Context");
      }
    });

    println!("\n--- Results for Category: {} ---", category);
    if results.is_empty() {
      println!("No matches found.");
    } else {
      for entry in results.iter() {
        let (msg, context) = entry.pair();
        println!("{}: {}", msg, context);
      }
    }
  }
}

fn main() {
  // 1. Initialize the Lens (Matrix)
  let projector = RandomProjector::new(512, 64);

  // 2. Setup the Indexer (Alice and Bob interaction)
  println!("--- Initializing Semantic Indexer ---");
  let indexer = SemanticIndexer::new(
    "Alice".to_string(),
    "Bob".to_string(),
    "interaction".to_string(),
    &projector
  );

  // 3. Test Dataset
  let test_data = vec![
    "Alice and Bob are meeting at the park".to_string(),      // Contextual Match
    "Bob was seen with Alice yesterday".to_string(),        // Structural Match
    "The price of gold is rising in London".to_string(),    // No Match
    "Alice emailed Bob the final report".to_string(),       // Action Match
    "I really like pizza with extra cheese".to_string(),   // No Match
  ];

  println!("\n--- Scanning Dataset ---");
  indexer.process_category("general", test_data, &projector);
  println!("\nScan Complete.");
  return
}
