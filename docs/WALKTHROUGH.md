# RUSTY-ROLODEX WEEKLY WALKTHROUGH

## TABLE OF CONTENTS
* [**Week 1 Walkthrough**](#rusty-rolodex---week-1-walkthrough)
    - [Initial Project Setup](#1-initial-project-setup)  
    - [Setting up project structure](#2-Setting-up-project-structure)
    - [Setup store.rs](#3-setup-storers)
    - [Setup domain.rs](#4-setup-domainrs)
    - [Program flow (main.rs)](#5-program-flow-mainrs)
    - [Interface (cli.rs)](#6-interface-clirs)
    - [Validation (validation.rs)](#7-validation-validationrs)
    - [Verification & Test](#8-verification--test)




## Rusty Rolodex - Week 1 Walkthrough

### 1. Initial Project Setup
First I created a new remote Github repo and a new local project directory and initialize git on the local repo with the remote repo:
```bash
mkdir rusty-rolodex
cd rusty-rolodex
echo "# rusty-rolodex" >> README.md
git init
git add README.md
git commit -m "first commit"
git branch -M main
git remote add origin https://github.com/uche09/rusty-rolodex.git
```

Then I used cargo to initialize a new rust project:  
`cargo init`  
this command created a directory structure:
```bash
rusty-rolodex
|
|
|--src/
|   |--main.rs
|
|--target/
|
|--.gitignore
|--Cargo.lock
|--Cargo.toml
```


### 2. Setting up project structure
Ran the following the command to setup the project structure as specified in the [project gist]:
```bash rusty-rolodex/:
mkdir docs test scripts examples
touch docs/CHANGELOG.md docs/WALKTHROUGH.md docs/weekly-notes.md
touch src/cli.rs src/domain.rs src/store.rs src/validation.rs
```

### 3. Setup store.rs
In src/store.rs I implement an abstraction that can be used to swith between different storage medium e.g in memory, or in file etc all from one place.
```rust src/store.rs:
src/store.rs:

pub struct Store {
    pub mem: Vec<Contact>,
    pub file: File,
}

impl Store {
    pub fn new() -> Self {
        create_file_parent();
        let file = File::create(FILE_PATH).unwrap();
        Store {
            mem: Vec::new(),
            file,
        }
    }
}
```


### 4. Setup domain.rs
I organized core logic specific to project context in this "domain" module such as adding contact, deleting contact, listing stored contacts etc while keeping abstraction in mind, it was done to the best of my current knowledge at the time of implementation.

```rust rc/domain.rs:
src/domain.rs:

pub struct Contact {
    pub name: String,
    pub phone: String,
    pub email: String,
}

pub enum Command {
    AddContact,
    ListContacts,
    DeleteContact,
    Exit,
}

pub struct ContactStore {
    store: Store,
}

impl ContactStore {
    pub fn new() -> Self {
        ContactStore {
            store: Store::new(),
        }
    }

    pub fn add_contact(&mut self, contact: Contact) {
        self.store.mem.push(contact);
    }

    pub fn contact_list(&self) -> &Vec<Contact> {
        &self.store.mem
    }

    pub fn delete_contact(&mut self, index: usize) -> Result<(), String> {
        if index < self.store.mem.len() {
            self.store.mem.remove(index);
            Ok(())
        } else {
            Err("No found".to_string())
        }
    }
}
```

### 5. Program flow (main.rs)
Now that I have the the core logic abstracted away, I can start piecing them together to create the program flow in main.rs
```rust src/main.rs
src/main.rs:

println!("\n\n--- Contact BOOK ---\n");

'outerloop: loop {
    cli::show_menu();

    let action = cli::get_input();

    match cli::get_command(&action) {
        Ok(command) => {
            // User entered valid command

            match command {
                Command::AddContact => {
                    ... // some code removed
                    storage.add_contact(new_contact);

                    println!("Contact added successfully!");
                    ... // some code removed
                }
                Command::ListContacts => {
                    ... // Some code removed
                    for contact in storage.contact_list().iter() {
                        println!();
                        println!("{}", cli::display_contact(contact));
                    }
                }
                Command::DeleteContact => {
                    ... // some code removed
                    storage.delete_contact(index);

                    println!("Contact deleted successfully!");
                }
                Command::Exit => {
                    println!("\nBye!");
                    exit(0);
                }
            }
        }
        Err(message) => {
            // User entered invalid command
            println!("{}", message);
            continue 'outerloop;
        }
    }
}
```


### 6. Interface (cli.rs)
While I was creating the program workflow, I identified areas that requires user interaction or I/O and created some I/O functions in cli.rs.
```rust src/cli.rs
src/cli.rs:

pub fn show_menu() {
    println!("\n");
    println!("1. Add Contact");
    println!("2. List Contacts");
    println!("3. Delete Contact");
    println!("4. Exit");
    print!("> ");
    io::stdout().flush().unwrap();
}

pub fn get_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

pub fn get_command(input: &str) -> Result<Command, String> {
    match input {
        "1" => Ok(Command::AddContact),
        "2" => Ok(Command::ListContacts),
        "3" => Ok(Command::DeleteContact),
        "4" => Ok(Command::Exit),
        _ => Err("Invalid command.".to_string()),
    }
}
```

### 7. Validation (validation.rs)
While I building the interface, I also identified inputs that needs validation, and I implemented validation functions for them in validation.rs
```rust src/validation.rs
src/validation.rs:

pub fn validate_name(name: &str) -> bool {
    // Must be alphabetic and non-empty
    // Name may contain spaces between alphabets
    name.chars().count() > 0 && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
}

pub fn validate_number(phone: &str) -> bool {
    // Must be at least 10 digits
    // Must contain only digits
    phone.chars().count() >= 10 && phone.chars().all(|c| c.is_ascii_digit())
}
```

### 8. Verification & Test
On completion of the work flow for the week 1 implementation of the project as specified in the [project gist], I ran the following command to check for errors and further improve code:
- `cargo check` This would check the program for any error without running the program or producing an executable file. Got some errors and warning that has been fixed before this documentation and are undocumented.
- `cargo clippy` This acts pretty much like `cargo check`. Highlighted some warning which were corrected too.
- `cargo fmt` Helped format the code properly, removing unnecessary white spaces, adding necessary white spaces and indentation etc.

These commands helped **verify** that the code base is error-free.  
Then I **manually** ran the code to **test** that is was working properly, here is the interaction:
```bash
rusty-rolodex$ cargo run

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

### 9. Push Changes
After a succuful test, I push the changes to the remote github repo.

```bash
git add .
git commit -m "<message>"
git push
```
`git push` command required me to input my github username and password (Personal access token) to complete execution.




[project gist]: (https://gist.github.com/Iamdavidonuh/062da8918a2d333b2150c74cae6bd525)