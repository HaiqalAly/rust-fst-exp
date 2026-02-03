# Rust FST Experiment

*A personal exploration of the `fst` crate for efficient fuzzy searching.*

> **Note:** I'm still learning! This started as a test script and grew into a complex experiment. Expect some bugs, inefficiencies in the code, or straight-up inaccuracies and nonsense I was about to spout in this README

## Usage

1.  **Prepare Dictionary:** Input must be strictly sorted by ASCII byte values.
    ```bash
    export LC_ALL=C && sort dict.txt -o dict.txt
    ```
    *Optional: Add weights for ranking (e.g., `love,1000`).*

2.  **Run:**
    ```bash
    # To run the REPL
    cargo run --release

    # OR use piping (example below)
    printf "love\n#q" | cargo run --release
    ```
    Type words to search. Type `#q` to exit.

## Key Insights & Benchmarks

### 1. Storage Efficiency
*   **Original File (`dict.txt`)**: **977 KB** (~103k words).
*   **Compressed FST (`dict.fst`)**: **279 KB**.
*   **Reduction**: The FST is **~29%** of the original size, achieving a **~71% reduction** in storage.

### 2. Massive Speedups
*   **Incremental Build:** Implemented `make`-like logic to skip rebuilding if `dict.fst` is fresh.
    *   **Debug Profile**:
        *   Fresh Build: **~352ms**
        *   Cached Startup: **~8.5µs** (No build performed)
        *   Speedup: **~41,000x** faster startup.
    *   **Release Profile**:
        *   Fresh Build: **~36ms** (Streaming) vs **~46.5ms** (Old In-Memory)
        *   Cached Startup: **~3.8µs** (High Perf) | **~7µs** (Balanced)
        *   Speedup: Even with optimized builds, skipping the work is **~10,000x** faster.
*   **Cold vs. Warm CPU:** Interactive shell queries (**~400µs - 1ms**) are on average **2-3x** slower than piped queries (**~350µs**) due to CPU power-saving latency. However, in "High Performance mode", repeat queries hit **~190µs** - **250µs**.

### 3. Zero-RAM Construction
*   **Streaming Build:** Switched from loading `Vec<String>` to streaming lines directly from disk.
    *   *Reduced RAM usage from O(N) to O(1).*
    *   *Build time dropped ~21%.*

### 4. Smart Search Features
*   **Weighted Ranking:** Modified to support `word,score` pairs. Results are ranked by: **Exact Match > High Score > Alphabetical**.
*   **Fuzzy Search:** `Levenshtein` distance 1 is instant (**~190µs** - **300µs**). Distance 2 is exponential (**~1.55ms**).

## Known Limitation
While the FST search is lightning-fast, the current implementation has three primary inefficiencies that scale poorly with large dictionaries or higher edit distances:

1. The "Collect-Sort" Bottleneck:
The code currently drains the entire search stream into a Vec, then sorts the entire result set just to pick the top 10. If a fuzzy search for a short word returns 5,000 matches, the CPU spends 99% of its time sorting 4,990 words you will never show the user.

2. Redundant String Conversion:
Inside the `sort_by_cached_key` block, the code performs `String::from_utf8_lossy(word).to_lowercase()` repeatedly.

3. Levenshtein Distance:
The search space for Levenshtein automata grows exponentially. In a large dictionary, `Levenshtein(2)` can return thousands of results, making the "Collect-Sort" bottleneck mentioned in point #1 even more severe.

## Status

**Maintenance Mode** (Feb 2026).<br>
This experiment has grown from a simple script into a complex optimization playground.<br>
I'm freezing it here to focus on other learnings.
