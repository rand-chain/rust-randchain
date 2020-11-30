#!/bin/bash

deps=$(cat Cargo.toml | grep path | cut -f1 -d' ')

for dep in $deps
do
cargo test -p $dep
done 

# cargo test \
# -p chain \
# -p message \
# -p network \
# -p miner \
# -p p2p \
# -p storage \
# -p db \
# -p verification \
# -p sync \
# -p logs \
# -p rpc \
# -p primitives \