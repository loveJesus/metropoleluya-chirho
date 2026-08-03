#!/usr/bin/env zsh
# For God so loved the world, that he gave his only begotten Son, that whosoever believeth in him should not perish, but have everlasting life. - John 3:16 (KJV)

emulate -LR zsh
setopt errexit nounset pipefail

SCRIPT_NAME_CHIRHO="${0:t}"
REGISTRY_FILE_CHIRHO="${PORTFOLIO_AGENT_REGISTRY_FILE_CHIRHO:-$HOME/.config/portfolio-agent-registry-chirho/projects-chirho.tsv}"
REGISTRY_DIR_CHIRHO="${REGISTRY_FILE_CHIRHO:h}"
REGISTRY_LOCK_DIR_CHIRHO="${REGISTRY_FILE_CHIRHO}.lock-chirho"
AGENT_LAUNCHER_CHIRHO="${PORTFOLIO_AGENT_LAUNCHER_CHIRHO:-$HOME/bin-chirho/agent-tmux-chirho.sh}"
TMUX_BIN_CHIRHO="${PORTFOLIO_AGENT_TMUX_BIN_CHIRHO:-tmux}"
BROKER_URL_CHIRHO="${PORTFOLIO_AGENT_BROKER_URL_CHIRHO:-http://127.0.0.1:37371}"
METROPOLELUYA_DIR_CHIRHO="${PORTFOLIO_AGENT_METROPOLELUYA_DIR_CHIRHO:-${0:A:h:h}}"
OPERATOR_SESSION_CHIRHO="${PORTFOLIO_AGENT_OPERATOR_SESSION_CHIRHO:-PORTFOLIO_CHIRHO}"
OPERATOR_AGENT_CHIRHO="${PORTFOLIO_AGENT_OPERATOR_AGENT_CHIRHO:-lj_chirho}"
AUTO_POOL_CHIRHO="${PORTFOLIO_AGENT_AUTO_POOL_CHIRHO:-gpt,claude,claude2}"

typeset -g PROJECT_KEY_CHIRHO=""
typeset -g PROJECT_PATH_CHIRHO=""
typeset -g PROJECT_SESSION_CHIRHO=""
typeset -g PROJECT_MODE_CHIRHO=""
typeset -g PROJECT_AGENT_CHIRHO=""
typeset -g PROJECT_AGY_TRUSTED_CHIRHO=""
typeset -g FOUND_KEY_CHIRHO=""
typeset -g SELECTED_MODE_CHIRHO=""
typeset -g SELECTED_AGENT_CHIRHO=""
typeset -g REGISTRY_LOCK_HELD_CHIRHO="0"

die_chirho() {
    print -u2 -- "error: $*"
    exit 1
}

note_chirho() {
    print -u2 -- "$*"
}

usage_chirho() {
    cat <<'EOF'
Portfolio agent registry

Register projects as warm tmux workspaces, then explicitly give each project at
most one standard model process. The existing agent-tmux-chirho.sh remains the
launcher and Metropoleluya registrar.

Usage:
  portfolio-agent-registry-chirho.sh register-chirho PATH [PROFILE] [OPTIONS]
  portfolio-agent-registry-chirho.sh import-tmux-chirho SESSION [PROFILE] [--registry-only-chirho]
  portfolio-agent-registry-chirho.sh list-chirho
  portfolio-agent-registry-chirho.sh status-chirho [KEY|all-chirho]
  portfolio-agent-registry-chirho.sh sync-chirho [KEY|all-chirho]
  portfolio-agent-registry-chirho.sh activate-chirho KEY|all-chirho [AGENT|auto]
  portfolio-agent-registry-chirho.sh rotate-chirho KEY
  portfolio-agent-registry-chirho.sh pause-chirho KEY|all-chirho
  portfolio-agent-registry-chirho.sh enforce-chirho KEY|all-chirho
  portfolio-agent-registry-chirho.sh trust-agy-chirho KEY
  portfolio-agent-registry-chirho.sh route-chirho KEY
  portfolio-agent-registry-chirho.sh send-chirho KEY BODY_FILE [TOPIC]
  portfolio-agent-registry-chirho.sh attach-chirho KEY
  portfolio-agent-registry-chirho.sh unregister-chirho KEY

Profiles:
  warm       Shell + Metropoleluya TUI, no model process. This is the default.
  active     One automatically selected model.
  auto       Alias for active.
  gpt        One Codex/GPT process.
  claude     One Claude process on the primary Claude account.
  claude2    One Claude process on the second Claude account.
  agy        One Agy process. Broker registration waits for trust-agy-chirho.

Register options:
  --key-chirho KEY          Override the derived registry key.
  --session-chirho SESSION  Override .env TMUX_SESSION_NAME_CHIRHO discovery.
  --registry-only-chirho    Write the row without creating/repairing tmux.

Safety contract:
  register/import default to warm. sync is non-destructive and never removes an
  existing model window. activate, rotate, pause, and enforce are explicit and
  may close only the standard agent windows managed by agent-tmux-chirho.sh.
  They never close the project shell, TUI, or a specially named worker window.

Examples:
  portfolio-agent-registry-chirho.sh register-chirho ~/dev-chirho/example-chirho
  portfolio-agent-registry-chirho.sh register-chirho ~/dev-chirho/example-chirho active
  portfolio-agent-registry-chirho.sh import-tmux-chirho 0 warm
  portfolio-agent-registry-chirho.sh activate-chirho example-chirho auto
  portfolio-agent-registry-chirho.sh rotate-chirho example-chirho
  portfolio-agent-registry-chirho.sh pause-chirho example-chirho
  portfolio-agent-registry-chirho.sh send-chirho example-chirho /tmp/direction-chirho.md
EOF
}

require_tool_chirho() {
    local tool_chirho="$1"
    command -v "$tool_chirho" >/dev/null 2>&1 || die_chirho "required tool not found: ${tool_chirho}"
}

ensure_registry_chirho() {
    mkdir -p "$REGISTRY_DIR_CHIRHO"
    chmod 700 "$REGISTRY_DIR_CHIRHO"
    if [[ ! -f "$REGISTRY_FILE_CHIRHO" ]]; then
        umask 077
        printf '# key_chirho\tpath_chirho\tsession_chirho\tmode_chirho\tagent_chirho\tagy_trusted_chirho\n' \
            > "$REGISTRY_FILE_CHIRHO"
    fi
    chmod 600 "$REGISTRY_FILE_CHIRHO"
}

validate_field_chirho() {
    local field_name_chirho="$1"
    local field_value_chirho="$2"
    [[ -n "$field_value_chirho" ]] || die_chirho "${field_name_chirho} cannot be empty"
    if [[ "$field_value_chirho" == *$'\t'* || "$field_value_chirho" == *$'\n'* || "$field_value_chirho" == *$'\r'* ]]; then
        die_chirho "${field_name_chirho} cannot contain tabs or newlines"
    fi
}

validate_session_chirho() {
    local session_name_chirho="$1"
    validate_field_chirho "tmux session" "$session_name_chirho"
    if [[ ! "$session_name_chirho" =~ '^[A-Za-z0-9_-]+$' ]]; then
        die_chirho "tmux session names may contain only letters, digits, '_' and '-': ${session_name_chirho}"
    fi
}

release_registry_lock_chirho() {
    local owner_pid_chirho=""
    [[ "$REGISTRY_LOCK_HELD_CHIRHO" == "1" ]] || return 0
    if [[ -f "$REGISTRY_LOCK_DIR_CHIRHO/owner-pid-chirho" ]]; then
        owner_pid_chirho=$(<"$REGISTRY_LOCK_DIR_CHIRHO/owner-pid-chirho")
    fi
    if [[ "$owner_pid_chirho" == "$$" ]]; then
        rm -f "$REGISTRY_LOCK_DIR_CHIRHO/owner-pid-chirho"
        rmdir "$REGISTRY_LOCK_DIR_CHIRHO" 2>/dev/null || true
    fi
    REGISTRY_LOCK_HELD_CHIRHO="0"
}

interrupt_with_lock_release_chirho() {
    release_registry_lock_chirho
    exit 130
}

terminate_with_lock_release_chirho() {
    release_registry_lock_chirho
    exit 143
}

acquire_registry_lock_chirho() {
    local attempt_chirho
    local owner_pid_chirho=""

    mkdir -p "$REGISTRY_DIR_CHIRHO"
    for attempt_chirho in {1..200}; do
        if mkdir "$REGISTRY_LOCK_DIR_CHIRHO" 2>/dev/null; then
            printf '%s\n' "$$" > "$REGISTRY_LOCK_DIR_CHIRHO/owner-pid-chirho"
            REGISTRY_LOCK_HELD_CHIRHO="1"
            return 0
        fi
        if [[ -f "$REGISTRY_LOCK_DIR_CHIRHO/owner-pid-chirho" ]]; then
            owner_pid_chirho=$(<"$REGISTRY_LOCK_DIR_CHIRHO/owner-pid-chirho")
            if [[ "$owner_pid_chirho" == <-> ]] && ! kill -0 "$owner_pid_chirho" 2>/dev/null; then
                rm -f "$REGISTRY_LOCK_DIR_CHIRHO/owner-pid-chirho"
                rmdir "$REGISTRY_LOCK_DIR_CHIRHO" 2>/dev/null || true
                continue
            fi
        fi
        sleep 0.05
    done
    die_chirho "timed out waiting for registry lock: ${REGISTRY_LOCK_DIR_CHIRHO}"
}

trap release_registry_lock_chirho EXIT
trap interrupt_with_lock_release_chirho INT
trap terminate_with_lock_release_chirho TERM HUP

command_mutates_state_chirho() {
    case "$1" in
        register-chirho|import-tmux-chirho|sync-chirho|activate-chirho|rotate-chirho|pause-chirho|enforce-chirho|trust-agy-chirho|unregister-chirho)
            return 0
            ;;
        *) return 1 ;;
    esac
}

canonical_path_chirho() {
    local input_path_chirho="$1"
    [[ -d "$input_path_chirho" ]] || die_chirho "project directory does not exist: ${input_path_chirho}"
    (
        cd "$input_path_chirho"
        pwd -P
    )
}

path_belongs_to_project_chirho() {
    local candidate_path_chirho="$1"
    local project_path_chirho="$2"
    [[ "$candidate_path_chirho" == "$project_path_chirho" \
        || "$candidate_path_chirho" == "$project_path_chirho"/* ]]
}

normalize_key_chirho() {
    local raw_key_chirho="$1"
    local normalized_key_chirho
    normalized_key_chirho=$(printf '%s' "$raw_key_chirho" \
        | tr '[:upper:]' '[:lower:]' \
        | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//; s/-+/-/g')
    [[ -n "$normalized_key_chirho" ]] || die_chirho "could not derive a project key from: ${raw_key_chirho}"
    if [[ "$normalized_key_chirho" != *-chirho ]]; then
        normalized_key_chirho="${normalized_key_chirho}-chirho"
    fi
    print -r -- "$normalized_key_chirho"
}

default_session_chirho() {
    local project_path_chirho="$1"
    local base_name_chirho="${project_path_chirho:t}"
    local session_name_chirho
    session_name_chirho=$(printf '%s' "$base_name_chirho" \
        | tr '[:lower:]' '[:upper:]' \
        | sed -E 's/[^A-Z0-9]+/_/g; s/^_+//; s/_+$//; s/_+/_/g')
    [[ -n "$session_name_chirho" ]] || die_chirho "could not derive a tmux session from: ${project_path_chirho}"
    if [[ "$session_name_chirho" != *_CHIRHO ]]; then
        session_name_chirho="${session_name_chirho}_CHIRHO"
    fi
    print -r -- "$session_name_chirho"
}

session_from_env_chirho() {
    local project_path_chirho="$1"
    local env_path_chirho
    local values_chirho
    local value_chirho

    for env_path_chirho in "$project_path_chirho/.env" "$project_path_chirho/.env-chirho"; do
        [[ -f "$env_path_chirho" ]] || continue
        values_chirho=$(sed -nE \
            's/^[[:space:]]*(export[[:space:]]+)?TMUX_SESSION_NAME_CHIRHO[[:space:]]*=[[:space:]]*(.*)[[:space:]]*$/\2/p' \
            "$env_path_chirho")
        [[ -n "$values_chirho" ]] || continue
        if [[ "$values_chirho" == *$'\n'* ]]; then
            die_chirho "multiple TMUX_SESSION_NAME_CHIRHO values in ${env_path_chirho}"
        fi
        value_chirho="$values_chirho"
        if [[ "$value_chirho" == \"*\" && "$value_chirho" == *\" ]]; then
            value_chirho="${value_chirho:1:-1}"
        elif [[ "$value_chirho" == \'*\' && "$value_chirho" == *\' ]]; then
            value_chirho="${value_chirho:1:-1}"
        fi
        validate_field_chirho "TMUX_SESSION_NAME_CHIRHO" "$value_chirho"
        print -r -- "$value_chirho"
        return 0
    done
    return 1
}

room_for_session_chirho() {
    local session_name_chirho="$1"
    local lower_name_chirho
    case "$session_name_chirho" in
        CAIRN_CHIRHO) print -r -- "cairn-chirho" ;;
        CBNETCHIRHO) print -r -- "crossroads-chirho" ;;
        HOTTP_CHIRHO) print -r -- "hottp-chirho" ;;
        LAMB) print -r -- "lamb-chirho" ;;
        *)
            lower_name_chirho="${(L)session_name_chirho}"
            lower_name_chirho="${lower_name_chirho%_chirho}"
            lower_name_chirho="${lower_name_chirho//_/-}"
            print -r -- "${lower_name_chirho}-chirho"
            ;;
    esac
}

valid_agent_chirho() {
    case "$1" in
        gpt|claude|claude2|agy) return 0 ;;
        *) return 1 ;;
    esac
}

window_name_for_agent_chirho() {
    case "$1" in
        claude) print -r -- "Hallelujah-claude" ;;
        gpt) print -r -- "Hallelujah-gpt" ;;
        agy) print -r -- "Hallelujah-agy" ;;
        claude2) print -r -- "Hallelujah-claude2" ;;
        opencode) print -r -- "Hallelujah-opencode" ;;
        *) return 1 ;;
    esac
}

identity_for_agent_chirho() {
    case "$1" in
        claude) print -r -- "claude_chirho" ;;
        gpt) print -r -- "gpt_chirho" ;;
        agy) print -r -- "agy_chirho" ;;
        claude2) print -r -- "claude2_chirho" ;;
        opencode) print -r -- "opencode_chirho" ;;
        *) return 1 ;;
    esac
}

load_project_chirho() {
    local wanted_key_chirho="$1"
    local row_key_chirho=""
    local row_path_chirho=""
    local row_session_chirho=""
    local row_mode_chirho=""
    local row_agent_chirho=""
    local row_trusted_chirho=""

    ensure_registry_chirho
    while IFS=$'\t' read -r row_key_chirho row_path_chirho row_session_chirho row_mode_chirho row_agent_chirho row_trusted_chirho; do
        [[ -n "$row_key_chirho" && "$row_key_chirho" != \#* ]] || continue
        if [[ "$row_key_chirho" == "$wanted_key_chirho" ]]; then
            PROJECT_KEY_CHIRHO="$row_key_chirho"
            PROJECT_PATH_CHIRHO="$row_path_chirho"
            PROJECT_SESSION_CHIRHO="$row_session_chirho"
            PROJECT_MODE_CHIRHO="$row_mode_chirho"
            PROJECT_AGENT_CHIRHO="$row_agent_chirho"
            PROJECT_AGY_TRUSTED_CHIRHO="$row_trusted_chirho"
            return 0
        fi
    done < "$REGISTRY_FILE_CHIRHO"
    return 1
}

find_key_by_path_chirho() {
    local wanted_path_chirho="$1"
    local row_key_chirho=""
    local row_path_chirho=""
    local row_session_chirho=""
    local row_mode_chirho=""
    local row_agent_chirho=""
    local row_trusted_chirho=""

    FOUND_KEY_CHIRHO=""
    ensure_registry_chirho
    while IFS=$'\t' read -r row_key_chirho row_path_chirho row_session_chirho row_mode_chirho row_agent_chirho row_trusted_chirho; do
        [[ -n "$row_key_chirho" && "$row_key_chirho" != \#* ]] || continue
        if [[ "$row_path_chirho" == "$wanted_path_chirho" ]]; then
            FOUND_KEY_CHIRHO="$row_key_chirho"
            return 0
        fi
    done < "$REGISTRY_FILE_CHIRHO"
    return 1
}

validate_unique_row_chirho() {
    local wanted_key_chirho="$1"
    local wanted_path_chirho="$2"
    local wanted_session_chirho="$3"
    local row_key_chirho=""
    local row_path_chirho=""
    local row_session_chirho=""
    local row_mode_chirho=""
    local row_agent_chirho=""
    local row_trusted_chirho=""

    ensure_registry_chirho
    while IFS=$'\t' read -r row_key_chirho row_path_chirho row_session_chirho row_mode_chirho row_agent_chirho row_trusted_chirho; do
        [[ -n "$row_key_chirho" && "$row_key_chirho" != \#* ]] || continue
        [[ "$row_key_chirho" == "$wanted_key_chirho" ]] && continue
        [[ "$row_path_chirho" != "$wanted_path_chirho" ]] \
            || die_chirho "path already belongs to ${row_key_chirho}: ${wanted_path_chirho}"
        [[ "$row_session_chirho" != "$wanted_session_chirho" ]] \
            || die_chirho "tmux session already belongs to ${row_key_chirho}: ${wanted_session_chirho}"
    done < "$REGISTRY_FILE_CHIRHO"
}

upsert_project_chirho() {
    local wanted_key_chirho="$1"
    local wanted_path_chirho="$2"
    local wanted_session_chirho="$3"
    local wanted_mode_chirho="$4"
    local wanted_agent_chirho="$5"
    local wanted_trusted_chirho="$6"
    local temp_path_chirho
    local replaced_chirho="0"
    local row_key_chirho=""
    local row_path_chirho=""
    local row_session_chirho=""
    local row_mode_chirho=""
    local row_agent_chirho=""
    local row_trusted_chirho=""

    ensure_registry_chirho
    temp_path_chirho=$(mktemp "${REGISTRY_FILE_CHIRHO}.tmp.XXXXXX")
    while IFS=$'\t' read -r row_key_chirho row_path_chirho row_session_chirho row_mode_chirho row_agent_chirho row_trusted_chirho; do
        if [[ "$row_key_chirho" == "$wanted_key_chirho" ]]; then
            printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
                "$wanted_key_chirho" "$wanted_path_chirho" "$wanted_session_chirho" \
                "$wanted_mode_chirho" "$wanted_agent_chirho" "$wanted_trusted_chirho" \
                >> "$temp_path_chirho"
            replaced_chirho="1"
        elif [[ -n "$row_key_chirho" ]]; then
            printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
                "$row_key_chirho" "$row_path_chirho" "$row_session_chirho" \
                "$row_mode_chirho" "$row_agent_chirho" "$row_trusted_chirho" \
                >> "$temp_path_chirho"
        fi
    done < "$REGISTRY_FILE_CHIRHO"
    if [[ "$replaced_chirho" == "0" ]]; then
        printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
            "$wanted_key_chirho" "$wanted_path_chirho" "$wanted_session_chirho" \
            "$wanted_mode_chirho" "$wanted_agent_chirho" "$wanted_trusted_chirho" \
            >> "$temp_path_chirho"
    fi
    chmod 600 "$temp_path_chirho"
    mv "$temp_path_chirho" "$REGISTRY_FILE_CHIRHO"
}

delete_project_chirho() {
    local wanted_key_chirho="$1"
    local temp_path_chirho
    local row_key_chirho=""
    local row_path_chirho=""
    local row_session_chirho=""
    local row_mode_chirho=""
    local row_agent_chirho=""
    local row_trusted_chirho=""

    ensure_registry_chirho
    temp_path_chirho=$(mktemp "${REGISTRY_FILE_CHIRHO}.tmp.XXXXXX")
    while IFS=$'\t' read -r row_key_chirho row_path_chirho row_session_chirho row_mode_chirho row_agent_chirho row_trusted_chirho; do
        [[ "$row_key_chirho" == "$wanted_key_chirho" ]] && continue
        [[ -n "$row_key_chirho" ]] || continue
        printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
            "$row_key_chirho" "$row_path_chirho" "$row_session_chirho" \
            "$row_mode_chirho" "$row_agent_chirho" "$row_trusted_chirho" \
            >> "$temp_path_chirho"
    done < "$REGISTRY_FILE_CHIRHO"
    chmod 600 "$temp_path_chirho"
    mv "$temp_path_chirho" "$REGISTRY_FILE_CHIRHO"
}

registry_keys_chirho() {
    local row_key_chirho=""
    local row_path_chirho=""
    local row_session_chirho=""
    local row_mode_chirho=""
    local row_agent_chirho=""
    local row_trusted_chirho=""

    ensure_registry_chirho
    while IFS=$'\t' read -r row_key_chirho row_path_chirho row_session_chirho row_mode_chirho row_agent_chirho row_trusted_chirho; do
        [[ -n "$row_key_chirho" && "$row_key_chirho" != \#* ]] || continue
        print -r -- "$row_key_chirho"
    done < "$REGISTRY_FILE_CHIRHO"
}

auto_pool_agents_chirho() {
    local -a pool_agents_chirho
    local pool_agent_chirho
    pool_agents_chirho=("${(@s:,:)AUTO_POOL_CHIRHO}")
    for pool_agent_chirho in "${pool_agents_chirho[@]}"; do
        pool_agent_chirho="${pool_agent_chirho//[[:space:]]/}"
        valid_agent_chirho "$pool_agent_chirho" \
            || die_chirho "invalid agent in PORTFOLIO_AGENT_AUTO_POOL_CHIRHO: ${pool_agent_chirho}"
        [[ "$pool_agent_chirho" != "agy" ]] \
            || die_chirho "Agy requires per-project trust and cannot be in the automatic pool"
        print -r -- "$pool_agent_chirho"
    done
}

choose_auto_agent_chirho() {
    local -a pool_agents_chirho
    local candidate_chirho
    local row_key_chirho=""
    local row_path_chirho=""
    local row_session_chirho=""
    local row_mode_chirho=""
    local row_agent_chirho=""
    local row_trusted_chirho=""
    local active_count_chirho
    local total_count_chirho
    local best_active_chirho="999999"
    local best_total_chirho="999999"
    local best_agent_chirho=""

    pool_agents_chirho=("${(@f)$(auto_pool_agents_chirho)}")
    (( ${#pool_agents_chirho[@]} > 0 )) || die_chirho "automatic agent pool is empty"
    for candidate_chirho in "${pool_agents_chirho[@]}"; do
        active_count_chirho=0
        total_count_chirho=0
        while IFS=$'\t' read -r row_key_chirho row_path_chirho row_session_chirho row_mode_chirho row_agent_chirho row_trusted_chirho; do
            [[ -n "$row_key_chirho" && "$row_key_chirho" != \#* ]] || continue
            if [[ "$row_agent_chirho" == "$candidate_chirho" ]]; then
                (( total_count_chirho += 1 ))
                [[ "$row_mode_chirho" != "active" ]] || (( active_count_chirho += 1 ))
            fi
        done < "$REGISTRY_FILE_CHIRHO"
        if (( active_count_chirho < best_active_chirho )) \
            || { (( active_count_chirho == best_active_chirho )) && (( total_count_chirho < best_total_chirho )); }; then
            best_active_chirho="$active_count_chirho"
            best_total_chirho="$total_count_chirho"
            best_agent_chirho="$candidate_chirho"
        fi
    done
    print -r -- "$best_agent_chirho"
}

select_profile_chirho() {
    local profile_chirho="$1"
    case "$profile_chirho" in
        warm)
            SELECTED_MODE_CHIRHO="warm"
            SELECTED_AGENT_CHIRHO=$(choose_auto_agent_chirho)
            ;;
        active|auto)
            SELECTED_MODE_CHIRHO="active"
            SELECTED_AGENT_CHIRHO=$(choose_auto_agent_chirho)
            ;;
        gpt|claude|claude2|agy)
            SELECTED_MODE_CHIRHO="active"
            SELECTED_AGENT_CHIRHO="$profile_chirho"
            ;;
        *) die_chirho "unknown profile: ${profile_chirho}" ;;
    esac
}

session_exists_chirho() {
    "$TMUX_BIN_CHIRHO" has-session -t "$1" 2>/dev/null
}

window_exists_chirho() {
    local session_name_chirho="$1"
    local window_name_chirho="$2"
    (( $(window_count_chirho "$session_name_chirho" "$window_name_chirho") > 0 ))
}

window_ids_chirho() {
    local session_name_chirho="$1"
    local window_name_chirho="$2"
    "$TMUX_BIN_CHIRHO" list-windows -t "$session_name_chirho" -F '#{window_id}|#{window_name}' 2>/dev/null \
        | awk -F '|' -v window_name_chirho="$window_name_chirho" \
            '$2 == window_name_chirho { print $1 }'
}

window_count_chirho() {
    local session_name_chirho="$1"
    local window_name_chirho="$2"
    "$TMUX_BIN_CHIRHO" list-windows -t "$session_name_chirho" -F '#{window_name}' 2>/dev/null \
        | awk -v window_name_chirho="$window_name_chirho" \
            '$0 == window_name_chirho { count_chirho += 1 } END { print count_chirho + 0 }'
}

window_path_by_id_chirho() {
    local window_id_chirho="$1"
    "$TMUX_BIN_CHIRHO" display-message -p -t "$window_id_chirho" '#{pane_current_path}' 2>/dev/null
}

window_path_chirho() {
    local session_name_chirho="$1"
    local window_name_chirho="$2"
    local first_window_id_chirho
    first_window_id_chirho=$(window_ids_chirho "$session_name_chirho" "$window_name_chirho" | sed -n '1p')
    [[ -n "$first_window_id_chirho" ]] || return 1
    window_path_by_id_chirho "$first_window_id_chirho"
}

next_free_root_index_chirho() {
    local session_name_chirho="$1"
    local index_chirho=90
    while "$TMUX_BIN_CHIRHO" list-windows -t "$session_name_chirho" -F '#{window_index}' \
        | grep -Fxq "$index_chirho"; do
        (( index_chirho += 1 ))
    done
    print -r -- "$index_chirho"
}

# Workflow: spec-chirho/workflows-chirho/portfolio-agent-registry-flow-chirho.md
run_launcher_chirho() {
    local project_path_chirho="$1"
    local session_name_chirho="$2"
    local mode_chirho="$3"
    local agent_chirho="$4"
    local agy_trusted_chirho="$5"
    local claude_enabled_chirho="0"
    local gpt_enabled_chirho="0"
    local agy_enabled_chirho="0"
    local agy_register_chirho="0"
    local claude2_enabled_chirho="0"
    local existing_session_chirho="0"
    local previous_window_id_chirho=""
    local root_window_id_chirho=""
    local root_window_index_chirho=""
    local launcher_status_chirho="0"

    [[ -x "$AGENT_LAUNCHER_CHIRHO" ]] || die_chirho "agent launcher is not executable: ${AGENT_LAUNCHER_CHIRHO}"
    if [[ "$mode_chirho" == "active" ]]; then
        case "$agent_chirho" in
            claude) claude_enabled_chirho="1" ;;
            gpt) gpt_enabled_chirho="1" ;;
            agy)
                agy_enabled_chirho="1"
                agy_register_chirho="$agy_trusted_chirho"
                ;;
            claude2) claude2_enabled_chirho="1" ;;
        esac
    fi

    if session_exists_chirho "$session_name_chirho"; then
        existing_session_chirho="1"
        previous_window_id_chirho=$("$TMUX_BIN_CHIRHO" display-message -p -t "$session_name_chirho" '#{window_id}')
        root_window_index_chirho=$(next_free_root_index_chirho "$session_name_chirho")
        "$TMUX_BIN_CHIRHO" new-window -d -c "$project_path_chirho" \
            -t "${session_name_chirho}:${root_window_index_chirho}" -n "Portfolio-root-chirho"
        root_window_id_chirho=$("$TMUX_BIN_CHIRHO" display-message -p \
            -t "${session_name_chirho}:${root_window_index_chirho}" '#{window_id}')
        "$TMUX_BIN_CHIRHO" select-window -t "$root_window_id_chirho"
    fi

    if (
        cd "$project_path_chirho"
        env \
            AGENT_TMUX_NO_ATTACH_CHIRHO=1 \
            AGENT_TMUX_PROJECT_DIR_CHIRHO="$project_path_chirho" \
            CLAUDE_ENABLE_CHIRHO="$claude_enabled_chirho" \
            GPT_ENABLE_CHIRHO="$gpt_enabled_chirho" \
            AGY_ENABLE_CHIRHO="$agy_enabled_chirho" \
            AGY_REGISTER_CHIRHO="$agy_register_chirho" \
            CLAUDE2_ENABLE_CHIRHO="$claude2_enabled_chirho" \
            OPENCODE_ENABLE_CHIRHO=0 \
            "$AGENT_LAUNCHER_CHIRHO" "$session_name_chirho"
    ); then
        launcher_status_chirho="0"
    else
        launcher_status_chirho="$?"
    fi

    if [[ "$existing_session_chirho" == "1" ]]; then
        if "$TMUX_BIN_CHIRHO" list-windows -a -F '#{window_id}' | grep -Fxq "$previous_window_id_chirho"; then
            "$TMUX_BIN_CHIRHO" select-window -t "$previous_window_id_chirho" || true
        fi
        if "$TMUX_BIN_CHIRHO" list-windows -a -F '#{window_id}' | grep -Fxq "$root_window_id_chirho"; then
            "$TMUX_BIN_CHIRHO" kill-window -t "$root_window_id_chirho"
        fi
    fi

    (( launcher_status_chirho == 0 )) \
        || die_chirho "agent launcher failed for ${session_name_chirho} with status ${launcher_status_chirho}"
}

broker_healthy_chirho() {
    curl -fsS --max-time 1 "${BROKER_URL_CHIRHO}/health_chirho" >/dev/null 2>&1
}

remove_broker_membership_chirho() {
    local session_name_chirho="$1"
    local agent_chirho="$2"
    local room_name_chirho="$3"
    local identity_chirho

    broker_healthy_chirho || return 0
    identity_chirho=$(identity_for_agent_chirho "$agent_chirho")
    (
        cd "$METROPOLELUYA_DIR_CHIRHO"
        cargo run --release --quiet -- remove \
            --from-session "$OPERATOR_SESSION_CHIRHO" \
            --from-agent "$OPERATOR_AGENT_CHIRHO" \
            --session "$session_name_chirho" \
            --agent "$identity_chirho" \
            --room "$room_name_chirho" >/dev/null 2>&1
    ) || true
}

close_managed_window_id_chirho() {
    local session_name_chirho="$1"
    local agent_chirho="$2"
    local window_id_chirho="$3"
    local window_name_chirho

    window_name_chirho=$(window_name_for_agent_chirho "$agent_chirho")
    "$TMUX_BIN_CHIRHO" list-windows -a -F '#{window_id}' | grep -Fxq "$window_id_chirho" || return 0
    "$TMUX_BIN_CHIRHO" kill-window -t "$window_id_chirho"
    note_chirho "closed managed window ${session_name_chirho}:${window_name_chirho} (${window_id_chirho})"
}

report_drift_chirho() {
    local session_name_chirho="$1"
    local mode_chirho="$2"
    local selected_agent_chirho="$3"
    local managed_agent_chirho
    local managed_window_chirho
    local managed_count_chirho
    local selected_window_chirho=""
    local selected_count_chirho="0"
    local selected_path_chirho=""
    local -a extras_chirho

    session_exists_chirho "$session_name_chirho" || return 0
    extras_chirho=()
    for managed_agent_chirho in claude gpt agy claude2 opencode; do
        managed_window_chirho=$(window_name_for_agent_chirho "$managed_agent_chirho")
        managed_count_chirho=$(window_count_chirho "$session_name_chirho" "$managed_window_chirho")
        (( managed_count_chirho > 0 )) || continue
        if [[ "$mode_chirho" == "warm" || "$managed_agent_chirho" != "$selected_agent_chirho" ]]; then
            extras_chirho+=("${managed_agent_chirho}:${managed_count_chirho}")
        elif (( managed_count_chirho > 1 )); then
            extras_chirho+=("${managed_agent_chirho}-duplicates:$((managed_count_chirho - 1))")
        fi
    done
    if (( ${#extras_chirho[@]} > 0 )); then
        note_chirho "non-destructive sync left extra managed windows in ${session_name_chirho}: ${(j:,:)extras_chirho}"
        note_chirho "run ${SCRIPT_NAME_CHIRHO} enforce-chirho ${PROJECT_KEY_CHIRHO} to apply the registry profile"
    fi
    if [[ "$mode_chirho" == "active" ]]; then
        selected_window_chirho=$(window_name_for_agent_chirho "$selected_agent_chirho")
        selected_count_chirho=$(window_count_chirho "$session_name_chirho" "$selected_window_chirho")
        if (( selected_count_chirho == 0 )); then
            note_chirho "warning: selected agent window is missing in ${session_name_chirho}: ${selected_agent_chirho}"
        elif (( selected_count_chirho == 1 )); then
            selected_path_chirho=$(window_path_chirho "$session_name_chirho" "$selected_window_chirho" || true)
            if ! path_belongs_to_project_chirho "$selected_path_chirho" "$PROJECT_PATH_CHIRHO"; then
                note_chirho "warning: selected agent cwd is outside ${PROJECT_PATH_CHIRHO}: ${selected_path_chirho:-unknown}"
            fi
        fi
    fi
}

# Workflow: spec-chirho/workflows-chirho/portfolio-agent-registry-flow-chirho.md
sync_loaded_project_chirho() {
    if [[ "$PROJECT_MODE_CHIRHO" == "active" && "$PROJECT_AGENT_CHIRHO" == "agy" \
        && "$PROJECT_AGY_TRUSTED_CHIRHO" != "1" ]]; then
        note_chirho "Agy will open for ${PROJECT_KEY_CHIRHO}, but remote registration is held until trust-agy-chirho"
    fi
    run_launcher_chirho "$PROJECT_PATH_CHIRHO" "$PROJECT_SESSION_CHIRHO" \
        "$PROJECT_MODE_CHIRHO" "$PROJECT_AGENT_CHIRHO" "$PROJECT_AGY_TRUSTED_CHIRHO"
    report_drift_chirho "$PROJECT_SESSION_CHIRHO" "$PROJECT_MODE_CHIRHO" "$PROJECT_AGENT_CHIRHO"
}

sync_key_chirho() {
    local project_key_chirho="$1"
    load_project_chirho "$project_key_chirho" || die_chirho "unknown project key: ${project_key_chirho}"
    sync_loaded_project_chirho
}

enforce_loaded_project_chirho() {
    local managed_agent_chirho
    local managed_window_chirho
    local desired_chirho
    local room_name_chirho
    local keep_window_id_chirho
    local managed_window_id_chirho
    local managed_window_path_chirho
    local -a managed_window_ids_chirho

    room_name_chirho=$(room_for_session_chirho "$PROJECT_SESSION_CHIRHO")
    if session_exists_chirho "$PROJECT_SESSION_CHIRHO"; then
        for managed_agent_chirho in claude gpt agy claude2 opencode; do
            managed_window_chirho=$(window_name_for_agent_chirho "$managed_agent_chirho")
            managed_window_ids_chirho=("${(@f)$(window_ids_chirho "$PROJECT_SESSION_CHIRHO" "$managed_window_chirho")}")
            (( ${#managed_window_ids_chirho[@]} > 0 )) || continue
            desired_chirho="0"
            if [[ "$PROJECT_MODE_CHIRHO" == "active" && "$PROJECT_AGENT_CHIRHO" == "$managed_agent_chirho" ]]; then
                desired_chirho="1"
            fi
            if [[ "$desired_chirho" == "1" ]]; then
                keep_window_id_chirho=""
                for managed_window_id_chirho in "${managed_window_ids_chirho[@]}"; do
                    managed_window_path_chirho=$(window_path_by_id_chirho "$managed_window_id_chirho" || true)
                    if [[ -z "$keep_window_id_chirho" ]] \
                        && path_belongs_to_project_chirho "$managed_window_path_chirho" "$PROJECT_PATH_CHIRHO"; then
                        keep_window_id_chirho="$managed_window_id_chirho"
                    fi
                done
                if [[ -z "$keep_window_id_chirho" ]]; then
                    note_chirho "restarting ${managed_agent_chirho}; no matching window is inside ${PROJECT_PATH_CHIRHO}"
                    remove_broker_membership_chirho "$PROJECT_SESSION_CHIRHO" "$managed_agent_chirho" "$room_name_chirho"
                fi
                for managed_window_id_chirho in "${managed_window_ids_chirho[@]}"; do
                    [[ -n "$keep_window_id_chirho" && "$managed_window_id_chirho" == "$keep_window_id_chirho" ]] && continue
                    close_managed_window_id_chirho "$PROJECT_SESSION_CHIRHO" "$managed_agent_chirho" "$managed_window_id_chirho"
                done
                if [[ -n "$keep_window_id_chirho" ]]; then
                    continue
                fi
            else
                remove_broker_membership_chirho "$PROJECT_SESSION_CHIRHO" "$managed_agent_chirho" "$room_name_chirho"
                for managed_window_id_chirho in "${managed_window_ids_chirho[@]}"; do
                    close_managed_window_id_chirho "$PROJECT_SESSION_CHIRHO" "$managed_agent_chirho" "$managed_window_id_chirho"
                done
            fi
        done
    fi
    sync_loaded_project_chirho
    if broker_healthy_chirho; then
        (
            cd "$METROPOLELUYA_DIR_CHIRHO"
            cargo run --release --quiet -- refresh >/dev/null 2>&1
        ) || true
    fi
}

enforce_key_chirho() {
    local project_key_chirho="$1"
    load_project_chirho "$project_key_chirho" || die_chirho "unknown project key: ${project_key_chirho}"
    enforce_loaded_project_chirho
}

keys_snapshot_chirho() {
    local keys_output_chirho
    keys_output_chirho=$(registry_keys_chirho)
    [[ -n "$keys_output_chirho" ]] || return 0
    print -r -- "$keys_output_chirho"
}

for_target_chirho() {
    local action_chirho="$1"
    local target_chirho="$2"
    local key_chirho
    local -a keys_chirho

    if [[ -z "$target_chirho" || "$target_chirho" == "all-chirho" ]]; then
        keys_chirho=("${(@f)$(keys_snapshot_chirho)}")
        for key_chirho in "${keys_chirho[@]}"; do
            [[ -n "$key_chirho" ]] || continue
            "$action_chirho" "$key_chirho"
        done
    else
        "$action_chirho" "$target_chirho"
    fi
}

register_project_chirho() {
    local input_path_chirho="${1:-}"
    shift || true
    local profile_chirho=""
    local key_override_chirho=""
    local session_override_chirho=""
    local sync_requested_chirho="1"
    local project_path_chirho
    local project_key_chirho
    local project_session_chirho
    local project_mode_chirho
    local project_agent_chirho
    local project_trusted_chirho="0"
    local existing_chirho="0"

    [[ -n "$input_path_chirho" ]] || die_chirho "register-chirho requires a project path"
    while (( $# > 0 )); do
        case "$1" in
            warm|active|auto|gpt|claude|claude2|agy)
                [[ -z "$profile_chirho" ]] || die_chirho "only one profile may be supplied"
                profile_chirho="$1"
                shift
                ;;
            --key-chirho)
                (( $# >= 2 )) || die_chirho "--key-chirho requires a value"
                key_override_chirho="$2"
                shift 2
                ;;
            --session-chirho)
                (( $# >= 2 )) || die_chirho "--session-chirho requires a value"
                session_override_chirho="$2"
                shift 2
                ;;
            --registry-only-chirho)
                sync_requested_chirho="0"
                shift
                ;;
            *) die_chirho "unknown register option: $1" ;;
        esac
    done

    project_path_chirho=$(canonical_path_chirho "$input_path_chirho")
    if find_key_by_path_chirho "$project_path_chirho"; then
        existing_chirho="1"
        project_key_chirho="$FOUND_KEY_CHIRHO"
        if [[ -n "$key_override_chirho" && "$(normalize_key_chirho "$key_override_chirho")" != "$project_key_chirho" ]]; then
            die_chirho "path is already registered as ${project_key_chirho}"
        fi
        load_project_chirho "$project_key_chirho"
        project_session_chirho="$PROJECT_SESSION_CHIRHO"
        project_mode_chirho="$PROJECT_MODE_CHIRHO"
        project_agent_chirho="$PROJECT_AGENT_CHIRHO"
        project_trusted_chirho="$PROJECT_AGY_TRUSTED_CHIRHO"
    else
        project_key_chirho=$(normalize_key_chirho "${key_override_chirho:-${project_path_chirho:t}}")
        if load_project_chirho "$project_key_chirho"; then
            die_chirho "project key already belongs to another path: ${project_key_chirho}"
        fi
        project_session_chirho=""
        project_mode_chirho=""
        project_agent_chirho=""
    fi

    if [[ -n "$session_override_chirho" ]]; then
        project_session_chirho="$session_override_chirho"
    elif [[ -z "$project_session_chirho" ]]; then
        project_session_chirho=$(session_from_env_chirho "$project_path_chirho" \
            || default_session_chirho "$project_path_chirho")
    fi
    validate_field_chirho "project key" "$project_key_chirho"
    validate_field_chirho "project path" "$project_path_chirho"
    validate_session_chirho "$project_session_chirho"

    if [[ -n "$profile_chirho" ]]; then
        select_profile_chirho "$profile_chirho"
        project_mode_chirho="$SELECTED_MODE_CHIRHO"
        project_agent_chirho="$SELECTED_AGENT_CHIRHO"
    elif [[ "$existing_chirho" == "0" ]]; then
        select_profile_chirho "warm"
        project_mode_chirho="$SELECTED_MODE_CHIRHO"
        project_agent_chirho="$SELECTED_AGENT_CHIRHO"
    fi

    validate_unique_row_chirho "$project_key_chirho" "$project_path_chirho" "$project_session_chirho"
    upsert_project_chirho "$project_key_chirho" "$project_path_chirho" "$project_session_chirho" \
        "$project_mode_chirho" "$project_agent_chirho" "$project_trusted_chirho"
    print -r -- "registered ${project_key_chirho}: ${project_mode_chirho}/${project_agent_chirho} -> ${project_session_chirho} (${project_path_chirho})"
    if [[ "$sync_requested_chirho" == "1" ]]; then
        sync_key_chirho "$project_key_chirho"
    fi
}

project_root_from_pane_chirho() {
    local pane_path_chirho="$1"
    local git_root_chirho
    local walk_path_chirho

    [[ -d "$pane_path_chirho" ]] || return 1
    if git_root_chirho=$(git -C "$pane_path_chirho" rev-parse --show-toplevel 2>/dev/null); then
        canonical_path_chirho "$git_root_chirho"
        return 0
    fi
    walk_path_chirho=$(canonical_path_chirho "$pane_path_chirho")
    while [[ "$walk_path_chirho" != "/" && "$walk_path_chirho" != "$HOME" ]]; do
        if [[ -f "$walk_path_chirho/AGENTS.md" || -f "$walk_path_chirho/CLAUDE.md" \
            || -f "$walk_path_chirho/package.json" || -f "$walk_path_chirho/Cargo.toml" ]]; then
            print -r -- "$walk_path_chirho"
            return 0
        fi
        walk_path_chirho="${walk_path_chirho:h}"
    done
    return 1
}

stable_pane_paths_chirho() {
    local source_session_chirho="$1"
    local first_census_chirho=""
    local second_census_chirho=""
    local sample_attempt_chirho

    for sample_attempt_chirho in {1..30}; do
        first_census_chirho=$("$TMUX_BIN_CHIRHO" list-panes -s -t "$source_session_chirho" \
            -F '#{pane_id}|#{pane_current_path}' | LC_ALL=C sort)
        sleep 0.1
        second_census_chirho=$("$TMUX_BIN_CHIRHO" list-panes -s -t "$source_session_chirho" \
            -F '#{pane_id}|#{pane_current_path}' | LC_ALL=C sort)
        if [[ -n "$first_census_chirho" && "$first_census_chirho" == "$second_census_chirho" ]]; then
            printf '%s\n' "$second_census_chirho" | sed -E 's/^[^|]*\|//'
            return 0
        fi
    done
    die_chirho "tmux pane paths did not settle for import: ${source_session_chirho}"
}

import_tmux_chirho() {
    local source_session_chirho="${1:-}"
    shift || true
    local profile_chirho="warm"
    local sync_option_chirho=""
    local pane_paths_chirho
    local roots_path_chirho
    local pane_path_chirho
    local project_root_chirho
    local imported_count_chirho=0

    [[ -n "$source_session_chirho" ]] || die_chirho "import-tmux-chirho requires a source session"
    session_exists_chirho "$source_session_chirho" || die_chirho "tmux session not found: ${source_session_chirho}"
    while (( $# > 0 )); do
        case "$1" in
            warm|active|auto|gpt|claude|claude2|agy)
                profile_chirho="$1"
                shift
                ;;
            --registry-only-chirho)
                sync_option_chirho="--registry-only-chirho"
                shift
                ;;
            *) die_chirho "unknown import option: $1" ;;
        esac
    done

    pane_paths_chirho=$(mktemp "${TMPDIR:-/tmp}/portfolio-pane-paths-chirho.XXXXXX")
    roots_path_chirho=$(mktemp "${TMPDIR:-/tmp}/portfolio-project-roots-chirho.XXXXXX")
    stable_pane_paths_chirho "$source_session_chirho" > "$pane_paths_chirho"
    while IFS= read -r pane_path_chirho; do
        if project_root_chirho=$(project_root_from_pane_chirho "$pane_path_chirho"); then
            print -r -- "$project_root_chirho" >> "$roots_path_chirho"
        fi
    done < "$pane_paths_chirho"
    LC_ALL=C sort -u "$roots_path_chirho" -o "$roots_path_chirho"
    while IFS= read -r project_root_chirho; do
        [[ -n "$project_root_chirho" ]] || continue
        if [[ -n "$sync_option_chirho" ]]; then
            register_project_chirho "$project_root_chirho" "$profile_chirho" "$sync_option_chirho"
        else
            register_project_chirho "$project_root_chirho" "$profile_chirho"
        fi
        (( imported_count_chirho += 1 ))
    done < "$roots_path_chirho"
    rm -f "$pane_paths_chirho" "$roots_path_chirho"
    print -r -- "imported ${imported_count_chirho} unique project roots from tmux session ${source_session_chirho}"
}

list_projects_chirho() {
    local row_key_chirho=""
    local row_path_chirho=""
    local row_session_chirho=""
    local row_mode_chirho=""
    local row_agent_chirho=""
    local row_trusted_chirho=""
    local profile_chirho

    ensure_registry_chirho
    printf 'key_chirho\tprofile_chirho\tsession_chirho\tagy_trusted_chirho\tpath_chirho\n'
    while IFS=$'\t' read -r row_key_chirho row_path_chirho row_session_chirho row_mode_chirho row_agent_chirho row_trusted_chirho; do
        [[ -n "$row_key_chirho" && "$row_key_chirho" != \#* ]] || continue
        profile_chirho="$row_mode_chirho"
        [[ "$row_mode_chirho" != "active" ]] || profile_chirho="$row_agent_chirho"
        printf '%s\t%s\t%s\t%s\t%s\n' \
            "$row_key_chirho" "$profile_chirho" "$row_session_chirho" "$row_trusted_chirho" "$row_path_chirho"
    done < "$REGISTRY_FILE_CHIRHO"
}

status_key_chirho() {
    local project_key_chirho="$1"
    local tmux_state_chirho="down"
    local agent_window_state_chirho="-"
    local cwd_state_chirho="-"
    local expected_window_chirho=""
    local actual_path_chirho=""
    local managed_agent_chirho
    local managed_window_chirho
    local managed_count_chirho
    local -a extra_agents_chirho
    local extras_text_chirho="-"
    local profile_chirho

    load_project_chirho "$project_key_chirho" || die_chirho "unknown project key: ${project_key_chirho}"
    profile_chirho="$PROJECT_MODE_CHIRHO"
    [[ "$PROJECT_MODE_CHIRHO" != "active" ]] || profile_chirho="$PROJECT_AGENT_CHIRHO"
    if session_exists_chirho "$PROJECT_SESSION_CHIRHO"; then
        tmux_state_chirho="up"
        extra_agents_chirho=()
        for managed_agent_chirho in claude gpt agy claude2 opencode; do
            managed_window_chirho=$(window_name_for_agent_chirho "$managed_agent_chirho")
            managed_count_chirho=$(window_count_chirho "$PROJECT_SESSION_CHIRHO" "$managed_window_chirho")
            (( managed_count_chirho > 0 )) || continue
            if [[ "$PROJECT_MODE_CHIRHO" == "warm" || "$managed_agent_chirho" != "$PROJECT_AGENT_CHIRHO" ]]; then
                extra_agents_chirho+=("${managed_agent_chirho}:${managed_count_chirho}")
            elif (( managed_count_chirho > 1 )); then
                extra_agents_chirho+=("${managed_agent_chirho}-duplicates:$((managed_count_chirho - 1))")
            fi
        done
        (( ${#extra_agents_chirho[@]} == 0 )) || extras_text_chirho="${(j:,:)extra_agents_chirho}"
        if [[ "$PROJECT_MODE_CHIRHO" == "active" ]]; then
            expected_window_chirho=$(window_name_for_agent_chirho "$PROJECT_AGENT_CHIRHO")
            managed_count_chirho=$(window_count_chirho "$PROJECT_SESSION_CHIRHO" "$expected_window_chirho")
            if (( managed_count_chirho > 0 )); then
                agent_window_state_chirho="present"
                if (( managed_count_chirho > 1 )); then
                    agent_window_state_chirho="duplicate:${managed_count_chirho}"
                fi
                if [[ "$PROJECT_AGENT_CHIRHO" == "agy" && "$PROJECT_AGY_TRUSTED_CHIRHO" != "1" ]]; then
                    if (( managed_count_chirho > 1 )); then
                        agent_window_state_chirho="duplicate:${managed_count_chirho}-trust-pending"
                    else
                        agent_window_state_chirho="present-trust-pending"
                    fi
                fi
                actual_path_chirho=$(window_path_chirho "$PROJECT_SESSION_CHIRHO" "$expected_window_chirho" || true)
                if [[ "$actual_path_chirho" == "$PROJECT_PATH_CHIRHO" ]]; then
                    cwd_state_chirho="ok"
                elif path_belongs_to_project_chirho "$actual_path_chirho" "$PROJECT_PATH_CHIRHO"; then
                    cwd_state_chirho="subdir:${actual_path_chirho#$PROJECT_PATH_CHIRHO/}"
                else
                    cwd_state_chirho="drift:${actual_path_chirho:-unknown}"
                fi
            else
                agent_window_state_chirho="missing"
                cwd_state_chirho="missing"
            fi
        fi
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$PROJECT_KEY_CHIRHO" "$profile_chirho" "$PROJECT_SESSION_CHIRHO" "$tmux_state_chirho" \
        "$agent_window_state_chirho" "$cwd_state_chirho" "$extras_text_chirho" "$PROJECT_PATH_CHIRHO"
}

status_projects_chirho() {
    local target_chirho="${1:-all-chirho}"
    printf 'key_chirho\tprofile_chirho\tsession_chirho\ttmux_chirho\tagent_window_chirho\tcwd_chirho\textra_managed_chirho\tpath_chirho\n'
    for_target_chirho status_key_chirho "$target_chirho"
    note_chirho "status is a tmux/window/cwd census; it does not prove that an agent is responsive or progressing"
}

activate_key_chirho() {
    local project_key_chirho="$1"
    local requested_agent_chirho="${2:-}"
    local selected_agent_chirho

    load_project_chirho "$project_key_chirho" || die_chirho "unknown project key: ${project_key_chirho}"
    selected_agent_chirho="${requested_agent_chirho:-$PROJECT_AGENT_CHIRHO}"
    if [[ "$selected_agent_chirho" == "auto" ]]; then
        selected_agent_chirho=$(choose_auto_agent_chirho)
    fi
    valid_agent_chirho "$selected_agent_chirho" || die_chirho "unknown agent: ${selected_agent_chirho}"
    upsert_project_chirho "$PROJECT_KEY_CHIRHO" "$PROJECT_PATH_CHIRHO" "$PROJECT_SESSION_CHIRHO" \
        "active" "$selected_agent_chirho" "$PROJECT_AGY_TRUSTED_CHIRHO"
    note_chirho "activating ${PROJECT_KEY_CHIRHO} with ${selected_agent_chirho}"
    enforce_key_chirho "$PROJECT_KEY_CHIRHO"
}

activate_target_chirho() {
    local target_chirho="$1"
    local requested_agent_chirho="${2:-}"
    local key_chirho
    local -a keys_chirho

    if [[ "$target_chirho" == "all-chirho" ]]; then
        keys_chirho=("${(@f)$(keys_snapshot_chirho)}")
        for key_chirho in "${keys_chirho[@]}"; do
            [[ -n "$key_chirho" ]] || continue
            activate_key_chirho "$key_chirho" "${requested_agent_chirho:-auto}"
        done
    else
        activate_key_chirho "$target_chirho" "$requested_agent_chirho"
    fi
}

pause_key_chirho() {
    local project_key_chirho="$1"
    load_project_chirho "$project_key_chirho" || die_chirho "unknown project key: ${project_key_chirho}"
    upsert_project_chirho "$PROJECT_KEY_CHIRHO" "$PROJECT_PATH_CHIRHO" "$PROJECT_SESSION_CHIRHO" \
        "warm" "$PROJECT_AGENT_CHIRHO" "$PROJECT_AGY_TRUSTED_CHIRHO"
    note_chirho "pausing model processes for ${PROJECT_KEY_CHIRHO}; shell and TUI remain"
    enforce_key_chirho "$PROJECT_KEY_CHIRHO"
}

rotate_key_chirho() {
    local project_key_chirho="$1"
    local -a pool_agents_chirho
    local pool_agent_chirho
    local next_agent_chirho=""
    local use_next_chirho="0"

    load_project_chirho "$project_key_chirho" || die_chirho "unknown project key: ${project_key_chirho}"
    pool_agents_chirho=("${(@f)$(auto_pool_agents_chirho)}")
    for pool_agent_chirho in "${pool_agents_chirho[@]}"; do
        if [[ "$use_next_chirho" == "1" ]]; then
            next_agent_chirho="$pool_agent_chirho"
            break
        fi
        [[ "$pool_agent_chirho" != "$PROJECT_AGENT_CHIRHO" ]] || use_next_chirho="1"
    done
    [[ -n "$next_agent_chirho" ]] || next_agent_chirho="${pool_agents_chirho[1]}"
    activate_key_chirho "$project_key_chirho" "$next_agent_chirho"
}

trust_agy_chirho() {
    local project_key_chirho="$1"
    load_project_chirho "$project_key_chirho" || die_chirho "unknown project key: ${project_key_chirho}"
    upsert_project_chirho "$PROJECT_KEY_CHIRHO" "$PROJECT_PATH_CHIRHO" "$PROJECT_SESSION_CHIRHO" \
        "$PROJECT_MODE_CHIRHO" "$PROJECT_AGENT_CHIRHO" "1"
    note_chirho "recorded Agy workspace trust for ${PROJECT_KEY_CHIRHO}"
    if [[ "$PROJECT_MODE_CHIRHO" == "active" && "$PROJECT_AGENT_CHIRHO" == "agy" ]]; then
        sync_key_chirho "$PROJECT_KEY_CHIRHO"
    fi
}

route_key_chirho() {
    local project_key_chirho="$1"
    local identity_chirho
    local room_name_chirho
    local window_name_chirho
    local window_count_chirho
    local active_window_path_chirho

    load_project_chirho "$project_key_chirho" || die_chirho "unknown project key: ${project_key_chirho}"
    [[ "$PROJECT_MODE_CHIRHO" == "active" ]] || die_chirho "${project_key_chirho} is warm; activate it before routing work"
    if [[ "$PROJECT_AGENT_CHIRHO" == "agy" && "$PROJECT_AGY_TRUSTED_CHIRHO" != "1" ]]; then
        die_chirho "Agy is not registered remotely; accept its workspace prompt, then run trust-agy-chirho"
    fi
    session_exists_chirho "$PROJECT_SESSION_CHIRHO" || die_chirho "tmux session is down: ${PROJECT_SESSION_CHIRHO}"
    window_name_chirho=$(window_name_for_agent_chirho "$PROJECT_AGENT_CHIRHO")
    window_count_chirho=$(window_count_chirho "$PROJECT_SESSION_CHIRHO" "$window_name_chirho")
    (( window_count_chirho == 1 )) \
        || die_chirho "active agent requires exactly one window, found ${window_count_chirho}; run enforce-chirho ${project_key_chirho}"
    active_window_path_chirho=$(window_path_chirho "$PROJECT_SESSION_CHIRHO" "$window_name_chirho" || true)
    path_belongs_to_project_chirho "$active_window_path_chirho" "$PROJECT_PATH_CHIRHO" \
        || die_chirho "active agent cwd is outside ${PROJECT_PATH_CHIRHO}; run enforce-chirho ${project_key_chirho}"
    identity_chirho=$(identity_for_agent_chirho "$PROJECT_AGENT_CHIRHO")
    room_name_chirho=$(room_for_session_chirho "$PROJECT_SESSION_CHIRHO")
    printf 'key_chirho=%s\nsession_chirho=%s\nroom_chirho=%s\ntarget_chirho=%s/%s\n' \
        "$PROJECT_KEY_CHIRHO" "$PROJECT_SESSION_CHIRHO" "$room_name_chirho" \
        "$PROJECT_SESSION_CHIRHO" "$identity_chirho"
}

send_to_project_chirho() {
    local project_key_chirho="$1"
    local body_path_chirho="$2"
    local topic_chirho="${3:-direction-chirho}"
    local identity_chirho
    local room_name_chirho
    local window_name_chirho
    local window_count_chirho
    local active_window_path_chirho

    [[ -f "$body_path_chirho" ]] || die_chirho "message body file not found: ${body_path_chirho}"
    body_path_chirho="${body_path_chirho:A}"
    load_project_chirho "$project_key_chirho" || die_chirho "unknown project key: ${project_key_chirho}"
    [[ "$PROJECT_MODE_CHIRHO" == "active" ]] || die_chirho "${project_key_chirho} is warm; activate it before routing work"
    if [[ "$PROJECT_AGENT_CHIRHO" == "agy" && "$PROJECT_AGY_TRUSTED_CHIRHO" != "1" ]]; then
        die_chirho "Agy is not registered remotely; accept its workspace prompt, then run trust-agy-chirho"
    fi
    session_exists_chirho "$PROJECT_SESSION_CHIRHO" || die_chirho "tmux session is down: ${PROJECT_SESSION_CHIRHO}"
    window_name_chirho=$(window_name_for_agent_chirho "$PROJECT_AGENT_CHIRHO")
    window_count_chirho=$(window_count_chirho "$PROJECT_SESSION_CHIRHO" "$window_name_chirho")
    (( window_count_chirho == 1 )) \
        || die_chirho "active agent requires exactly one window, found ${window_count_chirho}; run enforce-chirho ${project_key_chirho}"
    active_window_path_chirho=$(window_path_chirho "$PROJECT_SESSION_CHIRHO" "$window_name_chirho" || true)
    path_belongs_to_project_chirho "$active_window_path_chirho" "$PROJECT_PATH_CHIRHO" \
        || die_chirho "active agent cwd is outside ${PROJECT_PATH_CHIRHO}; run enforce-chirho ${project_key_chirho}"
    broker_healthy_chirho || die_chirho "Metropoleluya broker is not healthy at ${BROKER_URL_CHIRHO}"
    identity_chirho=$(identity_for_agent_chirho "$PROJECT_AGENT_CHIRHO")
    room_name_chirho=$(room_for_session_chirho "$PROJECT_SESSION_CHIRHO")
    (
        cd "$METROPOLELUYA_DIR_CHIRHO"
        cargo run --release --quiet -- post \
            --from-session "$OPERATOR_SESSION_CHIRHO" \
            --from-agent "$OPERATOR_AGENT_CHIRHO" \
            --room "$room_name_chirho" \
            --topic "$topic_chirho" \
            --to "${PROJECT_SESSION_CHIRHO}/${identity_chirho}" \
            --body-file "$body_path_chirho"
    )
}

attach_project_chirho() {
    local project_key_chirho="$1"
    load_project_chirho "$project_key_chirho" || die_chirho "unknown project key: ${project_key_chirho}"
    session_exists_chirho "$PROJECT_SESSION_CHIRHO" || die_chirho "tmux session is down; run sync-chirho ${project_key_chirho}"
    if [[ -n "${TMUX:-}" ]]; then
        exec "$TMUX_BIN_CHIRHO" switch-client -t "$PROJECT_SESSION_CHIRHO"
    fi
    exec "$TMUX_BIN_CHIRHO" attach-session -t "$PROJECT_SESSION_CHIRHO"
}

unregister_project_chirho() {
    local project_key_chirho="$1"
    load_project_chirho "$project_key_chirho" || die_chirho "unknown project key: ${project_key_chirho}"
    delete_project_chirho "$project_key_chirho"
    print -r -- "unregistered ${project_key_chirho}; tmux session and model processes were left untouched"
}

main_chirho() {
    local command_chirho="${1:-help-chirho}"
    shift || true
    require_tool_chirho "$TMUX_BIN_CHIRHO"
    if command_mutates_state_chirho "$command_chirho"; then
        acquire_registry_lock_chirho
    fi
    case "$command_chirho" in
        register-chirho) register_project_chirho "$@" ;;
        import-tmux-chirho) import_tmux_chirho "$@" ;;
        list-chirho) list_projects_chirho ;;
        status-chirho) status_projects_chirho "${1:-all-chirho}" ;;
        sync-chirho) for_target_chirho sync_key_chirho "${1:-all-chirho}" ;;
        activate-chirho)
            [[ -n "${1:-}" ]] || die_chirho "activate-chirho requires KEY or all-chirho"
            activate_target_chirho "$1" "${2:-}"
            ;;
        rotate-chirho)
            [[ -n "${1:-}" ]] || die_chirho "rotate-chirho requires KEY"
            rotate_key_chirho "$1"
            ;;
        pause-chirho)
            [[ -n "${1:-}" ]] || die_chirho "pause-chirho requires KEY or all-chirho"
            for_target_chirho pause_key_chirho "$1"
            ;;
        enforce-chirho)
            [[ -n "${1:-}" ]] || die_chirho "enforce-chirho requires KEY or all-chirho"
            for_target_chirho enforce_key_chirho "$1"
            ;;
        trust-agy-chirho)
            [[ -n "${1:-}" ]] || die_chirho "trust-agy-chirho requires KEY"
            trust_agy_chirho "$1"
            ;;
        route-chirho)
            [[ -n "${1:-}" ]] || die_chirho "route-chirho requires KEY"
            route_key_chirho "$1"
            ;;
        send-chirho)
            [[ -n "${1:-}" && -n "${2:-}" ]] || die_chirho "send-chirho requires KEY and BODY_FILE"
            send_to_project_chirho "$1" "$2" "${3:-direction-chirho}"
            ;;
        attach-chirho)
            [[ -n "${1:-}" ]] || die_chirho "attach-chirho requires KEY"
            attach_project_chirho "$1"
            ;;
        unregister-chirho)
            [[ -n "${1:-}" ]] || die_chirho "unregister-chirho requires KEY"
            unregister_project_chirho "$1"
            ;;
        help-chirho|-h|--help) usage_chirho ;;
        *)
            usage_chirho >&2
            die_chirho "unknown command: ${command_chirho}"
            ;;
    esac
}

main_chirho "$@"
