# Usecase

## UC-01 등록 및 준비

행위자: 사용자. 관련 기능: SF-01, SF-06, SF-09.
전제: Git 저장소와 사용자가 설치·인증한 CLI.
흐름: command/argv 프로필 등록 → 버전/help 확인 → 권한 capability 확인 → prepared.
예외: 실행 파일/인증 없음은 blocked; 알 수 없는 플래그나 파서는 실행 전에 실패.
인증 파일 내용은 수집하거나 로그에 기록하지 않습니다.

## UC-02 목표 계획 및 실행

행위자: 사용자/coordinator. 관련 기능: SF-02~04.
흐름: 목표 입력 → 관련 wiki/lessons 조회 → coordinator read-only 계획 → 3작업 스키마/DAG/범위 검증
→ 계획 검토 → worktree 생성 → ready 작업을 최대 3개 병렬 start → JSON 이벤트/로그 수집.
예외: 순환 의존성·겹친 소유 파일·경로 탈출은 차단; 실패 작업의 종속 작업은 blocked.

## UC-03 stop, 중간보고, 재지시

행위자: 사용자/coordinator. 관련 기능: SF-04/06/07.
흐름: 진행 이벤트와 현재 단계 표시 → stop 요청 → 프로세스 트리 종료 확인 → stopped 산출물 보존
→ diff/실패 증거 확인 → 새 attempt로 특정 세션 ID resume.
중간보고는 SQLite event로 우선 저장한 뒤 agentmemory outbox로 전달합니다.
로그에 출력이 없다는 이유만으로 작업을 실패시키지 않습니다.

## UC-04 검증 및 사람 병합

행위자: coordinator/사용자. 관련 기능: SF-03~06.
흐름: 작업 범위 검사 → 후보 commit 생성 → 전용 통합 worktree에 순서대로 적용
→ 통합 테스트 → 정확한 후보 diff와 근거 표시 → waiting_approval
→ approve/reject/revise → base/candidate/test hash 재확인 → 보호 branch에 승인된 후보 반영.
예외: 충돌은 해당 담당자에게 재작업; 승인 이후 변화는 승인 무효.
로컬 후보 commit 생성은 최종 branch 병합과 구분합니다. 자동 push는 없습니다.

## UC-05 지식자산 축적

행위자: coordinator/사용자. 관련 기능: SF-07.
흐름: 원본 수집 → 출처/내용 해시 기록 → wiki 초안 생성 → 근거·상충 주장 표시
→ index와 log 갱신 → 검토된 페이지와 lesson을 기억 검색 계층에 동기화.
예외: agentmemory 장애는 outbox 재시도; 오래된 출처는 stale 표시 후 재검증.
여러 작업의 wiki 초안은 작업별 경로에 작성하고 정본 반영은 한 writer가 담당합니다.
