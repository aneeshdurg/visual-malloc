#!/bin/bash
cargo web deploy
cp -r target/deploy/* .
cp _index.html index.html
