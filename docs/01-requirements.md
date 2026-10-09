# 요구사항 정의

기준일: 2026-10-09 KST. 사용자 요청이 기준이며 임의의 추가 제품 기능은 포함하지 않습니다.

| ID | 요구사항 | 우선순위 | 수용 기준 |
|---|---|---|---|
| R-01 | 기존 CLI 인증·구독 사용 | 필수 | Claude/Codex 실행에 오케스트레이터 API key 입력이 없음 |
| R-02 | subprocess I/O 제어 | 필수 | 명령/argv/cwd/env와 stdin, stdout, stderr, exit를 독립 수집 |
| R-03 | 3개 CLI와 임의 CLI 등록 | 필수 | Claude/Codex/OpenCode 프로필 및 같은 계약의 custom 프로필 |
| R-04 | 상위 목표를 정확히 3개 작업으로 분해 | MVP | 입력·범위·의존성·완료 조건을 가진 검증된 계획 |
| R-05 | worktree 병렬 실행 | MVP | 고정 base SHA, 작업별 branch/cwd, 동시 실행 상한 3 |
| R-06 | 결과·diff·테스트 수집 | MVP | 신규 파일을 포함한 변경 목록과 coordinator가 실행한 테스트 근거 |
| R-07 | 사람이 검토·승인 후 병합 | MVP | 승인한 base/candidate/diff/test 해시가 달라지면 병합 차단 |
| R-08 | 위험 작업 승인 | 필수 | 배포·push·외부 전송·시크릿 읽기는 기본 금지, 실행 전 승인 가능한 경로만 허용 |
| R-09 | 중단·resume·재지시·감사 | 필수 | 세션 ID와 attempt 보존, stop 후 산출물 유지, 재시작 복구 |
| R-10 | 공유 기억·lessons·중간보고 | 필수 | agentmemory 기록과 로컬 outbox, 장애 시 로컬 작업 지속 |
| R-11 | llm-wiki 지식자산 | 필수 | raw/schema/wiki, 출처·해시·index·append-only log |
| R-12 | Rust·가벼운 로컬 구조 | 필수 | 단일 실행 바이너리 배포, SQLite 포함, Docker/데스크톱 앱 선택 사항 |
| R-13 | 개발 절차·TDD | 필수 | 요구사항부터 테스트까지 추적, Red/Green 증거 없이 구현 완료 불가 |
| R-14 | 개발 도구 사용 | 필수 | rtk, Ponytail, Graphify, CocoIndex 역할과 사용 근거 기록 |
| R-15 | 사내 배포 라이선스 확인 | 필수 | 고정 버전·전이 의존성·모델·NOTICE를 포함한 배포 검토 |

첫 MVP는 등록→3작업 계획→병렬 worktree→JSON 수집→diff/테스트→사람 병합 승인까지입니다.
Docker, 자체호스트 HTTP transport, 데스크톱 앱, 비용 대시보드, 자동 배포는 후속 범위입니다.
자체호스트 **CLI**는 처음부터 공통 프로필 계약으로 지원하며 특정 벤더에 종속시키지 않습니다.

검증되지 않은 전제: DD는 Design Decision, QAS는 Quality Attribute Scenario로 해석합니다.
과거 학습 자료를 memory에서 검색했으나 이번 순서를 구체화한 기존 템플릿은 확인하지 못했습니다.
따라서 현재 명시한 사용자 순서와 표준 QAS 필드를 기준으로 작성했습니다.
