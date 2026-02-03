use std::io::{self, Write};
use std::collections::BinaryHeap;

use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Map, Streamer};
use memmap2::Mmap;

#[derive(PartialEq, Eq)]
struct SearchResult {
    key: Vec<u8>,
    value: u64,
    is_exact: bool,
}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // In a BinaryHeap (MaxHeap), pop() removes the Greatest item.
        // We want to KEEP it instead. x_x

        // 1. Exact Match: Non-exact is "Worse" (Greater)
        other.is_exact.cmp(&self.is_exact) 
        // 2. Score: Lower score is "Worse" (Greater)
            .then_with(|| other.value.cmp(&self.value))
            .then_with(|| self.key.cmp(&other.key))
    }
}

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
                println!("Enter a valid word.");
            }
            _ => {
                let start_search = std::time::Instant::now();
                let lev = Levenshtein::new(input.to_lowercase().as_str(), 1)?;
                let mut stream = map.search(lev).into_stream();

                let mut heap = BinaryHeap::with_capacity(11);
                let target_bytes = input.to_lowercase().into_bytes();

                // Only keep top 10 and pop the worst one if hit 11
                while let Some((key_bytes, value)) = stream.next() {
                    let is_exact = key_bytes == target_bytes;

                    // Constructing the struct
                    let res = SearchResult {
                        key: key_bytes.to_vec(),
                        value,
                        is_exact,
                    };

                    heap.push(res);

                    if heap.len() > 10 {
                        heap.pop();
                    }
                }
                
                // Only take top 10 words
                let top_10: Vec<_> = heap.into_sorted_vec();

                let duration_search = start_search.elapsed();
                println!("\nFound {} results in: {:?}", top_10.len(), duration_search);
                println!("{:-<30}", "");

                if top_10.is_empty() {
                    println!("No matches found.");
                } else {
                    for (i, item) in top_10.iter().enumerate() {
                        // Recover original string from bytes for display
                        let word = String::from_utf8_lossy(&item.key);
                        println!("{:2}. {:<15} (score: {})", i + 1, word, item.value);
                    }
                }

                println!("{:-<30}\n", "");
            }
        }
    }

    Ok(())
}