### Just a personal experiment on FST

I've been playing around with the `fst` crate to handle sets of strings efficiently. Here are my quick takeaways:

### What I Learned
1.  **Order Matters**: The `SetBuilder` requires keys to be inserted in **lexicographic order**. I had to sort my vector first.
2.  **Fuzzy Search**: Using the `Levenshtein` automaton allows for powerful approximate string matching (e.g., finding "food" when searching for "foo" with an edit distance of 2).

### Benchmarking & Performance
*   **Speed**: Execution is instantaneous even with the overhead of file I/O.
*   **Storage**: The resulting FST is incredibly compact. My 7 keys were compressed into just **64 bytes**.
*   *Next Step*: I need to benchmark this against a full dictionary (100k+ words) to really see the magnitude of the compression and lookup speed advantages and maybe use map instead of set.

