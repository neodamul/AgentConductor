# System Feature

| ID | 기능 | 목표 | MVP |
|---|---|---|---|
| SF-01 | 등록 가능한 CLI subprocess 프로필·파서 | BG-01 | 포함 |
| SF-02 | 3작업 계획과 의존성 스케줄러 | BG-02 | 포함 |
| SF-03 | worktree 수명주기·파일 범위·통합 후보 | BG-02/03 | 포함 |
| SF-04 | 로그·결과·diff·테스트 증거·resume | BG-03 | 포함 |
| SF-05 | 위험 작업 차단·사람 승인·병합 | BG-03 | 포함; 임의 CLI 도구의 완전한 중계는 제외 |
| SF-06 | SQLite 실행 상태·감사·복구 | BG-03 | 포함 |
| SF-07 | agentmemory와 로컬 wiki·lessons | BG-04 | 최소 연동 포함 |
| SF-08 | CLI 상태판·검토·승인 목록 | BG-03/05 | 포함; Ratatui TUI는 후속 |
| SF-09 | 라이선스/SBOM·TDD 검증 | BG-05 | 개발·릴리스 절차 포함 |

자동 계획은 지정된 coordinator CLI를 한 번 실행해 JSON 계획을 생성하고 Rust에서 검증합니다.
coordinator도 기존 로그인으로 실행합니다. plan 파일 수동 입력도 같은 검증 경로를 사용합니다.
