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
| add                   |    ~0.32    |   ~1.59    |    ~3.63     |    ~7.92    |    ~29.20   |   ~ 90.40    |
| list (sort + filter)  |    ~0.41    |   ~2.88    |    ~6.66     |    ~16.19   |    ~62.72   |   ~ 165.72   |
| Edit                  |    ~0.0003  |   ~0.0003  |    ~0.0003   |    ~0.0003  |    ~0.0003  |   ~ 0.0003   |
| Search --name         |    ~0.49    |   ~2.27    |    ~4.53     |    ~9.93    |    ~32.94   |   ~ 90.71    |
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

1. **Edit Operations Are Exceptionally Fast and Constant:**  
    Edit operations show remarkable performance characteristics across all dataset sizes:
    
    - Consistent ~0.0003 ms across 1k, 5k, 10k, 20k, 50k, and 100k contacts
    - Effectively O(1) in complexity, indicating direct lookups
    - The operation completes in microseconds regardless of dataset size
    
    **This implies that editing is not a bottleneck and users can modify contact information with virtually no perceivable delay.**

2. **List Operations Scale O(n log n) Due to Sorting:**  
    List operations with sorting and filtering show clear quadratic-logarithmic growth:
    
    - 1k contacts: ~0.41 ms
    - 100k contacts: ~165.72 ms
    - Growth factor: ~405x for 100x increase in data
    
    **This implies that sorting dominates the list operation cost, making it the most expensive user-facing operation in the system.**

3. **Core CRUD Operations Scale Linearly (O(n)):**  
    Add, search, and delete operations follow predictable linear scaling:
    
    - Add: 0.32 ms (1k) → 90.40 ms (100k)
    - Search --name: 0.49 ms (1k) → 90.71 ms (100k)
    - Delete --name: 0.34 ms (1k) → 91.36 ms (100k)
    - Approximate growth: ~280x for 100x increase in data
    
    **This implies that the system maintains efficient linear performance characteristics, allowing it to handle growing datasets predictably.**

4. **Search and Delete Have Nearly Identical Scaling Rates:**  
    Both search and delete operations consume approximately the same time across all dataset sizes:
    
    - 1k: search 0.49 ms vs delete 0.34 ms
    - 100k: search 90.71 ms vs delete 91.36 ms
    
    **This implies that both operations likely traverse similar index structures, and deleting is not significantly more expensive than searching.**

5. **File I/O Operations Scale Linearly and Are More Expensive Than In-Memory Operations:**  
    All file operations show clear linear growth:
    
    - JSON save: 2.06 ms (1k) → 307.98 ms (100k)
    - JSON read: 1.98 ms (1k) → 255.65 ms (100k)
    - TXT save: 2.17 ms (1k) → 336.56 ms (100k)
    - TXT read: 2.87 ms (1k) → 347.09 ms (100k)
    
    TXT and JSON show similar performance profiles, with TXT reading being slightly slower at larger scales.
    
    **This implies that file I/O is the primary bottleneck for persistence operations, but remains acceptable for contact management workflows.**

6. **Microoperations (Add, Search, Delete) Remain Fast Even at 100k Scale:**  
    Individual in-memory operations stay under 100 ms even with 100k contacts:
    
    - Fastest: Edit at ~0.0003 ms
    - Slowest among micro-ops: Search and Delete at ~90 ms
    
    **This implies that the system remains responsive for single-contact operations and interactive workflows.**

### Overall Assessment

The benchmark demonstrates a well-architected system with predictable performance characteristics:
- **Edit operations** provide near-instant feedback to users
- **List operations** are the performance constraint due to O(n log n) sorting requirements
- **Core CRUD operations** scale linearly and remain practical up to 100k contacts
- **File I/O** is the persistence bottleneck, not algorithmic complexity

The system is suitable for real-world contact management workloads with hundreds of thousands of contacts. For interactive use cases, single operations complete in milliseconds. For batch operations like full listing with sorting, users should expect sub-second response times up to 100k contacts.

**NOTE:** This benchmark was done on the worst-case scenario where contact names were formatted as User{i}, where i increases from 0 to sample size. This means the alphabetical index implemented in the project will group all contacts in a single set. In a real-world application with diverse naming conventions, data would be distributed across the index and performance would be even better, except of course in pathological worst-case scenarios just like this.