# QAS: Quality Attribute Scenario

아래 수치는 측정 목표이며 아직 실측값이 아닙니다. 테스트 증거로 검증해야 합니다.

| ID / 품질 | 자극원·자극 | 환경·대상 | 응답 | 측정 기준 |
|---|---|---|---|---|
| QAS-01 병렬성 | 사용자가 3개 ready 작업 start | 로컬/스케줄러 | 3 worktree에서 실행, 상한 준수 | mock 3개의 실행 구간 중첩, 동시 프로세스 ≤3 |
| QAS-02 안전성 | 미승인 병합·변경된 승인 후보 | 정상/병합 실행기 | 실행 전 거절 | 보호 ref 변경 0건, 감사 이벤트 존재 |
| QAS-03 복구성 | supervisor 강제 종료 | 실행 중/SQLite·프로세스 | 잔존 프로세스 식별, stale run 복구 | 산출물 유실 0, 중복 run 0, DB 복구 후 5초 내 상태 표시 목표 |
| QAS-04 중단성 | 사용자의 stop | 자식과 손자 프로세스 실행 중 | interrupt→term→kill 후 reap | 총 15초 내 프로세스 트리 종료 목표 |
| QAS-05 파서 신뢰성 | mock CLI가 분할 UTF-8·큰 stderr·알 수 없는 JSON 출력 | 실행 중/I/O | 원문 보존·정규화·유계 메모리 | 이벤트 누락/교착 0; malformed 출력은 진단, 조용히 성공 금지 |
| QAS-06 변경 격리 | agent가 소유 범위 밖 파일 수정 | worktree/검증기 | 범위 위반으로 검토 차단 | 신규·삭제·rename·symlink 탈출 사례 모두 검출 |
| QAS-07 가용성 | agentmemory 서비스 장애 | 작업 실행 중/outbox | SQLite에 보존 후 지연 동기화 | 작업 계속, 보고 유실 0, 중복 전송 시 중복 표시 억제 |
| QAS-08 자원 효율 | 3 mock 작업·10분 이벤트 스트림 | 지정 개발 host/Rust 코어 | log disk spool, 유계 queue | CLI/모델/sidecar 제외 코어 RSS <100MiB, 대기 CPU <1% 목표 |
| QAS-09 확장성 | 새로운 custom CLI 등록 | 정상/프로필 파서 | 동일 실행·검토 흐름 적용 | scheduler 수정 없이 argv+표준 JSONL fixture로 동작 |
| QAS-10 지식 신뢰성 | 출처 변경·상충 lesson | wiki/인덱스 | stale/conflicting 표시 | 근거 없는 정본 승격 0; source hash 변경 검출 |
| QAS-11 배포성 | 의존성/번들 변경 | 릴리스/라이선스 검증 | 허용 정책·고지·SBOM 확인 | 미검토 license/asset 0개일 때만 배포 후보 완료 |

연결: UC-02→QAS-01/05/06/08/09, UC-03→QAS-03/04/07,
UC-04→QAS-02/06, UC-05→QAS-07/10, UC-01→QAS-11.
