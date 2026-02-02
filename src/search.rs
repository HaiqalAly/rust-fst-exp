use std::io::{self, Write};

use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Map, Streamer};
use memmap2::Mmap;

pub fn search_fn() -> Result<(), Box<dyn std::error::Error>> {
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
                let mut stream = map.search(lev).into_stream();
                let mut result = Vec::new();

                while let Some((key_bytes, value)) = stream.next() {
                    let word = String::from_utf8(key_bytes.to_vec())?;

                    result.push((word, value));
                }

                result.sort_by_key(|(_, value)| std::cmp::Reverse(*value));

                let duration_search = start_search.elapsed();
                println!("Time to search: {:?}", duration_search);
                println!("{:#?}", result) 
            }
        }
    }

    Ok(())
}