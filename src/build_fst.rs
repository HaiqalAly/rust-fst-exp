use std::fs::File;
use std::io::{self};
use std::io::{BufRead, BufReader};

use fst::MapBuilder;

pub fn build_fst(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(input_path)?;

    let writer = io::BufWriter::new(File::create(output_path)?);
    let mut build = MapBuilder::new(writer)?;
    let reader = BufReader::new(file).lines();

    for line in reader {
        let line = line?;
        let mut key = line.as_str();
        let mut value = 0;

        if let Some((word, weight)) = line.split_once(","){
            key = word.trim();
            value = weight.trim().parse::<u64>().unwrap_or(0);
        } 
        build.insert(key, value)?;
    }

    build.finish()?;
    Ok(())
}
