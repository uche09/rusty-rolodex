# Rusty Rolodex
```
  _____________________ ____________________________  __
___  __ \_  __ \__  / __  __ \__  __ \__  ____/_  |/ /
__  /_/ /  / / /_  /  _  / / /_  / / /_  __/  __    / 
_  _, _// /_/ /_  /___/ /_/ /_  /_/ /_  /___  _    |  
/_/ |_| \____/ /_____/\____/ /_____/ /_____/  /_/|_|  
                                                                         
 Mastery Track (Beginner → Expert, One Project)                           
```

[![Rust Version](https://img.shields.io/badge/Rust-1.78+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Pull Request Test](https://github.com/uche09/rusty-rolodex/actions/workflows/ci.yml/badge.svg)](https://github.com/uche09/rusty-rolodex/actions/workflows/ci.yml)
<!-- [![License](https://img.shields.io/badge/license-MIT-green.svg?style=flat-square)](LICENSE) -->

---


Rusty Rolodex is a single evolving project — Rusty Rolodex — that grows from an in‑memory CLI address book into a production‑grade, test‑covered, CI/CD‑deployed tool.  
Check out more about [Rusty-Rolodex](https://gist.github.com/Iamdavidonuh/062da8918a2d333b2150c74cae6bd525)

![Project Demo](./docs/media/rolodex-demoV2.gif)

## Current Features
- Add a new contact with name, phone number and email
- View all saved contacts
- Search for a contact by name
- Delete a contact by name
- Basic user input handling and validation
<!-- - Save contacts to a file so that data is persistent across sessions.
- Update existing contacts.
- Display contacts alphabetically. -->


## What I Learned
Working on this project helped me practice and understand:
- **Structs**: Using structs to represent a `Contact` with name and phone fields.
- **Enum**: Using represent unique variants like user possible action/command.
- **Vectors**: Storing and managing a dynamic list of contacts.
- **Ownership and Borrowing**: Handling references properly when adding, searching, and deleting contacts.
- **Pattern Matching**: Using `match` statements to control user options and handle possible errors.
- **Input/Output**: Reading user input from the terminal and processing it.
- **Error Handling**: Managing common issues like invalid input and empty searches and also using rust powerful features for error handling like `Result<T, E>`.
- **Methods**: Using the `impl` construct to implement methods.
- **Trait**: How to implement traits (behavior) that can be implemented (inherited) by other constructs.
**Generic**: Reduce duplicated logic by implementing once and use it on any data types or constructs.
**GitHub Workflow**: This project doesn't just aim to improve my rust proficiency, but also aims at making me a better developer by applying standard professional level best practices. I added some branch protection ruleset and implemented a CI workflow using GitHub workflow.
<!-- - **Regex**: Validating user inputs like phone number and email using regex pattern. -->

## Challenges Encountered
- Managing **borrowing and ownership rules**, especially when parsing variables to some built-in construct without knowing if they consume the value (take ownership) or reference them by default.
- Handling **mutable and immutable references** correctly without causing borrow checker errors.
- Keeping the program **interactive** and **user-friendly** while avoiding panics.
- Rapping my head around some of the concept in rust can be challenging and requires indepth study, as some of these concepts are very technical, low level, or unique to rust and have never learned them anywhere before, and I have to learn them good enough to implement the next feature in the project within a very short and limited time.

And I was able to overcome these challenges to the best of my knowledge yet. And thankfully I always have `cargo clippy` to help me catch and properly explain the cause of errors.

## Example Usage

```bash
--- Contact BOOK ---



1. Add Contact
2. List Contacts
3. Delete Contact
4. Exit
> 1

Enter contact name 
* to go back: 


Invalid Name input.

Enter contact name 
* to go back: 
Uche

Enter contact number:
abcdefghijklmn

Invalid Number input.

Enter contact name 
* to go back: 
Uche

Enter contact number:
12345678901

Enter contact email.
uche.uche

Invalid email input.

Enter contact name 
* to go back: 
Uche

Enter contact number:
12345678901

Enter contact email.
uche@gmail.com
Are you sure you want to add this contact to your contact list 
Name: Uche
Number: 12345678901
Email: uche@gmail.com
? (y/n)
> y
Contact added successfully!


1. Add Contact
2. List Contacts
3. Delete Contact
4. Exit
> 2

Name: Uche
Number: 12345678901
Email: uche@gmail.com


1. Add Contact
2. List Contacts
3. Delete Contact
4. Exit
> 3
Search contact by name to DELETE or * to go back
uche
Name not found in contact list
Search contact by name to DELETE or * to go back
*   


1. Add Contact
2. List Contacts
3. Delete Contact
4. Exit
> 3
Search contact by name to DELETE or * to go back
Uche
Are you sure you want to delete this contact from your contact list 
Name: Uche
Number: 12345678901
Email: uche@gmail.com
? (y/n)
> y
Contact deleted successfully!


1. Add Contact
2. List Contacts
3. Delete Contact
4. Exit
> 2
No contact in contact list! 


1. Add Contact
2. List Contacts
3. Delete Contact
4. Exit
> 4

Bye!
```

## How to Run

1. Make sure you have Rust installed. If not, install it from [here](https://www.rust-lang.org/tools/install)

2. Clone the repository:

```bash
git clone https://github.com/uche09/rusty-rolodex.git
```

3. Navigate to the project directory:

```bash
cd rusty-rolodex
```

4. Build and run the project:

```bash
cargo run
```


### If you are excited about seeing how far I go in my rust journey, give me a star ⭐