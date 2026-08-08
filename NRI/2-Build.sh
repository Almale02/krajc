#!/bin/bash
set -e

mkdir -p "_Build"

cd "_Build"
cmake .. -DBUILD_SHARED_LIBS=OFF
cmake --build . --config Release -j $(nproc)
cmake --build . --config Debug -j $(nproc)
cd ..
