# Rust FST Experiment

A personal exploration of the **[fst](https://github.com/BurntSushi/fst)** crate in Rust, experimenting with efficient string (set/map) storage and fuzzy searching capabilities.

This project demonstrates building a Finite State Transducer (FST) from a dictionary file and performing interactive fuzzy searches using Levenshtein distance.

## Usage

Ensure you have a `dict.txt` file in the root directory containing a list of words (one per line).

```bash
cargo run --release
```

The program will:
1. Compile the dictionary into a compressed FST (`dict.fst`).
2. Launch a REPL where you can type words to find similar matches (within an edit distance of 1).
3. Type `#q` to exit.

## Key Learnings

1.  **Lexicographic Order is Mandatory**:
    The `MapBuilder` (and `SetBuilder`) requires keys to be inserted in sorted order. If the input source isn't sorted, it *must* be gathered and sorted in memory before insertion.

2.  **Fuzzy Search is Powerful**:
    Using the `Levenshtein` automaton allows for fast approximate string matching. Finding "food" when searching for "foo" works efficiently even against large datasets.

3.  **Map vs Set**:
    While a `Set` is sufficient for membership tests, using a `Map` allows associating values (like frequency or ID) with keys. In this experiment, I used `Map`.

4.  **CPU Cold Start is Real**:
    Interactive REPLs can be deceptive benchmarks. The OS and CPU aggressively scale down performance during user idle time, making individual queries appear up to 3x slower (0.8ms - 1.2ms) than they actually are (0.3ms).

## Performance Analysis & Benchmarks

### Storage Efficiency
FSTs provide significant compression over raw text.
*   **Original File (`dict.txt`)**: **977 KB** (~103k words).
*   **Compressed FST (`dict.fst`)**: **279 KB**.
*   **Reduction**: The FST is **~29%** of the original size, achieving a **~71% reduction** in storage.

### Execution Speed (Hot vs Cold)
Benchmarks revealed a massive discrepancy between manual interactive searches and piped/batched searches.

*   **Build Time**: **~53ms** to process 103,495 words.
*   **Search (Manual REPL)**: **~800µs - 1.2ms**. 
    *   *Note*: This higher latency is due to CPU frequency scaling (Cold Start) and wake-up latency when typing manually.
*   **Search (Piped / Warm)**: **~250µs - 370µs**.
    *   *Note*: This represents the true performance when the CPU is "warm", achieved by piping input (`printf | cargo run`). e.g, `printf "love\n#q" | cargo run --release`

*   **Insight**: The slowness in the REPL is **not** the FST traversal (which is < 0.4ms), but rather the **DFA Construction** (~500µs) and system wake-up overhead.

### Memory Mapping Strategy
*   **fs::read (Heap)**: Slightly faster (~335µs) for small files because it pre-loads everything into RAM.
*   **mmap (Memory Map)**: Slightly slower (~360µs) on first access due to page faults, but allows instant start-up and near-zero memory footprint for huge datasets.

