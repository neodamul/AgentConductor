# CLI subprocess 실행 계약

공식 문서와 2026-10-09 로컬 Codex help 확인을 기반으로 한 예시입니다.
버전별 contract test를 통과한 프로필만 실행합니다. 아래 shell redirection은 설명용이며
Rust 구현은 `Command`+argv+pipe로 실행하고 task 문자열을 shell에 넣지 않습니다.
각 명령은 해당 task worktree를 cwd로 실행합니다.

## Claude Code

```sh
claude -p --output-format stream-json --verbose \
  --permission-mode acceptEdits \
  < task.txt > stdout.jsonl 2> stderr.log

claude -p --resume SESSION_ID --output-format stream-json --verbose \
  --permission-mode acceptEdits \
  < feedback.txt > resume.jsonl 2> resume.log
```

stdin은 UTF-8 text를 쓰고 EOF를 보냅니다. token delta가 필요하면 `--include-partial-messages` 추가.
파서: system/init의 session_id, assistant의 content blocks, stream_event의 text_delta,
최종 result의 result/is_error/session_id/usage/cost를 추출합니다. stderr만 도구 로그라고 가정하지 않고
stdout tool_use/tool_result도 기록합니다. JSON 단일 출력은 `--output-format json`, `.result`와
`.structured_output`을 결과 envelope와 구분합니다.
acceptEdits는 범용 sandbox/안전 명령 승인과 같지 않습니다. 계획은 read-only 도구 정책,
구현은 제한된 tool 정책과 CLI sandbox를 별도 확인합니다. bypass 플래그는 기본값으로 쓰지 않습니다.

근거: [headless](https://code.claude.com/docs/en/headless),
[CLI reference](https://code.claude.com/docs/en/cli-reference).

## Codex

```sh
codex exec - --json --sandbox workspace-write \
  -o final.txt < task.txt > stdout.jsonl 2> stderr.log

codex exec resume SESSION_ID - --json \
  -o final.txt < feedback.txt > resume.jsonl 2> resume.log
```

새 실행과 resume의 옵션 집합은 다릅니다. 로컬 resume help에는 `--sandbox`, `-C`가 없으므로
새 실행 argv를 그대로 재사용하지 않습니다. 동일 cwd를 Command.current_dir로 지정하고
resume 권한 구성을 실제 버전에 맞춰 검증합니다. ephemeral은 session 저장이 필요하므로 쓰지 않습니다.
파서: thread.started.thread_id, turn.started, item.started/updated/completed,
agent_message.text, command_execution exit/status, file_change, turn.completed.usage,
turn.failed/error. exit 0과 turn 성공을 모두 검사합니다. `-o`는 최종 메시지 파일입니다.
`--output-schema result.schema.json`으로 최종 결과를 구조화할 수 있지만 JSONL 이벤트와는 별개입니다.
noninteractive exec가 범용 동기 사람 승인 UI를 제공한다고 가정하지 않습니다.
기존 CLI 인증을 재사용하며 `--last`는 병렬 운영에서 쓰지 않습니다.

근거: [noninteractive](https://developers.openai.com/codex/noninteractive/),
[CLI reference](https://developers.openai.com/codex/cli/reference/).

## OpenCode

```sh
opencode run --format json --file task.md \
  "첨부 작업 파일의 범위와 완료 조건에 따라 수행하세요" \
  > stdout.jsonl 2> stderr.log

opencode run --session SESSION_ID --format json --file feedback.md \
  "첨부 피드백에 따라 이 세션의 작업을 수정하세요" \
  > resume.jsonl 2> resume.log
```

stdin 동작을 추정하지 않고 공식 `--file`로 task/context를 전달합니다.
출력은 raw JSON events이며 고정 버전 fixture로 JSONL 여부와 sessionID/part 구조를 확인합니다.
일반적인 text, tool_use, step_start/step_finish, error를 정규화하지만 step_finish 한 건을
전체 작업 종료로 취급하지 않습니다. exit와 최종 text/세션 export를 함께 확인합니다.
표준 결과가 필요하면 result.json 파일도 계약에 지정하며 JSON event envelope와 구분합니다.
`opencode export SESSION_ID`는 추가 세션 근거 수집에 활용합니다. `--continue`는 사용하지 않습니다.
`--auto`를 기본 적용하지 않고 noninteractive 권한 거절/정지 동작을 smoke로 검증합니다.

오케스트레이터에는 API key/endpoint 등록이 없습니다. OpenCode가 자체적으로 사용하는
provider 인증은 별개입니다. Claude Code 로그인 토큰을 OpenCode로 복사하지 않습니다.
현재 공식 provider 문서는 Claude 구독 우회 사용의 제한을 명시하므로 Claude 작업은 native Claude CLI로
보냅니다. OpenCode는 이미 적법하게 설정한 지원 인증 또는 local provider를 사용합니다.

근거: [CLI](https://opencode.ai/docs/cli/), [providers](https://opencode.ai/docs/providers/).

## custom CLI와 공통 재시도

custom 프로필은 shell:false에 해당하는 executable+argv, stdin task JSON 또는 task 파일,
표준 JSONL `{type,session_id,payload}` 출력 계약을 받습니다.
자체호스트 HTTP는 별도 선택 transport이며 OpenAI-compatible completion endpoint를 요구하지 않습니다.

공통 전략은 [DD-07](../../docs/06-design-decisions.md)의 wall timeout/process group/attempt 정책입니다.
auth/config/권한 오류는 자동 retry하지 않습니다. partial writes가 있는 timeout은 diff 확인 후 resume.
재지시는 실패 테스트/충돌 파일/정확한 수정 범위를 담고 무제한 대화 대신 제한된 attempt를 생성합니다.
