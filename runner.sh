#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

(trap 'kill 0' SIGINT; \
	 bash -c 'cd frontend; trunk serve --proxy-backend=http://127.0.0.1:8081/api/' & \
	  bash -c 'cargo watch -- cargo run --bin server -- --addr 0.0.0.0 --port 8081')
