# DD: Design Decisions

## DD-01 Rust 단일 패키지

사용자 확정 언어는 Rust입니다. clap + Tokio(process/io/time/sync) + serde/serde_json + rusqlite로 시작합니다.
Git은 libgit2 추가 대신 설치된 git 실행 파일을 argv로 호출합니다. 셸 문자열 조합은 하지 않습니다.
HTTP 기억 연동에 reqwest를 선택적으로 사용하며 async trait 프레임워크는 도입하지 않습니다.
의존성 버전·feature와 라이선스는 구현 시작 시 lockfile 기준으로 검증합니다.
관련: SF-01/03/06/09, QAS-05/08/09/11.

## DD-02 coordinator가 상태를 소유

SQLite WAL을 단일 writer가 갱신합니다. tasks/edges/runs/events/artifacts/test_runs/approvals/
memory_outbox/lessons/knowledge_sources를 사용합니다. run ID + seq로 이벤트 중복을 억제합니다.
작업 상태: pending, running, waiting_approval, blocked, completed, failed.
실행 상태: prepared → ready → starting → running → stopping → stopped/completed/failed.
`start`, `stop`은 명령, `starting`, `stopping`은 전이 상태입니다.
개발 단계: plan → task → verify → complete. 세 상태 축을 하나의 enum에 혼합하지 않습니다.
process exit 0은 run 종료를 뜻하며 task 완료와 동일하지 않습니다.
관련: SF-02/04/06, QAS-02/03/04.

## DD-03 등록 가능한 프로필

AgentProfile: id, transport(cli), executable, argv, resume_argv, input_mode(text_stdin/file),
output_mode(json/jsonl), parser, environment overrides, permission capability,
timeout policy, allowed path scopes. worktree cwd는 task마다 coordinator가 주입합니다.
custom CLI는 같은 계약을 사용합니다. aipro의 실제 플래그는 문서 확인 후 등록합니다.
외부 HTTP는 추후 같은 결과 계약으로 추가하고 LLM completion API와 구분합니다.
부모 인증 환경/표준 HOME은 유지하고 API key를 별도 등록·복사하지 않습니다.
API-key/endpoint override 환경이 발견되면 값은 노출하지 않고 인증 경로를 알립니다.
관련: SF-01, QAS-09.

## DD-04 JSON 수집과 정규화

stdin/stdout/stderr는 별도 비동기 태스크로 처리합니다. text stdin은 쓰고 닫습니다.
stdout bytes를 disk spool한 뒤 UTF-8/JSON line 단위로 파싱합니다.
분할 line, CRLF, EOF 마지막 line, 알려지지 않은 이벤트, 최대 line 크기를 다룹니다.
stderr는 별도 로그로 drain합니다. bounded queue가 차도 pipe를 방치하지 않습니다.
NormalizedEvent: schema_version, run/task/agent/session ID, seq, observed_at,
kind(session/phase/progress/tool/result/usage/error/exit), payload, raw_offset.
Result: summary, evidence[], changed_files[], test evidence refs, artifacts[], exit, usage(nullable).
agent 보고와 coordinator가 측정한 Git/test 근거를 별도 필드로 둡니다.
외부 CLI 내부 도구 중 출력되지 않은 호출은 감사 로그 completeness=partial로 표시합니다.
관련: SF-04, QAS-05.

## DD-05 worktree와 통합 후보

고정 base SHA에서 각 `codex/ac-<goal>-<task>` branch/worktree를 만듭니다.
파일 소유 범위와 예상 신규 파일을 계획에 명시하고 glob 중첩을 검증합니다.
worktree는 변경 충돌을 줄이지만 OS 보안 격리가 아니며 shared Git metadata를 가집니다.
scope는 사후 검증, 강한 강제는 sandbox/권한 설정에서 수행합니다.
status --porcelain -z와 staged/unstaged diff 및 untracked 수집을 함께 사용합니다.
에이전트 정지 확인 후 후보 commit을 만들며 raw diff만으로 신규 파일을 누락시키지 않습니다.
테스트 에이전트는 합의된 인터페이스·테스트 스펙으로 병렬 작성합니다. 구현이 필요한 최종 테스트는
통합 후보에서 실행합니다. 세 작업이 같은 base를 썼다는 이유만으로 통합 성공을 가정하지 않습니다.
관련: SF-03/04, QAS-01/06.

## DD-06 사람 승인

승인은 action, repo, base SHA, candidate SHA, diff hash, test hash, 허용 범위와 만료를 묶습니다.
승인 전 merge/push/deploy/비허용 외부 전송/시크릿 접근은 금지합니다.
각 CLI의 permission flag를 같은 의미라고 취급하지 않습니다. plain headless CLI가
동기 승인 중계를 제공하지 않으면 도구를 거절하고 종료/보고한 뒤 승인된 새 attempt로 재개합니다.
stdout에서 tool_call을 본 시점에는 이미 실행됐을 수 있으므로 로그 파싱을 사전 승인으로 사용하지 않습니다.
MVP는 보호 branch 변경·배포 자격증명을 coordinator만 소유하도록 구성합니다.
위험 작업을 강제로 막을 수 없는 custom CLI는 read-only/sandbox 내 실행 또는 unsupported로 표시합니다.
임의 코드의 완전한 host 보안 격리는 worktree만으로 제공할 수 없습니다.
CLI 모델 서버로의 필요한 요청과 task가 임의로 하는 외부 전송도 구분해 정책에 기록합니다.
관련: SF-05, QAS-02/06.

## DD-07 timeout/retry/resume

제안 기본값: wall 30분, 무출력 5분은 경고만, SIGINT 5초→SIGTERM 5초→SIGKILL/reap.
POSIX process group, Windows Job Object로 손자 프로세스를 포함합니다.
1회 transient retry(2초부터 지수 backoff+jitter), 429/구독 소진은 cooldown 또는 blocked.
인증 오류·권한 거절·잘못된 flag는 자동 retry 금지. test failure는 새 feedback attempt.
부분 변경이 있는 timeout은 자동 재실행하지 않고 snapshot/diff를 먼저 확인합니다.
저장된 정확한 session ID로 같은 cwd와 compatible CLI version을 사용합니다.
--last/--continue는 병렬 자동화에서 사용하지 않습니다. CLI upgrade 시 fixture contract 검증.
관련: SF-04/06, QAS-03/04.

## DD-08 기억과 wiki

SQLite는 실행·승인의 원본, Markdown wiki는 검토 가능한 지식 원본,
agentmemory는 공유 맥락·보고·lesson의 검색/회상 계층입니다.
worker 세션은 공유하지 않고 `(goal, task, attempt, agent, provider_session)`로 연결합니다.
다음 worker에는 brief·범위·관련 wiki/lesson·검증 증거·실패 피드백만 전달합니다.
단일 agentmemory daemon을 기존 REST/MCP로 재사용합니다. structured slots를 진행 조회용으로
쓸 수 있으나 스케줄러 락/승인 원본은 맡기지 않습니다.
발송: SQLite transaction→outbox→memory_save/lesson 도구. project ID는 실제 session의 canonical ID를
확인해 사용하고 agentId를 생략한 기록은 shared, agentId 포함은 개인 기억으로 처리합니다.
임의 extra 필드는 서버 whitelist로 버려질 수 있어 content envelope와 로컬 매핑을 사용합니다.
재전송은 at-least-once이며 로컬 외부 ID 매핑으로 중복 표시를 억제합니다.
초기 keyless 검색은 BM25 중심; local embeddings는 명시적으로 활성화하고 자원 측정 후 사용합니다.
현재 upstream과 설치 스킬의 default embedding 설명이 달라 실제 고정 버전의 동작을 검증합니다.
agentmemory 장애/라이선스 배포 이슈 시 같은 memory port의 SQLite FTS5+Markdown backend를 사용합니다.
관련: SF-07, QAS-07/10/11.

## DD-09 개발 도구

rtk: 사람이 읽을 빌드/테스트/검색 출력 축약. CLI raw JSON 수집에는 필터를 적용하지 않습니다.
Ponytail: 기존 코드/표준 기능 우선, 필수 검증·에러 처리는 유지하며 과도한 확장 방지.
Graphify: code-only AST/cargo 관계로 ownership·영향 범위 확인. API-key 자동 backend 선택 금지.
CocoIndex Code: ccc grep 구조 검색부터 사용, local embeddings 검색은 필요 시 제한적으로 실행.
둘의 파생 index는 base/branch/hash로 식별하고 세 worktree가 같은 mutable index를 덮어쓰지 않습니다.
상주 모델·인덱서 3개를 띄우지 않고 coordinator가 공유 검색 결과를 배분합니다.
Python sidecar는 개발 보조/선택 기능이며 Rust 실행 코어에 임베딩 엔진을 추가하지 않습니다.
관련: SF-07/09, QAS-08/10.

## DD-10 구조와 UI

```text
src/
  main.rs, lib.rs
  profile.rs, protocol.rs
  adapters/{mod.rs,claude.rs,codex.rs,opencode.rs,custom.rs}
  process.rs, planner.rs, scheduler.rs, state.rs
  git.rs, verify.rs, approval.rs, memory.rs
tests/
  fixtures/{claude,codex,opencode}/
  process_contract.rs, workflow.rs, approval.rs, recovery.rs
  support/mock_agent.rs
migrations/
docs/01..07-*.md
knowledge/{raw/,schema.md,wiki/}
.agentconductor/                 # gitignore; runtime state
  state.sqlite, worktrees/, runs/<run-id>/
    prompt.txt, stdout.jsonl, stderr.log, events.jsonl
    result.json, diff.patch, tests.json
```

MVP UI는 `plan`, `run`, `status`, `logs`, `diff`, `approvals`, `approve`, `reject`, `revise`, `stop`, `resume`, `merge` CLI입니다.
실시간 로그는 tail, diff는 pager로 제공합니다. TUI/Ratatui, 데스크톱/Tauri는 후속.
token/cost는 제공값과 추정값을 구분하며 구독 청구액으로 단정하지 않습니다.
