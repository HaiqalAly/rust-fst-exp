### Personal experiment on FST

I've been playing around with the `fst` crate to handle sets of strings efficiently. Here are my quick takeaways:

### What I Learned
1.  **Order Matters**: The `MapBuilder` (previously `SetBuilder`) requires keys to be inserted in **lexicographic order**. I had to sort my vector first.
2.  **Fuzzy Search**: Using the `Levenshtein` automaton allows for powerful approximate string matching (e.g., finding "food" when searching for "foo" with an edit distance of 2).
3.  **Map vs Set**: I switched to using `Map` instead of `Set`, associating a value with each key (currently just `0`).

### Benchmarking & Performance
*   **Compilation**: Interesting finding I found is that `Map` and `MapBuilder` seem to make the program compile faster, though the difference is negligible for this small experiment. I was probably spouting nonsense though.
*   **Speed**:
    *   **Build Time**: Creating the FST from **103,495 words** took approximately **53ms**.
    *   **Search Time (Heap/fs::read)**: ~335µs. (Fastest, pre-loaded).
    *   **Search Time (Mmap)**: ~360µs. (Slightly slower due to page faults).
*   **Storage**: The compression is significant.
    *   Original `dict.txt`: **977 KB**
    *   Compressed `dict.fst`: **279 KB**
    *   **Reduction**: The FST is ~29% of the original file size, achieving a **~71% reduction** in storage.
*   **Memory Mapping (`memmap2`) vs `fs::read`**:
    *   **Result**: The memory mapped search was marginally slower (~25µs difference) for this small ~279KB file.
    *   **Reason**: `fs::read` loads the entire file into "hot" RAM immediately. `mmap` lazily loads pages on demand, incurring data access overhead (page faults) during the first search.
    *   **Verdict**: `fs::read` wins for small dictionaries. `mmap` is essential for massive datasets (GBs) where loading the whole file is impossible or startup time is critical.

### Interactive Performance (REPL Experiment)
I converted the program into an interactive REPL to isolate the **startup cost** from the **search cost**.
*   **Startup**: ~51ms (Building/Loading FST once).
*   **Per-Query Latency**:
    *   Search ("love", dist=1): **~370µs**
    *   Search ("funny", dist=1): **~241µs**
*   **Insight**: Once the FST is memory-mapped, the cost of constructing a Levenshtein automaton and streaming results is consistently **sub-millisecond**, making it suitable for real-time tasks like autocompletion.
