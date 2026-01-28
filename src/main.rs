use std::fs::File;
use std::io::{self, Write};
use std::io::{BufRead, BufReader};

use fst::{IntoStreamer, Map, MapBuilder};
use fst::automaton::Levenshtein;
use memmap2::Mmap;

// Adapted and built upon from the fst crate examples by the Legendary @burntsushi
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let start_build = std::time::Instant::now();
  build_fst("dict.txt", "dict.fst")?;
  let duration_build = start_build.elapsed();
  println!("Time to build: {:?}", duration_build);

  let data = std::fs::File::open("dict.fst")?;
  let mmap = unsafe { Mmap::map(&data)? };
  let map = Map::new(mmap)?;

  loop {
      print!("Enter a word to search (type #q to exit): ");
      io::stdout().flush()?;
      let mut input = String::new();
      std::io::stdin().read_line(&mut input)?;
      let input = input.trim();
      match input.to_lowercase().as_str() {
        "#q" => break,
        "" => {
          println!("Enter a valid word");
        }
        _ => {
          let start_search = std::time::Instant::now();
          let lev = Levenshtein::new(input.to_lowercase().as_str(), 1)?;
          let stream = map.search(lev).into_stream();
          let matches = stream.into_str_keys()?;
          let duration_search = start_search.elapsed();
          println!("Time to search: {:?}", duration_search);
          println!("{:#?}", matches)
        }
      }
  }
  Ok(())
}

fn build_fst(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
  let file  = File::open(input_path)?;
  let reader = BufReader::new(file).lines();

  let mut keys: Vec<String> = vec![];
  for line in reader {
    let line = line?;
    keys.push(line);
  }
  keys.sort();

  let writer = io::BufWriter::new(File::create(output_path)?);
  let mut build = MapBuilder::new(writer)?;

  for key in keys {
    build.insert(key, 0)?;
  }

  build.finish()?;
  Ok(())
}