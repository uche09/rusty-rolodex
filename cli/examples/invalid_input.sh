#!/bin/bash

# Example: showing validation errors

echo "Trying to add contact with invalid phone"
rusty-rolodex add --name "Bad Phone" --phone "abc123" --email "bad@example.com"

echo "Trying to add contact with invalid email"
rusty-rolodex add --name "No At Sign" --phone "+2348031123456" --email "foo.bar"
