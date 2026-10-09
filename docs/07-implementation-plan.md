# 구현 계획과 TDD

현재 수행 범위: Rust TDD 구현 중. T-01 검증 기반 완료, T-02 SQLite 상태 기반 구현/검증 중.

각 작업: **plan**(연결된 R/BG/SF/UC/QAS/DD·수용 조건) → **task**(Red→Green→Refactor)
→ **verify**(반례·통합·리뷰) → **complete**(증거·보고·lesson 기록).
QAS는 품질 목표, 테스트는 이를 검증하는 수단입니다. 둘을 같은 문서로 대체하지 않습니다.

| 작업 | 선행 | DD | 먼저 실패시킬 테스트 | 완료 증거 |
|---|---|---|---|---|
| T-01 프로필·계획 스키마 | 설계 문서 | 01/03 | 3작업 외 수, DAG cycle, 겹친 scope, argv injection | 스키마/경계 테스트 |
| T-02 SQLite 상태·감사 | T-01 | 02 | 불법 전이, 중복 event, stale lease 복구 | 재시작 테스트 |
| T-03 subprocess supervisor | T-01 | 04/07 | stdin EOF, stderr flood, UTF-8 분할, stop 손자 종료 | mock process contract |
| T-04 세 CLI parser/resume | T-03 | 03/04/07 | raw fixture→정규화, 오류 종료, 정확한 session 선택 | version 고정 fixture; 실 CLI smoke 별도 |
| T-05 worktree/scheduler | T-02/03 | 05 | 3개 실제 병렬, base 고정, 의존성 block, 범위 위반 | 임시 Git repo 통합 |
| T-06 diff/test/승인/병합 | T-04/05 | 05/06 | untracked 포함, 실패 테스트, 무승인/변경 후보 병합 거부 | 보호 branch 전후 SHA |
| T-07 memory/wiki | T-02 | 08/09 | 서비스 장애 시 outbox, 중복 전송, source stale 검출 | offline fallback/출처 검증 |
| T-08 CLI E2E/릴리스 | T-01~07 | 10 | 등록→3작업→검증→승인→병합, 거절/재지시 | QAS 실측·license/SBOM |

검증 명령(구현 이후): `rtk cargo test`, `rtk cargo clippy --all-targets -- -D warnings`,
`rtk cargo fmt --check`, lockfile 기반 `cargo deny check licenses`와 SBOM 생성.
유료 모델 호출은 unit test에 사용하지 않고 mock executable과 캡처 fixture로 재현합니다.
실제 CLI smoke는 기존 인증을 사용하는 별도 integration check로 기록합니다.
완료 조건: tests passed + 통합 후보 테스트 + 사람 승인과 정확한 SHA 일치.

추적 예시: R-07 → BG-03 → SF-05 → UC-04 → QAS-02 → DD-06 → T-06 → approval_changed_head_rejected.
전체 대응은 각 문서의 ID 연결을 사용하며 코드 구현 시 테스트명·증거 파일을 추가합니다.

## 배포 조건

단일 실행 바이너리로 배포합니다. rusqlite의 bundled feature로 SQLite를 포함하며 코어에
Python/Node sidecar를 필수로 두지 않습니다. release build에 profile 최적화를 적용합니다.
OS/architecture별 산출물, 파일 checksum, dependency/license manifest를 릴리스 근거로 관리합니다.
