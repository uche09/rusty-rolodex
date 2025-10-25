# RUSTY-ROLODEX WEEKLY WALKTHROUGH


## Rusty Rolodex - Week 5 Walkthrough

### PartialEq
+ I implemented the `PartialEq` trait to define the basis of equality (name and number) between contacts, as oppose to initially deriving the trait on Contact struct.

### DateTime Fields
+ Some compulsory fields (`created_at` and `updated_at`) has been added to the Contact struct. They are automatically assigned when you use the `Contact::new()` constructor.

### Update Sort command
+ Listing of contact can now be sorted by date created (`created_at`) or by date updated (`updated_at`).
+ List `--reverse` | `-r` sub-command to sort in reverse, e.g. sort date created from newest to oldest.

### Added Edit Command
+ I added the `edit` command to modify contact information.
+ edit --name <current_name> --phone <current_phone> [OPTION] --newname <update_name> --newphone <update_phone> --newemail <update_email> --newtag <update_tag> .

### Import & Export (.csv)
+ I was able to implement a module that allows importing contacts from a .csv file from any permited storage location by running the import command and the source file path (`rusty-rolodex import --src /home/uche/Downloads/sample_contacts.csv`).
+ I also implemented an export feature to export/backup contacts to any permited location location in storage by running the import command and the destination path `rusty-rolodex export --des /home/uche/Downloads`.




### Demo week 4
![Demo GIF](./media/rolodex-demoV4.gif)






## Rusty Rolodex - Week 6 Walkthrough

### Index & Search
+ I implemented a search feature that allows searching of contact via contact's name or contact's email domain (yahoo.com).
+ The underlying implementation of the search feature includes:
    - I created an in-memory index with a Hashmap to organize contact alphabeticaly by name, and by email domain.
    - The indexing process **concurrently** iterates through halves of the contactlist, and organize them by name (alphabetically) or by domain, depending on which function is called.
    - By default the Search command searches by name, or email domain if specified otherwise.
    - The search retrieves the first character of the search string as key when searching by name, or the uses the search string as key when searching by email domain. This key is used to retrieve a smaller set of the entire contactlist that matches the key, and again **concurrently** iterates through halves of this smaller matching list and uses the `rust-fuzzy-search` module to filter and collect contacts whose name/email-domain closely resembles the user provided search string, using distance of >= 7.

### Generic Store
+ I refactored the storage handlers from having `JsonStore` for handling json storage and `TxtStore` for handling txt storage, into a single generic `Store` struct that uses the same function for both json txt storage hence reducing duplicate codes.


### Micro-Benchmarking
+ This benchmark is more like a stress test.
+ This benchmarking was done with the `criterion` crate.
+ The 5k and 100k (denoting five thousand and hundred thousand repectively) doesn't only represent the size of the sample data, but also represent the amount of task/work (iteration) done to get each reading.
+ That means each reading reading simply says, for example *"it takes approx this amount of time to iteratively perform 5 thousand searches in a data set of 5 thousand contacts"*.
+ This Benchmark mostly assumes worst case scenario. For example the search benchmark literally matches the entre 5/100 thousand contacts.


### Demo week 4
![Demo GIF](./media/rolodex-demoV4.gif)




[project gist]: (https://gist.github.com/Iamdavidonuh/062da8918a2d333b2150c74cae6bd525)