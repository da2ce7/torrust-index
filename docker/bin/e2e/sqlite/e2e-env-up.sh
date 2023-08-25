#!/bin/bash


docker compose build


TORRUST_INDEX_CONFIG=$(cat config-index.sqlite.local.toml) \
TORRUST_TRACKER_CONFIG=$(cat config-tracker.local.toml) \
TORRUST_TRACKER_API_TOKEN=${TORRUST_TRACKER_API_TOKEN:-MyAccessToken} \
docker compose up -d
