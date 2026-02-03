use std::fs;
use std::path::Path;

mod build_fst;
mod search;
use build_fst::build_fst;
use search::search_fn;

// Adapted and built upon from the fst crate examples by the Legendary @burntsushi
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_build = std::time::Instant::now();

    let fst_path = Path::new("dict.fst");
    let txt_path = Path::new("dict.txt");

    let mut should_build = !fst_path.exists();

    // Check if the txt file is newer, if yes rebuild, if not use cached one
    if !should_build {
      let fst_metadata = fs::metadata(fst_path)?;
      let txt_metadata = fs::metadata(txt_path)?;

      let fst_time = fst_metadata.modified()?;
      let txt_time = txt_metadata.modified()?;

      if txt_time > fst_time {
        should_build = true;
      }
    }

    if should_build {
      build_fst(txt_path.to_str().expect("something's wrong but idk what it is!"), fst_path.to_str().expect("something's wrong but idk what it is!"))?;
    }

    let duration_build = start_build.elapsed();
    println!("Time to build: {:?}", duration_build);

    search_fn()?;
    Ok(())
}
