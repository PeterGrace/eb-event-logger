#!/bin/bash
if [[ ! -z "$DEBUG" ]]
then
    echo "Debug requested, sleeping infinity."
    sleep infinity
    exit 1
fi

/opt/eb-event-logger/eb-event-logger
