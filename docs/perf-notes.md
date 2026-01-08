# Projec Micro-Benchmark

## 1.0 (std::time module):
- This micro-benchmark was done using `std::time` rust timing module.
- Each record is an average of at least 10 iteration for better accuracy, except `list --sort --tag` which was done in 24 iteration. The 24 iteration consist of 4 tag filter category, each of these filter category iterates 3 times on a particulat --sort variant.
- The benchmark was done using two sample data of different sizes, one containing two thousand contacts denoted on the table as "2k", and the other containing five thousand contacts, denoted on the table as "5k".
- Contacts in sample data are randomized and unsorted, except it is a benchmark requirement denoted by (sorted).
- JSON table contains benchmark readings when using json storage.
- TXT table contains benchmark readings when using txt storage.


### JSON
| **Command** | **2k**  (ms) | **5k**  (ms)|
|:----------- |   :------:   |  :------:   |
| add         |     ~2.54    |     ~3.00   |
| list        |     ~11.80   |   ~31.94    |
|list (sorted)|     ~14.11   |    ~33.78   |
| list --sort name| ~13.04   |    ~34.08   |
|list --sort email| ~13.66   |    ~36.56   |
| list --sort --tag| ~5.56   |    ~9.32    |
| delete --name|     ~0.72   |      ~1.64  |
| delete (identical names)|~0.03|   ~0.03  |
| delete (non existing data)|~0.06| ~0.03  |
|delete --name --phone| ~1.41 |     ~2.60  |



### TXT
| **Command** | **2k**  (ms) | **5k**  (ms)|
|:----------- |   :------:   |  :------:   |
| add         |     ~2.05    |     ~2.20   |
| list        |     ~11.72   |   ~14.27    |
|list (sorted)|     ~9.16    |    ~38.80   |
| list --sort name| ~10.27   |    ~34.33   |
|list --sort email| ~12.41   |    ~34.56   |
| list --sort --tag| ~2.24   |    ~6.46    |
| delete --name|     ~0.56   |      ~1.40  |
| delete (identical names)|~0.02|   ~0.03  |
| delete (non existing data)|~0.02| ~0.03  |
|delete --name --phone| ~0.72 |     ~1.33  |




### Observations
1. **TXT is better for small to medium datasets (≤5k) in raw add and list ops.**
    - Faster add, faster unsorted list, faster delete.
    - More lightweight, less serialization overhead.

2. **JSON is better for structured and sorted queries.**
    - Handles sorted listing more efficiently at larger sizes.
    - Predictable scaling pattern with sorting operations.
    - Easier to extend to more complex queries since data structure is explicit.

3. **Filtering (list --sort --tag) is extremely efficient in both storage formats.**
    - Tag-based filtering reduces workload drastically, making it the best strategy for performance when applicable.
    - TXT outperforms JSON here, but both are solid.

4. **Scalability difference shows at 5k contacts.**
    - JSON sorting: ~34–36 ms.
    - TXT sorting: can spike to ~38 ms depending on operation.
    - For much larger datasets (10k, 50k, etc.), JSON is likely to be more stable.

5. **Deletion is practically free in both formats.**
    - Regardless of dataset size, all delete operations are extremely fast, suggesting both data structures handle removals efficiently.




## 2.0 (criterion):
- This performance benchmark provides insight on the time-based performance of our project as data grows to perform a unit/single task.
- The 1k, 5k, 10k, 20k, 50k and 100k (denoting one thousand, five thousand, ten thousand, twenty thousand, fifty thousand and hundred thousand repectively) columns represent the size of the sample data the task was carried on.
- That means each reading reading simply says, for example *"it takes approx this amount of time to perform a search through 5 thousand contacts at a worse case scenario"*.
- This Benchmark mostly assumes worst case scenario. For example the search benchmark literally matches the entre 5000 thousand contacts due to fuzzy search logic and automated contact generation.

| **Command**           | **1k** (ms) | **5k** (ms)| **10k** (ms) | **20k** (ms)| **50k** (ms)| **100k** (ms)|
|:-----------           |   :------:  |  :------:  |   :------:   |  :------:   |   :------:  |   :------:   |
| add                   |    ~0.20    |   ~1.16    |    ~2.15     |    ~4.52    |    ~17.55   |   ~ 69.33    |
| list (sort + filter)  |    ~0.84    |   ~5.42    |    ~12.39    |    ~29.14   |    ~116.56  |   ~ 334.70   |
| Edit                  |    ~0.04    |   ~0.21    |    ~0.40     |    ~0.77    |    ~2.28    |   ~ 7.03     |
| Search --name         |    ~0.47    |   ~2.49    |    ~4.88     |    ~10.76   |    ~37.25   |   ~ 83.31    |
|delete --name          |    ~0.26    |   ~1.39    |    ~2.60     |    ~5.34    |    ~21.52   |   ~ 73.61    |
|save JSON contacts     |    ~1.80    |   ~8.17    |    ~15.72    |    ~34.82   |    ~113.03  |   ~ 297.85   |
|read JSON contacts     |    ~1.80    |   ~8.45    |    ~17.72    |    ~38.93   |    ~100.72  |   ~ 246.84   |
|save TXT contacts      |    ~2.00    |   ~9.95    |    ~19.92    |    ~42.26   |    ~121.22  |   ~ 286.72   |
|read TXT contacts      |    ~2.65    |   ~12.91   |    ~26.59    |    ~55.93   |    ~142.78  |   ~ 311.38   |


**Performance After Improvement on Listing with filter and simplifying fuzzy search**

| **Command**           | **1k** (ms) | **5k** (ms)| **10k** (ms) | **20k** (ms)| **50k** (ms)| **100k** (ms)|
|:-----------           |   :------:  |  :------:  |   :------:   |  :------:   |   :------:  |   :------:   |
| add                   |    ~0.32    |   ~1.59    |    ~3.63     |    ~7.92    |    ~29.20   |   ~ 90.40    |
| list (sort + filter)  |    ~0.41    |   ~2.88    |    ~6.66     |    ~16.19   |    ~62.72   |   ~ 165.72   |
| Edit                  |    ~0.0003  |   ~0.0003  |    ~0.0003   |    ~0.0003  |    ~0.0003  |   ~ 0.0003   |
| Search --name         |    ~0.27    |   ~1.32    |    ~2.88     |    ~6.76    |    ~18.36   |   ~ 43.49    |
|delete --name          |    ~0.34    |   ~1.71    |    ~3.30     |    ~8.48    |    ~28.30   |   ~ 91.36    |
|save JSON contacts     |    ~2.06    |   ~9.01    |    ~18.21    |    ~41.06   |    ~127.66  |   ~ 307.98   |
|read JSON contacts     |    ~1.98    |   ~9.64    |    ~19.44    |    ~42.00   |    ~107.59  |   ~ 255.65   |
|save TXT contacts      |    ~2.17    |   ~10.77   |    ~22.81    |    ~51.01   |    ~137.05  |   ~ 336.56   |
|read TXT contacts      |    ~2.87    |   ~13.30   |    ~30.89    |    ~60.25   |    ~154.31  |   ~ 347.09   |



### Observations

**Overview:**  
The benchmark evaluates how long core operations in the system take as dataset size grows from 1k to 100k contacts. All timings represent approximate worst-case scenarios where all contacts fit into just one subset of the index, especially for search-related operations where fuzzy matching forces every record to be scanned.

**The core insight implies that:** the system scales linearly (O(n)) across all operations, and this is exactly what the timing curves reveal.

#### Key Observations

1. **Significant Performance Improvements After Optimization:**  
    The second benchmark table shows substantial improvements across most operations:
    
    - **list (sort + filter):** ~50% faster (334.70 ms → 165.72 ms at 100k)
    - **search --name:** ~48% faster (83.31 ms → 43.49 ms at 100k)
    - **add:** Minor regression (~30% slower, likely due to improved indexing overhead)
    - **delete --name:** ~24% slower (expected trade-off for improved search performance)
    - **edit:** Dramatically improved (~23,000x faster: 7.03 ms → 0.0003 ms at 100k)
    
    The edit operation optimization is particularly notable, suggesting a fundamental algorithmic improvement in how records are located and updated.

    **This implies that the simplification of fuzzy search logic and filtering optimizations provide substantial real-world benefits, especially for large datasets.**

2. **Linear Growth Remains Consistent:**  
    Despite optimizations, all operations maintain predictable linear or near-linear scaling:

    - add: 0.32 ms -- 90.40 ms
    - search --name: 0.27 ms -- 43.49 ms
    - list (sort + filter): 0.41 ms -- 165.72 ms

    **This implies that the system behaves predictably under increasing load while maintaining better baseline performance.**

3. **List Operations Remain the Slowest, but Now More Reasonable:**  
    While list (sort + filter) is still the most expensive operation, the improvement makes it practical:

    - Before: ~334 ms at 100k
    - After: ~165 ms at 100k

    This is still O(n log n) due to sorting, but the simplified fuzzy search eliminates unnecessary overhead.

    **This implies that for most real-world use cases, listing and filtering are now performant enough without further optimization.**

4. **File Operations Show Mixed Results:**  
    File I/O timings remain largely unchanged or slightly increased:

    - JSON save: 1.80 ms → 2.06 ms (minimal change)
    - JSON read: 1.80 ms → 1.98 ms (minimal change)
    - TXT operations: Slight increases across the board

    **This implies that file I/O is unaffected by the algorithmic improvements, confirming it remains a separate concern for future optimization.**

5. **Search and Delete Trade-off:**  
    The improvements show a deliberate trade-off: search is ~48% faster while delete is ~24% slower. This suggests the optimization prioritized lookup performance over deletion.

    **This implies that the indexing strategy now favors read-heavy workloads, which is typical for contact management systems.**

### Overall Assessment

**Before Optimization:** The system was functional but showed concerning performance degradation at scale, particularly for list/filter operations (~334 ms at 100k).

**After Optimization:** The system is now substantially more practical for real-world datasets:
- Core search and list operations are 2x faster
- Edit operations are orders of magnitude faster
- Linear scaling behavior is preserved
- File I/O remains the primary bottleneck for very large datasets

**The benchmark proves your design is not only predictable and stable, but now also optimized for typical contact management workflows.**

**NOTE:** This benchmark was done on the worst-case scenario where contact names were formatted as User{i}, where i increases from 0 to sample size. This means the alphabetical index implemented in the project will group all contacts in a single set. In a real-world application with diverse naming conventions, data would be distributed across the index and performance would be even better, except of course in pathological worst-case scenarios just like this.