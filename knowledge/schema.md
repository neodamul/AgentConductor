# 지식자산 규칙

`raw/`는 출처 기록 원본, `wiki/`는 유지·연결하는 지식, 이 파일은 유지 규칙입니다.
패턴 출처: [Karpathy LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f).

1. 원문/관찰을 raw에 저장하고 source URL/path, retrieved_at, hash, revision을 기록합니다.
2. 저작권 있는 웹 자료는 링크·메타데이터·허용된 짧은 발췌만 저장합니다. 전체 복사를 기본으로 하지 않습니다.
3. wiki 페이지는 관련 출처·검증일·사실/설계 제안/미확인을 구분합니다.
4. 새로운 출처는 기존 페이지와 연결하고 상충 주장을 지우지 말고 표시합니다.
5. index.md는 주제 목록, log.md는 append-only 기록입니다. 매 ingest/query 결과 반영 시 갱신합니다.
6. lesson은 상황·실패·원인·대응·증거·confidence·적용 범위·재검증 조건을 가집니다.
7. 검증 없는 agent의 추정은 draft입니다. 성공 보고만으로 accepted로 승격하지 않습니다.
8. source hash/코드 commit이 바뀌면 관련 주장을 stale로 표시합니다.
9. multi-agent 초안은 작업별 경로에 쓰고 정본 반영은 coordinator 한 writer가 수행합니다.
10. 승인/시크릿은 wiki에 저장하지 않습니다. 실행 진행상황은 run 링크·요약만 기록합니다.

자체 관리: Markdown+Git 이력+SQLite FTS5. agentmemory는 검색 가능한 복제와 lesson 회상 계층입니다.
Graphify/CocoIndex의 파생 인덱스는 삭제 후 재생성 가능해야 하며 원본으로 취급하지 않습니다.
