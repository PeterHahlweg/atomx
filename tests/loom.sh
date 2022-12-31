#!/bin/bash

LOGFILE="loom.log.json"

if [ -f "$LOGFILE" ]; then
    rm "$LOGFILE"
fi

LOOM_LOG=none \
LOOM_LOCATION=1 \
LOOM_CHECKPOINT_INTERVAL=1 \
LOOM_CHECKPOINT_FILE="$LOGFILE" \
RUSTFLAGS="--cfg loom" \
RUST_BACKTRACE=1 \
cargo test \
    --test loom \
    --features="loom/checkpoint" \
    --release \
    -- \
    --nocapture
