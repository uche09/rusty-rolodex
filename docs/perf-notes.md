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



### Observations

**Overview:**  
The benchmark evaluates how long core operations in the system take as dataset size grows from 1k to 100k contacts. All timings represent approximate worst-case scenarios where the all contacts fits into just one subset of the index, especially for search-related operations where fuzzy matching forces every record to be scanned.

**The core insight implies that:** the system scales linearly (O(n)) across all operations, and this is exactly what the timing curves reveal.

#### Key Observations
1. **Linear Growth Across the Board:**  
    Every operation shows clear linear scaling.
    For example:  

    - add: 0.20 ms -- 69.33 ms

    - search --name: 0.47 ms -- 83.31 ms

    - list (sort + filter): 0.84 ms -- 334.70 ms

    This is expected because all these operations perform work proportional to the dataset size, searching, sorting, filtering, or serializing data.

    **This implies that the system behaves predictably under increasing load, but will continue to slow proportionally as data increases.**

2. **Search and List Operations Are Naturally the Slowest**

    The list (sort + filter) operation is consistently the highest in cost:

    - At 100k, it reaches ~334 ms, the highest in the table.

    This is because sorting is at least O(n log n) which is done on the entire data in the dataset and combined with filters, it becomes the most computationally expensive operation.

    **This implies that for larger datasets, this is the main candidate for optimization.**

3. **File Operations Become Expensive at Scale**

    Saving and reading contacts (both JSON and TXT) show significant growth:

    - JSON save: 1.80 ms -- 297.85 ms

    - JSON read: 1.80 ms -- 246.84 ms

    - TXT read: 2.65 ms -- 311.38 ms

    These operations involve full serialization/deserialization of all contacts, which is inherently O(n).
    TXT is notably slower than JSON for reading at higher volumes due to parsing overhead.

    **This implies that for production-scale data, switching to a database or incremental file updates will be necessary.**

4. **Editing and Deleting Remain Cheap**

    Both edit and delete are extremely fast relative to other operations:

    - edit: 0.04 ms -- 7.03 ms

    - delete: 0.26 ms → 73.61 ms

    They grow linearly, but their baseline cost is low because they only manipulate a single entry once located.

    **This implies that the bottleneck in these operations is finding the record, not updating it.**

### Overall Assessment

- Search-heavy tasks will slow down significantly as data grows, especially with fuzzy logic requiring full scans.

- File I/O is the major bottleneck at scale, due to full reload/rewrite of the dataset.

**NOTE:** This benchmark was done on the worst-case scenario where contacts names were formatted as User{i}, where i increases from 0 - sample size. This means the alphabetical index implemented in project will group all contacts in a single set.  
But all things being equal, in a real world application the data would be splited and search would perform even better, except of course in worst-case scenario just like this.


The benchmark proves your design is predictable and stable, even if not optimized for massive datasets.