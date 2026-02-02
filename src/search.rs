use std::io::{self, Write};

use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Map, Streamer};
use memmap2::Mmap;

pub fn search_fn() -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::File::open("dict.fst")?;
    // SAFETY: We assume the dictionary file is not being modified concurrently
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
                    result.push((key_bytes.to_vec(), value));
                }

                // Tie breaker (lowercase first > capitalized, e.g love > Love)
                let target = input.to_lowercase();
                result.sort_by_cached_key(|(word, value)| {
                    let word_str = String::from_utf8_lossy(word).to_lowercase();

                    let not_exact = word_str != target;
                    
                    (
                        not_exact,
                        std::cmp::Reverse(*value),
                        word_str,
                        std::cmp::Reverse(word.clone())
                    )
                });
                
                // Only take top 10 words
                let top_10: Vec<(String, u64)> = result.into_iter()
                    .take(10)
                    .filter_map(|(bytes, value)| {
                        String::from_utf8(bytes).ok().map(|s| (s, value))
                    })
                    .collect();

                let duration_search = start_search.elapsed();
                println!("Time to search: {:?}", duration_search);
                println!("{:#?}", top_10) 
            }
        }
    }

    Ok(())
}