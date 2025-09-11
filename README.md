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

See latest features and changes in [CHANGELOG.md](./docs/CHANGELOG.md) and [WALKTHROUGH.md](./docs/WALKTHROUGH.md)

![Project Demo](./docs/media/rolodex-demoV2.gif)

## Current Features
- Add a new contact with name, phone number and email
- View all saved contacts
- Search for a contact by name
- Delete a contact by name
- Command line argument input handling and validation using clap
- Save contacts to a file so that data is persistent across sessions.
- List sorted contacts.


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
- **Regex**: Validating user inputs like name, phone number and email using regex pattern.
- **JSON**: Used `serde_json` to parse and store contact in json format.
- **Clap**: Introduced Command Line Argument Parser in project with `clap` crate.
- **Test**: Over the course of this project I've been constantly writing unit test to make sure functionalities works as expected, which is also required to pass the github workflow I set up.

## Challenges Encountered
- Managing **borrowing and ownership rules**, especially when parsing variables to some built-in construct without knowing if they consume the value (take ownership) or reference them by default.
- Handling **mutable and immutable references** correctly without causing borrow checker errors.
- Keeping the program **interactive** and **user-friendly** while avoiding panics.
- Rapping my head around some of the concept in rust can be challenging and requires indepth study, as some of these concepts are very technical, low level, or unique to rust and have never learned them anywhere before, and I have to learn them good enough to implement the next feature in the project within a very short and limited time.

And I was able to overcome these challenges to the best of my knowledge yet. And thankfully I always have `cargo clippy` to help me catch and properly explain the cause of errors.

## Example Usage

```bash
cargo run add --name Jerry --phone 08861473537
Contact added successfully

cargo run add --name Alice --phone +234123456789 --email ailce@gmail.com
Contact added successfully

cargo run add --name james --phone +2348881454872 --email ja.mes@gmail.com
Contact added successfully

cargo run add --name Jerry --phone +2348861473537
Error: Validation("Contact with this name and number already exist")

cargo run add --name daniel --phone 07099512124 --email bigD@yahoo.com
Contact added successfully

cargo run list
  1. Alice                +234123456789   ailce@gmail.com
  2. Jerry                08861473537     
  3. daniel               07099512124     bigD@yahoo.com
  4. james                +2348881454872  ja.mes@gmail.com

cargo run list --sort name
  1. Alice                +234123456789   ailce@gmail.com
  2. daniel               07099512124     bigD@yahoo.com
  3. james                +2348881454872  ja.mes@gmail.com
  4. Jerry                08861473537     

cargo run list --sort email
  1. Jerry                08861473537     
  2. Alice                +234123456789   ailce@gmail.com
  3. daniel               07099512124     bigD@yahoo.com
  4. james                +2348881454872  ja.mes@gmail.com

cargo run add --name Jerry --phone 09422138746 --email jex@gmail.com
Contact added successfully

cargo run add --name 'Dr Sam' --phone 08111111111 --email info@samclinic.ng
Contact added successfully

cargo run list
  1. Alice                +234123456789   ailce@gmail.com
  2. Dr Sam               08111111111     info@samclinic.ng
  3. Jerry                08861473537     
  4. Jerry                09422138746     jex@gmail.com
  5. daniel               07099512124     bigD@yahoo.com
  6. james                +2348881454872  ja.mes@gmail.com


cargo run delete --name Jerry

Deleting failed
Found multiple contacts with this name: Jerry, please provide number. See help

cargo run delete -h
Delete a contact by name provide optional number is case name match multiple contacts

Usage: rusty-rolodex delete [OPTIONS] --name <NAME>

Options:
      --name <NAME>      Name of contact to delete
      --number <NUMBER>  Contact number to delete
  -h, --help             Print help


cargo run delete --name Jerry --number 08861473537
Contact deleted successfully

cargo run delete --name Daniel
Contact Not found

cargo run delete --name daniel
Contact deleted successfully

cargo run list
  1. Alice                +234123456789   ailce@gmail.com
  2. Dr Sam               08111111111     info@samclinic.ng
  3. Jerry                09422138746     jex@gmail.com
  4. james                +2348881454872  ja.mes@gmail.com

cargo run list --sort name
  1. Alice                +234123456789   ailce@gmail.com
  2. Dr Sam               08111111111     info@samclinic.ng
  3. james                +2348881454872  ja.mes@gmail.com
  4. Jerry                09422138746     jex@gmail.com

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