# int2DDS 실습 강의 자료

int2DDS의 Rust API로 DDS를 실습하는 강의 자료입니다. 강의 슬라이드와 실습 코드가 들어 있습니다.

## 구성

```text
int2DDS-lecture/
├─ int2dds-lecture-for-beginner.pptx   강의 슬라이드 (3일 과정, Rust)
├─ int2dds-lecture.pptx                이전 2일 과정 슬라이드 (C 기준)
├─ dds-lecture-rs/                     실습 코드 (Rust)
│  ├─ Cargo.toml                       실습 프로그램 19개 ([[bin]])
│  ├─ common/                          lib.rs, lab.rs (공통 헬퍼), shutdown.rs (Ctrl+C 종료 처리)
│  ├─ lab1_hello/                      hello.idl, hello.rs (생성), pub.rs, sub.rs
│  ├─ lab2_instance_lifecycle/         floor.idl, floor.rs (생성), pub.rs, sub.rs
│  ├─ lab3_discovery/                  sub.rs
│  ├─ lab4_partition/                  pub.rs, sub.rs
│  ├─ lab5_data_receive/               sub_read.rs, sub_listener.rs, sub_waitset.rs
│  ├─ lab6_qos_compatibility/          pub.rs, sub.rs
│  ├─ lab7_qos_test/                   pub.rs, sub.rs, recipes.rs
│  ├─ lab8_xml/                        sub_xml.rs, sub_profile.rs, sub_config.rs, *.xml
│  └─ board/                           board_pub.rs, board_sub.rs, board.xml, led.py (Raspberry Pi)
```

이전 C 실습 코드(`dds-lecture-src/`)는 삭제했습니다. 필요하면 git 기록에서 꺼낼 수 있습니다.

### 실습 목록

| 실습 | 세션 | 내용 | 폴더 |
|---|---|---|---|
| 1 | 5 · 기본 Pub/Sub 구현 | HelloWorld pub/sub 왕복 | `lab1_hello` |
| 2 | 6 · Topic · Key · Instance | 복합 키 타일의 Instance 생명주기 | `lab2_instance_lifecycle` |
| 3 | 7 · Discovery | 발견된 Participant 확인 | `lab3_discovery` |
| 4 | 8 · Domain과 Partition | Domain · Partition 격리 | `lab4_partition` |
| 5 | 9 · 데이터 소비 | read/take, Listener, WaitSet | `lab5_data_receive` |
| 6 | 10 · QoS 프레임워크 | QoS 불일치와 런타임 변경 | `lab6_qos_compatibility` |
| 7 | 11 · QoS 정책별 동작과 설정 | QoS 레시피 (Late Joiner, 헬스 모니터링, Failover, 데이터율 제한) | `lab7_qos_test` |
| 8 | 12 · XML 활용 | XML로 타입 · QoS · 구성 정의 | `lab8_xml` |

실습은 두 갈래로 이어집니다. HelloWorld(키 없음)는 실습 1 → 3 → 4 → 6 → 7, TilePressure(복합 키)는 실습 2 → 5 → 8입니다.

실습 2 · 4 · 7은 Raspberry Pi에서 LED로도 확인합니다(`board/`, [보드 실습](#보드-실습-raspberry-pi)). 보드에서는 다시 빌드하지 않고 `board.xml`만 고칩니다.

## 준비물

- Windows 10/11 x64. Linux x86_64(WSL Ubuntu)에서도 같은 명령으로 빌드되고 실습 1이 동작하는 것을 확인했습니다. aarch64(보드) 크로스 빌드는 아직 확인하지 않았습니다. 보드 프로그램은 PC에서 `--text`로 확인했습니다.
- Rust 1.89 이상: [rustup](https://rustup.rs)으로 설치합니다. Windows에서는 MSVC 링커가 필요해서 Visual Studio 2022 Build Tools("C++를 사용한 데스크톱 개발" 워크로드)도 설치합니다.
- Git: `Cargo.toml`이 int2DDS를 [GitHub](https://github.com/IntellectusCorp/int2DDS)의 `develop` 브랜치에서 받습니다. 따로 clone할 필요는 없고 첫 빌드 때 cargo가 받아 옵니다(인터넷 필요).
  - 실제로 쓰는 커밋은 `Cargo.lock`에 고정됩니다. 그래서 develop이 바뀌어도 수강생마다 같은 버전으로 빌드됩니다.
  - 최신 develop으로 올릴 때는 `cargo update -p int2dds`를 실행하고, 실습을 다시 확인한 뒤 `Cargo.lock`을 커밋합니다.
- 실습 3 · 4에서 패킷을 볼 때: Wireshark (Npcap loopback 어댑터 포함)

## 빌드

```powershell
cd dds-lecture-rs
cargo build --release
```

- 결과물은 `target\release\`에 생깁니다. `lab*.exe` 17개와 `board_pub.exe` · `board_sub.exe`입니다. 처음 빌드는 의존성까지 컴파일해서 몇 분 걸립니다.
- 하나만 빌드할 때: `cargo build --release --bin lab1_pub`
- 고친 뒤에는 같은 명령을 다시 실행하면 바뀐 것만 다시 빌드합니다.

### IDL에서 Rust 타입 만들기 (세션 5)

`hello.rs`와 `floor.rs`는 `int2dds-idl`로 만든 파일입니다. 이미 들어 있어서 빌드에는 필요 없고, IDL을 고쳤을 때만 다시 만듭니다.

```powershell
# 생성기를 한 번 설치해 둔다 (~\.cargo\bin\int2dds-idl.exe)
cargo install --git https://github.com/IntellectusCorp/int2DDS.git --branch develop int2dds-idl
int2dds-idl -r lab1_hello\hello.rs lab1_hello\hello.idl
int2dds-idl -r lab2_instance_lifecycle\floor.rs lab2_instance_lifecycle\floor.idl
```

인자 없이 `int2dds-idl`만 실행하면 IDL 파일 → 언어 → 출력 폴더를 차례로 고르는 마법사 모드가 됩니다.

생성된 파일은 `#[derive(DdsType)]`을 붙인 구조체입니다. 이 derive가 만든 코드가 `speedy`와 `log` 크레이트를 직접 참조해서 `Cargo.toml`에 두 의존성이 들어 있습니다.

## 실행 공통

### 터미널 준비

PowerShell 창을 두 개 엽니다(왼쪽 sub, 오른쪽 pub). 실습 7 C는 세 개가 필요합니다. 각 창에서 한 번씩 실행합니다.

```powershell
cd dds-lecture-rs
$env:Path = "$PWD\target\release;$env:Path"
lab1_sub
```

- PATH 줄은 exe 이름만으로 실행하려고 넣는 것입니다. `.\target\release\lab1_sub.exe`처럼 경로로 실행해도 됩니다.
- `cargo run --release --bin lab1_sub -- --domain 1`처럼 실행해도 됩니다. `--` 뒤가 프로그램 인자입니다.
- 실습 8의 기본 XML 경로는 빌드할 때 절대 경로로 들어갑니다. 인자로 상대 경로를 줄 때는 `dds-lecture-rs`에서 실행하세요.

### 공통 동작

- 출력 앞의 `[  1234 ms]`는 프로그램 시작 후 경과 시간입니다.
- sub는 모두 Ctrl+C를 누를 때까지 계속 돕니다.
- pub는 정해진 개수를 다 쓰면 스스로 끝납니다. `--hold`를 주면 Ctrl+C까지 남습니다. `lab4_pub`, `lab6_pub`, `lab7_pub`는 `--count` 기본값이 0(무한)입니다.
- 종료는 Ctrl+C입니다. `cleanup`을 찍고 정리한 뒤 끝나고, 정리 중에 멈추면 한 번 더 누르면 즉시 종료됩니다.
- 종료 코드는 `0` 정상, `1` API 실패(`Error: …`로 에러 이름 출력), `2` 인자 오류, `3` `--crash-after`입니다.
- **없는 인자나 잘못된 값을 주면 바로 `error: unexpected argument …`를 찍고 끝납니다.** `--help`로 인자 목록을 볼 수 있습니다.
- **빌드 직후 첫 실행은 몇 초 동안 출력이 멈출 수 있습니다.** Windows Defender가 새 exe를 처음 검사할 때 생기는 것으로 보이고, 두 번째 실행부터는 없어집니다. 수업 전에 한 번씩 실행해 두거나 `target\` 폴더를 Defender 검사 예외에 넣습니다.

### 공통 인자

- `--domain N` (기본 0): 모든 프로그램에 있습니다. 단 `lab8_sub_config`에는 없고 `app.xml`의 `domain_id="0"`으로 정해집니다.
- `--topic NAME` (기본 `hello_world_topic`): 실습 1, 3, 4, 6, 7 프로그램에 있습니다. 실습 2와 5는 `TilePressure`로 고정입니다.

### 여럿이 함께 실습할 때

같은 네트워크에서 모두 도메인 0을 쓰면 서로의 pub/sub가 붙습니다. 좌석 번호를 도메인으로 쓰고 모든 명령에 `--domain N`을 붙이세요. `lab8_sub_config`만 예외라서, 각자 `lab8_xml\app.xml`의 `domain_id`를 고치고 짝이 되는 `lab2_pub`에도 같은 `--domain`을 줍니다.

Domain ID는 0~232만 됩니다(포트 공식 7400 + 250·d). Windows는 예약된 포트 범위가 있어서 큰 Domain ID에서 `bind … os error 10013`이 날 수 있습니다. 좌석 번호처럼 작은 값을 쓰세요.

### 실습별 프로그램 짝

| 실습 | sub | pub |
|---|---|---|
| 1 | `lab1_sub` | `lab1_pub` |
| 2 | `lab2_sub` | `lab2_pub` |
| 3 | `lab3_sub` | `lab1_pub` |
| 4 | `lab4_sub` | `lab4_pub` |
| 5 | `lab5_sub_read` / `lab5_sub_listener` / `lab5_sub_waitset` | `lab2_pub --hold` |
| 6 | `lab6_sub` | `lab6_pub` |
| 7 | `lab7_sub` | `lab7_pub` (C는 두 개) |
| 8 | `lab8_sub_xml` / `lab8_sub_profile` / `lab8_sub_config` | `lab2_pub` |

## 실습별 실행

기본적으로 sub를 먼저 띄웁니다. **실습 7 A와 실습 8의 2 · 3단계만 pub이 먼저**입니다(늦게 참여하는 쪽을 보는 실험).

기대 출력은 Windows 11에서 int2DDS 0.1.8(develop, 커밋 `60c5b3a8`)로 실행한 결과입니다. 현재 `Cargo.lock`의 커밋 `aed6bb83`에서는 실습 1 · 2 · 4 · 6(변경 불가) · 7 C · 8 1'을 다시 실행해 같은 결과를 확인했습니다. **(미확인)** 표시는 직접 실행해 보지 않은, 코드 기준의 예상 동작입니다.

### 실습 1 · HelloWorld

**`lab1_pub`**

| 인자 | 기본 | 뜻 |
|---|---|---|
| `--count N` | 10 | 쓸 개수 |
| `--period-ms N` | 500 | 간격(ms) |
| `--no-wait` | 끔 | 매칭을 기다리지 않고 바로 write |
| `--hold` | 끔 | 다 쓴 뒤에도 Ctrl+C까지 유지 |
| `--crash-after S` | 0 | S초 뒤 정리 없이 종료 (exit 3) |

- 기본 동작: `pub ready` 뒤 Reader가 붙을 때까지 기다렸다가 10개를 쓰고 끝납니다(약 5초).
- **`lab1_sub`**는 `--domain`, `--topic`만 받습니다.

| 케이스 | sub | pub | 기대 출력 |
|---|---|---|---|
| 기본 | `lab1_sub` | `lab1_pub` | pub: `publication_matched: current_count 0 -> 1 (total 1)` → `write index=1..10` → `cleanup`<br>sub: `recv index=1..10 message=hello` |
| 도메인 다름 | `lab1_sub` | `lab1_pub --domain 1` | pub은 `pub ready: domain=1`에서 멈추고 sub에는 아무것도 오지 않음. 에러도 없음 |
| 토픽 다름 | `lab1_sub --topic hello_world_topic2` | `lab1_pub --no-wait` | sub에 아무것도 오지 않음 |
| 매칭 안 기다림 | `lab1_sub` | `lab1_pub --no-wait --period-ms 300` | sub가 `recv index=2`쯤부터 받음 (매칭 전에 쓴 것은 버려짐) |
| sub 둘 | `lab1_sub` 두 개 | `lab1_pub` | 둘 다 1..10을 받음 |

### 실습 2 · Instance 생명주기 (Active Floor)

타일 4개 (1,1) (1,2) (2,1) (2,2)가 각각 하나의 Instance입니다(키: `row`, `col`).

**`lab2_pub`**

| 인자 | 기본 | 뜻 |
|---|---|---|
| `--cycles N` | 5 | N주기째에 (1,1) dispose, (1,2) unregister, 2N주기 뒤 종료 |
| `--period-ms N` | 500 | 주기(ms) |
| `--autodispose 0 또는 1` | 1 | unregister 시 자동 dispose |
| `--events` | 끔 | StepEvent 토픽도 발행 |
| `--no-wait` | 끔 | 매칭을 기다리지 않음 |
| `--hold` | 끔 | 남은 (2,1) (2,2)를 Ctrl+C까지 계속 씀 |
| `--crash-after S` | 0 | S초 뒤 정리 없이 종료 |
| `--profile Lib::Prof` | 없음 | Writer QoS를 XML 프로파일로 (실습 8). 이때 `--autodispose`는 무시됨 |
| `--profile-file F` | `lab8_xml/team_qos.xml` | 프로파일 파일 |

**`lab2_sub`**는 `--domain`, `--events`(StepEvent도 구독)를 받습니다.

| 케이스 | sub | pub | 기대 출력 |
|---|---|---|---|
| 기본 | `lab2_sub` | `lab2_pub` | 처음 `view=NEW inst=ALIVE` 4줄 → 이후 `NOT_NEW` → 5주기 뒤 `tile(1,1) -> NOT_ALIVE_DISPOSED (valid_data=false)`, `tile(1,2) -> NOT_ALIVE_DISPOSED` → pub 종료 직후 `tile(2,1)`, `tile(2,2) -> NOT_ALIVE_NO_WRITERS` |
| autodispose 끔 | `lab2_sub` | `lab2_pub --autodispose 0` | `tile(1,2) -> NOT_ALIVE_NO_WRITERS` |
| 정상 종료 (미확인) | `lab2_sub` | `lab2_pub --hold` 몇 초 뒤 Ctrl+C | 즉시 (2,1) (2,2)가 `NOT_ALIVE_NO_WRITERS` |
| 강제 종료 (미확인, 약 100초) | `lab2_sub` | `lab2_pub --hold --crash-after 5` | pub `simulated crash: exit without cleanup` 후 약 100초 뒤 `NOT_ALIVE_NO_WRITERS` |
| 도전 | `lab2_sub --events` | `lab2_pub --events` | `EVENT step on tile(r,c) ts=…` |

### 실습 3 · Discovery

- **`lab3_sub`**(`--domain`, `--topic`)는 `lab1_sub`에 1초마다 `participants=N [핸들]  matched writers=M` 출력을 더한 것입니다.
- pub은 `lab1_pub`를 씁니다.
- Wireshark는 Npcap loopback 어댑터를 캡처하고 표시 필터에 `rtps`를 넣습니다.

| 케이스 | sub | pub | 기대 출력 |
|---|---|---|---|
| 기본 | `lab3_sub` | `lab1_pub --count 3` | `participants=0 []` → `participants=1 [xxxxxxxx ]  matched writers=1` → `recv index=1..3` → pub 종료 1~2초 뒤 `participants=0` |
| 도메인 다름 (미확인) | `lab3_sub` | `lab1_pub --domain 1` | `participants=0` 유지 |
| 강제 종료 (약 100초) | `lab3_sub` | `lab1_pub --hold --crash-after 5` | pub이 죽은 뒤에도 `participants=1` 유지 (10초 넘게 확인), 약 100초 뒤 목록에서 빠짐 (미확인) |

### 실습 4 · Domain과 Partition

**`lab4_pub`**

| 인자 | 기본 | 뜻 |
|---|---|---|
| `--partition P[,P2]` | 없음 (`""`) | Publisher 파티션 |
| `--count N` | 0 (무한) | 쓸 개수 |
| `--period-ms N` | 500 | 간격(ms) |

**`lab4_sub`** (1초마다 `participants=N  matched writers=M` 출력)

| 인자 | 기본 | 뜻 |
|---|---|---|
| `--partition P[,…]` | 없음 | 와일드카드 `floor-*` 가능 |
| `--switch-after S` | 0 | S초 뒤 파티션 변경 |
| `--switch-to P[,…]` | 없음 | 바꿀 파티션 |

| 케이스 | sub | pub | 기대 출력 |
|---|---|---|---|
| 도메인 불일치 (미확인) | `lab4_sub --domain 1` | `lab4_pub --domain 0` | `participants=0` 유지 |
| 파티션 불일치 | `lab4_sub` | `lab4_pub --partition floor-1` | `participants=1  matched writers=0` 유지 |
| 런타임 변경 | `lab4_sub --switch-after 5 --switch-to floor-*` | `lab4_pub --partition floor-1` | 5초에 `partition switched to (floor-*)` → `recv index=…` → `matched writers=1` |
| 하나만 겹침 | `lab4_sub --partition floor-2` | `lab4_pub --partition floor-1,floor-2` | 즉시 매칭, recv |

- pub은 무한이라 Ctrl+C로 끕니다.
- PowerShell에서 `floor-1,floor-2`와 `floor-*`는 따옴표 없이 그대로 넘어갑니다.
- 런타임 변경은 `subscriber.set_qos()` 한 번이면 Reader가 새 파티션으로 다시 광고됩니다.

### 실습 5 · 데이터 수신 방식

세 경우 모두 pub은 `lab2_pub --hold`이고, 끝낼 때는 양쪽 다 Ctrl+C입니다.

| 프로그램 | 인자 | 동작 |
|---|---|---|
| `lab5_sub_read` | `--domain` | KEEP_LAST(8). 1초마다 read(NOT_READ) / 다시 read(NOT_READ) / read(READ) / take(ANY) / read(ANY)의 5단계 |
| `lab5_sub_listener` | `--domain`, `--slow-ms N` (0) | 콜백 방식. `--slow-ms`는 콜백 안에서 N ms 잠듦 |
| `lab5_sub_waitset` | `--domain`, `--timeout-ms N` (1000) | WaitSet + GuardCondition (Ctrl+C 시 깨움) |

| 케이스 | sub | 기대 출력 |
|---|---|---|
| 폴링 · 마스크 | `lab5_sub_read` | `1 read(NOT_READ) … sample_state=NOT_READ` 전부 → `2 read(NOT_READ) again NO_DATA` → `3 read(READ)` 첫 샘플 `READ` → `4 take(ANY)` 전부 → `5 read(ANY) NO_DATA` → `----` |
| Listener | `lab5_sub_listener` | `on_subscription_matched: current_count=1 (+1) total=1` → `on_data_available #n: tile(r,c) p=…`, pub 종료 시 `(-1)` |
| Listener 지연 | `lab5_sub_listener --slow-ms 1000` | 콜백마다 `callback sleeping 1000 ms (receive thread blocked)`, 1초에 하나씩 처리되며 점점 밀림 |
| WaitSet | `lab5_sub_waitset`를 **먼저** 띄우고 2~3초 뒤 pub | pub 전에는 매초 `wait: TIMEOUT` → `tile(r,c) p=…` → Ctrl+C 시 루프를 빠져나와 `cleanup` |

### 실습 6 · QoS 호환성 (일부러 안 맞추기)

양쪽 다 1초마다 `matched=… incompatible_qos total=… last=…`를 찍습니다.

**`lab6_pub`**

| 인자 | 기본 | 뜻 |
|---|---|---|
| `--reliability reliable/best_effort` | reliable | |
| `--deadline-ms N` | 0 (무한) | 약속하는 DEADLINE |
| `--ownership shared/exclusive` | shared | |
| `--strength N` | 0 | OWNERSHIP_STRENGTH |
| `--fix-deadline-after S` | 0 | S초 뒤 DEADLINE을 set_qos로 변경 |
| `--fix-deadline-ms N` | 0 | 바꿀 값 |
| `--try-immutable` | 끔 | 3초 뒤 RELIABILITY 변경 시도 |
| `--period-ms N` | 500 | 간격(ms) |
| `--count N` | 0 (무한) | 쓸 개수 |

- **`lab6_sub`**는 `--reliability`, `--deadline-ms`(요구값), `--ownership`을 받습니다.
- `best_effort`, `exclusive`가 아닌 값은 모두 기본값으로 처리됩니다.

| 케이스 | sub | pub | 기대 출력 |
|---|---|---|---|
| RELIABILITY | `lab6_sub --reliability reliable` | `lab6_pub --reliability best_effort` | 양쪽 `…_INCOMPATIBLE_QOS: last_policy=RELIABILITY` (total=2까지), `matched=0` |
| DEADLINE | `lab6_sub --deadline-ms 200` | `lab6_pub --period-ms 100 --deadline-ms 300` | `last_policy=DEADLINE`, `matched=0` |
| OWNERSHIP | `lab6_sub --ownership exclusive` | `lab6_pub --ownership shared` | `last_policy=OWNERSHIP`, `matched=0` |
| 런타임 수정 | `lab6_sub --deadline-ms 200` | `lab6_pub --period-ms 100 --deadline-ms 300 --fix-deadline-after 5 --fix-deadline-ms 200` | 5초에 `set_qos(deadline=200 ms) -> Ok(())` → 양쪽 `*_MATCHED: current_count=1 (+1)` → `recv index=…` |
| 변경 불가 | (sub 없어도 됨) | `lab6_pub --try-immutable` | 3초에 `set_qos(reliability change) -> Err(ImmutablePolicy)` |

DEADLINE을 쓰는 케이스에서는 pub의 쓰기 주기(`--period-ms`)를 DEADLINE보다 짧게 줍니다. 길면 매칭된 뒤에 DEADLINE_MISSED가 계속 찍힙니다.

### 실습 7 · QoS 레시피

| `--recipe` | QoS |
|---|---|
| `none` (기본) | 기본 QoS (VOLATILE, KEEP_LAST 1) |
| `hard_state` | RELIABLE + TRANSIENT_LOCAL + HISTORY (`--depth` / `--keep-all`) |
| `health` | DEADLINE (`--deadline-ms`) + LIVELINESS MANUAL_BY_TOPIC (`--lease-ms`) |
| `failover` | EXCLUSIVE + strength + LIVELINESS AUTOMATIC (`--lease-ms`) |

pub과 sub에는 같은 `--recipe`를 줍니다.

**`lab7_pub`**

| 인자 | 기본 | 뜻 |
|---|---|---|
| `--recipe` | none | |
| `--name X` | A | sub에서 `from=X`로 보임 |
| `--count N` | 0 (무한) | |
| `--period-ms N` | 500 | 100 미만이면 write 로그 생략 |
| `--depth N` | 1 | |
| `--keep-all` | 끔 | |
| `--deadline-ms N` | 500 | |
| `--lease-ms N` | 2000 | |
| `--strength N` | 100 | |
| `--pause-after S` | 0 | S초 뒤 write 멈춤 |
| `--assert` | 끔 | 멈춘 동안 assert_liveliness |
| `--hold` | 끔 | |
| `--crash-after S` | 0 | |

**`lab7_sub`**

| 인자 | 기본 | 뜻 |
|---|---|---|
| `--recipe` | none | |
| `--depth N` | 1 | |
| `--keep-all` | 끔 | |
| `--deadline-ms N` | 1000 | 요구값 |
| `--lease-ms N` | 2000 | |
| `--tbf-ms N` | 0 | TIME_BASED_FILTER |
| `--rate` | 끔 | 1초마다 `rate: N samples/s` |
| `--gaps` | 끔 | index가 건너뛰면 `GAP: expected n got m` |

**A · Late Joiner** (pub 먼저, 다 쓴 뒤 약 5초 지나서 sub. 끝나면 둘 다 Ctrl+C)

| pub (먼저) | sub (나중) | 기대 출력 |
|---|---|---|
| `lab7_pub --recipe hard_state --depth 10 --count 10 --hold` | `lab7_sub --recipe hard_state --depth 10` | `recv index=1..10 from=A` |
| `lab7_pub --recipe hard_state --depth 1 --count 10 --hold` | `lab7_sub --recipe hard_state --depth 1` | `recv index=10`만 |
| `lab7_pub --count 10 --hold` | `lab7_sub` | 매칭은 되지만 recv 없음 (VOLATILE) |

**B · 헬스 모니터링** (sub 먼저)

| sub | pub | 기대 출력 |
|---|---|---|
| `lab7_sub --recipe health --deadline-ms 1000 --lease-ms 2000` | `lab7_pub --recipe health --deadline-ms 500 --lease-ms 2000 --pause-after 5 --period-ms 400` | 5초에 pub `paused: no more write`<br>마지막 write 약 1초 뒤 sub `REQUESTED_DEADLINE_MISSED: total=1` (이후 매초)<br>약 2초 뒤 `LIVELINESS_CHANGED: alive=0 (-1) not_alive=1 (+1)`, `NOT_ALIVE_NO_WRITERS (valid_data=false)`<br>pub에는 0.5초마다 `OFFERED_DEADLINE_MISSED`와 `LIVELINESS_LOST: total=1` |
| (같음) | 위 pub 명령 + `--assert` | DEADLINE_MISSED만 반복되고 LIVELINESS는 유지 |

`--period-ms 400`을 주는 이유: 기본 500ms면 쓰기 주기가 약속한 DEADLINE 500ms와 같아서, 멈추기 전에도 pub에 `OFFERED_DEADLINE_MISSED`가 매 주기 찍힙니다.

**C · Failover** (창 셋, sub → A → B 순서)

| 창 | 명령 | 기대 출력 |
|---|---|---|
| sub | `lab7_sub --recipe failover --lease-ms 1000` | `from=A`만 보임 |
| A | `lab7_pub --recipe failover --strength 100 --name A --lease-ms 1000` | |
| B | `lab7_pub --recipe failover --strength 50 --name B --lease-ms 1000` | |
| A에서 Ctrl+C (미확인) | | 곧바로 `from=B` |
| A에 `--crash-after 4`를 붙여 띄운 경우 | | A가 죽고 약 1초 뒤 `LIVELINESS_CHANGED: alive=1 (-1) not_alive=1 (+1)`와 함께 `from=B` |
| A 재시작 | | `from=A`로 복귀 |

**D · 데이터율 제한** (sub 먼저)

| sub | pub | 기대 출력 |
|---|---|---|
| `lab7_sub --tbf-ms 100 --rate` | `lab7_pub --period-ms 10` (Ctrl+C로 끔) | `rate: 9~10 samples/s` (pub은 초당 100개를 씀) |
| `lab7_sub --gaps` | `lab7_pub --period-ms 1 --count 300` | `GAP: expected 6 got 9` 같은 줄이 여러 개 |

- `--period-ms 0 --count 1000`처럼 주면 매칭 전에 다 쓰고 끝나서 수신이 0입니다.
- `--period-ms 0`만 주고(무한) Ctrl+C로 끊어도 GAP이 보이지만, 3초에 500줄 넘게 쏟아집니다.

### 실습 8 · XML

pub은 `lab2_pub`입니다.

| XML 파일 (`lab8_xml/`) | 내용 |
|---|---|
| `tile.xml` | TilePressure 타입 |
| `tile_lab.xml` | 같은 타입을 `lab` 모듈 안에 둔 것 (`lab::TilePressure`) |
| `team_qos.xml` | `Team::HardState` QoS 프로파일 |
| `app.xml` | 타입, QoS, 도메인 0, 트리 `Apps::Monitor` / `sub::tile` |

| 프로그램 | 인자 (기본) |
|---|---|
| `lab8_sub_xml` | `--xml` (tile.xml), `--type` (TilePressure), `--topic` (TilePressure), `--domain` |
| `lab8_sub_profile` | `--profile-file` (team_qos.xml), `--xml` (tile.xml), `--domain` |
| `lab8_sub_config` | `--config` (app.xml), `--reader` (sub::tile). **`--domain` 없음** |

| 단계 | 순서 | sub | pub | 기대 출력 |
|---|---|---|---|---|
| 1 · 타입만 XML | sub → pub | `lab8_sub_xml` | `lab2_pub` | `matched writers=1`, `tile(r,c) p=… view=NEW inst=ALIVE` |
| 1' · 타입 이름만 다름 | sub → pub | `lab8_sub_xml --xml lab8_xml/tile_lab.xml --type lab::TilePressure` | `lab2_pub` | **매칭되고 데이터가 옴.** 양쪽이 TypeObject를 보내면 이름이 아니라 구조(할당 가능성)로 매칭하기 때문 |
| 2 · QoS도 XML | **pub 먼저**, 약 3초 뒤 sub | `lab8_sub_profile` | `lab2_pub --profile Team::HardState --no-wait --hold` | `effective reader qos: reliability=Reliable durability=TransientLocal history=KeepLast(1)` 뒤 최신값이 바로 옴 |
| 3 · 구성 전체를 XML | **pub 먼저** | `lab8_sub_config` | 위와 같음 (도메인 0) | `sub ready (config): reader=sub::tile reliability=Reliable durability=TransientLocal`, `matched writers=1`, 타일 값 |
| 도전 · 필드 이름 다름 | sub → pub | `tile.xml` 복사본에서 `row`를 `rwo`로 고치고 `lab8_sub_xml --xml <복사본>` | `lab2_pub --no-wait` | `matched writers=0` 유지 (`pressure`를 `float64`로 바꿔도 같음) |

- 2 · 3단계에서 pub은 약 2.5초에 (1,1)을 dispose하고 (1,2)를 unregister합니다. 그래서 늦게 붙은 sub에는 `NOT_ALIVE_DISPOSED (valid_data=false)` 두 줄과 **(2,1), (2,2)의 값만** 옵니다.
- 4개를 다 보려면 pub에 `--cycles 100`을 더해 dispose를 50초 뒤로 미룹니다.

### 보드 실습 (Raspberry Pi)

DDS와 QoS는 Rust 프로그램(`board_pub`, `board_sub`)이 맡고, Python(`led.py`)은 `board_sub`가 내보내는 `led N on 밝기|off|blink|fast` 줄을 GPIO LED로 옮기기만 합니다. 타입 · QoS · Partition · Domain은 모두 `board.xml`에 있어서, **동작을 바꿀 때는 `board.xml`만 고치고 다시 실행**합니다. Pi에서는 빌드하지 않습니다.

| 역할 | 프로그램 | `board.xml` 안의 이름 |
|---|---|---|
| Sensor (타일 4개를 씀) | `board_pub` | `Board::Sensor` / `pub::tile` |
| Display (받은 상태를 LED로) | `board_sub \| python3 led.py` | `Board::Display` / `sub::tile` |

**준비 (처음 한 번)**

1. PC(WSL Ubuntu)에서 aarch64용으로 빌드합니다. 아직 확인하지 않은 절차입니다.
   ```bash
   rustup target add aarch64-unknown-linux-gnu
   sudo apt install -y gcc-aarch64-linux-gnu
   export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
   export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
   cargo build --release --target aarch64-unknown-linux-gnu --bin board_pub --bin board_sub
   ```
2. 각 Pi의 `~/lab`에 네 파일을 복사합니다: `target/aarch64-unknown-linux-gnu/release/board_pub`, `board_sub`, `board/board.xml`, `board/led.py`.
3. Display Pi에 `gpiozero`가 없으면 `sudo apt install -y python3-gpiozero`.
4. 팀마다 `board.xml`의 `domain_id`를 하나 정하고, 팀의 Pi는 모두 같은 값으로 둡니다. 같은 Wi-Fi에서 다른 팀 LED와 섞이지 않게 하는 유일한 장치입니다(보드 프로그램에는 `--domain`이 없습니다).

**배선 (Display Pi)**: LED 0 · 1 · 2 · 3 ← BCM 17 · 27 · 22 · 23, LED마다 330Ω 저항을 직렬로 달고 반대쪽은 GND. 타일 (1,1)(1,2)(2,1)(2,2)가 LED 0 · 1 · 2 · 3입니다. 다른 핀은 `led.py --pins a,b,c,d`.

| LED | 뜻 |
|---|---|
| 켜짐 (밝기 = pressure / 100) | ALIVE |
| 꺼짐 | NOT_ALIVE_DISPOSED |
| 느린 깜빡 | NOT_ALIVE_NO_WRITERS (Writer가 떠남 · LIVELINESS 만료) |
| 빠른 깜빡 | REQUESTED_DEADLINE_MISSED (갱신이 끊김) |

| 프로그램 | 인자 (기본) |
|---|---|
| `board_pub` | `--config` (board.xml), `--participant` (Board::Sensor), `--writer` (pub::tile), `--tiles` (11,12,21,22), `--pressure` (80), `--period-ms` (500), `--count` (0 = 계속), `--dispose-after` S, `--crash-after` S |
| `board_sub` | `--config` (board.xml), `--participant` (Board::Display), `--reader` (sub::tile) |
| `led.py` | `--pins` (17,27,22,23), `--text` (GPIO 없이 글자로) |

보드 없이 PC에서 연습할 때는 `dds-lecture-rs`에서 아래처럼 실행합니다. 아래 기대 출력은 모두 PC에서 이 방법으로 확인했습니다.

```powershell
cmd /c "board_sub --config board/board.xml | python board/led.py --text"   # Display
board_pub --config board/board.xml                                          # Sensor (다른 터미널)
```

- Windows PowerShell 5.1은 프로그램끼리의 파이프 출력을 앞 프로그램이 끝날 때까지 모아 둡니다. 그래서 `cmd /c`로 감쌉니다(cmd · bash에서는 바로 보입니다).
- 실습 7 · 통합 실습처럼 XML을 고칠 때는 `board/board.xml`을 복사해 고치고 `--config`로 그 파일을 줍니다. Sensor를 둘 띄울 때는 파일도 둘입니다.

**실습 2 · 보드** (`board.xml` 그대로, Display 먼저)

| Sensor | 기대 LED |
|---|---|
| `./board_pub` | 4개 켜짐 0.80 |
| `./board_pub --pressure 30` | 0.30 |
| `./board_pub --dispose-after 5` | 5초 뒤 꺼짐 |
| 실행 중 Ctrl+C | 바로 느린 깜빡 |
| 실행 중 전원 뽑기 (`--crash-after 5`로 대신 가능) | 약 100초 동안 켜진 채, 그 뒤 느린 깜빡 (Participant lease) |

**실습 4 · 보드**

- Display의 `<subscriber_qos>` 안 `<element>`를 `floor-2`로 → `matched writers=0`. `floor-*`로 → 붙음. 와일드카드는 Display(Subscriber) 쪽에만 씁니다. int2DDS는 Publisher 쪽 와일드카드로 `*` 하나만 받습니다.
- 한쪽 Pi의 `domain_id`만 1로 → 서로 보이지 않음.

**실습 7 · 보드** (`base_name`의 `Recipe::Basic`을 바꿈)

| 단계 | 고칠 곳 | 실행 | 기대 LED |
|---|---|---|---|
| A · Late Joiner | 두 Pi 모두 `Recipe::HardState` | Sensor `./board_pub --count 1`, 5초 뒤 Display | 4개 켜짐 (Basic이면 꺼진 채) |
| B · 헬스 | 두 Pi 모두 `Recipe::Health` | Display 먼저, Sensor `./board_pub --count 6` → 이어서 Sensor 전원 뽑기 | 쓰기가 멈추면 빠른 깜빡, 전원을 뽑으면 2초 뒤 느린 깜빡 |
| C · Failover | 모두 `Recipe::Failover`, Sensor B만 `<ownership_strength><value>50</value>` | Pi 1: `./board_pub --pressure 100`, Pi 2: Display와 `./board_pub --pressure 30` → Pi 1 전원 뽑기 | 1초 뒤 1.00 → 0.30 |
| D · 불일치 | Display의 `datareader_qos`만 `Recipe::HardState` | Display, Sensor | sub 로그 `REQUESTED_INCOMPATIBLE_QOS: last_policy=DURABILITY` |

적용된 값은 시작 줄 `pub ready (board.xml): …`, `sub ready (board.xml): …`에서 확인합니다.

**통합 실습 (선택) · 정답 예시**: `Recipe`에 아래 프로파일을 더하고, 모든 Pi의 `base_name`을 `Recipe::Field`로 바꿉니다(Sensor B만 strength 50).

```xml
<qos_profile name="Field">
  <datawriter_qos>
    <reliability><kind>RELIABLE_RELIABILITY_QOS</kind></reliability>
    <durability><kind>TRANSIENT_LOCAL_DURABILITY_QOS</kind></durability>
    <history><kind>KEEP_LAST_HISTORY_QOS</kind><depth>1</depth></history>
    <deadline><period><sec>0</sec><nanosec>500000000</nanosec></period></deadline>
    <liveliness><kind>AUTOMATIC_LIVELINESS_QOS</kind><lease_duration><sec>1</sec><nanosec>0</nanosec></lease_duration></liveliness>
    <ownership><kind>EXCLUSIVE_OWNERSHIP_QOS</kind></ownership>
  </datawriter_qos>
  <datareader_qos>
    <reliability><kind>RELIABLE_RELIABILITY_QOS</kind></reliability>
    <durability><kind>TRANSIENT_LOCAL_DURABILITY_QOS</kind></durability>
    <history><kind>KEEP_LAST_HISTORY_QOS</kind><depth>1</depth></history>
    <deadline><period><sec>1</sec><nanosec>0</nanosec></period></deadline>
    <liveliness><kind>AUTOMATIC_LIVELINESS_QOS</kind><lease_duration><sec>1</sec><nanosec>0</nanosec></lease_duration></liveliness>
    <ownership><kind>EXCLUSIVE_OWNERSHIP_QOS</kind></ownership>
  </datareader_qos>
</qos_profile>
```

Sensor A `./board_pub --pressure 100`, Sensor B `./board_pub --pressure 30 --count 40`, 몇 초 뒤 Display를 켭니다. 바로 1.00 → A 전원을 뽑으면 0.30 → B가 다 쓰고 멈추면 빠른 깜빡 → B 전원을 뽑으면 느린 깜빡.

- 정책은 `Field` 안에 모두 적습니다. 빠뜨린 정책은 기본값이 됩니다(Reader 기본 reliability는 BEST_EFFORT).
- `Field`의 `datawriter_qos`에 `base_name="Recipe::HardState"`를 달아 상속하면 int2DDS develop에서는 적은 정책까지 모두 기본값이 됩니다. 쓰지 않습니다.

## 문제 해결

| 증상 | 원인 | 조치 |
|---|---|---|
| `failed to load source for dependency int2dds` | GitHub에 접속하지 못함 (프록시 · 방화벽) | 인터넷 연결 확인. 사내망이면 `CARGO_NET_GIT_FETCH_WITH_CLI=true`로 시스템 git 설정을 쓰게 함 |
| `link.exe not found` | MSVC 링커 없음 | Visual Studio 2022 Build Tools 설치 |
| `lab1_sub`를 인식하지 못함 | 그 창에서 PATH 줄을 치지 않음 | PATH 줄 실행 또는 `.\target\release\lab1_sub.exe` |
| 빌드 직후 첫 실행이 몇 초 멈춤 | Windows Defender 첫 실행 검사(추정) | 한 번 더 실행하거나 `target\`을 검사 예외에 추가 |
| `bind … os error 10013` | Domain ID가 커서 Windows 예약 포트에 걸림 | 작은 Domain ID 사용 |
| 아무것도 안 옴 | 도메인, 토픽, 타입, QoS, 파티션 중 하나가 다름 | `lab3_sub`로 참가자 확인 → `lab6_sub`처럼 incompatible_qos 확인 |
| pub이 `pub ready`에서 멈춤 | `lab1_pub`, `lab2_pub`는 매칭을 기다림 | sub를 띄우거나 `--no-wait` |
| 실습 8에서 exit 1 | 상대 경로 `--xml`을 주고 다른 폴더에서 실행 | `dds-lecture-rs`에서 실행 |
| 강제 종료 뒤 NO_WRITERS가 안 옴 | 기본 lease가 100초 | 기다리거나 실습 7 B · C로 대체 |
| 로그 두 줄이 한 줄에 섞임 | 콜백 스레드와 메인 스레드가 동시에 출력 | 표시 문제일 뿐 동작과는 무관 |
| 다른 PC와 통신이 안 됨 | Windows 방화벽이 UDP를 막음 | 방화벽에서 실습 exe의 UDP 허용 |
| Pi끼리(Wi-Fi) 서로 안 보임 | 공유기가 멀티캐스트를 막거나 AP(클라이언트) 격리가 켜짐 | 격리 끄기 · 같은 공유기 사용. 멀티캐스트만 막혔으면 `INT2DDS_INITIAL_PEERS=상대IP`로 SPDP를 유니캐스트로도 보냄(포트 생략 = 7400, domain d면 `상대IP:7400+250×d`). 코드를 보고 쓴 것이고 아직 확인하지 않음. 격리가 켜져 있으면 이것으로도 안 됨 |
| 다른 팀 LED가 같이 움직임 | 팀끼리 `board.xml`의 `domain_id`가 같음 | 팀마다 다른 `domain_id` |

## 라이선스

int2DDS는 Apache License 2.0을 따릅니다.
