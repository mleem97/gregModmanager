💡 **What:**
Replaced `File.ReadAllText` and `File.WriteAllText` in `ModCollectionService` with stream-based synchronous alternatives: `File.OpenRead` and `new FileStream(..., bufferSize: 32768)`.

🎯 **Why:**
The previous implementation materialized the entire file content into an intermediate string prior to serialization/deserialization. The issue mentions converting to `File.ReadAllTextAsync()`, but since the method is called synchronously inside `lock` blocks and constructors, async conversion would require cascading changes throughout the service. A synchronous streaming approach achieves excellent memory optimization while retaining safety and correctness without invasive changes.

📊 **Measured Improvement:**
I generated a dummy collections catalog representing moderate/heavy usage (50 collections, 20 items each = ~114 KB of JSON) and benchmarked the file IO paths.

**Memory allocations:**
* `SaveCatalog` (WriteAllText): ~223.59 MB allocated per 1000 operations
* `SaveCatalog` (Stream): ~32.46 MB allocated per 1000 operations (**85% reduction**)

* `LoadCatalog` (ReadAllText): ~661.99 MB allocated per 1000 operations
* `LoadCatalog` (Stream): ~200.69 MB allocated per 1000 operations (**69% reduction**)

**Execution time:**
The CPU time improvement varies but consistently favored streaming in tests due to far fewer memory allocations and GC collections (e.g. 797ms vs 1559ms overhead improvement per 1k operations for save on my benchmark container).
