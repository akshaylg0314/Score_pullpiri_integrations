#!/bin/bash
# SPDX-FileCopyrightText: Copyright 2024 LG Electronics Inc.
# SPDX-License-Identifier: Apache-2.0

# Apply each artifact YAML one by one

for yaml in ./resources/driver-distraction-10sec.yaml ./resources/driver-distraction-5sec.yaml; do
	echo "Applying artifact: $yaml"
	curl --location 'http://0.0.0.0:47099/api/artifact' \
		--header 'Content-Type: text/plain' \
		--data "$(cat $yaml)"
	echo # newline for readability
done