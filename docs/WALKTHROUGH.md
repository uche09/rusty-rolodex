# RUSTY-ROLODEX WEEKLY WALKTHROUGH


## Rusty Rolodex - Week 7 Walkthrough

### Synchronization workflow
- **Merge Policy**: I implemented the ***last-write-wins*** merge policy for the synchronization workflow.
- **Assumptions**:
    - Contacts with same id and same created_at timestamp are considered the same contact.
    - Contacts with same id but different created_at timestamps are considered id collision or date conflict.

### Tested Senarios
- SCENARIO 1: Same contact edited in two places
- SCENARIO 2: Remote file contains new contacts
- SCENARIO 3: Local delete vs remote edit
- SCENARIO 4: Import fails halfway
- SCENARIO 5: Conflicting or missing timestamps
- SCENARIO 6: Duplicate entries from different devices
- EDGE CASES
    - Empty remote
    - All contacts deleted locally but not deleted in remote
    - Index updates after merge
    - Complex scenario: multiple contacts with add, update, delete, keep operations

### Partial Corruption Handling
- Data is rolled back to last known good state (snapshot) is any error occurs during sync.

### Duplicate Detection:
- Contact implements `PartialEq` which defines equality based on identical id or identical name and phone number.
- Contacts with different ids but identical name and phone number are considered duplicates and ignored during merge.



<!-- ### Demo week 4
![Demo GIF](./media/rolodex-demoV4.gif) -->




[project gist]: (https://gist.github.com/Iamdavidonuh/062da8918a2d333b2150c74cae6bd525)