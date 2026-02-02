# Rust FST Experiment

> Please keep in mind that I'm still learning and at the very beginning. Expect some bugs, inefficiencies, or straight-up inaccuracies and nonsense I was about to spout in this README. I’ll admit this is getting a bit out of hand, so I think the experiment will be wrapping up soon.

A personal exploration of the **[fst](https://github.com/BurntSushi/fst)** crate in Rust, experimenting with efficient string (set/map) storage and fuzzy searching capabilities.

This experiment demonstrates building a Finite State Transducer (FST) from a dictionary file and performing interactive fuzzy searches using Levenshtein distance.


## Usage

Ensure you have a `dict.txt` file in the root directory containing a list of words (one per line).

**Important:** The dictionary must be strictly sorted by ASCII byte values (not case-insensitive dictionary order). Use this command to prepare your file:

```bash
export LC_ALL=C && sort dict.txt -o dict.txt
```

Then run the project:

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
*   **Search (Manual REPL)**: **~800µs - 1.2ms** (Balanced Mode) vs **~240µs - 560µs** (High Perf Mode).
    *   *Note*: The higher latency is primarily due to CPU frequency scaling (Cold Start). In my experience, switching the OS to "High Performance" eliminates this governor latency, bringing manual search speeds closer to warm/piped performance.
*   **Search (Piped / Warm)**: **~250µs - 370µs**.
    *   *Note*: This represents the true performance when the CPU is "warm", achieved by piping input (`printf | cargo run`) e.g, `printf "love\n#q" | cargo run --release`.
*   **Insight**: The slowness in the REPL is **not** the FST traversal (which is < 0.4ms), but rather the **DFA Construction** (~500µs) and system wake-up overhead.

### Incremental Build Optimization
Implemented a "lazy" build system that checks file modification timestamps (similar to `make`) before rebuilding the FST.

*   **Logic**: Comparing `mtime` of `dict.txt` vs `dict.fst`.
*   **Results** (Debug Profile):
    *   **Fresh Build**: ~352ms
    *   **Cached Startup**: **~8.5µs** (No build performed)
    *   **Speedup**: ~41,000x faster startup.
*   **Results** (Release Profile):
    *   **Fresh Build**: ~36ms (Streaming) vs ~46.5ms (In-Memory Sort)
    *   **Cached Startup**: **~3.8µs** (High Perf) - **~7.2µs** (Balanced)
    *   **Speedup**: Even with optimized builds, skipping the work is **~6,000x - 12,000x faster**.

### Streaming Build vs In-Memory Sort
Switched from loading the entire dictionary into a `Vec<String>` to streaming lines directly from disk into the `MapBuilder`.

*   **Logic**: Pre-sorting `dict.txt` using system `sort` (LC_ALL=C) allows O(1) memory usage during construction.
*   **Memory Impact**:
    *   **Old Way**: O(N) RAM usage. With 100MB text, it used ~100MB+ RAM to sort.
    *   **New Way**: O(1) RAM usage. Uses practically zero memory regardless of file size.
*   **Time Impact**: Build time dropped from **~46.5ms** to **~36.5ms** (~21% faster) by avoiding the overhead of allocating and moving 100k strings in Rust.

### Memory Mapping Strategy
*   **fs::read (Heap)**: Slightly faster (~335µs) for small files because it pre-loads everything into RAM.
*   **mmap (Memory Map)**: Slightly slower (~360µs) on first access due to page faults, but allows instant start-up and near-zero memory footprint for huge datasets.

### Weighted Transducer (Ranking Results)
Enhanced the FST to finally make use of the map (key, value) pair abilities for a ranked result, effectively creating a weighted search engine. For now, you'll have to manually edit the dictionary and assign the value yourself though. 

*   **Implementation**: Parsed `word,score` pairs from `dict.txt` (e.g., `love,1000`).
*   **Search Logic**: Results are no longer just alphabetical. They are collected and sorted descending by score at query time.
*   **Outcome**: Searching for "lov" now promotes "love" (Score: 1000) to the top, while "clove" (Score: 0) sinks to the bottom, mimicking modern autocomplete behavior.

