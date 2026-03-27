#!/bin/bash


# Example: add two contacts and list them

echo "Example: adding contacts and listing them"

# Clean slate
rm -f ./.instance/contacts.json

echo "Adding Ada Lovelace..."
rusty-rolodex add --name "Ada Lovelace" --phone "+2348031234567" --email "ada@example.com"

echo "Adding Alice..."
rusty-rolodex add --name "Alice" --phone "12025550123" --email "alice@navy.mil"

echo "Listing all contacts sorted by name..."
rusty-rolodex list --sort name

echo "Listing all contacts sorted by email:"
rusty-rolodex list --sort email