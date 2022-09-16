#!/bin/bash
set -ex

tag="${1:-dev}"

docker build -t "fakelogs:${tag}" .

