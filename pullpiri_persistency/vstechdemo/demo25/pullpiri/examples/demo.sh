#!/bin/bash

BODY=$(< ./resources/helloworld.yaml)
#BODY=$(< ./resources/adas-emergency.yaml)
#BODY=$(< ./resources/adas-manual.yaml)
#BODY=$(< ./resources/adas-autonomous.yaml)
#BODY=$(< ./resources/infotainment_startup.yaml)
#BODY=$(< ./resources/infotainment_emergencypopup.yaml)

curl --location 'http://0.0.0.0:47099/api/artifact' \
--header 'Content-Type: text/plain' \
--data "${BODY}"