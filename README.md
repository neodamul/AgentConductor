# AgentConductor

Rust로 만드는 로컬 CLI 에이전트 오케스트레이터의 요구사항·조사·설계 저장소입니다.
현재는 **Rust TDD 구현 중**입니다. 프로필·3작업 계획 검증과 SQLite 실행 상태 기반을 구현했습니다.
전체 MVP 워크플로는 아직 완료되지 않았습니다.

개발 순서: 요구사항 정의 → Business goal → System Feature → Usecase → QAS → DD → 구현.
각 작업은 plan → task → verify → complete로 관리하고, 구현은 TDD로 진행합니다.

- [지식 위키](knowledge/wiki/index.md)
- [요구사항](docs/01-requirements.md)
- [Business goal](docs/02-business-goals.md)
- [System Feature](docs/03-system-features.md)
- [Usecase](docs/04-usecases.md)
- [QAS](docs/05-qas.md)
- [DD](docs/06-design-decisions.md)
- [TDD 구현 작업](docs/07-implementation-plan.md)
- [OSS 조사](knowledge/wiki/oss-comparison.md)
- [CLI 실행 규격](knowledge/wiki/cli-protocols.md)
- [라이선스 검토](knowledge/wiki/license-review.md)

LLM API key/호환 endpoint는 오케스트레이터 설정에 두지 않습니다.
각 CLI는 사용자가 설치하고 인증한 실행 파일을 그대로 사용합니다.

배포는 단일 `agentconductor` 실행 바이너리입니다. SQLite는 정적 포함하고 Rust 런타임·Node·Python을
코어 실행에 요구하지 않습니다. Git과 AI CLI는 기존 설치를 사용하며 운영체제/아키텍처별로 빌드합니다.
agentmemory·Graphify·CocoIndex는 선택 연동/개발 도구이므로 필수 배포물에 번들하지 않습니다.
