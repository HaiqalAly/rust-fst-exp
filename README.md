# Fast-FST: High-Performance Fuzzy Search

*A personal experimentation of the `fst` crate for efficient fuzzy searching.*

> **Note:** I'm still learning Rust. This project unexpectedly grew from a simple test script into a complex optimization experiment. The code definitely has bugs and inefficiencies, and some concepts here might be beyond my current understanding.

![Benchmark repeat query](docs/query.png)<br>
## Usage

1.  **Prepare Dictionary:** Input must be strictly sorted by ASCII byte values.
    ```bash
    export LC_ALL=C && sort dict.txt -o dict.txt
    ```
    *Optional: Add weights for ranking (e.g., `love,1000`).*

2.  **Run:**
    ```bash
    cargo run --release
    ```
    The interactive TUI will launch.
    *   **Type** to search instantly.
    *   **Esc** or **Ctrl+C** to exit.

## Key Insights & Benchmarks

### 1. Storage Efficiency
*   **Original File (`dict.txt`)**: **977 KB** (~103k words).
*   **Compressed FST (`dict.fst`)**: **279 KB**.
*   **Reduction**: The FST is **~29%** of the original size, achieving a **~71% reduction** in storage.

### 2. Massive Speedups
*   **Heap-Based Top-K Rangking:** Used `std::collections::BinaryHeap` to limit search result to top 10 items in real-time.
    *   **Memory Efficiency:** We no longer store the entire fuzzy match result in RAM before sorting.
    *   **Algorithmic View:** Complexity drops from **O(N log N)** to **O(N log 10)**
*   **Incremental Build:** Implemented `make`-like logic to skip rebuilding if `dict.fst` is fresh.
    *   **Debug Profile**:
        *   Fresh Build: **~352ms**
        *   Cached Startup: **~8.5µs** (No build performed)
        *   Speedup: **~41,000x** faster startup.
    *   **Release Profile**:
        *   Fresh Build: **~36ms** (Streaming) vs **~46.5ms** (Old In-Memory)
        *   Cached Startup: **~3.8µs** (High Perf) | **~7µs** (Balanced)
        *   Speedup: Even with optimized builds, skipping the work is **~10,000x** faster.
*   **Search Latency:** This was tested on a laptop and the performance depends heavily on CPU power states:
    *   **Balanced Mode:** ~700µs - 1ms.
    *   **High Performance:** ~190µs - 350µs.

### 3. Zero-RAM Construction
*   **Streaming Build:** Switched from loading `Vec<String>` to streaming lines directly from disk.
    *   *Reduced RAM usage from O(N) to O(1).*
    *   *Build time dropped ~21%.*

### 4. Smart Search Features
*   **Weighted Ranking:** Modified to support `word,score` pairs. Results are ranked by: **Exact Match > High Score > Alphabetical**.
*   **Fuzzy Search:** `Levenshtein` distance 1 is instant (**~190µs** - **300µs**). Distance 2 is exponential (**~1.55ms**).

## Known Limitation

1. **Levenshtein Distance:**
Even with the Heap optimization, a `Levenshtein` distance of 2+ on a massive dictionary is significantly slower. While we no longer struggle to sort those results, the FST still has to find them, which involves traversing a much larger state machine.