#!/usr/bin/env bash

# Runs independent silent checks concurrently, then replays failed diagnostics
# in declaration order so scheduling cannot make suite output nondeterministic.
run_parallel_checks() {
	if (( $# == 0 )); then
		return 0
	fi

	local work
	work=$(mktemp -d "${TMPDIR:-/tmp}/aedicule-parallel-checks.XXXXXX")
	local -a checks=("$@")
	local -a pids=()
	local index status failures=0

	for index in "${!checks[@]}"; do
		"${checks[index]}" >"$work/$index.log" 2>&1 &
		pids[index]=$!
	done

	for index in "${!checks[@]}"; do
		wait "${pids[index]}"
		status=$?
		if (( status != 0 )); then
			failures=$((failures + 1))
			if [[ -s $work/$index.log ]]; then
				sed -n '1,$p' "$work/$index.log"
			fi
		fi
	done

	rm -rf -- "$work"
	if (( failures > 255 )); then
		return 255
	fi
	return "$failures"
}
