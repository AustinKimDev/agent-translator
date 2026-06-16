# Agent Translator PRD

## 문제

AI agent 도구마다 설정 파일, 지시문 위치, 훅, 보조 문서 참조 방식이 다르다. 사용자는 Claude, Codex, Antigravity, Cursor 등 여러 도구를 병행하면서 같은 정책과 작업 방식을 반복해서 옮겨 적고, 차이를 수동으로 추적해야 한다.

## 목표

Agent Translator는 여러 AI agent 설정을 분석하고, 차이를 설명하며, 다른 도구의 설정 형태로 이전할 수 있게 돕는 Rust 기반 CLI/GUI 도구다.

## 사용자

- 여러 agent 도구를 동시에 쓰는 개인 개발자
- 팀 공통 agent 규칙을 여러 도구로 배포해야 하는 엔지니어링 팀
- 기존 프로젝트의 agent 설정을 점검하고 표준화하려는 리드 개발자

## 핵심 유스케이스

1. 프로젝트 안의 AI agent 설정 파일을 찾아 어떤 도구용인지 식별한다.
2. 설정 본문과 연결된 Markdown 문서를 함께 읽어 분석 입력으로 사용한다.
3. 도구별 설정의 차이를 사용자가 이해할 수 있는 형태로 요약한다.
4. 한 도구의 설정을 다른 도구 설정으로 이전할 초안을 만든다.
5. 훅처럼 특정 도구에서만 지원하는 기능을 감지하고, 이전 가능 여부와 대체 방안을 표시한다.

## 제품 형태

### CLI

초기 구현 대상이다. 자동화, CI, 빠른 로컬 점검에 사용한다.

- `agent-translator scan <path>`: 설정 후보 파일과 연결 문서를 분석한다.
- `agent-translator diff <path>`: 감지된 설정 간 차이를 요약한다.
- `agent-translator migrate <source> <target> <path>`: 이전 초안을 출력한다.

### GUI

후속 구현 대상이다. CLI와 같은 라이브러리 코어를 사용한다.

- 설정 파일 목록
- 도구별 차이 비교
- Migration preview
- 훅/미지원 기능 경고

## MVP 범위

- Rust crate와 CLI 진입점 생성
- 표준 라이브러리만 사용
- 설정 도구 종류 모델링: Claude, Codex, Antigravity, Cursor, Unknown
- 대표 파일명 감지
- 설정 본문에서 hook 관련 기능 플래그를 단어 단위로 감지
- Markdown inline/link-reference 링크 중 로컬 `.md` 링크 추출
- scan 결과를 정규화 모델로 변환
- scan 결과를 텍스트와 JSON으로 출력
- diff 결과에서 도구별 feature mismatch를 텍스트와 JSON으로 출력
- migration은 실제 AI 호출 전 단계인 dry-run plan 출력

## 비범위

- 실제 Codex exec 호출
- GUI 구현
- 외부 crate 기반 Markdown 파서
- 네트워크 링크 크롤링
- 설정 파일 자동 덮어쓰기

## 성공 기준

- `cargo test`가 통과한다.
- `cargo run -- scan <path>`가 대표 agent 설정 파일을 감지한다.
- `cargo run -- scan --json <path>`가 정규화된 agent 배열을 출력한다.
- linked Markdown 파일이 scan 결과에 포함된다.
- `cargo run -- diff <path>`가 감지된 설정 수와 feature mismatch를 출력한다.
- `cargo run -- diff --json <path>`가 feature mismatch 배열을 출력한다.
- `cargo run -- migrate claude codex <path>`가 소스/타깃과 처리 단계를 담은 dry-run plan을 출력한다.
- `cargo run -- migrate claude codex --json <path>`가 dry-run plan을 JSON으로 출력한다.

## 리스크

- 각 도구의 설정 규칙은 빠르게 바뀐다.
- 훅 기능은 도구마다 실행 모델과 보안 경계가 달라 1:1 변환이 불가능할 수 있다.
- Markdown 링크 파싱을 표준 라이브러리로 시작하면 CommonMark 전체 호환은 제공하지 않는다.
- JSON 출력은 외부 crate 없이 수동 formatter로 시작했으므로 schema 확장 시 escaping과 안정성 테스트를 같이 늘려야 한다.
