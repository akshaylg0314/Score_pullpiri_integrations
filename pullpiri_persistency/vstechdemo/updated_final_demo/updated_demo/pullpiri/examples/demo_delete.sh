#!/bin/bash
# Script to delete artifacts one by one using the API

for yaml in ./resources/driver-distraction-10sec.yaml ./resources/driver-distraction-5sec.yaml; do
  echo "Deleting artifact: $yaml"
  curl --location --request DELETE 'http://0.0.0.0:47099/api/artifact' \
    --header 'Content-Type: text/plain' \
    --data "$(cat $yaml)"
  echo # newline for readability
done
