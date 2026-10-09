# AgentConductor

Rust로 만드는 로컬 CLI 에이전트 오케스트레이터의 요구사항·조사·설계 저장소입니다.
현재는 **설계 단계**이며 실행 가능한 MVP는 아직 없습니다.

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
