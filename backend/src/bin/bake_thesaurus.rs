use bytecheck::{CheckBytes, check_bytes, rancor::Failure};
use rkyv::{Archive, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::env;

#[derive(Archive, Serialize, CheckBytes, Debug)]
#[repr(C)]
pub struct SynonymDatabase {
  // FIXED: HashMap requires the 'Map' wrapper for O(1) binary layout
  //#[rkyv(with = rkyv::with::Map)]
  //#[rkyv(with = rkyv::with::Map<()>)]
  pub map: HashMap<String, Vec<String>>,
}

fn load_from_csv(path: &str) -> anyhow::Result<HashMap<String, Vec<String>>> {
  let mut reader = csv::Reader::from_path(path)?;
  let mut data_map = HashMap::new();

  for result in reader.records() {
    let record = result?;
    let verb = record[0].to_string();
    let synonyms = vec![
      record[1].to_string(),
      record[2].to_string(),
      record[3].to_string(),
      record[4].to_string(),
      record[5].to_string(),
    ];
    data_map.insert(verb, synonyms);
  }
  Ok(data_map)
}

fn main() -> anyhow::Result<()> {
  let args: Vec<String> = env::args().collect();
  let mut data_map = load_from_csv(&args[1])?;

  let database = SynonymDatabase { map: data_map };
  let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&database).expect("Failed to serialize");
  //let bytes = rkyv::to_bytes::<_, 1024>(&database).expect("Failed to serialize");

  let mut file = File::create("thesaurus.bin")?;
  file.write_all(&bytes)?;

  println!("Success: thesaurus.bin generated with {} categories.", database.map.len());
  Ok(())
}
