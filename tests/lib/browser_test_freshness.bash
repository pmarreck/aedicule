#!/usr/bin/env bash

BROWSER_TEST_MAX_AGE_SECONDS=$((2 * 24 * 60 * 60))

# Classifies browser-integration freshness from injected time and commit state.
# Empty or malformed success metadata is due; unchanged HEAD and clock skew are not.
browser_test_is_due() {
	local now_epoch=$1 last_epoch=$2 last_commit=$3 current_commit=$4
	local head_advanced=$5 age
	[[ $now_epoch =~ ^[0-9]+$ && $current_commit != '' ]] || return 1
	[[ $last_epoch =~ ^[0-9]+$ && $last_commit != '' ]] || return 0
	[[ $head_advanced == 1 ]] || return 1
	(( now_epoch >= last_epoch )) || return 1
	age=$((now_epoch - last_epoch))
	(( age >= BROWSER_TEST_MAX_AGE_SECONDS ))
}

# Pure reminder renderer: it emits one stable line only when the injected state is due.
browser_test_reminder() {
	if browser_test_is_due "$@"; then
		printf 'Browser integration is due after new commits; run ./test_browser.\n'
	fi
}

# Reads one worktree-local successful-run marker without mutating repository state.
read_browser_test_success() {
	local marker=$1 line
	BROWSER_TEST_LAST_EPOCH=
	BROWSER_TEST_LAST_COMMIT=
	[[ -r $marker ]] || return 0
	while IFS= read -r line; do
		case "$line" in
			epoch=*) BROWSER_TEST_LAST_EPOCH=${line#epoch=} ;;
			commit=*) BROWSER_TEST_LAST_COMMIT=${line#commit=} ;;
		esac
	done <"$marker"
}

# Atomically records success only after the complete live-browser gate has passed.
record_browser_test_success() {
	local marker=$1 epoch=$2 commit=$3 temporary
	mkdir -p -- "${marker%/*}"
	temporary=$(mktemp "$marker.XXXXXX")
	printf 'epoch=%s\ncommit=%s\n' "$epoch" "$commit" >"$temporary"
	mv -f -- "$temporary" "$marker"
}

# Keeps automated output clean; interactive successful suites get the pure reminder.
remind_browser_test_if_due() {
	local repository=$1 marker now_epoch current_commit head_advanced reminder
	[[ -t 2 ]] || return 0
	marker=$(git -C "$repository" rev-parse --git-path aedicule-test-browser-success \
		2>/dev/null) || return 0
	case "$marker" in
		/*) ;;
		*) marker="$repository/$marker" ;;
	esac
	read_browser_test_success "$marker"
	now_epoch=$(date +%s) || return 0
	current_commit=$(git -C "$repository" rev-parse HEAD 2>/dev/null) || return 0
	head_advanced=1
	if [[ -n $BROWSER_TEST_LAST_COMMIT ]]; then
		head_advanced=0
		if [[ $BROWSER_TEST_LAST_COMMIT != "$current_commit" ]]; then
			if git -C "$repository" merge-base --is-ancestor \
				"$BROWSER_TEST_LAST_COMMIT" "$current_commit" 2>/dev/null; then
				head_advanced=1
			elif ! git -C "$repository" cat-file -e \
				"$BROWSER_TEST_LAST_COMMIT^{commit}" 2>/dev/null; then
				head_advanced=1
			fi
		fi
	fi
	reminder=$(browser_test_reminder "$now_epoch" "$BROWSER_TEST_LAST_EPOCH" \
		"$BROWSER_TEST_LAST_COMMIT" "$current_commit" "$head_advanced")
	[[ -z $reminder ]] || printf '%s\n' "$reminder" >&2
}
