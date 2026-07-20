#!/usr/bin/env bash

# Captures stdout, stderr, and the exit status without temporary files so CLI
# tests remain isolated on hosts that do not contain Peter's personal dotfiles.
capture() {
	if (( $# == 0 )); then
		printf 'capture: no command given\n' >&2
		return 2
	fi

	local variable
	for variable in out err rc; do
		if ! declare -p "$variable" >/dev/null 2>&1; then
			printf 'capture: required variable "%s" is not declared\n' "$variable" >&2
			return 2
		fi
	done

	local captured_out captured_err captured_rc
	. <(
		{
			captured_err=$(
				{
					captured_out=$(
						set -o pipefail
						"$@" 2> >(cat >&3) | cat
					)
					captured_rc=$?
					declare -p captured_out captured_rc >&4
				} 3>&1
			)
			declare -p captured_err >&4
		} 4>&1
	)

	out=$captured_out
	err=$captured_err
	rc=$captured_rc
}
