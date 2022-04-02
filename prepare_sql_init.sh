#!/bin/sh

sudo mysqldump --no-data fbets > sql_init/2_schema.sql
sudo mysqldump fbets ROLE NAVACCESS ROLE_NAVACCESS > sql_init/3_data.sql --no-create-info
