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
- This performance benchmark provides insight on the time-based performance of our project to perform a unit/single task as the data grows.
- The 1k, 5k, 10k, 20k, 50k and 100k (denoting one thousand, five thousand, ten thousand, twenty thousand, fifty thousand and hundred thousand repectively) columns represent the size of the sample data the task was carried on.
- That means each reading simply says, for example *"it takes approx this amount of time to perform a search through 5 thousand contacts"*.
- The elements of the sample data were randomly generated to mimic real life application/use to an extent.

| **Command**           | **1k** (ms) | **5k** (ms)| **10k** (ms) | **20k** (ms)| **50k** (ms)| **100k** (ms)|
|:-----------           |   :------:  |  :------:  |   :------:   |  :------:   |   :------:  |   :------:   |
| add                   |    ~0.24    |   ~1.16    |    ~2.29     |    ~4.97    |    ~16.51   |   ~ 54.08    |
| list (sort + filter)  |    ~0.16    |   ~1.18    |    ~2.70     |    ~6.01    |    ~19.68   |   ~ 56.35    |
| Edit                  |    ~0.20    |   ~1.17    |    ~2.15     |    ~4.94    |    ~15.20   |   ~ 51.50    |
| Search --name         |    ~0.40    |   ~1.62    |    ~3.14     |    ~5.69    |    ~14.44   |   ~ 30.13    |
|delete --name          |    ~0.21    |   ~1.08    |    ~2.37     |    ~5.30    |    ~16.31   |   ~ 51.67    |
|save JSON contacts     |    ~2.06    |   ~8.53    |    ~18.39    |    ~40.98   |    ~111.06  |   ~ 269.75   |
|read JSON contacts     |    ~2.14    |   ~8.64    |    ~19.27    |    ~45.92   |    ~122.46  |   ~ 261.91   |
|save TXT contacts      |    ~2.21    |   ~9.00    |    ~18.31    |    ~38.71   |    ~118.02  |   ~ 257.01   |
|read TXT contacts      |    ~2.87    |   ~13.22   |    ~29.21    |    ~61.44   |    ~165.73  |   ~ 341.50   |
|increment Index        |    ~0.21    |   ~1.06    |    ~2.27     |    ~4.89    |    ~15.71   |   ~ 48.14    |
|decrement Index        |    ~0.24    |   ~1.10    |    ~2.34     |    ~4.81    |    ~13.89   |   ~ 48.51    |



### Observations

**Overview:**  
The Benchmark 2.0 data illustrates the time-based performance of core operations as the dataset grows from 1k to 100k contacts. All operations exhibit linear scaling (O(n)), with some variations due to algorithmic differences.

#### Key Observations

1. **CRUD Operations (Add, Edit, Delete) Scale Linearly:**  
   - Add: ~0.24 ms (1k) → ~54.08 ms (100k)  
   - Edit: ~0.24 ms (1k) → ~52.28 ms (100k)  
   - Delete --name: ~0.21 ms (1k) → ~51.67 ms (100k)  
   These operations show consistent linear growth, indicating efficient data structures for insertions, updates, and removals.

2. **Search Operations Are Efficient:**  
   - Search --name: ~0.40 ms (1k) → ~30.13 ms (100k)  
   Search is faster than add/edit/delete at larger scales, suggesting the effect of multithreading and the concurrent search.

3. **List Operations Incur Sorting Overhead:**  
   - List (sort + filter): ~0.16 ms (1k) → ~56.35 ms (100k)  
   The time increases more steeply than pure linear, implying O(n log n) complexity due to sorting, making it the slowest user-facing operation.

4. **File I/O Is the Primary Bottleneck:**  
   - Save JSON: ~2.06 ms (1k) → ~269.75 ms (100k)  
   - Read JSON: ~2.14 ms (1k) → ~261.91 ms (100k)  
   - Save TXT: ~2.21 ms (1k) → ~257.01 ms (100k)  
   - Read TXT: ~2.87 ms (1k) → ~341.50 ms (100k)  
   Persistence operations scale linearly but are significantly slower than in-memory operations, with TXT reading being the most expensive at scale.

5. **Index Operations Are Fast and Consistent:**  
   - Increment Index: ~0.21 ms (1k) → ~48.14 ms (100k)  
   - Decrement Index: ~0.24 ms (1k) → ~48.51 ms (100k)  
   These micro-operations remain under 50 ms even at 100k, indicating lightweight index management.

### Overall Assessment

The system demonstrates solid performance for contact management up to 100k entries, with in-memory operations staying under 100 ms. File I/O dominates persistence costs, but remains viable for typical use. Sorting in list operations introduces the most variability, suggesting optimizations like lazy sorting or pagination could improve user experience for large datasets. The linear scaling ensures predictability, making the application suitable for growing workloads without unexpected performance degradation.

**NOTE:** This benchmark was done on data sets with randomized elements (contacts), this randomize selection helped mimic real world use cases to an extent.