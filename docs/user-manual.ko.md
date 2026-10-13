# AgentConductor 사용 설명서

버전 **0.1.16** · [English](user-manual.md) · [프로젝트 소개](../README.md)

## 1. 데스크톱 앱 사용

### 1.1 설치와 실행

macOS 11 이상 Apple silicon 또는 Windows x86_64에서 사용합니다. Windows에는 WebView2 Runtime이 필요합니다.
Git과 이미 로그인된 Claude Code, Codex, OpenCode 또는 사내 CLI를 설치하세요.
프로젝트는 일반 폴더여도 됩니다. Git 저장소와 최초 커밋은 필수가 아닙니다.
앱은 기존 CLI 인증을 사용하며 모델 API 키 등록이나 로컬 UI 웹서버를 요구하지 않습니다.

GitHub Releases에서 운영체제에 맞는 ZIP과 `SHA256SUMS`를 받습니다. 체크섬을 확인하고 압축을 푼 뒤
`AgentConductor.app` 또는 `agentconductor.exe`를 실행합니다.
패키지에는 실행파일, 두 언어 매뉴얼, 실제 화면, 의존성 라이선스 및 SBOM이 포함됩니다.
앱 제목 옆의 작은 숫자가 실행 중인 버전입니다.

### 1.2 워크스페이스와 설정

워크스페이스에는 목표, 계획, 역할 배정과 실행 이력이 저장됩니다. 이름과 목표를 입력하고
**찾아보기…** 또는 직접 입력으로 기존 프로젝트 폴더를 지정한 뒤 **워크스페이스 저장**을 누르세요.
기본 실행 방식은 공유 폴더입니다. Git worktree 격리는 필요할 때 선택합니다.
이름을 저장하면 제목 표시줄도 바로 갱신됩니다.

**워크스페이스 ▾**에서 생성, 열기, 전환, 계획 규칙, 설정 내보내기/가져오기, 메신저를 사용합니다.
Agent Room에서도 같은 제목 표시줄을 사용합니다. 기존 실행이 있으면 적절한 작업 화면이 열리며,
워크스페이스로 돌아오는 것만으로 새 계획이 시작되지는 않습니다.
저장하지 않은 편집 내용이 있으면 이동 전에 선택을 요청합니다.

현재 워크스페이스도 **삭제**할 수 있습니다. 계획 생성 중이면 **계획 중단 후 삭제**가
소유한 프로세스와 이미 시작된 메타데이터 수집의 종료를 기다립니다. 현재 항목을 삭제하면 최근에 사용한 다른 워크스페이스를 엽니다. 남은 항목이 없으면 빈 워크스페이스를 만듭니다.
구현 작업이 실행 중이면 Agent Room에서 **실행 중단**을 먼저 누릅니다.
앱이 관리하는 저장된 워크스페이스만 삭제되며 프로젝트 원본은 보존됩니다.
이력은 `~/.agentconductor/workspace-trash/<id>`에 옮겨져 복구할 수 있습니다.
**최근 목록에서 숨기기**는 데이터 삭제가 아닙니다.

![워크스페이스와 목표 설정](assets/ui-workspace-korean-current.png)

**설정**에서 다음 항목을 관리합니다.

- **화면 언어**: 영어 또는 한국어. 입력한 목표, 지시, 원본 로그와 에이전트 출력은 번역하지 않습니다.
- **MCP 서버 등록…**: OpenCode 형식의 JSON/JSONC로 로컬 명령 또는 원격 서버를 등록합니다.
  인증은 환경변수 참조를 권장합니다. 도구를 조회한 뒤 읽기 전용 Jira/Confluence 가져오기를 연결합니다.
- **신호 전송 방식**: 기본 로컬 Rust 또는 agentmemory 우선과 로컬 폴백.
- **캐시 정리…**: 경로와 크기를 검토한 뒤 일회성 메타데이터를 영구 삭제합니다.
- **에이전트 도구 설치 안내…**: agentmemory, RTK, Ponytail, Graphify, CocoIndex의 선택적 설치 안내입니다.
  앱이 자동 설치하거나 실행하지 않으며 기본 동작에 필수인 도구도 아닙니다.

### 1.3 에이전트 등록과 모델

앱이 관리하는 워크스페이스는 `~/.agentconductor/agents.json`의 공통 CLI 목록을 사용합니다.
워크스페이스를 삭제해도 에이전트 목록은 삭제되지 않습니다. 역할 배정은 워크스페이스별입니다.
카드 드래그 또는 위로/아래로 이동으로 순서를 바꿀 수 있습니다.
없는 에이전트를 가리키는 역할은 강조 표시되며 재배정 전에는 계획을 만들 수 없습니다.

**+ 추가**에서 Claude, Codex, OpenCode 또는 Custom을 선택합니다.
연필은 수정, 쓰레기통은 등록 제거입니다.

- 실행파일은 비동기로 탐색합니다. 실패하면 수정 창의 **찾아보기…**로 파일을 지정합니다.
- 인자는 JSON 배열입니다. 로그인은 해당 CLI에서 관리합니다.
- 실행 권한은 별도 드롭다운이 아니라 CLI 종류에 맞춰 정해집니다.
  표준 Claude/Codex 프로필은 기본적으로 전체 접근 권한으로 실행해 CLI 자체 도구 승인 절차를 생략합니다.
  실행파일을 변경하거나 사내 CLI를 등록하면 이 권한을 자동 승계하지 않습니다.
  Custom/OpenCode의 외부 샌드박스 또는 읽기 전용 구성은 운영자가 검증해야 합니다.
- 프로필마다 최대 3개 세션을 동시에 실행할 수 있습니다. 같은 계정은 같은 쿼터 그룹을 사용합니다.
- 대체 에이전트와 그 모델을 지정합니다. 상호 대체도 가능하지만 이미 시도한 에이전트를 반복하지 않습니다.
- 지원 CLI의 모델 목록은 생성 요청 없이 조회합니다. **Models**로 갱신합니다.
  모델 조회가 없는 사내 CLI에는 운영자가 제공한 ID를 입력하고 실제 모델 인자를 `{model}`에 연결합니다.
- 사용량 갱신은 기본 **5분**, 에이전트별 **1–60분** 범위에서 수정합니다.

**프로필 적용**은 비동기로 저장하고 성공하면 창을 닫습니다. 오류는 창 안에 표시됩니다.
실행 구성을 바꾸면 계획을 다시 검토해야 합니다. 모델/쿼터 관측만 갱신한 경우에는 계획을 버리지 않습니다.
메타데이터 수집은 한 번에 하나씩 수행되며 대기 요청이 다른 편집을 막지 않습니다.

![에이전트 실행파일과 갱신 주기 설정](assets/ui-agent-settings-current.png)

### 1.4 목표, 문서 가져오기와 역할

상단의 **워크스페이스 설정**, **에이전트 역할**, **계획 검토** 탭으로 이동합니다. 탭을 바꿔도 입력은 유지되며 실제 저장은 **워크스페이스 저장** 또는 **계획 수정 저장**을 눌러야 합니다. 계획 생성이 끝나면 검토 탭이 자동 선택됩니다.
목표와 계획 코디네이터를 지정합니다. MCP 서버 등록 후 목표에
`jira:EMM-52599`, `confluence:53553565`처럼 명시적으로 문서 참조를 넣을 수 있습니다.
계획 생성 전에 문서를 미리 보고 선택한 근거를 적용하세요. 서버 등록만으로 명령이 실행되지는 않습니다.
문서나 목표가 바뀌면 다시 검토합니다.

검증, 개발, 문서화, 지식 자산화, 테스트 역할별로 에이전트와 선택적 모델/대체 에이전트를 지정합니다.
**기본값 제안**은 등록된 팀을 사용합니다. 코디네이터는 역할에 맞는 작업을 제안하고 Rust 엔진이 실행과 상태를 관리합니다.

목표에는 짧은 영어 폴더 이름이 붙습니다. 예를 들어 “사다리 게임”은 `ladder_game`이며
한국어 발음을 로마자로 옮기는 방식이 아닙니다. 목표 폴더는 `<프로젝트>/<목표>/` 바로 아래에 만듭니다.
이전 위치의 폴더는 사용 시 이동합니다. 일반 폴더의 사본 이력은 앱이 별도로 관리합니다.
목표 폴더에는 `plan/`, `phase/`, `task/`, `verification/`, `completed/`가 있습니다.
상태 파일을 직접 수정하지 마세요. 앱이 기록의 식별자와 일관성을 검사합니다.

### 1.5 계획 검토와 승인

**계획 생성**은 임시 사본에서 분석과 순서가 있는 단계를 요청합니다.
코디네이터가 시작되기 전에도 준비 메시지와 경과 시간이 표시됩니다.
오류가 나면 원인과 로그 위치를 확인하세요.
**계획 읽기…**는 기존 Markdown 계획을 변환하며 최대 64 KiB, 완료 항목은 제외합니다.
**계획 직접 작성**은 코디네이터를 실행하지 않습니다. 직접 작업과 테스트 명령을 채웁니다.

모든 단계의 제안을 먼저 검토할 수 있습니다. 단계 제목에서 순서와 상태를 확인하고
접힌 작업을 펼쳐 지시, 에이전트/모델, 담당 파일, 의존성, 테스트를 수정하세요.
**계획 수정 저장** 후 현재 단계에 대해 **계획 승인 후 실행**을 누릅니다.
비어 있거나 범위가 겹치는 작업, 바뀐 기준 소스, 오래된 다이제스트는 승인할 수 없습니다.

**목표 및 진행**에는 단계 태그와 설명이 표시됩니다.
단계를 병합한 뒤 다음 단계 계획 버튼은 저장된 다음 단계를 읽으며 코디네이터를 새로 호출하지 않습니다.
기존 단계형 목표의 계획 생성은 현재 단계와 현재 프로필을 불러옵니다.
**계획 규칙…**은 컴퓨터 공통 규칙 또는 워크스페이스 전용 규칙을 편집합니다.

일반 폴더의 계획은 임시 사본에서 만듭니다. 승인한 공유 실행은 선택한 폴더에서 직접 작업하며, Git 메타데이터는 `~/.agentconductor/project-history/`에 저장하고 프로젝트에 `.git`을 만들지 않습니다. 워크트리 모드는 작업을 격리합니다. 공유 실행 중에는 변경이 이미 프로젝트에 보이므로 최종 반영 전 diff를 검토하세요.
커밋된 Git 프로젝트는 깨끗한 기준이 필요합니다. **커밋 후 계속** 전에는 파일 목록을 확인하세요.
운영체제 메타데이터는 로컬 제외로 처리할 수 있으며 `.gitignore`를 바꾸지 않습니다.
일반 폴더의 공유 실행에서는 기존 로컬 의존성을 사용합니다. 워크트리에서는 의존성을 별도로 준비해야 합니다.

### 1.6 Agent Room과 피드백

상태 배너가 실행 상태, 작업/단계 진행률, 실패 원인과 다음 행동을 알려줍니다.
왼쪽 목록은 굵은 상태 글자와 초록/노랑/빨강 표시를 사용합니다.
실행 중에는 깜빡이고 결정 대기 작업도 강조됩니다. 동작 줄이기 설정에서는 애니메이션을 줄입니다.
사용량 배지는 워크스페이스 에이전트 카드에 표시하며 작업 목록에서는 제외합니다.

작업을 선택하면 **활동**, **Diff 및 테스트**, **원본 로그**, **학습**을 볼 수 있습니다.
활동에는 CLI와 유사한 메시지, 도구 실행과 결과 요약이 표시되고 JSON 원문은 원본 로그에 남습니다.
Thinking은 접을 수 있습니다. 출력이 없거나 정체된 경고는 관측 상태이며 반드시 실패를 뜻하지는 않습니다.
작업 뒤의 코디네이터 항목을 선택하면 통합 단계, 도구 결정과 테스트 출력을 볼 수 있습니다.

- **피드백 보내기** 또는 ⌘/Ctrl+Enter는 해당 작업만 수정 지시합니다. 유휴 실행은 기록 후 이어집니다.
- **피드백으로 수정**은 실패 작업을 선택하고 실패 출력에 따른 지시 초안을 만듭니다.
- 작업 검사기에서 에이전트와 관측된 모델을 바꿀 수 있습니다. 기존 파일과 로그는 유지됩니다.
- **재개 시각**은 제한된 세션을 예약합니다. **실행 중단**, **재개**, **복구**로 소유한 실행을 관리합니다.
- **팀 브리핑**은 현재 작업, 최근 활동, 신선도, 제한과 다음 행동을 읽기 전용으로 보여줍니다.

앱을 다시 열면 중단된 실행과 소유 프로세스의 종료를 확인한 뒤 바뀌지 않은 작업을 자동 재개할 수 있습니다. 새 작업 진행이 없으면 최대 두 번 시도하며, 사람이 중단한 실행은 재개하지 않습니다. 소유권이 불명확하면 복구를 멈추고 최종 후보 승인을 해제합니다. 재개하지 못하면 보존한 근거를 확인하세요.

![오프라인 검증 실행의 Agent Room](assets/ui-agent-room-current.png)

### 1.7 도구 결정과 최종 검증

아래 도구 게이트는 `native_callback` 보호 프로필에 적용됩니다. 표준 전체 접근 Claude/Codex 프로필은 CLI 자체 도구 승인을 생략하지만 계획과 최종 후보 승인은 유지합니다.

보호된 작업 공간 안의 일반 읽기·편집·검색·빌드·테스트는 즉시 실행할 수 있습니다.
지원하지 않거나 안전하지 않은 요청은 평가 전에 거절합니다.
나머지 요청은 워크스페이스 코디네이터가 허용/거절을 판단하고 이유를 기록합니다.
유효한 판단을 받지 못하면 사람이 결정합니다. 별도 모델 API 서비스는 만들지 않습니다.

도구 요청 창에서 읽기 쉬운 행동 설명, 원본 입력, 실시간 활동과 마감 시간을 확인합니다.
**한 번 허용**, **거절**, **나중에 결정**으로 사람에게 온 요청을 처리합니다.
승인은 작업, 세션, 계획, 작업 공간, 정확한 입력에 묶이며 최대 900초 후 만료되고 한 번만 소비됩니다.
도구 결정 대기는 작업 제한 시간에서 제외합니다. 타임아웃 작업은 차단되며 재개할 수 있습니다.
도구 승인은 최종 병합 승인이 아닙니다.

**Diff 및 테스트**에서 후보를 확인하세요. 바뀐 검증 보고서에 Blocking/차단 항목이 있으면
해당 작업의 통합 검증은 실패합니다. 문제를 수정하고 다시 검증하세요.
**승인**은 후보 커밋, 기준 소스, diff와 통과한 테스트에 묶입니다.
이후 Git은 **승인된 후보 병합**, 원본 폴더 직접 실행은 **변경 수락**, 이전 사본 실행은 **변경 적용**을 별도로 실행합니다. 직접 실행 후보를 거절하면 이후의 수동 편집을 덮어쓰지 않는 경우에만 담당 파일을 기준 상태로 복원합니다.
근거가 바뀌면 승인은 취소됩니다. 최종 반영은 사람이 승인합니다.

모든 작업 완료 후 통합 검증이 실패하면 등록된 코디네이터가 근거를 검토하고 필요한 작업을 수정해 재실행할 수 있습니다. 최대 세 번의 검토 또는 유효하지 않은 응답 뒤에는 멈춥니다. 이후 사람이 수정하거나 재배정하세요.

### 1.8 사용량 제한과 대체 에이전트

유료 CLI 카드에는 `64%/100% ↻` 같은 남은 사용량을 표시합니다.
Enterprise는 남은 할당량/전체 할당량의 비율이며, 시간 제한은 제공자의 기준(5시간/주간, 일간/주간 등)을 사용합니다.
마우스를 올리면 원래 수치, 초기화 시각, 출처와 조회 시각을 확인할 수 있습니다.
전체 할당량이 없는 크레딧은 비율로 만들지 않습니다. 토큰 수나 예상 비용도 구독 잔여량이 아닙니다.
조회 불가는 `—`, 오래된 관측은 비율을 숨깁니다. 소진된 구간은 재개 가능 시각을 표시합니다.

조회는 비동기이며 출력과 시간이 제한되고 생성 요청을 하지 않습니다.
Claude는 도구·훅·프로젝트 설정을 끈 격리 메타데이터 세션을 사용합니다.
해당 수집 세션에서만 폴더 신뢰 질문에 답할 수 있습니다. 테마/로그인 초기 설정은 CLI에서 완료해야 합니다.
Codex는 기존 ChatGPT 로그인이 필요합니다. 실제 Enterprise 형식과 사내 CLI는 운영자 검증이 필요합니다.

제한은 같은 쿼터 그룹에 적용합니다. 현재 소유자가 종료된 뒤 대체 에이전트가 새 세션으로 이어갑니다.
파일과 로그는 유지합니다. 대체 대상이 모두 제한되면 기다립니다.
가능 시각을 알면 예약하고, 모르면 재개 시각 지정·재배정·모델 변경을 선택하세요.
인증/설정 오류는 자동 재시도하지 않습니다. 이동 후 다시 검증하고 승인해야 합니다.

### 1.9 메모리, 공유와 캐시

로컬 SQLite 체크포인트와 감사 기록이 기준 상태입니다.
협력 프로토콜을 지원하는 사내 CLI는 안전한 경계에서 지시를 조회합니다.
등록만으로 임의의 CLI가 실행 중 steering을 지원하게 되는 것은 아닙니다.

기본 **로컬 Rust**는 agentmemory에 연결하지 않습니다.
**agentmemory 우선**은 외부 전송 시도를 먼저 기록하며, 서버 불가 또는 재시작 후 결과가 불확실하면
같은 로컬 지시를 재전송 없이 유지합니다. 클라이언트는 8초 제한입니다.
메모리 저장 영수증은 수신자의 작업 수락이 아닙니다. 다른 컴퓨터의 에이전트로 전송하는 기능은 아닙니다.

검토한 학습에는 근거와 소스 해시가 있으며 Markdown wiki에도 기록됩니다.
소스가 바뀌면 오래된 학습으로 처리하고 권한을 부여하지 않습니다. 메모리 동기화/가져오기는 선택 사항입니다.

**설정 내보내기…**는 프로필, 역할과 검증된 목표 아카이브를 공유합니다.
시크릿·환경변수 값·로컬 모델 관측은 컴퓨터에 남습니다. 공유 전에 아카이브를 검토하고
가져온 권한 구성은 해당 컴퓨터에서 확인하세요.

**메신저…**는 Slack/webhook과 지정 승인자의 허용/거절 응답을 지원합니다.
인증 값은 로컬에 남으며 목표 본문은 명시적으로 선택한 경우만 보냅니다.
원격 응답은 계획이나 병합을 승인하지 않습니다. 실제 사내 메신저와 인증은 별도 검증하세요.

**캐시 정리…**는 완료된 조회/메타데이터/모델 캐시를 미리 보여줍니다. 영구 삭제는 취소할 수 없습니다.
실행 중·소유권 불명·별칭 데이터, 현재 진단, 프로젝트, 검토 계획, diff, 테스트, 감사와 복구 자료는 보존합니다.
유휴 시작 또는 수집 완료 시 최근 8개와 보호 참조를 유지하며 별도 상시 정리 프로세스는 없습니다.
Cargo 빌드 캐시는 앱 캐시와 별개입니다.

## 2. CLI 참조

`RUN`은 저장소 밖의 실행 디렉터리, `TASK`는 작업 ID, `AGENT`는 등록된 에이전트 ID입니다.
실행 전에 깨끗한 기준 소스를 준비하고 정확한 다이제스트를 검토하세요.

### 2.1 워크스페이스와 에이전트

| 명령 | 기능 |
|---|---|
| `agentconductor` | 기본 워크스페이스를 엽니다. |
| `agentconductor ui-new <workspace-dir>` | 지정한 워크스페이스를 엽니다. |
| `agentconductor ui <run-dir>` | 기존 실행의 Agent Room을 엽니다. |
| `agentconductor profiles <profiles.json>` | 검토할 CLI 프로필 템플릿을 씁니다. |
| `agentconductor models-discover <profiles.json> AGENT <new-dir>` | 생성 없이 CLI 모델 목록을 조회합니다. |
| `agentconductor catalog-import <profiles.json> AGENT <catalog.json>` | 운영자가 관측한 모델 목록을 가져옵니다. |

### 2.2 계획과 실행

| 명령 | 기능 |
|---|---|
| `agentconductor plan <profiles.json> <coordinator> <repo> <new-plan-dir> "<goal>"` | 읽기 전용 코디네이터로 계획을 만듭니다. --from은 Markdown, --rules는 규칙, --phases는 단계형 목표입니다. |
| `agentconductor validate <plan.json>` | 계획을 검증하고 승인 다이제스트를 출력합니다. |
| `agentconductor run <plan.json> <repo> <new-run-dir> <digest> [--phase slug-dir phase-id]` | 정확히 승인한 계획을 실행합니다. --phase는 목표 단계의 이력을 연결합니다. |
| `agentconductor status RUN` | 실행과 작업 상태를 보여줍니다. |
| `agentconductor update` | 최신 GitHub 릴리스를 이 실행 파일 위에 설치합니다(재시작하지 않음). |
| `agentconductor logs RUN TASK` | 해당 작업의 로그를 보여줍니다. |
| `agentconductor progress RUN TASK` | 현재 시도의 정규화된 최근 이벤트를 보여줍니다. |
| `agentconductor diff RUN` | 후보 diff와 테스트 근거를 확인합니다. |
| `agentconductor approve RUN <candidate-sha>` | 정확한 후보 커밋을 승인합니다. |
| `agentconductor reject RUN "<reason>"` | 후보를 이유와 함께 거절합니다. |
| `agentconductor merge RUN` | 승인된 후보를 반영합니다. |

### 2.3 세션·제한·복구

| 명령 | 기능 |
|---|---|
| `agentconductor sessions RUN` | 작업별 세션을 보여줍니다. |
| `agentconductor briefing RUN` | 현재 팀을 읽기 전용으로 브리핑합니다. |
| `agentconductor quota-refresh RUN AGENT` | 등록 에이전트의 계정 사용량을 조회합니다. |
| `agentconductor schedule RUN TASK <YYYY-MM-DDTHH:MM:SS+HH:MM>` | 제한된 세션의 재개 시각을 지정합니다. |
| `agentconductor reassign RUN TASK AGENT [model]` | 작업을 다른 에이전트에 재배정합니다. |
| `agentconductor session-model RUN TASK <model>` | 세션 모델을 관측된 모델로 변경합니다. |
| `agentconductor revise RUN TASK "<feedback>"` | 최대 16 KiB의 작업 피드백을 기록합니다. |
| `agentconductor stop RUN "<reason>"` | 이유와 함께 소유한 실행을 중단하고 파일과 로그를 보존합니다. |
| `agentconductor resume RUN` | 중단한 실행을 명시적으로 재개합니다. |
| `agentconductor resume-due RUN` | 예약 시각에 도달한 작업을 이어갑니다. |
| `agentconductor watch RUN` | 실행 중 재개 시각에 도달한 작업을 확인합니다. |
| `agentconductor recover RUN` | 중단된 코디네이터의 소유 프로세스가 없음을 확인하고 복구합니다. |

### 2.4 도구 요청

| 명령 | 기능 |
|---|---|
| `agentconductor tool-requests RUN` | 도구 승인 요청 목록을 보여줍니다. |
| `agentconductor tool-approve RUN <request-sha256> [seconds]` | 정확한 요청을 한 번 허용합니다. |
| `agentconductor tool-reject RUN <request-sha256>` | 정확한 요청을 거절합니다. |
| `agentconductor tool-refresh RUN <request-sha256>` | 오래된 요청의 현재 근거를 다시 검토합니다. |

### 2.5 리뷰

| 명령 | 기능 |
|---|---|
| `agentconductor review RUN <coordinator>` | 코디네이터에게 제한된 결과 리뷰를 요청합니다. |
| `agentconductor apply-review RUN <proposal-sha256>` | 저장된 리뷰를 적용합니다. 실행파일 도움말의 정확한 형식을 사용하세요. |
| `agentconductor review-apply RUN <coordinator>` | 검토한 리뷰를 적용합니다. 병합 승인은 아닙니다. |

### 2.6 학습과 메모리

| 명령 | 기능 |
|---|---|
| `agentconductor knowledge-digest <lesson.json>` | 학습 초안의 검토 다이제스트를 만듭니다. |
| `agentconductor knowledge-accept RUN <lesson.json> <digest>` | 정확히 검토한 학습을 수락하고 자산화합니다. |
| `agentconductor knowledge-status RUN` | 학습 근거의 상태와 신선도를 보여줍니다. |
| `agentconductor recall RUN "<query>"` | 현재 소스에 유효한 검토 학습을 조회합니다. |
| `agentconductor memory-sync RUN <port>` | 검토 요약을 로컬 agentmemory에 동기화합니다. |
| `agentconductor memory-publish <port> <envelope.json>` | 체크포인트 봉투를 게시합니다. |
| `agentconductor memory-read <port> <memory-id>` | 정확한 메모리 ID 하나를 읽습니다. |
| `agentconductor memory-import RUN <port> <memory-id>` | 검증된 체크포인트 관측을 실행에 가져옵니다. |
| `agentconductor memory-discover RUN <port>` | 작업의 정확한 주소로 체크포인트 관측을 찾고 가져옵니다. |

### 2.7 협력 체크포인트

| 명령 | 기능 |
|---|---|
| `agentconductor checkpoint-issue RUN <issue.json>` | 작업자에게 pause/handoff 지시를 기록합니다. |
| `agentconductor checkpoint-poll RUN <address.json>` | 작업자가 안전한 경계에서 지시를 조회합니다. |
| `agentconductor checkpoint-ack RUN <ack.json>` | 수신한 지시의 상태를 확인 응답합니다. |
| `agentconductor checkpoint-report RUN <progress.json>` | 크기가 제한된 진행 보고를 기록합니다. |

### 2.8 진단

| 명령 | 기능 |
|---|---|
| `agentconductor normalize <claude\|codex\|opencode\|custom> <stdout.jsonl>` | CLI JSONL을 표준 이벤트로 정규화합니다. |

협력 CLI는 `cooperative: checkpoint_v1`과 정확한 `{session_id}` 인자를 사용합니다.
`AC_CHECKPOINT_RUN_DIR`, `AC_CHECKPOINT_ADDRESS_FILE`, `AC_CHECKPOINT_EXECUTABLE`이 전달됩니다.
handoff를 수락·완료하고 프로세스가 종료되면 검토된 대체 에이전트로 이어갑니다. pause는 중단합니다.
학습 초안에는 context/failure/cause/response/evidence/confidence와 소스 path/sha256을 기록합니다.
JSON 필드 이름과 CLI 인자는 번역하지 않습니다. 자세한 형식은 실행파일의 `--help`와 영어 프로토콜 문서를 확인하세요.

## 3. 검증 범위와 문제 해결

패키지 실행파일은 macOS·Windows의 결정적 오프라인 작업자로 검증합니다.
Git/일반 폴더, 공유/worktree, 병렬 실행과 미승인 반영 거절을 검사합니다.
이 검증이 실제 사내 에이전트·Enterprise 계정·메신저까지 검증했다는 뜻은 아닙니다.

이전에 실제로 확인한 CLI 계약은 Claude Code 2.1.287, Codex 작업/재개 0.162.0 및 쿼터 0.162.1,
OpenCode 2.0.25, agentmemory 0.9.29입니다. 설치 버전은 다를 수 있으며 알 수 없는 출력은
조회 불가 또는 안전한 실패로 처리합니다.

- 실행파일 없음/Windows 오류 193: 실제 실행파일 또는 npm .cmd를 지정합니다. 수정 → 찾아보기로 재설정할 수 있습니다.
- 사용량 조회 불가: 툴팁을 보고 CLI 테마/로그인을 완료한 뒤 새로고침합니다. 전체 할당량 없이 비율을 만들지 않습니다.
- 역할 에이전트 없음: 강조된 배정을 등록된 에이전트로 바꿉니다.
- 기준 소스 변경: 파일 목록을 확인해 커밋/스태시하거나 중단하고 다시 계획합니다.
- 차단/실패: 원인과 테스트 출력을 확인하고 피드백으로 수정하거나 설정을 고친 뒤 재개합니다.
- 소유권 불명: 소유한 프로세스의 종료가 확인된 뒤 복구합니다. 다시 열 때 소유 프로세스의 종료가 확인된 실행만 제한적으로 자동 재개합니다. 사람이 중단한 실행은 재개하지 않습니다.
- 승인 근거 변경: 계획 또는 검증을 다시 합니다. 일반 폴더 변경 적용은 백업/영수증을 남기며 중단된 적용은 임의 재실행하지 않습니다.
- MCP 타임아웃: 등록한 명령·서버와 제한 시간을 확인합니다. 원본 로그는 비공개로 관리하세요.
- 창이 이미 열림: 기존 창을 사용하거나 먼저 닫습니다.

관련 문서: [MCP 연결과 문서 가져오기](mcp-sources.md), [영어 사용 설명서](user-manual.md).

## 선택 도구 명령

앱이 설치하지 않습니다. 슬래시 명령은 해당 에이전트 CLI, 나머지는 터미널에서 직접 실행합니다.

- `brew install uv`
- `winget install astral-sh.uv`
- `npx -y @agentmemory/agentmemory@latest`
- `/plugin marketplace add rohitg00/agentmemory`
- `/plugin install agentmemory`
- `codex plugin marketplace add rohitg00/agentmemory`
- `codex plugin add agentmemory@agentmemory`
- `/plugin marketplace add DietrichGebert/ponytail`
- `/plugin install ponytail@ponytail`
- `codex plugin marketplace add DietrichGebert/ponytail`
- `codex plugin add ponytail@ponytail`
- `brew install rtk`
- `winget install rtk-ai.rtk`
- `rtk init -g`
- `rtk init -g --codex`
- `rtk gain`
- `uv tool install graphifyy`
- `graphify install`
- `graphify install --platform codex`
- `graphify update .`
- `uv tool install --upgrade 'cocoindex-code[full]'`
- `/plugin marketplace add cocoindex-io/cocoindex-code`
- `/plugin install cocoindex-code@cocoindex-code`
- `codex mcp add cocoindex-code -- ccc mcp`
- `ccc init`
- `ccc index`
