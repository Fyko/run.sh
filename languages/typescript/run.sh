#!/bin/sh
set -e

printf %s "$1" | tsx -p || true
