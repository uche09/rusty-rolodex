# Projec Micro-Benchmark

## Note:
- This micro-benchmark was done using `std::time` rust timing module.
- Each record is an average of at least 10 iteration for better accuracy, except `list --sort --tag` which was done in 24 iteration. The 24 iteration consist of 4 tag filter category, each of these filter category iterates 3 times on a particulat --sort variant.
- The benchmark was done using two sample data of different sizes, one containing two thousand contacts denoted on the table as "2k", and the other containing five thousand contacts, denoted on the table as "5k".
- Contacts in sample data are randomized and unsorted, except it is a benchmark requirement denoted by (sorted).
- JSON table contains benchmark readings when using json storage.
- TXT table contains benchmark readings when using txt storage.


## JSON
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



## TXT
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




# Search Performance

## Search
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
