#!/usr/bin/env zsh
# For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV)

emulate -LR zsh
setopt errexit nounset pipefail

SCRIPT_DIR_CHIRHO="${0:A:h}"
REGISTRY_SCRIPT_CHIRHO="$SCRIPT_DIR_CHIRHO/portfolio-agent-registry-chirho.sh"
TEMP_ROOT_CHIRHO=$(mktemp -d "${TMPDIR:-/tmp}/portfolio-agent-registry-tests-chirho.XXXXXX")
REGISTRY_FILE_CHIRHO="$TEMP_ROOT_CHIRHO/config-chirho/projects-chirho.tsv"
LAUNCH_LOG_CHIRHO="$TEMP_ROOT_CHIRHO/launches-chirho.tsv"
FAKE_LAUNCHER_CHIRHO="$TEMP_ROOT_CHIRHO/fake-agent-launcher-chirho.sh"
SESSION_PREFIX_CHIRHO="PORTFOLIO_TEST_${$}_CHIRHO"
DELTA_SESSION_CHIRHO="${SESSION_PREFIX_CHIRHO}_DELTA"
IMPORT_SESSION_CHIRHO="${SESSION_PREFIX_CHIRHO}_IMPORT"

cleanup_chirho() {
    tmux kill-session -t "$DELTA_SESSION_CHIRHO" 2>/dev/null || true
    tmux kill-session -t "$IMPORT_SESSION_CHIRHO" 2>/dev/null || true
    rm -rf "$TEMP_ROOT_CHIRHO"
}
trap cleanup_chirho EXIT INT TERM

fail_chirho() {
    print -u2 -- "FAIL: $*"
    exit 1
}

assert_equal_chirho() {
    local expected_chirho="$1"
    local actual_chirho="$2"
    local label_chirho="$3"
    [[ "$actual_chirho" == "$expected_chirho" ]] \
        || fail_chirho "${label_chirho}: expected '${expected_chirho}', got '${actual_chirho}'"
}

assert_contains_chirho() {
    local haystack_chirho="$1"
    local needle_chirho="$2"
    local label_chirho="$3"
    [[ "$haystack_chirho" == *"$needle_chirho"* ]] \
        || fail_chirho "${label_chirho}: missing '${needle_chirho}'"
}

assert_not_contains_chirho() {
    local haystack_chirho="$1"
    local needle_chirho="$2"
    local label_chirho="$3"
    [[ "$haystack_chirho" != *"$needle_chirho"* ]] \
        || fail_chirho "${label_chirho}: unexpectedly contained '${needle_chirho}'"
}

registry_field_chirho() {
    local project_key_chirho="$1"
    local field_number_chirho="$2"
    awk -F '\t' -v project_key_chirho="$project_key_chirho" -v field_number_chirho="$field_number_chirho" \
        '$1 == project_key_chirho { print $field_number_chirho; exit }' "$REGISTRY_FILE_CHIRHO"
}

run_registry_chirho() {
    env \
        PORTFOLIO_AGENT_REGISTRY_FILE_CHIRHO="$REGISTRY_FILE_CHIRHO" \
        PORTFOLIO_AGENT_LAUNCHER_CHIRHO="$FAKE_LAUNCHER_CHIRHO" \
        PORTFOLIO_AGENT_BROKER_URL_CHIRHO="http://127.0.0.1:9" \
        PORTFOLIO_AGENT_TEST_LOG_CHIRHO="$LAUNCH_LOG_CHIRHO" \
        "$REGISTRY_SCRIPT_CHIRHO" "$@"
}

mkdir -p "$TEMP_ROOT_CHIRHO/config-chirho"
cat > "$FAKE_LAUNCHER_CHIRHO" <<'EOF'
#!/usr/bin/env zsh
# For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV)
printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$PWD" "$1" "$CLAUDE_ENABLE_CHIRHO" "$GPT_ENABLE_CHIRHO" \
    "$AGY_ENABLE_CHIRHO" "$AGY_REGISTER_CHIRHO" "$CLAUDE2_ENABLE_CHIRHO" \
    "$OPENCODE_ENABLE_CHIRHO" >> "$PORTFOLIO_AGENT_TEST_LOG_CHIRHO"
EOF
chmod +x "$FAKE_LAUNCHER_CHIRHO" "$REGISTRY_SCRIPT_CHIRHO"

ALPHA_PATH_CHIRHO="$TEMP_ROOT_CHIRHO/alpha-chirho"
BETA_PATH_CHIRHO="$TEMP_ROOT_CHIRHO/beta-chirho"
GAMMA_PATH_CHIRHO="$TEMP_ROOT_CHIRHO/gamma-chirho"
DELTA_PATH_CHIRHO="$TEMP_ROOT_CHIRHO/delta-chirho"
EPSILON_PATH_CHIRHO="$TEMP_ROOT_CHIRHO/epsilon-chirho"
ZETA_PATH_CHIRHO="$TEMP_ROOT_CHIRHO/zeta-chirho"
mkdir -p "$ALPHA_PATH_CHIRHO" "$BETA_PATH_CHIRHO" "$GAMMA_PATH_CHIRHO" \
    "$DELTA_PATH_CHIRHO" "$EPSILON_PATH_CHIRHO" "$ZETA_PATH_CHIRHO"
BETA_CANONICAL_CHIRHO=$(cd "$BETA_PATH_CHIRHO" && pwd -P)
touch "$ALPHA_PATH_CHIRHO/AGENTS.md" "$BETA_PATH_CHIRHO/AGENTS.md" \
    "$GAMMA_PATH_CHIRHO/AGENTS.md" "$DELTA_PATH_CHIRHO/AGENTS.md"
printf 'TMUX_SESSION_NAME_CHIRHO=%s_ALPHA\n' "$SESSION_PREFIX_CHIRHO" > "$ALPHA_PATH_CHIRHO/.env"
printf 'TMUX_SESSION_NAME_CHIRHO=%s_BETA\n' "$SESSION_PREFIX_CHIRHO" > "$BETA_PATH_CHIRHO/.env"
printf 'TMUX_SESSION_NAME_CHIRHO=%s_GAMMA\n' "$SESSION_PREFIX_CHIRHO" > "$GAMMA_PATH_CHIRHO/.env"
printf 'TMUX_SESSION_NAME_CHIRHO=%s\n' "$DELTA_SESSION_CHIRHO" > "$DELTA_PATH_CHIRHO/.env"

run_registry_chirho register-chirho "$ALPHA_PATH_CHIRHO" >/dev/null
assert_equal_chirho "warm" "$(registry_field_chirho alpha-chirho 4)" "new projects default warm"
assert_equal_chirho "gpt" "$(registry_field_chirho alpha-chirho 5)" "first auto assignment"
assert_contains_chirho "$(tail -n 1 "$LAUNCH_LOG_CHIRHO")" $'\t0\t0\t0\t0\t0\t0' \
    "warm sync starts no model"

run_registry_chirho register-chirho "$BETA_PATH_CHIRHO" active >/dev/null
run_registry_chirho register-chirho "$GAMMA_PATH_CHIRHO" active >/dev/null
assert_equal_chirho "claude" "$(registry_field_chirho beta-chirho 5)" "active pool balances second project"
assert_equal_chirho "claude2" "$(registry_field_chirho gamma-chirho 5)" "active pool balances third project"
assert_equal_chirho "1" "$(awk -F '\t' '$1 ~ /beta-chirho$/ { print $3 }' "$LAUNCH_LOG_CHIRHO")" \
    "beta enables only primary Claude"
assert_equal_chirho "1" "$(awk -F '\t' '$1 ~ /gamma-chirho$/ { print $7 }' "$LAUNCH_LOG_CHIRHO")" \
    "gamma enables only Claude2"

run_registry_chirho activate-chirho alpha-chirho auto >/dev/null
assert_equal_chirho "active" "$(registry_field_chirho alpha-chirho 4)" "activation persists mode"
assert_equal_chirho "gpt" "$(registry_field_chirho alpha-chirho 5)" "activation chooses least active model"
assert_equal_chirho "1" "$(tail -n 1 "$LAUNCH_LOG_CHIRHO" | awk -F '\t' '{ print $4 }')" \
    "activation enables only GPT"

run_registry_chirho register-chirho "$ALPHA_PATH_CHIRHO" --registry-only-chirho >/dev/null
assert_equal_chirho "active" "$(registry_field_chirho alpha-chirho 4)" \
    "re-register without a profile preserves state"

STALE_PATH_CHIRHO="$TEMP_ROOT_CHIRHO/stale-lock-chirho"
mkdir -p "$STALE_PATH_CHIRHO" "${REGISTRY_FILE_CHIRHO}.lock-chirho"
printf '99999999\n' > "${REGISTRY_FILE_CHIRHO}.lock-chirho/owner-pid-chirho"
run_registry_chirho register-chirho "$STALE_PATH_CHIRHO" --registry-only-chirho >/dev/null
assert_equal_chirho "warm" "$(registry_field_chirho stale-lock-chirho 4)" \
    "a dead lock owner does not strand the registry"

typeset -a CONCURRENT_PIDS_CHIRHO
for CONCURRENT_INDEX_CHIRHO in {1..6}; do
    CONCURRENT_PATH_CHIRHO="$TEMP_ROOT_CHIRHO/concurrent-${CONCURRENT_INDEX_CHIRHO}-chirho"
    mkdir -p "$CONCURRENT_PATH_CHIRHO"
    touch "$CONCURRENT_PATH_CHIRHO/AGENTS.md"
    run_registry_chirho register-chirho "$CONCURRENT_PATH_CHIRHO" \
        --registry-only-chirho >/dev/null &
    CONCURRENT_PIDS_CHIRHO+=("$!")
done
for CONCURRENT_PID_CHIRHO in "${CONCURRENT_PIDS_CHIRHO[@]}"; do
    wait "$CONCURRENT_PID_CHIRHO" || fail_chirho "concurrent registry writer failed"
done
CONCURRENT_ROWS_CHIRHO=$(awk -F '\t' '$1 ~ /^concurrent-[0-9]+-chirho$/ { count_chirho += 1 } END { print count_chirho + 0 }' \
    "$REGISTRY_FILE_CHIRHO")
if [[ "$CONCURRENT_ROWS_CHIRHO" != "6" ]]; then
    print -u2 -- "concurrent registry evidence:"
    sed -n '/^concurrent-/p' "$REGISTRY_FILE_CHIRHO" >&2
fi
assert_equal_chirho "6" "$CONCURRENT_ROWS_CHIRHO" "registry lock prevents lost concurrent rows"

mkdir -p "$TEMP_ROOT_CHIRHO/other-chirho/alpha-chirho"
if run_registry_chirho register-chirho "$TEMP_ROOT_CHIRHO/other-chirho/alpha-chirho" \
    --registry-only-chirho >/dev/null 2>&1; then
    fail_chirho "a derived-key collision overwrote an existing project"
fi

tmux new-session -d -s "$DELTA_SESSION_CHIRHO" -c "$DELTA_PATH_CHIRHO" -n "Hallelujah"
DELTA_SHELL_ID_CHIRHO=$(tmux display-message -p -t "$DELTA_SESSION_CHIRHO" '#{window_id}')
tmux move-window -s "$DELTA_SHELL_ID_CHIRHO" -t "${DELTA_SESSION_CHIRHO}:2"
tmux new-window -d -t "${DELTA_SESSION_CHIRHO}:1" -c "$DELTA_PATH_CHIRHO" -n "Hallelujah-claude" "sleep 300"
tmux new-window -d -t "${DELTA_SESSION_CHIRHO}:3" -c "$DELTA_PATH_CHIRHO" -n "Hallelujah-gpt" "sleep 300"
tmux new-window -d -t "${DELTA_SESSION_CHIRHO}:8" -c "$DELTA_PATH_CHIRHO" -n "Hallelujah-gpt" "sleep 300"
tmux new-window -d -t "${DELTA_SESSION_CHIRHO}:4" -c "$DELTA_PATH_CHIRHO" -n "Hallelujah-agy" "sleep 300"
tmux new-window -d -t "${DELTA_SESSION_CHIRHO}:5" -c "$DELTA_PATH_CHIRHO" -n "Hallelujah-claude2" "sleep 300"
tmux new-window -d -t "${DELTA_SESSION_CHIRHO}:6" -c "$DELTA_PATH_CHIRHO" -n "Hallelujah-opencode" "sleep 300"
run_registry_chirho register-chirho "$DELTA_PATH_CHIRHO" gpt --registry-only-chirho >/dev/null
DELTA_DUPLICATE_STATUS_CHIRHO=$(run_registry_chirho status-chirho delta-chirho 2>/dev/null)
assert_contains_chirho "$DELTA_DUPLICATE_STATUS_CHIRHO" "duplicate:2" \
    "status refuses to collapse duplicate active windows into one"
assert_contains_chirho "$DELTA_DUPLICATE_STATUS_CHIRHO" "gpt-duplicates:1" \
    "status names the duplicate drift"
if run_registry_chirho route-chirho delta-chirho >/dev/null 2>&1; then
    fail_chirho "remote routing accepted a duplicate active identity"
fi
run_registry_chirho pause-chirho delta-chirho >/dev/null
DELTA_WINDOWS_CHIRHO=$(tmux list-windows -t "$DELTA_SESSION_CHIRHO" -F '#{window_name}')
assert_contains_chirho "$DELTA_WINDOWS_CHIRHO" "Hallelujah" "pause keeps the project shell"
assert_not_contains_chirho "$DELTA_WINDOWS_CHIRHO" "Hallelujah-gpt" "pause closes GPT"
assert_not_contains_chirho "$DELTA_WINDOWS_CHIRHO" "Hallelujah-claude" "pause closes Claude"
assert_not_contains_chirho "$DELTA_WINDOWS_CHIRHO" "Hallelujah-agy" "pause closes Agy"
assert_not_contains_chirho "$DELTA_WINDOWS_CHIRHO" "Hallelujah-claude2" "pause closes Claude2"
assert_not_contains_chirho "$DELTA_WINDOWS_CHIRHO" "Hallelujah-opencode" "pause closes opencode"
assert_equal_chirho "warm" "$(registry_field_chirho delta-chirho 4)" "pause persists warm mode"

tmux new-window -d -t "${DELTA_SESSION_CHIRHO}:3" -c "$BETA_PATH_CHIRHO" -n "Hallelujah-gpt" "sleep 300"
run_registry_chirho register-chirho "$DELTA_PATH_CHIRHO" gpt --registry-only-chirho >/dev/null
if run_registry_chirho route-chirho delta-chirho >/dev/null 2>&1; then
    fail_chirho "remote routing accepted an agent rooted in another project"
fi
DELTA_WRONG_CWD_STATUS_CHIRHO=$(run_registry_chirho status-chirho delta-chirho 2>/dev/null)
assert_contains_chirho "$DELTA_WRONG_CWD_STATUS_CHIRHO" "drift:${BETA_CANONICAL_CHIRHO}" \
    "status reports cross-project cwd drift"
run_registry_chirho pause-chirho delta-chirho >/dev/null

tmux new-window -d -t "${DELTA_SESSION_CHIRHO}:3" -c "$DELTA_PATH_CHIRHO" -n "Hallelujah-gpt" "sleep 300"
run_registry_chirho register-chirho "$DELTA_PATH_CHIRHO" gpt --registry-only-chirho >/dev/null
DELTA_ROUTE_CHIRHO=$(run_registry_chirho route-chirho delta-chirho)
assert_contains_chirho "$DELTA_ROUTE_CHIRHO" "target_chirho=${DELTA_SESSION_CHIRHO}/gpt_chirho" \
    "one in-project agent exposes the canonical route"
run_registry_chirho pause-chirho delta-chirho >/dev/null

git -C "$EPSILON_PATH_CHIRHO" init -q
git -C "$ZETA_PATH_CHIRHO" init -q
tmux new-session -d -s "$IMPORT_SESSION_CHIRHO" -c "$EPSILON_PATH_CHIRHO" -n "Epsilon-chirho"
tmux new-window -d -t "$IMPORT_SESSION_CHIRHO" -c "$ZETA_PATH_CHIRHO" -n "Zeta-chirho"
EPSILON_CANONICAL_CHIRHO=$(cd "$EPSILON_PATH_CHIRHO" && pwd -P)
ZETA_CANONICAL_CHIRHO=$(cd "$ZETA_PATH_CHIRHO" && pwd -P)
for IMPORT_READY_ATTEMPT_CHIRHO in {1..40}; do
    IMPORT_PANE_CENSUS_CHIRHO=$(tmux list-panes -s -t "$IMPORT_SESSION_CHIRHO" -F '#{pane_current_path}')
    if [[ "$IMPORT_PANE_CENSUS_CHIRHO" == *"$EPSILON_CANONICAL_CHIRHO"* \
        && "$IMPORT_PANE_CENSUS_CHIRHO" == *"$ZETA_CANONICAL_CHIRHO"* ]]; then
        break
    fi
    sleep 0.05
done
assert_contains_chirho "$IMPORT_PANE_CENSUS_CHIRHO" "$EPSILON_CANONICAL_CHIRHO" \
    "tmux import fixture exposes epsilon cwd"
assert_contains_chirho "$IMPORT_PANE_CENSUS_CHIRHO" "$ZETA_CANONICAL_CHIRHO" \
    "tmux import fixture exposes zeta cwd"
IMPORT_OUTPUT_CHIRHO=$(run_registry_chirho import-tmux-chirho "$IMPORT_SESSION_CHIRHO" warm --registry-only-chirho)
if [[ -z "$(registry_field_chirho epsilon-chirho 4)" || -z "$(registry_field_chirho zeta-chirho 4)" ]]; then
    print -u2 -- "import pane census:"
    print -u2 -r -- "$IMPORT_PANE_CENSUS_CHIRHO"
    print -u2 -- "import output:"
    print -u2 -r -- "$IMPORT_OUTPUT_CHIRHO"
    print -u2 -- "import registry evidence:"
    sed -n '/^epsilon-chirho\|^zeta-chirho/p' "$REGISTRY_FILE_CHIRHO" >&2
fi
assert_equal_chirho "warm" "$(registry_field_chirho epsilon-chirho 4)" "tmux import finds first git root"
assert_equal_chirho "warm" "$(registry_field_chirho zeta-chirho 4)" "tmux import finds second git root"

STATUS_OUTPUT_CHIRHO=$(run_registry_chirho status-chirho delta-chirho 2>/dev/null)
assert_contains_chirho "$STATUS_OUTPUT_CHIRHO" $'delta-chirho\twarm\t' "status reports registered profile"
assert_contains_chirho "$STATUS_OUTPUT_CHIRHO" $'\tup\t-\t-\t-' "status reports warm tmux census"

run_registry_chirho register-chirho "$EPSILON_PATH_CHIRHO" agy --registry-only-chirho >/dev/null
if run_registry_chirho route-chirho epsilon-chirho >/dev/null 2>&1; then
    fail_chirho "an untrusted Agy project exposed a remote route"
fi
run_registry_chirho trust-agy-chirho epsilon-chirho >/dev/null
assert_equal_chirho "1" "$(registry_field_chirho epsilon-chirho 6)" "Agy trust is persisted per project"

print -r -- "PASS: portfolio agent registry isolated tests"
