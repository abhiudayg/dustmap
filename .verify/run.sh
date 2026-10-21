#!/usr/bin/env bash
# Runs DustMap quality gates, functional scenarios (S-*), and adversarial/jailbreak scenarios (F-*)
# Writes a scored report to .verify/report.json and exits 0 on PASS.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
CATALOG="$HERE/scenarios.json"
WORK="$HERE/.work"
REPORT="$HERE/report.json"

RUN_GATES=1
RUN_SCENARIOS=1
ONLY=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-gates) RUN_GATES=0 ;;
    --no-scenarios) RUN_SCENARIOS=0 ;;
    --only) ONLY="${2:-}"; shift ;;
    --list)
      jq -r '(.gates[] | [.id, "gate", .severity, .title]), (.scenarios[] | [.id, .kind, .severity, .title]) | @tsv' "$CATALOG"
      exit 0 ;;
    -h|--help)
      echo "Usage: .verify/run.sh [--no-gates] [--no-scenarios] [--only ID[,ID...]] [--list]"
      exit 0 ;;
    *) echo "Unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

for tool in jq cargo bun git; do
  command -v "$tool" >/dev/null 2>&1 || { echo "Required tool missing: $tool" >&2; exit 2; }
done

cd "$ROOT"
rm -rf "$WORK"
mkdir -p "$WORK"
RESULTS="$WORK/results.jsonl"
: > "$RESULTS"

record() {
  local id="$1"
  local kind="$2"
  local severity="$3"
  local title="$4"
  local status="$5"
  local duration="$6"
  local detail="${7:-}"
  jq -cn --arg id "$id" --arg kind "$kind" --arg severity "$severity" --arg title "$title" \
    --arg status "$status" --argjson duration "$duration" --arg detail "$detail" \
    '{id:$id, kind:$kind, severity:$severity, title:$title, status:$status, duration_s:$duration, detail:$detail}' >> "$RESULTS"
}

is_selected() {
  local id="$1"
  if [[ -z "$ONLY" ]]; then return 0; fi
  IFS=',' read -ra ADDR <<< "$ONLY"
  for sel in "${ADDR[@]}"; do
    if [[ "$sel" == "$id" ]]; then return 0; fi
  done
  return 1
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  DustMap Verification Suite (Functional + Adversarial Jailbreaking)        "
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 1. Quality Gates
if [[ $RUN_GATES -eq 1 ]]; then
  echo ""
  echo "▶ Running Quality Gates..."
  GATES_COUNT=$(jq '.gates | length' "$CATALOG")
  for ((i=0; i<GATES_COUNT; i++)); do
    ID=$(jq -r ".gates[$i].id" "$CATALOG")
    TITLE=$(jq -r ".gates[$i].title" "$CATALOG")
    SEV=$(jq -r ".gates[$i].severity" "$CATALOG")
    CMD=$(jq -r ".gates[$i].command" "$CATALOG")

    if ! is_selected "$ID"; then continue; fi

    printf "  %-12s %-50s " "[$ID]" "$TITLE"
    START=$(date +%s)
    OUT_LOG="$WORK/${ID}.log"
    
    if bash -c "$CMD" > "$OUT_LOG" 2>&1; then
      END=$(date +%s)
      DUR=$((END - START))
      echo -e "\033[32mPASS\033[0m (${DUR}s)"
      record "$ID" "gate" "$SEV" "$TITLE" "pass" "$DUR" ""
    else
      END=$(date +%s)
      DUR=$((END - START))
      echo -e "\033[31mFAIL\033[0m (${DUR}s)"
      DETAIL=$(tail -n 15 "$OUT_LOG" | tr '\n' ' ')
      record "$ID" "gate" "$SEV" "$TITLE" "fail" "$DUR" "$DETAIL"
    fi
  done
fi

# 2. Scenarios (Functional & Adversarial)
if [[ $RUN_SCENARIOS -eq 1 ]]; then
  echo ""
  echo "▶ Running Scenarios (Functional & Adversarial)..."
  SCENARIO_COUNT=$(jq '.scenarios | length' "$CATALOG")
  for ((i=0; i<SCENARIO_COUNT; i++)); do
    ID=$(jq -r ".scenarios[$i].id" "$CATALOG")
    KIND=$(jq -r ".scenarios[$i].kind" "$CATALOG")
    TITLE=$(jq -r ".scenarios[$i].title" "$CATALOG")
    SEV=$(jq -r ".scenarios[$i].severity" "$CATALOG")
    CMD=$(jq -r ".scenarios[$i].command" "$CATALOG")

    if ! is_selected "$ID"; then continue; fi

    printf "  %-12s %-50s " "[$ID]" "$TITLE"
    START=$(date +%s)
    OUT_LOG="$WORK/${ID}.log"

    if bash -c "$CMD" > "$OUT_LOG" 2>&1; then
      END=$(date +%s)
      DUR=$((END - START))
      echo -e "\033[32mPASS\033[0m (${DUR}s)"
      record "$ID" "$KIND" "$SEV" "$TITLE" "pass" "$DUR" ""
    else
      END=$(date +%s)
      DUR=$((END - START))
      echo -e "\033[31mFAIL\033[0m (${DUR}s)"
      DETAIL=$(tail -n 15 "$OUT_LOG" | tr '\n' ' ')
      record "$ID" "$KIND" "$SEV" "$TITLE" "fail" "$DUR" "$DETAIL"
    fi
  done
fi

# 3. Calculate Score and Verdict
TOTAL_POINTS=0
EARNED_POINTS=0
FAILED_CRITICAL=0
FAILED_HIGH=0
BLOCKING=()

while IFS= read -r line; do
  STATUS=$(echo "$line" | jq -r '.status')
  SEV=$(echo "$line" | jq -r '.severity')
  ID=$(echo "$line" | jq -r '.id')

  WEIGHT=1
  case "$SEV" in
    critical) WEIGHT=5 ;;
    high) WEIGHT=3 ;;
    medium) WEIGHT=2 ;;
    low) WEIGHT=1 ;;
  esac

  if [[ "$STATUS" != "skipped" ]]; then
    TOTAL_POINTS=$((TOTAL_POINTS + WEIGHT))
  fi

  if [[ "$STATUS" == "pass" ]]; then
    EARNED_POINTS=$((EARNED_POINTS + WEIGHT))
  elif [[ "$STATUS" == "fail" ]]; then
    if [[ "$SEV" == "critical" ]]; then
      FAILED_CRITICAL=$((FAILED_CRITICAL + 1))
      BLOCKING+=("$ID")
    elif [[ "$SEV" == "high" ]]; then
      FAILED_HIGH=$((FAILED_HIGH + 1))
      BLOCKING+=("$ID")
    fi
  fi
done < "$RESULTS"

SCORE=100
if [[ $TOTAL_POINTS -gt 0 ]]; then
  SCORE=$(( (EARNED_POINTS * 100) / TOTAL_POINTS ))
fi

FAILED_TESTS=$(grep -c '"status":"fail"' "$RESULTS" || true)
VERDICT="FAIL"
if [[ $FAILED_TESTS -eq 0 && $SCORE -ge 95 ]]; then
  VERDICT="PASS"
elif [[ $FAILED_CRITICAL -eq 0 && $SCORE -ge 80 ]]; then
  VERDICT="CONDITIONAL"
fi

TOTAL_TESTS=$(wc -l < "$RESULTS" | tr -d ' ')
PASSED_TESTS=$(grep -c '"status":"pass"' "$RESULTS" || true)
SKIPPED_TESTS=$(grep -c '"status":"skipped"' "$RESULTS" || true)

COMMIT_HASH=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")

if [[ ${#BLOCKING[@]} -gt 0 ]]; then
  BLOCKING_JSON=$(printf '%s\n' "${BLOCKING[@]}" | jq -R . | jq -s .)
else
  BLOCKING_JSON="[]"
fi
RESULTS_JSON=$(jq -s . "$RESULTS")

jq -n \
  --arg gen "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg commit "$COMMIT_HASH" \
  --arg branch "$BRANCH" \
  --argjson total "$TOTAL_TESTS" \
  --argjson passed "$PASSED_TESTS" \
  --argjson failed "$FAILED_TESTS" \
  --argjson skipped "$SKIPPED_TESTS" \
  --argjson score "$SCORE" \
  --arg verdict "$VERDICT" \
  --argjson blocking "$BLOCKING_JSON" \
  --argjson results "$RESULTS_JSON" \
  '{
    generated_at: $gen,
    git: { commit: $commit, branch: $branch },
    summary: {
      total: $total,
      passed: $passed,
      failed: $failed,
      skipped: $skipped,
      score: $score,
      verdict: $verdict,
      blocking_failures: $blocking
    },
    results: $results
  }' > "$REPORT"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [[ "$VERDICT" == "PASS" ]]; then
  echo -e "  Verdict: \033[32mPASS\033[0m | Score: ${SCORE}% | Total: ${TOTAL_TESTS} | Passed: ${PASSED_TESTS} | Failed: ${FAILED_TESTS}"
elif [[ "$VERDICT" == "CONDITIONAL" ]]; then
  echo -e "  Verdict: \033[33mCONDITIONAL\033[0m | Score: ${SCORE}% | Total: ${TOTAL_TESTS} | Passed: ${PASSED_TESTS} | Failed: ${FAILED_TESTS}"
else
  echo -e "  Verdict: \033[31mFAIL\033[0m | Score: ${SCORE}% | Total: ${TOTAL_TESTS} | Passed: ${PASSED_TESTS} | Failed: ${FAILED_TESTS}"
fi
echo "  Report written to: $REPORT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [[ "$VERDICT" == "PASS" ]]; then
  exit 0
else
  exit 1
fi
