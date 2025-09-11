#!/bin/bash

# Example: deleting a contact

echo "Assuming contacts.json already has Ada Lovelace and Alice"

echo "Deleting Ada Lovelace..."
rusty-rolodex delete --name "Ada Lovelace"

echo "Listing remaining contacts..."
rusty-rolodex list
