# 사내 배포 라이선스 검토

확인일: 2026-10-09 KST. 프로젝트 최상위 라이선스와 일부 부속 고지를 확인한 **예비 검토**입니다.
전체 전이 의존성/배포본/모델을 검사한 결과가 아니며 배포 적합성을 확정하지 않습니다.
개발 보조 실행, 코드 복사, 제품 번들 배포를 각각 구분합니다.

| 구성요소 | 확인 결과 | 현재 취급 |
|---|---|---|
| conduct / Agetor / agent-deck | MIT | 코드/번들 사용 시 해당 버전 license·copyright 보존; 전이 의존성 별도 |
| Vibe Kanban | Apache-2.0 | 수정·고지·NOTICE 확인 후 참고/재사용 후보 |
| ao | API license=null, 조회 tree에 LICENSE 없음 | 코드 복사·재배포 제외; 허가 확인 전 채택 불가 |
| [agentmemory](https://github.com/rohitg00/agentmemory/blob/main/LICENSE) | Apache-2.0 | 선택 외부 서비스; 엔진도 함께 확인 |
| [iii](https://github.com/iii-hq/iii/blob/main/LICENSE.spdx) | `Apache-2.0 AND Elastic-2.0` | permissive-only로 분류 금지; 사내 법무/OSS 정책 검토 대상 |
| [RTK](https://github.com/rtk-ai/rtk/blob/develop/LICENSE) | Apache-2.0 | 개발 보조; 기본 제품 번들 제외 |
| [Ponytail](https://github.com/DietrichGebert/ponytail/blob/main/LICENSE) | MIT | 스킬을 복제 배포하면 고지 포함; 현재 공식 스킬을 읽어 적용 |
| [Graphify](https://github.com/Graphify-Labs/graphify/blob/v8/LICENSE) | Apache-2.0, 기존 MIT 기여분 존재 | [NOTICE](https://github.com/Graphify-Labs/graphify/blob/v8/NOTICE)와 LICENSE-MIT도 보존 대상 검토 |
| [CocoIndex Code](https://github.com/cocoindex-io/cocoindex-code/blob/main/LICENSE) / [CocoIndex](https://github.com/cocoindex-io/cocoindex/blob/main/LICENSE) | Apache-2.0 | 개발 보조; Python 패키지·embedding 모델의 조건은 별도 검증 |
| Claude Code / Codex / OpenCode | CLI별 라이선스와 서비스 이용 조건 별개 | 사용자가 설치한 CLI 실행; 인증 파일/CLI 바이너리를 제품에 번들하지 않음 |

agentmemory upstream install은 iii/v0.22.1을 고정합니다. 해당 tag를 GitHub API로 풀어 확인한
commit `e7de3820d1e558f3762edf95e4440552444d48d3`의 LICENSE.spdx에도 같은 AND 선언이 있습니다.
SPDX 문서의 DataLicense CC0-1.0은 metadata의 라이선스이며 엔진 자체의 permissive 허가가 아닙니다.
Elastic 조건을 포함한 전체 파일 범위·서비스 제한은 실제 배포 형태와 정확한 원문으로 검토해야 합니다.

MVP 배포 기본 구성은 Rust 코어와 로컬 상태/wiki입니다. agentmemory는 기존 로컬 서비스에 연결할 수
있게 두고 iii를 자동 다운로드/번들하지 않습니다. 선택 설치가 라이선스 검토를 면제하지는 않습니다.
사내 정책상 허용되지 않으면 SQLite FTS5+Markdown 자체 memory backend를 사용합니다.

## 릴리스 완료 게이트

- Cargo.lock과 feature/platform별 resolved dependency, npm/Python sidecar 및 binary/model revision 고정.
- 모든 direct/transitive component의 SPDX·copyright·license 원문과 SBOM 생성.
- unknown/custom/GPL/AGPL/Elastic 등은 자동 허용하지 않고 사내 정책 검토.
- MIT/Apache도 license/NOTICE/변경 고지 요구를 실제 원문 기준으로 적용.
- Graphify 기존 MIT 부분, local embedding 모델 가중치, 설치 스크립트와 CLI 서비스 약관 별도 검사.
- 번들 목록과 검토 목록이 일치할 때만 배포 후보를 complete로 표시.

아직 Cargo.lock과 실행 배포본이 없으므로 전이 의존성 검증은 **미수행**입니다.
