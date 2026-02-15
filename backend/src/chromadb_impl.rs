use chromadb::v1::client::{ChromaClient, ChromaClientOptions};
use chromadb::v1::collection::{ChromaCollection, QueryOptions};
use embed_anything::embeddings::embed::Embedder;
use std::collections::HashMap;
use futures::executor::block_on;
use anyhow::Result;

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
    results.push(format!("{} was seen with {}", k1, k2));
    results
  }
}

pub struct ChromaIndexer {
  embedder: Embedder,
  collection: ChromaCollection,
}

impl ChromaIndexer {
  pub async fn new(collection_name: &str) -> Result<Self> {
    // Connect to local ChromaDB (Default: http://localhost:8000)
    let client = ChromaClient::new(ChromaClientOptions::default());
    let collection = client.get_or_create_collection(collection_name, None).await?;

    let embedder = Embedder::from_pretrained_hf(
      "openai/clip-vit-base-patch32", 
      None, None, None
    ).unwrap();

    Ok(Self { embedder, collection })
  }

  // Replaces MaskBaker; seeds the collection with expanded forensic context
  pub async fn seed_context(&self, k1: &str, k2: &str, context: &str) -> Result<()> {
    let expander = ActionExpander::new();
    let variations = expander.generate_all_possibilities(k1, k2, context);

    for (i, text) in variations.iter().enumerate() {
      let embed_result = block_on(self.embedder.embed(&[text.as_str()], None, None)).unwrap();
      let vector = embed_result[0].to_dense().expect("Embed error");

      self.collection.upsert(
        vec![format!("ctx_{}", i)],
        Some(vec![vector]),
        None,
        Some(vec![text.clone()]),
        None
      ).await?;
    }
    Ok(())
  }

  // Replaces process_category; returns findings to an outside program
  pub async fn scan_dataset(&self, messages: Vec<String>) -> Result<HashMap<String, f32>> {
    let mut matches = HashMap::new();

    for msg in messages {
      let embed_result = block_on(self.embedder.embed(&[msg.as_str()], None, None)).unwrap();
      let vector = embed_result[0].to_dense().expect("Embed error");

      // Query nearest neighbors using HNSW
      let query_results = self.collection.query(QueryOptions {
        query_embeddings: Some(vec![vector]),
        n_results: Some(1),
        ..Default::default()
      }).await?;

      if let Some(distances) = &query_results.distances {
        let score = distances[0][0];
        // Threshold for "Detected Context" (Distance < 0.5 is a strong match)
        if score < 0.5 {
          matches.insert(msg, score);
        }
      }
    }
    Ok(matches)
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  println!("--- Initializing ChromaDB Indexer ---");
  let indexer = ChromaIndexer::new("forensics_case_001").await?;

  indexer.seed_context("Alice", "Bob", "interaction").await?;

  let test_data = vec![
    "Alice and Bob are meeting at the park".to_string(),
    "Bob was seen with Alice yesterday".to_string(),
    "The price of gold is rising in London".to_string(),
    "Alice emailed Bob the final report".to_string(),
  ];

  println!("\n--- Scanning Dataset ---");
  let results = indexer.scan_dataset(test_data).await?;

  // Return or output results for the outside program
  for (msg, score) in results {
    println!("[MATCH] Score {:.4}: {}", score, msg);
  }

  Ok(())
}
