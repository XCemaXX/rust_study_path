#! /bin/bash

echo "[workspace]" > Cargo.toml
echo 'resolver = "2"'>> Cargo.toml
echo "members = [" >> Cargo.toml

find . -maxdepth 2 -mindepth 2 -type d \
    | grep -v -e "./.git" -e "./target/" -e "./*/rustc*" \
              -e "13_executable_packer/samples" \
              -e "13_executable_packer/minipak" \
    | cut -c3- \
    | sort \
    | xargs printf '    "%s",\n' >> Cargo.toml

echo "]" >> Cargo.toml

# This script was written due to limited familiarity with Cargo workspaces.
# It generates a workspace Cargo.toml by scanning the directory tree.
# In fact, Cargo can manage this automatically:
# if you run `cargo new/init --lib some/path` inside an existing workspace,
# the new crate will be added to the workspace members automatically.