use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Map, Streamer};
use memmap2::Mmap;

#[derive(PartialEq, Eq, Clone)]
pub struct SearchResult {
    pub key: Vec<u8>,
    pub value: u64,
    pub is_exact: bool,
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
        other
            .is_exact
            .cmp(&self.is_exact)
            // 2. Score: Lower score is "Worse" (Greater)
            .then_with(|| other.value.cmp(&self.value))
            .then_with(|| self.key.cmp(&other.key))
    }
}

pub struct Dictionary {
    map: Map<Mmap>,
}

impl Dictionary {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::File::open(path)?;
        // SAFETY: We assume the dictionary file is not being modified concurrently
        let mmap = unsafe { Mmap::map(&data)? };
        let map = Map::new(mmap)?;
        Ok(Self { map })
    }

    pub fn search(
        &self,
        query: &str,
    ) -> Result<(Vec<SearchResult>, Duration), Box<dyn std::error::Error>> {
        if query.is_empty() {
            return Ok((vec![], Duration::from_micros(0)));
        }

        let start_search = Instant::now();
        let query_lower = query.to_lowercase();
        let lev = Levenshtein::new(&query_lower, 1)?;
        let mut stream = self.map.search(lev).into_stream();

        let mut heap = BinaryHeap::with_capacity(10);
        let target_bytes = query_lower.into_bytes();

        // Only keep top 10; skip items that are worse than the current worst in the heap
        while let Some((key_bytes, value)) = stream.next() {
            let is_exact = key_bytes == target_bytes;

            let res = SearchResult {
                key: key_bytes.to_vec(),
                value,
                is_exact,
            };

            if heap.len() < 10 {
                heap.push(res);
            } else if heap.peek().is_none_or(|worst| res < *worst) {
                heap.pop();
                heap.push(res);
            }
        }

        let top_10 = heap.into_sorted_vec();
        Ok((top_10, start_search.elapsed()))
    }
}
