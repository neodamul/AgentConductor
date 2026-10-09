# 지식·개발 진행 기록

## [2026-10-09] plan | 요구사항과 OSS 조사

CLI subprocess·기존 인증·worktree·사람 승인 요구 조사.
Rust/TDD, agentmemory, llm-wiki, RTK/Ponytail/Graphify/CocoIndex 요구 추가 반영.
기존 OSS 비교와 정확한 headless/resume 계약을 [프로토콜](cli-protocols.md)에 정리.

## [2026-10-09] task | 설계 문서 작성

요구사항 → BG → SF → UC → QAS → DD → TDD 구현 계획을 연결.
이 항목은 문서 작성 task이며 제품 구현 task의 시작을 의미하지 않습니다.
초기 구현은 단일 Rust 코어·SQLite·CLI/pager, memory와 indexing은 선택 sidecar.

## [2026-10-09] verify | 라이선스와 계약 확인

ao 라이선스 미확인으로 재사용 제외. agentmemory Apache-2.0과 iii의
Apache-2.0 AND Elastic-2.0을 구분. conduct의 Codex/OpenCode 기본 text 출력,
Vibe Kanban의 app-server/serve 경로와 요청한 exec/run 계약 차이 기록.
제품 실행·TDD test·성능·전이 의존성 배포 검증은 아직 수행하지 않음.

## [2026-10-09] complete | 조사·설계 산출물

Markdown 15개, 내부 링크 오류 0, 요구사항부터 구현 계획까지 단계 문서 7/7 확인.
조사·설계 단계 완료이며 T-01~T-08 구현 완료를 뜻하지 않습니다.
agentmemory에 결과·판단 근거·남은 검증을 공유 기억으로 기록했습니다.
