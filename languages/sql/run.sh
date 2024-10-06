#!/bin/sh
set -e

printf %s "$1" > program.sql
sqlite3 :memory: < program.sql || true
