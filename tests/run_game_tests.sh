#!/usr/bin/env bash
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DERIVED="/tmp/hw-test-derived"
BIN="$DERIVED/Build/Products/Debug/HelloWorld"
BUILD_LOG="$DERIVED/build.log"
FAILURES=0

pass() { echo "PASS: $1"; }
fail() { echo "FAIL: $1"; FAILURES=$((FAILURES + 1)); }

run_with_timeout() {
  local seconds="$1"
  shift
  if command -v timeout >/dev/null 2>&1; then
    timeout "$seconds" "$@"
  elif command -v gtimeout >/dev/null 2>&1; then
    gtimeout "$seconds" "$@"
  else
    perl -e 'alarm shift; exec @ARGV' "$seconds" "$@"
  fi
}

check_build() {
  mkdir -p "$DERIVED"
  if xcodebuild -project HelloWorld.xcodeproj -scheme HelloWorld -configuration Debug \
      -derivedDataPath "$DERIVED" build CODE_SIGNING_ALLOWED=NO MACOSX_DEPLOYMENT_TARGET=12.0 \
      >"$BUILD_LOG" 2>&1; then
    pass "build"
  else
    fail "build (see $BUILD_LOG)"
    return
  fi

  if grep -E 'HelloWorld/(Game|Participant|main)\.(cpp|hpp|h|m|mm|swift).*:.*(warning|error):' "$BUILD_LOG" >/dev/null 2>&1; then
    fail "build produced warning/error in HelloWorld/Game, Participant, or main"
  else
    pass "no Game/Participant/main warning or error in build log"
  fi
}

check_case() {
  local name="$1" input="$2" expected="$3"
  local out rc result_lines last

  if [[ ! -x "$BIN" ]]; then
    fail "$name: executable missing at $BIN"
    return
  fi

  set +e
  if [[ "$input" == "__EMPTY__" ]]; then
    out="$(run_with_timeout 10 "$BIN" </dev/null 2>&1)"
  else
    out="$(run_with_timeout 10 "$BIN" < "$input" 2>&1)"
  fi
  rc=$?
  set -e

  if [[ $rc -ne 0 ]]; then
    fail "$name: exit code $rc (expected 0)"
    return
  else
    pass "$name: exit code 0"
  fi

  result_lines="$(printf '%s\n' "$out" | grep -c '^RESULT:' || true)"
  if [[ "$result_lines" -ne 1 ]]; then
    fail "$name: RESULT line count $result_lines (expected 1)"
  else
    pass "$name: exactly one RESULT line"
  fi

  last="$(printf '%s\n' "$out" | awk 'NF{line=$0} END{print line}')"
  if [[ "$last" == "$expected" ]]; then
    pass "$name: marker '$expected'"
  else
    fail "$name: marker '$last' (expected '$expected')"
  fi
}

check_line_limits() {
  local f lines ok=1
  while IFS= read -r f; do
    [[ -z "$f" ]] && continue
    lines="$(wc -l < "$f" | tr -d ' ')"
    if [[ "$lines" -gt 300 ]]; then
      fail "line limit: $f has $lines lines (>300)"
      ok=0
    fi
  done < <(find HelloWorld tests -type f \( -name '*.cpp' -o -name '*.hpp' -o -name '*.sh' \) 2>/dev/null | sort)
  if [[ $ok -eq 1 ]]; then
    pass "all .cpp/.hpp/.sh in HelloWorld/ and tests/ are <= 300 lines"
  fi
}

check_person_unchanged() {
  if git diff --quiet HEAD -- HelloWorld/Person.hpp HelloWorld/Person.cpp; then
    pass "HelloWorld/Person.hpp and Person.cpp unchanged"
  else
    fail "HelloWorld/Person.hpp or Person.cpp modified"
  fi
}

echo "== build =="
check_build

echo "== cases =="
check_case "win" "tests/fixtures/win.in" "RESULT: WIN"
check_case "moved" "tests/fixtures/moved.in" "RESULT: ELIMINATED (moved)"
check_case "timeout" "tests/fixtures/timeout.in" "RESULT: ELIMINATED (timeout)"
check_case "eof" "tests/fixtures/eof.in" "RESULT: ABORTED (EOF)"
check_case "invalid" "tests/fixtures/invalid.in" "RESULT: ABORTED (invalid input)"
check_case "empty-stdin" "__EMPTY__" "RESULT: ABORTED (EOF)"

echo "== static checks =="
check_line_limits
check_person_unchanged

if [[ $FAILURES -ne 0 ]]; then
  echo "RESULT_SUMMARY: FAIL ($FAILURES failure(s))"
  exit 1
fi

echo "RESULT_SUMMARY: PASS"
exit 0
