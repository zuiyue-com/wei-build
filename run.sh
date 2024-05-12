#!/bin/bash

HTTPS_PROXY=http://127.0.0.1:7897/
HTTP_PROXY=http://127.0.0.1:7897/
ALL_PROXY=socks://127.0.0.1:7897/

cargo run build
