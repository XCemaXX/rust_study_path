#! /bin/bash

echo "[workspace]" > Cargo.toml
echo 'resolver = "2"'>> Cargo.toml
echo "members = [" >> Cargo.toml

find . -maxdepth 2 -mindepth 2 -type d \
    | grep -v -e "./.git" -e "./target/" -e "./00_hello_world/hello_raw" \
    | cut -c3- \
    | sort \
    | xargs printf '    "%s",\n' >> Cargo.toml

echo "]" >> Cargo.toml