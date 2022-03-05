#!/bin/sh

sea-orm-cli generate entity -o src/entities
sea-orm-cli generate entity -o .tmp --with-serde both
mv .tmp/navaccess.rs src/entities/navaccess.rs
rm -r .tmp
