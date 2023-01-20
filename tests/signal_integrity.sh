#!/bin/bash

CARGO_CMD1='cargo test --test signal_integrity data_integrity_on_concurrent_access_signal'
CARGO_CMD2='cargo test --release --test signal_integrity data_integrity_on_concurrent_access_signal'
# cargo clean
while [ true ]
do
    echo "-----------------------------------------------------------------------------------------"
    echo "$CARGO_CMD1"
    result="$($CARGO_CMD1 | tee | grep -i panicked)"
    if [[ $result ]]; then
        echo "Found a value: $result" >&2
    fi
    echo "$CARGO_CMD2"
    result="$($CARGO_CMD2 | tee | grep -i panicked)"
    if [[ $result ]]; then
        echo "Found a value: $result" >&2
    fi
done