# RUSTY-ROLODEX WEEKLY WALKTHROUGH


## Rusty Rolodex - Week 6 Walkthrough

### Data Structure
+ The `Store` struct now uses `HashMap<Uuid, Contact>` to store contact list in memory.
+ This Data structure was picked for it fast lookup by key, fast insert and delete operation.
+ The HashMap picked over the initial Vec due the tradeoff of the Vec that poses significant bottleneck:
    - Both DS has fast insert/delete operation, but Vec delete operation is more expensive when item is deleted towards the middle index, while HashMap is always O(1).
    - Vec (Array) is preferable for storing fixed or moderatelly growing data, as it became a problem identifying contacts uniquelly by their index position for fast random access as the vec constantlly grew or shrink without searching through the entire Vec at O(n). This is problem made HashMap stood out for me as it allows items to be uniquelly identified always by their key while retaining its fast O(n) lookup by key, and fast insert and delete operation.

### Index & Search
+ I implemented a search feature that allows searching of contact via contact's name or contact's email domain (yahoo.com).
+ The underlying implementation of the search feature includes:
    - I created an in-memory index with a Hashmap to organize contact alphabeticaly by name, and by email domain.
    - The indexing process **concurrently** iterates through multiple chunks (depending on the data size) of the contactlist, and organize them by name (alphabetically) or by domain, depending on which function is called.
    - By default the Search command searches by name, or email domain if specified otherwise.
    - The search retrieves the first character of the search string as key when searching by name, or the uses the search string as key when searching by email domain. This key is used to retrieve a smaller set of the entire contactlist that matches the key, and again **concurrently** iterates through multiple chunks (depending on the data size) of this smaller matching list and uses the `rust-fuzzy-search` module to filter and collect contacts whose name/email-domain closely resembles the user provided search string, using distance of >= 7.

### Generic Store
+ I refactored the storage handlers from having `JsonStore` for handling json storage and `TxtStore` for handling txt storage, into a single generic `Store` struct that uses the same function for both json txt storage hence reducing duplicate codes.


### Micro-Benchmarking
+ This benchmark is more like a stress test.
+ This benchmarking was done with the `criterion` crate.
+ The 1k, 5k, 10k, 20k, 50k and 100k (denoting one thousand, five thousand, ten thousand, twenty thousand, fifty thousand and hundred thousand repectively) columns represent the size of the sample data the task was carried on.
+ That means each reading reading simply says, for example *"it takes approx this amount of time to perform a search through 5 thousand contacts at a worse case scenario"*.
+ This Benchmark mostly assumes worst case scenario. For example the search benchmark literally matches the entre 5000 thousand contacts due to fuzzy search logic and automated contact generation.

See the performance information and the benchmark summary in the [performance note](./perf-notes.md)


<!-- ### Demo week 4
![Demo GIF](./media/rolodex-demoV4.gif) -->




[project gist]: (https://gist.github.com/Iamdavidonuh/062da8918a2d333b2150c74cae6bd525)