#!/usr/bin/env bash
# Stages the shipped skills as wheel data, installed to <prefix>/share/htmlpress/skills.
set -euo pipefail
cd "$(dirname "$0")/.."
rm -rf htmlpress.data
mkdir -p htmlpress.data/data/share/htmlpress
cp -R ../.skills htmlpress.data/data/share/htmlpress/skills
