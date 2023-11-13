#!/bin/bash

# Delete the databases and recreate them.

./contrib/dev-tools/container/e2e/mysql/e2e-env-down.sh

# Index

# Database credentials
MYSQL_USER="root"
MYSQL_PASSWORD="root_secret_password"
MYSQL_HOST="localhost"
MYSQL_DATABASE="torrust_index_e2e_testing"

# Create the MySQL database for the index. Assumes MySQL client is installed.
echo "Creating MySQL database $MYSQL_DATABASE for E2E testing ..."
mysql -h $MYSQL_HOST -u $MYSQL_USER -p$MYSQL_PASSWORD -e "DROP DATABASE IF EXISTS $MYSQL_DATABASE; CREATE DATABASE $MYSQL_DATABASE;"

# Tracker

# Delete tracker database
rm -f ./storage/tracker/lib/database/torrust_tracker_e2e_testing.db

# Generate storage directory if it does not exist
mkdir -p "./storage/tracker/lib/database"

# Generate the sqlite database for the tracker if it does not exist
if ! [ -f "./storage/tracker/lib/database/torrust_tracker_e2e_testing.db" ]; then
    sqlite3 ./storage/tracker/lib/database/torrust_tracker_e2e_testing.db "VACUUM;"
fi

./contrib/dev-tools/container/e2e/mysql/e2e-env-up.sh
