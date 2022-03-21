#!/bin/sh

sea-orm-cli generate entity -o src/sql_entities
sea-orm-cli generate entity -o .tmp --with-serde both
mv .tmp/navaccess.rs src/sql_entities/navaccess.rs
rm -r .tmp
