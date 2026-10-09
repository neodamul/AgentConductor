# CLI 오케스트레이터 OSS 비교

확인일: 2026-10-09 KST. README·실행 어댑터·GitHub default branch commit/API를 확인했습니다.
실제 설치·성능·전체 권한 모델의 E2E 검증은 수행하지 않았습니다.
활동 날짜는 최신 default branch commit의 UTC 날짜이며 pushed_at/크롤링 날짜와 다릅니다.

| 프로젝트 | 라이선스 / 최근 commit | 아키텍처·UI·병렬/격리 | 인증·프로세스 I/O | 수집·재지시·부족한 기능 |
|---|---|---|---|---|
| [conduct](https://github.com/ldlac/conduct) | MIT / 2026-06-21 | TS/Node, Ink TUI, 작업별 worktree/process | 추가 key 불필요; Claude 양방향 stream-json; Codex는 기본 `exec`, OpenCode 기본 `run` | diff·수동 merge·상태 파일·대화 지원. 기본 Codex/OpenCode JSON 수집 아님; DAG/자동 역할 분해/custom registry/공통 승인 보완 필요 |
| [ao](https://github.com/Mats2208/ao) | **미확인**, API license=null, tree에 LICENSE 없음 / 2026-07-08 | Node JS brain/router+workers, worktree, 로컬 웹/PTY | Claude stdin+stream-json; Codex exec JSONL; OpenCode run JSON; 기존 CLI 인증 | 계획·tier routing·session·review/test/merge 흐름. OpenCode `--auto`, read-only도 미강제. 라이선스 불명으로 코드 재사용 제외 |
| [Vibe Kanban](https://github.com/BloopAI/vibe-kanban) | Apache-2.0 / 2026-09-19 | Rust backend+React 웹, worktree, 병렬 executor | CLI 자식 프로세스, 별도 completion key가 기본 필수는 아님. Claude stream-json; Codex app-server JSON-RPC; OpenCode serve+local HTTP | 보드·diff·follow-up·승인 기능이 풍부. 요청한 exec/run stdout 계약과 다르고 코어가 큰 편; 임의 CLI·3작업 자동 DAG·wiki는 확장 검토 필요 |
| [Agetor](https://github.com/alamops/agetor) | MIT / 2026-10-08 | Bun/Electrobun+React native WebView, SQLite/worktree | 기존 harness 인증; Claude/Codex tmux 경로 및 Codex app-server 관련 코드 | 보드·대화·pipeline. 확인 README에 OpenCode 없음; Rust/TUI/headless JSONL 최소 구조와 차이 |
| [agent-deck](https://github.com/asheshgoplani/agent-deck) | MIT / 2026-10-04 | Go TUI, tmux session/worktree 관리 | 이미 인증된 CLI 운영, 주로 terminal 세션 제어 | 즉시 여러 CLI 관리에 적합. 표준 headless JSONL 결과·자동 분해/DAG·검증 근거 승인 파이프라인은 별도 확인·확장 필요 |

실행 소스 근거:
[conduct agents](https://github.com/ldlac/conduct/blob/main/src/core/agents.ts),
[ao Claude](https://github.com/Mats2208/ao/blob/main/src/providers/claude.js),
[ao Codex](https://github.com/Mats2208/ao/blob/main/src/providers/codex.js),
[ao OpenCode](https://github.com/Mats2208/ao/blob/main/src/providers/opencode.js),
[Vibe Codex](https://github.com/BloopAI/vibe-kanban/blob/main/crates/executors/src/executors/codex.rs),
[Vibe OpenCode](https://github.com/BloopAI/vibe-kanban/blob/main/crates/executors/src/executors/opencode.rs).

## 추천

즉시 작업별 CLI/리뷰를 사용하려면 **Vibe Kanban**, 터미널·가벼움을 우선하면 **conduct**를 평가합니다.
conduct는 `pnpm start /path/to/repo`로 별도 작업을 만들고 diff 확인 후 수동 merge를 사용할 수 있습니다.
두 도구 모두 상위 목표를 역할별 3개 작업으로 자동 분해하는 요구 전체를 충족한다고 보장하지 않습니다.

이번 요구는 Rust + 직접 exec/run JSON I/O + custom CLI + 실행 전 승인 + wiki/TDD 추적성입니다.
확인한 후보를 그대로 사용하는 것으로는 부족하므로 **작은 Rust 코어**를 권장합니다.
기존 UI를 처음부터 복제하지 않고 CLI/pager부터 구현합니다. Vibe Kanban의 허가된 Rust 구현은
프로세스/로그 설계 참고 대상으로 삼되 현재 계약과 전이 의존성을 확인하지 않은 코드 복사는 하지 않습니다.
ao는 아이디어 비교 자료로만 사용합니다. 모든 OSS 중 적합한 프로젝트가 없다는 단정은 아닙니다.
