#!/bin/bash

TORRUST_IDX_BACK_USER_UID=${TORRUST_IDX_BACK_USER_UID:-1000} \
    docker compose build

TORRUST_IDX_BACK_USER_UID=${TORRUST_IDX_BACK_USER_UID:-1000} \
    TORRUST_IDX_BACK_CONFIG=$(cat config-index.mysql.local.toml) \
    TORRUST_IDX_BACK_MYSQL_DATABASE="torrust_index_backend_e2e_testing" \
    TORRUST_TRACKER_CONFIG=$(cat config-tracker.local.toml) \
    TORRUST_TRACKER_API_TOKEN=${TORRUST_TRACKER_API_TOKEN:-MyAccessToken} \
    docker compose up -d

