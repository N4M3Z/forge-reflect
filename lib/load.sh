#!/usr/bin/env bash
# Forge Core — content loader.
# Source this file to get the load_context() function.
#
# Reads a module's config (config.yaml → defaults.yaml fallback),
# resolves system + user content paths, transforms frontmatter per
# metadata mapping, processes !`command` blocks, and outputs content.
#
# Ships with every forge module as lib/load.sh (standalone fallback).
# When FORGE_LIB is set, the Core version wins. Eventually rewritten in Rust.
#
# Usage:
#   source load.sh
#   load_context "$MODULE_ROOT" "$PROJECT_ROOT"
#   load_context "$MODULE_ROOT" "$PROJECT_ROOT" "Section Header"
#   load_context "$MODULE_ROOT" "$PROJECT_ROOT" --index-only
#   load_user_content "$MODULE_ROOT" "$PROJECT_ROOT"

# --- Dependencies: parser.sh and strip-front.sh ---
# Source from FORGE_LIB if available, otherwise use embedded versions.

if [ -n "${FORGE_LIB:-}" ] && [ -f "$FORGE_LIB/parser.sh" ]; then
  # shellcheck source=parser.sh
  source "$FORGE_LIB/parser.sh"
elif ! type parse_yaml_list &>/dev/null; then
  # Embedded fallback: minimal parser (standalone mode)
  parse_yaml_list() {
    local file="$1" key="$2"
    [ -f "$file" ] || return 0
    awk -v key="$key" '
      $0 ~ "^" key ":" {
        if (match($0, /\[.*\]/)) {
          content = substr($0, RSTART + 1, RLENGTH - 2)
          n = split(content, items, ",")
          for (i = 1; i <= n; i++) {
            val = items[i]
            gsub(/^[[:space:]]*["'"'"']?|["'"'"']?[[:space:]]*$/, "", val)
            if (val != "") print val
          }
          exit
        }
        in_list = 1; next
      }
      in_list {
        if (/^[a-zA-Z_]/ || /^---/) exit
        if (/^[[:space:]]*-/) {
          val = $0
          sub(/^[[:space:]]*-[[:space:]]*/, "", val)
          gsub(/^["'"'"']|["'"'"']$/, "", val)
          if (val != "") print val
        }
      }
    ' "$file"
  }

  resolve_path() {
    local path="$1" module_root="$2" project_root="$3"
    if [[ "$path" == /* ]] && [ -e "$path" ]; then
      echo "$path"
    elif [ -e "$module_root/$path" ]; then
      echo "$module_root/$path"
    elif [ -e "$project_root/$path" ]; then
      echo "$project_root/$path"
    fi
  }

  parse_yaml_map() {
    local file="$1" key="$2"
    [ -f "$file" ] || return 0
    awk -v key="$key" '
      $0 ~ "^" key ":" {
        val = $0; sub("^" key ":[[:space:]]*", "", val)
        if (val == "" || val ~ /^#/) { in_map = 1; next }
      }
      in_map {
        if (/^[a-zA-Z_]/ || /^---/ || /^$/) exit
        if (/^[[:space:]]+[a-zA-Z_-]+:/) {
          src = $0; sub(/^[[:space:]]+/, "", src)
          idx = index(src, ":")
          k = substr(src, 1, idx - 1)
          v = substr(src, idx + 1)
          gsub(/^[[:space:]]*/, "", v)
          if (match(v, /^\[.*\]/)) {
            content = substr(v, RSTART + 1, RLENGTH - 2)
            n = split(content, items, ",")
            for (i = 1; i <= n; i++) {
              item = items[i]
              gsub(/^[[:space:]]*["'"'"']?|["'"'"']?[[:space:]]*$/, "", item)
              if (item != "") print k "\t" item
            }
          } else {
            gsub(/["'"'"']?[[:space:]]*$/, "", v)
            gsub(/^["'"'"']/, "", v)
            if (k != "" && v != "") print k "\t" v
          }
        }
      }
    ' "$file"
  }
fi

if [ -n "${FORGE_LIB:-}" ] && [ -f "$FORGE_LIB/strip-front.sh" ]; then
  # shellcheck source=strip-front.sh
  source "$FORGE_LIB/strip-front.sh"
elif ! type strip_front &>/dev/null; then
  # Embedded fallback: minimal frontmatter stripper (standalone mode)
  strip_front() {
    local file="$1"
    [ -f "$file" ] || return 1
    awk '
      /^---$/ && !started { started = 1; skip = 1; next }
      /^---$/ && skip      { skip = 0; next }
      skip                 { next }
      !body && /^# /       { body = 1; next }
      { body = 1; print }
    ' "$file"
  }
fi

# --- Config resolution ---

# _resolve_config MODULE_ROOT
# Returns config.yaml if it exists, else defaults.yaml.
# Follows .env.example/.env pattern: defaults.yaml is checked into git,
# config.yaml is gitignored and created by the user to override.
_resolve_config() {
  local module_root="$1"
  if [ -f "$module_root/config.yaml" ]; then
    echo "$module_root/config.yaml"
  elif [ -f "$module_root/defaults.yaml" ]; then
    echo "$module_root/defaults.yaml"
  fi
}

# --- Metadata transformation ---

# transform_front FILE FIELD_MAP
# Extract frontmatter from FILE, keep only mapped fields, rename as configured.
# FIELD_MAP is tab-separated "output_name\tsource_field" pairs (one per line).
# Multiple rows with the same output_name enable fallback chains (first match wins).
# Convention (Airbyte/dbt style): output field = key, source field = value.
# Returns: transformed frontmatter (---...---) + stripped body.
# If FIELD_MAP is empty, strips all frontmatter (backward compatible).
transform_front() {
  local file="$1"
  local field_map="$2"
  [ -f "$file" ] || return 1

  if [ -z "$field_map" ]; then
    # No mapping configured — strip everything (backward compatible)
    strip_front "$file"
    return
  fi

  # Build lookup: source_field → output_name
  # parse_yaml_map returns "output_name\tsource_field" (key\tvalue from config)
  local -A source_to_output=()
  while IFS=$'\t' read -r output_name source_field; do
    [ -n "$source_field" ] && source_to_output["$source_field"]="$output_name"
  done <<< "$field_map"

  # Two-pass: extract mapped frontmatter, then emit body
  local in_fm=false fm_done=false kept_lines=""
  local -A emitted=()
  local body_content
  body_content=$(strip_front "$file")

  # Extract frontmatter fields (first match wins per output field)
  while IFS= read -r line; do
    if [ "$line" = "---" ] && [ "$in_fm" = false ] && [ "$fm_done" = false ]; then
      in_fm=true
      continue
    fi
    if [ "$line" = "---" ] && [ "$in_fm" = true ]; then
      fm_done=true
      break
    fi
    if [ "$in_fm" = true ]; then
      if [[ "$line" =~ ^([a-zA-Z_-]+):[[:space:]]*(.*) ]]; then
        local fkey="${BASH_REMATCH[1]}"
        local fval="${BASH_REMATCH[2]}"
        if [ -n "${source_to_output[$fkey]+x}" ]; then
          local output_name="${source_to_output[$fkey]}"
          if [ -z "${emitted[$output_name]+x}" ]; then
            kept_lines+="$output_name: $fval"$'\n'
            emitted["$output_name"]=1
          fi
        fi
      fi
    fi
  done < "$file"

  # Emit: transformed frontmatter + body
  if [ -n "$kept_lines" ]; then
    printf '%s\n%s---\n' "---" "$kept_lines"
  fi
  [ -n "$body_content" ] && printf '%s\n' "$body_content"
}

# --- Main function ---

# load_context MODULE_ROOT PROJECT_ROOT [SECTION_HEADER] [--index-only]
#
# Reads config from MODULE_ROOT (config.yaml → defaults.yaml fallback).
# Loads system then user content paths. Transforms frontmatter per metadata
# mapping. Processes !`command` blocks (Dynamic Context Injection) in body.
#
# --index-only: emit only transformed metadata (no body). For session-start
# hooks where the body is lazy-loaded via skills or file reads.
load_context() {
  local module_root="$1"
  local project_root="$2"
  shift 2
  local section_header="" index_only=false
  while [ $# -gt 0 ]; do
    case "$1" in
      --index-only) index_only=true ;;
      *) section_header="$1" ;;
    esac
    shift
  done
  local config
  config=$(_resolve_config "$module_root")
  [ -z "$config" ] && return 0
  local output=""

  # Read metadata field mapping from config (if configured)
  local field_map=""
  if type parse_yaml_map &>/dev/null; then
    field_map=$(parse_yaml_map "$config" "metadata")
  fi

  # _load_entry FILE — process a single file with metadata transformation
  _load_entry() {
    local file="$1"
    local content
    content=$(transform_front "$file" "$field_map")
    if [ "$index_only" = true ]; then
      if [ -n "$field_map" ]; then
        # Extract only the metadata block (between --- delimiters)
        content=$(echo "$content" | awk '/^---$/{n++; print; next} n==1{print} n>=2{exit}')
      else
        # No metadata mapping — nothing to index
        content=""
      fi
    fi
    # Process !`command` blocks (Dynamic Context Injection for non-Claude providers)
    if [ "$index_only" = false ] && [[ "$content" == *'!`'* ]]; then
      local rendered=""
      while IFS= read -r cline; do
        if [[ "$cline" =~ ^\!\`.+\`$ ]]; then
          local cmd="${cline#!\`}"
          cmd="${cmd%\`}"
          local cmd_out
          cmd_out=$(eval "$cmd" 2>/dev/null) || true
          [ -n "$cmd_out" ] && rendered+="$cmd_out"$'\n'
        else
          rendered+="$cline"$'\n'
        fi
      done <<< "$content"
      content="$rendered"
    fi
    [ -n "$content" ] && output+="$content"$'\n'
  }

  # _load_path ENTRY — resolve and load a file or directory
  _load_path() {
    local entry="$1"
    [ -z "$entry" ] && return
    local resolved
    resolved=$(resolve_path "$entry" "$module_root" "$project_root")
    [ -z "$resolved" ] && return

    if [ -f "$resolved" ]; then
      _load_entry "$resolved"
    elif [ -d "$resolved" ]; then
      for f in "$resolved"/*.md; do
        [ -f "$f" ] || continue
        _load_entry "$f"
      done
    fi
  }

  # Load system entries (Tier 2)
  while IFS= read -r entry; do
    _load_path "$entry"
  done < <(parse_yaml_list "$config" "system")

  # Load user entries (Tier 3)
  while IFS= read -r entry; do
    _load_path "$entry"
  done < <(parse_yaml_list "$config" "user")

  # Output with optional section header
  if [ -n "$output" ]; then
    if [ -n "$section_header" ]; then
      printf '## %s\n\n%s' "$section_header" "$output"
    else
      printf '%s' "$output"
    fi
  fi
}

# --- User content loader ---

# load_user_content MODULE_ROOT PROJECT_ROOT
#
# Loads only user content paths from config (config.yaml → defaults.yaml).
# Strips frontmatter, emits body only. Called by SKILL.md !`command`
# preprocessing to inject user extensions at skill invocation time.
load_user_content() {
  local module_root="$1" project_root="$2"
  local config
  config=$(_resolve_config "$module_root")
  [ -z "$config" ] && return 0
  local output=""

  _uc_emit_file() {
    local content
    content=$(strip_front "$1")
    [ -n "$content" ] && output+="$content"$'\n'
  }

  _uc_emit_path() {
    local entry="$1"
    [ -z "$entry" ] && return
    local resolved
    resolved=$(resolve_path "$entry" "$module_root" "$project_root")
    [ -z "$resolved" ] && return
    if [ -f "$resolved" ]; then
      _uc_emit_file "$resolved"
    elif [ -d "$resolved" ]; then
      for f in "$resolved"/*.md; do
        [ -f "$f" ] || continue
        _uc_emit_file "$f"
      done
    fi
  }

  while IFS= read -r entry; do
    [ -n "$entry" ] && _uc_emit_path "$entry"
  done < <(parse_yaml_list "$config" "user")

  [ -n "$output" ] && printf '%s' "$output"
}
