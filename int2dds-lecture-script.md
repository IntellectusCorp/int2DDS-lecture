# int2DDS 실습 강의 진행 시나리오 · 발표 스크립트

기준 자료는 `docs/int2dds-lecture.pptx`(69장)와 `docs/dds-lecture-src/labs`(실습 코드, README 기대 결과는 2026-09-14 Windows 실측)이다. 이 문서는 강사용이다. 슬라이드에 없는 말, 터미널에 칠 명령, 나올 출력, 던질 질문을 세션 순서대로 적었다.

## 0. 읽는 법

| 표기 | 뜻 |
|---|---|
| **S12** | 덱의 12번째 슬라이드(물리 순서). 푸터 쪽번호는 아직 리터럴이라 어긋나 있으니 쓰지 않는다 |
| **말** | 강사가 하는 말. 그대로 읽어도 되게 썼다 |
| **시연** | 강사가 프로젝터에서 실행한다 |
| **실습** | 수강생이 따라 실행한다 |
| **질문** | 수강생에게 던지는 질문과 답 |
| **정정** | 덱과 실제 동작이 다른 곳. 덱을 고치기 전까지 구두로 바로잡는다 |

터미널은 왼쪽 열 = sub, 오른쪽 열 = pub로 고정한다. 명령은 `labs` 폴더에서 다음을 한 번 실행한 뒤의 형태다.

```powershell
$env:Path = "$PWD\build\Release;$env:Path"
```

출력 앞의 `[  1234 ms]`는 프로그램 시작 후 경과 시간이다. 모든 프로그램은 Ctrl+C로 정리 후 종료한다. `--domain`, `--topic`, `--count`, `--period-ms`, `--hold`, `--crash-after`, `--no-wait`는 공통 인자다(부록 C).

## 1. 전체 구성

### 1.1 시간표(안)

덱에는 세션별 시간이 없다. 아래는 실습 4단계(1.2)를 기준으로 잡은 제안이다.

| Day | 세션 | 슬라이드 | 이론 | 실습 | 실습 프로그램 |
|---|---|---|---|---|---|
| 1 | 0 · 환경 준비 | S1–S3 | 30분 | – | `lab1_pub` `lab1_sub` |
| 1 | 1 · DDS의 기본 | S4–S10 | 40분 | – | – |
| 1 | 2 · 기본 Pub/Sub 구현 | S11–S19 | 40분 | 30분 · 실습 1 | `lab1_*` |
| 1 | 3 · Topic · Key · Instance | S20–S27 | 35분 | 35분 · 실습 2 | `lab2_*` |
| 1 | 4 · Discovery | S28–S30 | 20분 | 30분 · 실습 3 | `lab3_sub` + `lab1_pub` |
| 1 | 5 · Domain과 Partition | S31–S35 | 20분 | 25분 · 실습 4 | `lab4_*` |
| 1 | 6 · 데이터 소비 | S36–S41 | 35분 | 35분 · 실습 5 | `lab5_sub_*` + `lab2_pub` |
| 2 | 복습 · Q&A | – | 15분 | – | – |
| 2 | 7 · QoS 프레임워크 | S42–S50 | 45분 | 30분 · 실습 6 | `lab6_*` |
| 2 | 8 · QoS 정책별 동작과 설정 | S51–S60 | 50분 | 60분 · 실습 7 | `lab7_*` |
| 2 | 9 · XML 활용 | S61–S69 | 40분 | 40분 · 실습 8 | `lab9_sub_*` + `lab2_pub` |
| 2 | 마무리 | (장표 추가 예정) | 20분 | – | – |

Day 1은 375분, Day 2는 300분(휴식·점심 별도). Day 2에 한 시간쯤 여유가 있다. 자유 실험 시간이나 Day 1 실습 재시도로 쓴다.

두 실습 계보를 첫날 첫 시간에 말해 둔다. HelloWorld(키 없음)는 실습 1 → 3 → 4 → 6 → 7, TilePressure(복합 키)는 실습 2 → 5 → 8이다. 실습 3부터는 새 코드를 짜지 않고 앞 실습의 pub/sub에 인자와 QoS만 얹는다.

### 1.2 실습 진행 4단계

1. **시연(5분)** 강사가 기본 실행을 한 번 한다. 출력을 한 줄씩 읽어 주며 어디를 볼지 짚는다.
2. **따라하기(5–10분)** 수강생이 같은 명령을 친다. 전원이 기대 출력을 봤는지 손들기로 확인한다.
3. **변형(10–15분)** 인자만 바꿔 다시 실행한다. 변형마다 실행 전에 "무엇이 달라질까"를 먼저 묻고, 예측과 결과를 비교한다.
4. **확인 포인트(5분)** 실습 슬라이드 오른쪽 열을 하나씩 답한다. 이 문서의 **질문** 항목이 그 답이다.

코드는 처음부터 짜지 않는다. 읽고, 실행하고, 인자를 바꾸고, 시간이 남으면 한 줄 고쳐 재빌드한다.

```powershell
cmake --build build --config Release --target lab1_pub
```

### 1.3 강의장 세팅

- **강사 PC** PowerShell 두 개를 프로젝터에 좌우로 스냅한다(왼쪽 sub, 오른쪽 pub). 폰트는 20pt 이상. 세 번째 창은 100초짜리 실험(실습 2·3·7)을 미리 띄워 두는 용도다. 코드는 VS Code 밝은 테마로 보여 준다. Wireshark는 세션 4부터 쓴다.
- **수강생 PC** Windows 11, Visual Studio 2022 Build Tools(C++ 데스크톱), CMake 3.10 이상. S3의 "직접빌드 or dll제공" 스티커는 결정이 필요하다. 권장은 SDK 패키지 제공이다. 헤더 두 개(`int2dds-ffi.h`, `int2dds_cdr.h`), `int2dds_ffi.dll`, `int2dds_ffi.dll.lib`, `int2dds-idl.exe`를 한 폴더로 나눠 주고 다음처럼 구성한다. Rust 설치와 리포 빌드는 원하는 사람만 한다.

```powershell
cmake -S . -B build -DINT2DDS_INCLUDE_DIR=<sdk>\include -DINT2DDS_LIB_DIR=<sdk>\lib -DINT2DDS_IDL=<sdk>\bin\int2dds-idl.exe
cmake --build build --config Release
```

- **네트워크** 같은 LAN에서 전원이 도메인 0을 쓰면 수강생끼리 pub/sub가 서로 붙는다. 세 가지 중 하나를 정한다.
  - (A) 좌석 번호를 도메인으로 쓴다. 실습 1–7은 모든 명령에 `--domain N`을 붙이고, 실습 8은 `app.xml`의 `domain_id`를 각자 고친다. 권장.
  - (B) 강의장 Wi-Fi가 클라이언트 격리(AP isolation)면 도메인 0 그대로 쓴다. 멀티캐스트가 막히는지 사전에 확인한다.
  - (C) 일부러 도메인 0을 공유하고, 세션 4에서 "참가자가 30개 보인다"를 데모로 쓴다. 나머지 세션은 혼란이 커서 권하지 않는다.
  - 이 문서의 명령은 도메인 0 기준이다.

### 1.4 D-1 체크리스트

- 강사 PC에서 `cargo build --release -p int2dds-ffi -p int2dds-idl` 후 labs를 다시 빌드하고, `lab1_sub`/`lab1_pub` 왕복을 확인한다.
- Wireshark에 Npcap loopback 어댑터가 보이는지 확인하고, 표시 필터 `rtps`를 저장해 둔다.
- 방화벽 팝업을 미리 허용한다. 실행 경로에 한글·공백이 없는지 본다.
- 100초가 걸리는 실험(실습 2 `--crash-after`, 실습 3 `--crash-after`)을 한 번 돌려 시간을 몸으로 익힌다.
- 덱의 "실습내용 확인 전" 스티커 8장(S19·S27·S30·S35·S41·S50·S60·S69)을 지우고, 부록 B의 정정 항목을 반영할지 정한다.
- 수강생 배포물: SDK 패키지 + `labs` 폴더(build 제외) + README. 사전 빌드 안내를 보낸다.

## 2. Day 1

### 세션 0 · 환경 준비 (S1–S3, 30분)

**S1 표지** — **말** "이틀 동안 int2DDS로 DDS를 배웁니다. 슬라이드는 짧게, 터미널은 길게 갑니다. 오늘 만드는 프로그램은 두 개뿐이고, 나머지 이틀은 그 두 개에 인자와 QoS를 얹어 가며 동작이 어떻게 바뀌는지 보는 시간입니다."

**S2 실습 일정** — **말** "왼쪽이 세션, 오른쪽이 실습입니다. 실습 1의 HelloWorld가 3·4·6·7로, 실습 2의 TilePressure가 5·8로 이어집니다. 실습 3부터는 새 코드가 아니라 앞 실습의 pub/sub를 그대로 씁니다. 오늘 오후에 배운 걸 내일 다시 쓰니, 오늘 실습에서 막히면 바로 손을 들어 주세요."

**S3 실습 환경 준비** — **시연** labs 폴더에서 PowerShell 두 개를 열고 `$env:Path` 한 줄을 친 뒤 왼쪽 `lab1_sub`, 오른쪽 `lab1_pub`. 왼쪽에 `recv index=1..10 message=hello`가 찍히면 통과다.

**실습** 전원이 같은 것을 한다. 실패 유형별 대응은 부록 A. 자주 나오는 셋만 여기 적는다.

| 증상 | 원인 | 처치 |
|---|---|---|
| exe가 뜨자마자 죽음 | `int2dds_ffi.dll`을 못 찾음 | `build\Release`에 dll이 있는지, `$env:Path`를 그 창에서 쳤는지 |
| 둘 다 떠 있는데 `recv`가 없음 | 방화벽 팝업을 거부했거나 경로에 한글 | 인바운드 UDP 허용, ASCII 경로로 이동 |
| pub가 `pub ready` 뒤 멈춤 | 매칭 대기 중(정상) | sub가 살아 있는지 확인 |

**말** "CMake가 세 가지를 대신 했습니다. `int2dds_ffi.dll.lib`로 링크하고, dll을 실행 파일 옆에 복사하고, IDL에서 C 헤더를 만들었습니다. 세션 2에서 그 헤더를 직접 열어 봅니다."

### 세션 1 · DDS의 기본 (S4–S10, 40분, 이론)

**S5 DCPS와 RTPS의 위치** — **말** "여러분이 부르는 함수는 전부 위쪽 DCPS입니다. Participant, Topic, Writer, Reader, QoS. 아래 RTPS는 그 호출을 패킷으로 바꾸는 층이고, 이틀 동안 RTPS를 직접 만지는 순간은 Wireshark로 들여다볼 때뿐입니다. 그래도 RTPS를 알아야 하는 이유는 하나입니다. 데이터가 안 올 때 어느 층에서 막혔는지 알아야 고칩니다."

**S6 DCPS 계층** — **말** "Domain은 정수 하나입니다. 같은 번호끼리만 서로 보입니다. 그 안에 Participant가 들어가고, Participant가 Publisher와 Subscriber를 만들고, 그 안에 Writer와 Reader가 있습니다. Writer와 Reader는 직접 연결되지 않습니다. Topic이라는 이름으로만 만납니다. 실습 1에서 이 순서를 코드로 그대로 봅니다."

**S7 DCPS Entity의 역할** — **말** "표의 세 번째 열을 보세요. Domain이 다르면 에러가 아니라 그냥 안 보입니다. Topic은 이름과 타입이 같아야 합니다. Partition은 Publisher와 Subscriber에만 붙습니다. QoS는 Writer가 제공하고 Reader가 요구합니다. 이 네 줄이 내일까지 나올 '안 붙는 이유'의 전부입니다." 하단 콜아웃을 짚는다. "정리는 두 줄입니다. `delete_contained_entities` 다음 `delete_participant`. 실습 코드 `common/shutdown.h`의 `cleanup_participant`가 이겁니다."

**S8 RTPS 계층** — **말** "샘플 하나의 여정입니다. write를 부르면 Writer History Cache에 들어가고, DATA 서브메시지로 나가서 Reader History Cache에 쌓입니다. read/take는 거기서 꺼냅니다. 가운데 HEARTBEAT와 ACKNACK은 RELIABLE일 때 빠진 번호를 되찾는 대화입니다. 밑의 Locator는 주소와 포트인데, 이걸 서로 어떻게 알아내느냐가 세션 4 Discovery입니다."

**S9 RTPS Submessage의 역할** — **말** "외울 필요 없습니다. 세션 4에서 Wireshark를 켜면 DATA(p), DATA(w), DATA(r), HEARTBEAT, ACKNACK이 순서대로 지나갑니다. 그때 이 표로 돌아옵니다." Wireshark 라벨이 표기 그대로라는 점을 미리 말한다.

**S10 RTPS 메세지** — **정정** 현재 비어 있다(글자 "ㅌ" 하나). 채운다면 Wireshark 캡처 한 장(Header + INFO_TS + DATA 해부)이 맞고, 비워 둘 거면 발표에서 건너뛴다.

### 세션 2 · 기본 Pub/Sub 구현 (S11–S19, 이론 40분 + 실습 30분)

**S12 기본 Pub/Sub 구현 과정** — **말** "네 단계입니다. IDL을 쓰고, 코드를 생성하고, 애플리케이션을 쓰고, 두 터미널에서 돌립니다. 타입을 바꾸면 2번부터 다시 합니다. 실습 코드는 CMake가 2번을 빌드 중에 해 줍니다."

**S13 IDL 작성** — **말** "실습 1의 `hello.idl` 그대로입니다. module은 이름 범위, struct가 토픽의 데이터 타입, 멤버는 이름·타입·순서·크기입니다. `string<128>`은 최대 128자라는 약속이고, C에서는 `char message[129]`가 됩니다. `@key`는 세션 3에서 나옵니다."

**S14 코드 생성** — **시연** 터미널에서 직접 한 번 생성하고 헤더를 연다.

```powershell
int2dds-idl -c build\gen\hello.h lab1_hello\hello.idl
code build\gen\hello.h
```

**말** "네 가지를 찾으세요. `struct HelloWorld`, `HelloWorld_serialize_for`, `HelloWorld_deserialize_cdr`, `HelloWorld_create_topic`. 이 넷이 세션 2 코드에 나오는 생성물의 전부입니다." **정정** 덱의 마법사 프롬프트는 한글로 적혀 있지만 실제 출력은 영문(`IDL file path`, `Languages to generate`, `Output directory`)이다.

**S15 3a Publisher** — **말** "위에서 아래로 Publisher, Topic, QoS, Writer입니다. `serialize_for`는 Writer의 DataRepresentation에 맞춰 바이트를 만들고, `write_serialized`는 그 바이트를 Writer에 넘깁니다. 반환이 OK라는 건 Writer가 받았다는 뜻이지 Reader가 받았다는 뜻이 아닙니다. 매칭을 기다리는 루프는 실습 편의용입니다. 실제 코드에서 write는 매칭과 무관하게 부를 수 있고, 안 붙어 있으면 그냥 버려집니다. 실습 1 변형 3에서 그걸 봅니다."

**S16 3b Subscriber** — **말** "`take_serialized`는 Reader 캐시에서 하나 꺼내고 캐시에서 지웁니다. `NO_DATA`는 에러가 아니라 지금 없다는 뜻이라 200ms 쉬고 다시 봅니다. `valid_data`가 false면 버퍼가 비어 있으니 역직렬화를 부르지 않습니다. 세션 3에서 이게 언제 오는지 봅니다. 버퍼가 작으면 `BUFFER_TOO_SMALL`과 필요한 크기를 돌려주고 샘플은 그대로 남아 있습니다."

**S17·S18 RTI Connext 비교** — **말** "같은 HelloWorld를 RTI로 쓰면 오른쪽입니다. 차이 셋만 기억하세요. 첫째, int2DDS는 모든 함수가 반환 코드를 돌려주고 결과는 마지막 out 인자로 나옵니다. RTI는 포인터를 돌려주고 NULL이 실패입니다. 둘째, RTI는 타입 등록 뒤 토픽 생성인데 int2DDS는 `HelloWorld_create_topic` 한 줄이 둘을 합니다. 셋째, RTI는 타입별 Writer로 구조체째 write하고, int2DDS는 Writer가 하나뿐이라 직렬화한 바이트로 write합니다. RTI를 써 본 분은 이 표만 보면 옮길 수 있습니다."

**S19 실습 1 · 첫 pub/sub 왕복** — 런시트

**질문** 실행 전에 묻는다. "pub을 먼저 띄우면 어떻게 될까요?" 답: `lab1_pub`는 Reader가 붙을 때까지 기다린다(`pub ready` 뒤 정지). sub가 뜨면 그때 10개를 쓴다.

| 단계 | sub 터미널 | pub 터미널 | 기대 출력 |
|---|---|---|---|
| 기본 | `lab1_sub` | `lab1_pub` | pub `publication_matched: current_count 0 -> 1 (total 1)` 뒤 `write index=1..10`, sub `recv index=1..10 message=hello`. pub는 `cleanup` 후 종료, sub는 계속 대기 |
| 변형 1 | `lab1_sub` | `lab1_pub --domain 1` | pub는 `pub ready: domain=1`에서 정지, sub는 아무것도 없음. 양쪽 다 에러 없음 |
| 변형 2 | `lab1_sub --topic hello_world_topic2` | `lab1_pub` | 변형 1과 같은 현상 |
| 변형 3 | `lab1_sub` | `lab1_pub --no-wait` | sub가 index 2(또는 3)부터 받는다 |
| sub 둘 | `lab1_sub` ×2 (창 셋) | `lab1_pub` | 두 sub 모두 1..10, pub는 `current_count`가 2까지 오른다(0→2 또는 0→1→2) |

**말** 변형 1·2 뒤: "반환값은 전부 OK였습니다. DDS는 상대가 없다고 에러를 내지 않습니다. 조용히 안 옵니다. 그래서 세션 4에서 '누가 보이는지' 세는 코드를 넣습니다." 변형 3 뒤: "기본 DURABILITY가 VOLATILE이라 매칭 전에 쓴 샘플은 버려집니다. 늦게 온 Reader에게 과거를 주는 방법은 내일 실습 7입니다." sub 둘 뒤: "1:N은 공짜입니다. pub 코드는 한 글자도 안 바꿨습니다."

**손대기(시간이 남으면)** `lab1_hello/pub.c`의 `"hello"`를 `"hello #%d"`(인자 `i`)로 바꾸고 `--target lab1_pub`으로 재빌드해 sub 출력이 바뀌는지 본다.

**질문** 확인 포인트 답. current_count가 0→1이 되는 시점은 pub 첫 로그 줄이다. 그 시각과 sub의 첫 `recv` 시각(둘 다 `[ms]`)을 비교하면 디스커버리에 걸린 시간이 보인다.

### 세션 3 · Topic · Key · Instance (S20–S27, 이론 35분 + 실습 35분)

**S21 Topic · Instance · 샘플** — **말** "토픽을 테이블로 생각하세요. 키가 행을 고르고, write 한 번이 그 행에 값을 하나 더 얹습니다. `vehicle_id = 7`이 한 인스턴스, 샘플 t1부터 t4가 그 인스턴스의 역사입니다. 키가 없으면 행이 하나뿐인 테이블입니다. 그러면 소유권도, 인스턴스별 Deadline도, dispose도 의미가 없습니다. 실습 1의 HelloWorld가 그런 토픽이고, 오늘 실습 2가 키 있는 토픽입니다."

**S22 IDL에서 key 정의** — **말** "`@key`가 붙은 멤버가 인스턴스를 고릅니다. 두 개면 복합 키입니다. 실습 2의 TilePressure는 row와 col 두 개가 키라 타일 하나가 인스턴스 하나입니다. 오른쪽 '인스턴스 단위 동작'을 보세요. HISTORY, DEADLINE, OWNERSHIP이 전부 인스턴스 단위로 돕니다. 내일 QoS를 배울 때 이 줄이 계속 나옵니다."

**S23 Instance 생명주기** — 하단 띠의 세 문장을 그대로 읽는다. **말** "ALIVE는 살아 있는 Writer가 하나 이상이고 dispose되지 않은 상태. DISPOSED는 Writer가 '이 인스턴스는 없다'고 선언한 상태. NO_WRITERS는 선언 없이 Writer가 다 사라진 상태입니다. 기본 설정에서는 unregister도 DISPOSED가 됩니다. 이따 실습에서 둘을 구분해 봅니다." **정정** 그림의 화살표 라벨 두 곳은 사실과 다르다. "unregister_instance() 호출 → NO_WRITERS"는 `autodispose=false`일 때만 맞고, "Liveliness를 회복 → ALIVE" 경로는 int2DDS에 없다(되살아나는 길은 다시 write뿐). 오타 "Write가"도 두 곳.

**S24 Instance 관련 API 1·2** — **말** "register_instance는 Writer 안에 표 한 줄을 만들고 핸들을 돌려줍니다. 와이어로는 아무것도 안 나갑니다. 생략해도 됩니다. write가 키를 보고 알아서 등록합니다. 그래도 쓰는 이유는 핸들을 미리 받아 두면 dispose할 때 샘플을 다시 만들 필요가 없기 때문입니다. 넘기는 건 키가 아니라 직렬화한 샘플 전체입니다. 사전 조건 하나. `TilePressure_create_topic`처럼 타입 정보를 함께 등록하는 경로로 만든 토픽이어야 합니다. 아니면 `PRECONDITION_NOT_MET`입니다."

**S25 Instance 관련 API 3·4** — **말** "dispose는 '이 인스턴스는 더 이상 없다', unregister는 '나는 더 이상 이 인스턴스를 안 맡는다'입니다. 코드 차이는 dispose가 핸들로, unregister가 샘플의 키로 대상을 고른다는 것뿐입니다. 순서 함정이 하나 있습니다. dispose한 뒤 같은 인스턴스를 unregister하면 `INVALID_ARGUMENT`입니다. 둘 중 하나로 끝내세요. 그리고 기본값 `autodispose_unregistered_instances = true`라서 unregister도 Reader에는 DISPOSED로 보입니다. NO_WRITERS를 보려면 false로 바꿔야 합니다."

**S26 Reader 쪽** — **말** "Reader가 볼 것은 `valid_data`입니다. true면 데이터, false면 상태 알림입니다. 상태 알림에는 버퍼가 비어 있으니 `instance_state`만 읽습니다. 그런데 어느 타일인지는 어떻게 알까요. 핸들뿐입니다. 그래서 실습 2 sub는 데이터가 올 때 핸들과 좌표를 기억해 뒀다가 알림이 오면 이름을 붙입니다."

**S27 실습 2 · Active Floor** — 런시트

**시연 전에 코드 두 곳을 보여 준다.** `lab2_floor/floor.idl`(복합 키, 도전용 StepEvent)과 `lab2_floor/pub.c`의 주석 1~4(register → write → dispose → unregister). 타일 좌표가 1부터 시작하는 이유를 미리 말한다. "키가 전부 0이면 인스턴스 핸들 16바이트가 전부 0이 되어 FFI가 NIL로 취급합니다. 실제 시스템에서는 키 0을 피하거나 핸들 대신 키로 dispose하세요."

**세 번째 창을 먼저** 100초 실험(변형 4)을 실습 시작과 동시에 띄워 두고 실습 끝에 돌아와 본다.

| 단계 | sub 터미널 | pub 터미널 | 기대 출력 |
|---|---|---|---|
| 기본 | `lab2_sub` | `lab2_pub` | pub: `register tile(1,1) handle=…` ×4 → 주기마다 `write tile(r,c) p=…` ×4(밟힌 타일은 +50) → 5주기 뒤 `dispose tile(1,1)`, `unregister tile(1,2) (autodispose=1)` → 10주기 뒤 `cleanup`. sub: 처음엔 `view=NEW inst=ALIVE`, 이후 `NOT_NEW`; 5주기 뒤 `tile(1,1) -> NOT_ALIVE_DISPOSED (valid_data=false)`, `tile(1,2) -> NOT_ALIVE_DISPOSED (valid_data=false)`; pub 종료 직후 `tile(2,1)`, `tile(2,2) -> NOT_ALIVE_NO_WRITERS` |
| 변형 1 | `lab2_sub` | `lab2_pub --autodispose 0` | `tile(1,2) -> NOT_ALIVE_NO_WRITERS` |
| 변형 2 | `lab2_sub` | `lab2_pub --hold` 후 Ctrl+C | 남은 타일 둘이 즉시 `NOT_ALIVE_NO_WRITERS` |
| 변형 3 (창 셋, 미리) | `lab2_sub` | `lab2_pub --hold --crash-after 5` | pub `simulated crash: _exit without cleanup` 뒤 약 100초 동안 아무 신호 없음 → `NOT_ALIVE_NO_WRITERS` |
| 도전 | `lab2_sub --events` | `lab2_pub --events` | 밟힌 타일마다 `EVENT step on tile(r,c) ts=…` |

**정정 S27** 확인 포인트 "unregister된 타일: NOT_ALIVE_NO_WRITERS"는 `--autodispose 0`일 때만 맞다. 기본값에서는 DISPOSED다. 확인 포인트 순서를 기본 → 변형 1로 읽는다.

**말** 기본 뒤: "dispose와 unregister가 똑같이 DISPOSED로 보였습니다. 기본값이 그렇습니다. 그럼 NO_WRITERS는 언제 볼까요." 변형 1 뒤: "autodispose를 끄니 unregister가 '내가 손을 뗀다'가 됐고, 이 Writer가 마지막이라 NO_WRITERS입니다." 변형 2 뒤: "정상 종료는 Participant가 '나 간다'를 보내니 즉시입니다." 변형 3 뒤: "강제 종료는 아무 말 없이 사라진 겁니다. 기본 Liveliness가 AUTOMATIC에 무한이라 Participant lease 100초가 지나야 압니다. 100초를 1초로 줄이는 게 내일 실습 7의 LIVELINESS입니다."

**질문** "row와 col을 키에서 빼면?" 답은 토론으로 충분하다. 타일 넷이 인스턴스 하나가 되어 마지막 write가 이전 값을 덮고, dispose 한 번에 바닥 전체가 죽는다. 코드로 해 보려면 `floor.idl`에서 `@key`를 지우고 재빌드하는데, pub의 dispose(1,1) 뒤 unregister(1,2)가 같은 인스턴스를 겨냥해 `INVALID_ARGUMENT`로 종료될 가능성이 있다(미실측). 시연은 사전 검증 후에만.

### 세션 4 · Discovery (S28–S30, 이론 20분 + 실습 30분)

**S29 Discovery** — **말** "두 단계입니다. SPDP는 '나 여기 있다'를 멀티캐스트 239.255.0.1로 주기적으로 외칩니다. 포트는 7400 더하기 도메인 곱하기 250이라 도메인 0은 7400, 도메인 1은 7650입니다. 그래서 도메인이 다르면 포트부터 다릅니다. Participant를 서로 알고 나면 SEDP로 '내 Writer는 이 토픽, 이 타입, 이 QoS'를 유니캐스트로 교환하고, 그걸 보고 매칭을 판정합니다. 아래 표가 진단 순서입니다. 참가자가 보이나, 엔드포인트가 보이나, 토픽 이름, 타입, QoS, 파티션. 위에서부터 하나씩 지웁니다."

**S30 실습 3 · Discovery 눈으로 확인** — 런시트

**시연 준비** Wireshark를 Npcap loopback 어댑터로 캡처 시작, 필터 `rtps`. 같은 PC의 두 프로세스는 loopback으로 오가므로 이 어댑터가 없으면 아무것도 안 보인다. `lab3_sub`는 실습 1 sub에 1초마다 발견된 Participant 목록과 매칭된 Writer 수를 찍는 코드가 붙어 있다(`report()`). **정정 S30** 절차 "sub에 get_discovered_participants 조회를 1초 루프로 추가"는 이미 되어 있다. 코드를 열어 보여 주는 것으로 대신한다.

| 단계 | sub 터미널 | pub 터미널 | 기대 출력 |
|---|---|---|---|
| 기본 | `lab3_sub` | `lab1_pub --count 3` | sub `participants=0 []  matched writers=0` → pub 뜨면 `participants=1 [xxxxxxxx ]  matched writers=1` → `recv index=1..3` → pub 종료 직후 `participants=0` |
| Wireshark | 위 실행 중 | | 먼저 `DATA(p)`(목적지 239.255.0.1:7400), 그다음 `DATA(w)`·`DATA(r)`(유니캐스트), 그다음 사용자 `DATA`와 `HEARTBEAT`/`ACKNACK` |
| 변형 1 | `lab3_sub` | `lab1_pub --domain 1` | `participants=0` 유지. Wireshark에서 pub의 SPDP가 7650으로 나가는 것을 찾는다 |
| 변형 2 (창 셋, 미리) | `lab3_sub` | `lab1_pub --hold --crash-after 5` | 목록이 약 100초 뒤에 준다 |
| 선택 | `$env:INT2DDS_META_PORT=7500; lab3_sub` | `$env:INT2DDS_META_PORT=7500; lab1_pub` | 유니캐스트 메타 포트가 7500·7502로 고정된다(참가자마다 +2). 캡처에서 포트로 구분하기 쉬워진다 |

**말** 기본 뒤: "목록에 자기 자신은 없습니다. 상대만 셉니다. pub를 Ctrl+C로 끄면 즉시 빠지는데, 정상 종료가 SPDP dispose를 보내기 때문입니다. 100초짜리는 저 창에서 기다리고 있습니다." Wireshark 뒤: "S9 표를 다시 보세요. p, w, r은 내장 Writer의 DATA입니다. 사용자 데이터가 오기 전에 이 셋이 먼저 지나가야 합니다. 세션 5에서 'p, w, r은 오가는데 사용자 DATA가 없는' 상태를 일부러 만듭니다."

**질문** 확인 포인트. SEDP는 어느 주소·포트로 오가는가: 각 참가자의 유니캐스트 메타 포트(도메인 0 기본 7410부터, 참가자마다 +2). 도메인을 바꾸면 목록이 비고 반환 코드는 OK인가: 그렇다. SEDP까지 오가는데 매칭이 없다면 다음은: 토픽 이름 → 타입 → QoS → 파티션.

### 세션 5 · Domain과 Partition (S31–S35, 이론 20분 + 실습 25분)

**S32 Domain은 벽, Partition은 방** — **말** "Domain이 다르면 Discovery 자체가 안 됩니다. 벽입니다. Partition은 같은 Domain 안의 방입니다. Participant는 서로 보이는데 Writer와 Reader가 방 이름이 하나라도 겹쳐야 붙습니다. 이름을 안 주면 빈 문자열 방이고, 빈 방은 `*`과는 붙지만 `floor-1`과는 안 붙습니다."

**S33 Partition 매칭 규칙** — **정정** 표 8행 `'floor-*'`(Publisher) / `'floor-1'`(Subscriber) = O는 int2DDS에서 X다. 와일드카드는 Subscriber 쪽 문자열의 `prefix*`와 `*suffix`만 해석하고, Publisher 쪽은 `*` 단독만, `?`는 양쪽 다 미지원이다. 표를 고치기 전까지는 "int2DDS는 Publisher 쪽 `floor-*`를 리터럴로 본다"고 말한다. **말** "규칙 셋. 하나라도 겹치면 매칭. 와일드카드는 구독 쪽에서. 양쪽 다 와일드카드면 실패."

**S34 Partition 설정** — **말** "Partition은 Writer가 아니라 Publisher QoS, Reader가 아니라 Subscriber QoS입니다. 변경 가능한 QoS라 런타임에 바꿀 수 있는데, int2DDS는 Subscriber `set_qos`만으로는 SEDP 재광고가 안 나갑니다. Reader의 `get_qos`, `set_qos`를 한 번 더 불러야 새 파티션이 상대에게 갑니다. 코드 아래 세 줄이 그겁니다." 오른쪽 열의 비용 문장을 짚는다. "Partition은 문자열이라 공짜, Domain은 Participant가 하나 더 필요해 포트와 스레드가 듭니다."

**S35 실습 4 · 격리 실험** — 런시트. **정정 S35** "실습 3의 pub/sub"는 `lab4_pub`/`lab4_sub`다(실습 3 sub에 파티션 인자와 런타임 변경을 더한 것).

| 단계 | sub 터미널 | pub 터미널 | 기대 출력 |
|---|---|---|---|
| Domain 불일치 | `lab4_sub --domain 1` | `lab4_pub --domain 0` | sub `participants=0  matched writers=0` 유지 |
| Partition 불일치 | `lab4_sub` | `lab4_pub --partition floor-1` | sub `participants=1  matched writers=0`. Wireshark에 DATA(p)·DATA(w)·DATA(r)는 있고 사용자 DATA는 없음 |
| 런타임 변경 | `lab4_sub --switch-after 5 --switch-to floor-*` | `lab4_pub --partition floor-1` | 5초 뒤 sub `partition switched to (floor-*)` → `matched writers=1` → `recv index=…` |
| 하나만 겹침 | `lab4_sub --partition floor-2` | `lab4_pub --partition floor-1,floor-2` | 즉시 매칭, 데이터 흐름 |

PowerShell 5.1은 `floor-*`와 `floor-1,floor-2`를 확장하지 않고 그대로 넘긴다. 따옴표가 필요 없다.

**말** Partition 불일치 뒤: "세션 4에서 예고한 상태입니다. 참가자는 보이고, SEDP도 오갔고, 그런데 매칭이 0입니다. 표에서 봤듯 이건 Incompatible QoS 상태도 안 남깁니다. 파티션 불일치는 가장 조용한 실패입니다." 런타임 변경 뒤: "코드 재시작 없이 붙었습니다. 34장 세 줄이 한 일입니다."

**질문** 확인 포인트 넷은 표의 네 행이 순서대로 답한다.

### 세션 6 · 데이터 소비 (S36–S41, 이론 35분 + 실습 35분)

**S37 Reader Cache · read와 take** — **말** "Reader 캐시는 인스턴스별 줄입니다. read는 보고 남기고, take는 보고 지웁니다. 샘플마다 READ/NOT_READ, 인스턴스마다 NEW/NOT_NEW와 ALIVE/NOT_ALIVE가 붙습니다. 지금까지 쓴 `take_serialized`는 'NOT_READ만' 꺼내는 기본 마스크였습니다. 마스크를 주는 `_w_states` 계열로 바꾸면 READ 샘플도 다시 볼 수 있습니다. 실습 5의 첫 프로그램이 이 다섯 가지를 순서대로 부릅니다."

**S38 SampleInfo** — **말** "`valid_data`가 첫 번째입니다. false면 버퍼가 비어 있는데 크기 값은 채워지므로, 검사 없이 역직렬화하면 초기화되지 않은 메모리를 읽습니다. 그다음 세 상태, 그다음 어느 인스턴스·어느 Writer에서 왔는지 핸들 둘, 그다음 Writer의 write 시각입니다. 세대 카운트는 인스턴스가 NOT_ALIVE에서 되살아난 횟수인데, 실습 2에서 dispose 뒤 다시 write하면 1이 됩니다."

**S39 Listener** — **말** "필요한 콜백만 채우고 나머지는 NULL, 그리고 마스크에 넣은 상태만 불립니다. 함수를 등록해도 마스크에 없으면 안 불립니다. 주의 두 가지. int2DDS는 콜백을 RTPS 수신 스레드에서 동기 호출합니다. 콜백에서 1초 자면 그 Participant의 수신이 1초 멈춥니다. 그리고 콜백 안에서 자기 Entity를 지울 수 없습니다. 콜백에서는 take만 하고 처리는 짧게, 무거운 일은 큐에 넘기세요."

**S40 WaitSet + ReadCondition** — **말** "Listener가 미들웨어 스레드에서 불리는 게 싫으면 WaitSet입니다. 내 스레드가 `wait`에서 자다가 조건이 켜지면 깹니다. ReadCondition은 '안 읽은 샘플이 있다', GuardCondition은 내가 켜는 스위치라 종료 신호로 씁니다. 이 코드는 활성 목록을 안 받고 Guard를 먼저 확인한 뒤 나머지를 ReadCondition으로 배치 take합니다. batch는 SampleSeq로 오고 length, get_data, get_info, delete 순서입니다."

**S41 실습 5 · 세 가지 수신 방식** — 런시트. pub는 세 경우 모두 `lab2_pub --hold`(타일 4개를 계속 쓴다).

| 단계 | sub 터미널 | pub 터미널 | 기대 출력 |
|---|---|---|---|
| 폴링·마스크 | `lab5_sub_read` | `lab2_pub --hold` | 1초마다 다섯 줄 묶음. `1 read_w_info` 도착 샘플 전부(`sample_state=NOT_READ`) → `2 read_w_info again NO_DATA` → `3 read_w_states(READ)` 첫 샘플이 `sample_state=READ`로 → `4 take_w_states(ANY)` 전부 → `5 read_w_states(ANY) NO_DATA` |
| Listener | `lab5_sub_listener` | (계속) | `on_subscription_matched: current_count=1 (+1)` → `on_data_available #n: tile(r,c) p=…`. pub를 껐다 켜면 `(-1)`, `(+1)` |
| Listener 지연 | `lab5_sub_listener --slow-ms 1000` | (계속) | `callback sleeping 1000 ms (receive thread blocked)` 뒤 다음 콜백에 샘플이 몰려 온다 |
| WaitSet | `lab5_sub_waitset` (pub 없이 먼저) | 나중에 `lab2_pub --hold` | 매초 `wait: TIMEOUT (2)` → pub가 뜨면 `tile(r,c) p=…` → Ctrl+C 시 GuardCondition으로 루프 탈출, `cleanup` |

**정정 S41** 확인 포인트 "read 2회: 같은 샘플이 다시 오는가"는 틀렸다. `read_serialized_w_info`는 NOT_READ만 읽으므로 두 번째 호출은 `NO_DATA`다. 절차의 "`read_serialized_w_states(NOT_READ)`로 바꿔 재시도"도 NOT_READ가 아니라 READ 또는 ANY 마스크여야 같은 샘플이 다시 온다. 이 프로그램은 3단계에서 READ 마스크로 한 번만 부른다. 단일 샘플 read를 READ/ANY로 루프 돌리면 같은 첫 샘플만 계속 오니 한 번만 부른다.

**말** 폴링 뒤: "2번이 NO_DATA인 게 핵심입니다. 기본 마스크는 '안 읽은 것'이라 read 한 번으로 전부 READ가 됐습니다. 3번에서 READ 마스크로 부르니 같은 샘플이 READ 표시로 다시 옵니다. 4번 take 뒤엔 5번이 비어 있습니다." Listener 지연 뒤: "1초 잔 동안 pub는 두 주기를 썼고, 다음 콜백에 그만큼 몰려 왔습니다. 수신 스레드가 멈춘 겁니다." WaitSet 뒤: "TIMEOUT은 반환 코드 2이고 에러가 아닙니다. 주기 작업을 끼워 넣는 자리입니다."

**질문** 확인 포인트 답. read 2회: 두 번째는 NO_DATA(위 정정). take 후 read: 비어 있다(5단계). Listener sleep: 수신이 그만큼 밀린다. WaitSet 타임아웃: `INT2DDS_RET_TIMEOUT`(2).

## 3. Day 2

복습 15분은 Day 1의 "안 붙는 이유" 네 가지(Domain, Topic 이름, Partition, QoS)를 칠판에 다시 쓰고, 오늘은 그중 QoS를 하루 종일 본다고 연다.

### 세션 7 · QoS 프레임워크 (S42–S50, 이론 45분 + 실습 30분)

**S43 QoS란 무엇인가** — **말** "'무엇을 보내는가'는 토픽과 타입, '어떻게 보내는가'가 QoS입니다. 정책 하나는 이름과 값이고 코드가 아니라 선언입니다. 네 단계로 돕니다. 선언하고, Entity에 붙이고, Discovery에서 Writer의 Offered와 Reader의 Requested를 대조해 판정하고, 매칭 뒤에도 위반은 Status로 드러납니다. 어제 Partition 불일치가 조용했던 것과 달리, QoS 불일치는 이유가 남습니다. 오늘 실습 6이 그 이유를 읽는 연습입니다."

**S44 5가지 카테고리** — **말** "22개를 외우지 말고 다섯 묶음으로 보세요. 가용성은 '늦게 온 Reader에게도 가나', 전달은 '어떤 보장과 순서로', 적시성은 '시간 약속', 자원은 '얼마나 쌓나', 구성은 통신과 무관한 태그입니다. 오늘 실습에서 만지는 건 가용성의 DURABILITY·HISTORY, 전달의 RELIABILITY·OWNERSHIP, 적시성의 DEADLINE·LIVELINESS, 자원의 TIME_BASED_FILTER입니다."

**S45 정책이 적용되는 Entity** — **말** "첫 줄이 대부분입니다. Topic, Writer, Reader에 붙습니다. Partition은 Pub/Sub에만, TIME_BASED_FILTER는 Reader에만, OWNERSHIP_STRENGTH는 Writer에만. 매칭 판정은 항상 Writer와 Reader 사이입니다."

**S46 RxO** — **말** "Reader가 요구하고 Writer가 제공합니다. 정책 하나라도 안 맞으면 매칭이 안 됩니다. 예시를 보세요. Reader가 50ms 안에 갱신을 요구하는데 Writer는 100ms만 약속합니다. Reader에는 REQUESTED_INCOMPATIBLE_QOS, Writer에는 OFFERED_INCOMPATIBLE_QOS가 남습니다. 방향은 정책마다 다릅니다. RELIABLE Writer는 BEST_EFFORT Reader와 붙지만 반대는 안 됩니다."

**S47 핵심 QoS 한눈에** — **말** "'Enable 후 변경' 열을 보세요. RELIABILITY, HISTORY, DURABILITY, LIVELINESS, OWNERSHIP은 생성 뒤 못 바꿉니다. DEADLINE, LIFESPAN, PARTITION은 바꿀 수 있습니다. 실습 6에서 RELIABILITY를 런타임에 바꾸려다 `IMMUTABLE_POLICY`를 받는 걸 봅니다."

**S48 QoS 설정과 RxO 호환성 확인** — **말** "흐름은 기본 QoS 생성, 정책 설정, Entity 생성, QoS 해제입니다. 기간은 전부 나노초 int64라 `MS()` 매크로를 씁니다. 기본값을 외워 두세요. Writer는 RELIABLE, Reader는 BEST_EFFORT, HISTORY는 KEEP_LAST(1)입니다. 실습 코드가 Reader를 RELIABLE로 올리는 이유가 이겁니다."

**S49 QoS 불일치 진단** — **말** "Reader에서 `requested_incompatible_qos_status`, Writer에서 `offered_incompatible_qos_status`를 읽습니다. `last_policy_id`가 어느 정책인지 알려 줍니다. 진단 순서는 어제와 같습니다. Discovery 됐나, current_count가 0인가, Incompatible 카운트가 올라가나, 아무 이벤트도 없으면 Domain·Topic·Type·Partition입니다."

**S50 실습 6 · 일부러 안 맞추기** — 런시트. `lab6_pub`/`lab6_sub`는 실습 1 pub/sub에 QoS 인자와 세 콜백(incompatible_qos·matched·deadline_missed)을 붙인 것이다. 양쪽 다 1초마다 `matched=… incompatible_qos total=… last=…`를 찍는다.

| 단계 | sub 터미널 | pub 터미널 | 기대 출력 |
|---|---|---|---|
| RELIABILITY | `lab6_sub --reliability reliable` | `lab6_pub --reliability best_effort` | 양쪽 `*_INCOMPATIBLE_QOS: last_policy=RELIABILITY total=1`, `matched=0` |
| DEADLINE | `lab6_sub --deadline-ms 50` | `lab6_pub --deadline-ms 100` | `last_policy=DEADLINE`, `matched=0` |
| OWNERSHIP | `lab6_sub --ownership exclusive` | `lab6_pub --ownership shared` | `last_policy=OWNERSHIP`, `matched=0` |
| 런타임 수정 | `lab6_sub --deadline-ms 50` | `lab6_pub --deadline-ms 100 --fix-deadline-after 5 --fix-deadline-ms 50` | 5초 뒤 pub `set_qos(deadline=50 ms) -> OK` → 양쪽 `PUBLICATION/SUBSCRIPTION_MATCHED: current_count=1 (+1)` → sub `recv index=…` |
| 불변 정책 | (없어도 됨) | `lab6_pub --try-immutable` | 3초 뒤 `set_qos(reliability change) -> IMMUTABLE_POLICY (22)` |

DEADLINE 50/100ms에 쓰기 주기 500ms면 매칭 전후로 DEADLINE_MISSED 콜백이 초당 열 번 찍힌다. 조용히 보려면 `--period-ms 100`에 pub `--deadline-ms 300`, sub `--deadline-ms 200`, 수정은 `--fix-deadline-ms 200`으로 한다.

**정정 S50** 확인 포인트 "pub의 QoS를 고치면 재시작 없이 매칭되나"의 답은 "DEADLINE만". RELIABILITY와 OWNERSHIP은 enable 후 불변이라 재시작해야 한다.

**말** 첫 세 케이스 뒤: "세 번 다 참가자는 보이고 매칭은 0인데, 어제와 달리 `last_policy`가 이유를 말해 줍니다. 로그를 못 보는 시스템이라면 이 Status 하나가 디버깅 시간을 반으로 줄입니다." 런타임 수정 뒤: "DEADLINE은 변경 가능 정책이라 set_qos 한 번에 SEDP가 다시 나가고 붙었습니다." 불변 정책 뒤: "22번, IMMUTABLE_POLICY. 생성 전에 정하라는 뜻입니다."

**질문** 확인 포인트 답. 어느 조합이 실패하나: 표의 세 행. last_policy_id가 맞나: 콜백과 상태 줄 둘 다 같은 이름. Wireshark에서 SEDP까지 오가는가: 그렇다, DATA(w)·DATA(r)는 있고 사용자 DATA만 없다.

### 세션 8 · QoS 정책별 동작과 설정 (S51–S60, 이론 50분 + 실습 60분)

**S52 RELIABILITY** — **말** "BEST_EFFORT는 빠진 걸 안 찾습니다. 센서 스트림처럼 다음 값이 곧 오는 데이터에 씁니다. RELIABLE은 HEARTBEAT와 ACKNACK으로 빠진 번호를 되찾고 순서를 지킵니다. 명령, 설정, 이벤트에 씁니다. 대신 자원이 차면 write가 max_blocking_time만큼 기다립니다. RxO는 RELIABLE Writer가 둘 다 만족, BEST_EFFORT Writer는 RELIABLE Reader를 못 만족합니다. 어제 실습 6 첫 케이스가 그거였습니다."

**S53 HISTORY와 RESOURCE_LIMITS** — **말** "HISTORY는 무엇을 남기나, RESOURCE_LIMITS는 얼마나 남기나입니다. KEEP_LAST(depth)는 인스턴스별 최신 depth개, 기본은 1입니다. KEEP_ALL은 자원 한도까지 전부. 실습 7 A에서 depth 1과 10의 차이를 봅니다."

**S54 DURABILITY** — **말** "늦게 온 Reader에게 얼마나 과거를 주나입니다. VOLATILE은 안 줍니다. 실습 1 변형 3에서 앞 샘플이 사라진 이유입니다. TRANSIENT_LOCAL은 Writer가 살아 있는 동안 History에 남은 만큼 줍니다. TRANSIENT와 PERSISTENT는 외부 서비스가 필요하고 int2DDS는 미구현입니다. RxO는 Writer 수준이 Reader 요구 이상이어야 합니다."

**S55 동작 1 · Late Joiner** — **말** "레시피입니다. Writer는 RELIABLE + TRANSIENT_LOCAL + KEEP_LAST(1), Reader는 RELIABLE + TRANSIENT_LOCAL. Reader도 TRANSIENT_LOCAL을 요구해야 과거를 받습니다. VOLATILE Reader는 붙긴 하지만 과거를 안 달라고 한 겁니다. 반대로 TRANSIENT_LOCAL Reader는 VOLATILE Writer와 아예 안 붙습니다. 실습 8에서 이걸 프로파일로 다시 씁니다."

**S56 DEADLINE · LIVELINESS** — **말** "DEADLINE은 인스턴스가 period 안에 갱신되나, LIVELINESS는 Writer가 살아 있나입니다. 다른 질문입니다. Writer가 살아 있어도 값을 안 보내면 DEADLINE 위반이고, LIVELINESS는 멀쩡합니다. AUTOMATIC은 미들웨어가 알아서, MANUAL_BY_TOPIC은 write나 assert_liveliness를 내가 lease 안에 불러야 합니다. 어제 실습 2의 100초가 Participant 단위 기본 lease였고, 오늘은 Writer 단위로 1~2초까지 줄입니다."

**S57 동작 2 · 헬스 모니터링** — **말** "Writer는 0.5초 갱신 약속에 2초 lease, Reader는 1초 안에 갱신 요구에 같은 lease. Reader 콜백 둘을 답니다. write가 멈추면 1초 뒤 REQUESTED_DEADLINE_MISSED, 2초 뒤 LIVELINESS_CHANGED입니다. 순서가 중요합니다. 실습 7 B에서 시각을 재 봅니다."

**S58·S59 OWNERSHIP · Failover** — **말** "EXCLUSIVE면 인스턴스별로 strength가 가장 높은 살아 있는 Writer 하나만 Reader에 보입니다. A가 100, B가 50이면 B의 샘플은 버려집니다. A가 liveliness를 잃거나 DEADLINE을 어기면 B가 이어받습니다. Reader 코드는 그대로입니다. kind는 양쪽이 같아야 붙습니다. 실습 6 세 번째 케이스가 그거였습니다. 키 없는 토픽이면 토픽 전체에서 주인 하나입니다."

**S60 실습 7 · 레시피 실험** — 런시트. `lab7_pub`/`lab7_sub`는 실습 1 pub/sub에 `--recipe`로 `recipes.h`의 QoS 함수 셋을 고르게 한 것이고, 그 헤더가 절차 E의 답이다(관용구 3종을 함수로. 프로파일 버전은 실습 8의 `team_qos.xml`). sub 콜백은 matched·deadline_missed·liveliness_changed·incompatible_qos 넷이다.

**A · Late Joiner** (pub를 먼저 띄우고 다 쓴 뒤 sub를 늦게)

| sub 터미널 (나중) | pub 터미널 (먼저) | 기대 출력 |
|---|---|---|
| `lab7_sub --recipe hard_state --depth 10` | `lab7_pub --recipe hard_state --depth 10 --count 10 --hold` | sub `recv index=1..10 from=A` |
| `lab7_sub --recipe hard_state --depth 1` | `lab7_pub --recipe hard_state --depth 1 --count 10 --hold` | `recv index=10`만 |
| `lab7_sub` | `lab7_pub --count 10 --hold` | 아무것도 안 옴(VOLATILE) |

**정정 S60** 확인 포인트 "A: sub가 첫 샘플(index=1)부터 받는가"는 Writer와 Reader 둘 다 HISTORY depth ≥ 10일 때만 그렇다. S55 레시피의 KEEP_LAST(1)이면 마지막 하나만 온다. 키 없는 HelloWorld라 "인스턴스별 최신 1개"가 곧 "마지막 1개"다.

**B · 헬스 모니터링**

| sub 터미널 | pub 터미널 | 기대 출력 |
|---|---|---|
| `lab7_sub --recipe health --deadline-ms 1000 --lease-ms 2000` | `lab7_pub --recipe health --deadline-ms 500 --lease-ms 2000 --pause-after 5` | 5초에 pub `paused: no more write` → 약 1초 뒤 sub `REQUESTED_DEADLINE_MISSED: total=1 instance=…`(이후 매초) → 약 2초 뒤 `LIVELINESS_CHANGED: alive=0 (-1) not_alive=1 (+1)`와 `NOT_ALIVE_NO_WRITERS (valid_data=false)`, pub에는 `LIVELINESS_LOST` |
| (같음) | 위에 `--assert` 추가 | DEADLINE_MISSED만 반복, LIVELINESS는 유지 |

**말** "deadline이 liveliness보다 먼저 왔습니다. 1초 대 2초. 그리고 `--assert`를 주면 '살아는 있는데 값을 안 보내는' 상태가 구분됩니다. 프로세스가 죽은 것과 프로세스가 멈춘 것을 나눠 감시하는 게 이 레시피의 목적입니다."

**C · Failover** (창 셋: sub, A, B)

| 창 | 명령 | 기대 출력 |
|---|---|---|
| sub | `lab7_sub --recipe failover --lease-ms 1000` | `recv index=… from=A`만. B는 안 보임 |
| A | `lab7_pub --recipe failover --strength 100 --name A --lease-ms 1000` | |
| B | `lab7_pub --recipe failover --strength 50 --name B --lease-ms 1000` | |
| A 종료 | A 창 Ctrl+C 또는 `--crash-after 10`으로 다시 띄움 | 약 1초 안에 sub `from=B`(실측 0.7초). Ctrl+C면 즉시 |
| A 재시작 | A 명령 다시 | `from=A`로 복귀 |

**말** "전환 지연이 lease_duration 근처입니다. AUTOMATIC lease 1초라 프로세스가 죽으면 1초 안에 압니다. A가 돌아오면 strength가 높으니 바로 되찾습니다. Reader는 한 줄도 안 바꿨습니다."

**D · 데이터율 제한**

| sub 터미널 | pub 터미널 | 기대 출력 |
|---|---|---|
| `lab7_sub --tbf-ms 100 --rate` | `lab7_pub --period-ms 10` | `rate: 10 samples/s` 근처. Wireshark 패킷 수는 100/s 그대로 |
| `lab7_sub --gaps` | `lab7_pub --period-ms 0 --count 1000` | `GAP: expected n got m` 여러 줄 |

**말** "TIME_BASED_FILTER는 int2DDS에서 Reader 쪽 필터입니다. 네트워크는 그대로고 애플리케이션에 주는 것만 줄입니다. 대역폭을 아끼려면 Writer 주기를 줄여야 합니다. 두 번째는 KEEP_LAST(1) Reader에 빠르게 쓰면 take 전에 밀려난 샘플이 GAP으로 보이는 겁니다. 실습 5의 `lab5_sub_read`가 depth 8을 준 이유입니다."

D를 다른 실험과 동시에 돌렸을 때 sub가 매칭 뒤 멈춘 사례가 한 번 있었다(단독 재현 없음). 그러면 sub만 다시 띄운다.

**질문** 확인 포인트 답. B: deadline 1초가 liveliness 2초보다 먼저. C: 전환 ≈ lease, 복귀는 즉시. D: 수신률 ~10Hz, 패킷은 그대로. KEEP_LAST(1) 중간값 유실: `--gaps`.

### 세션 9 · XML 활용 (S61–S69, 이론 40분 + 실습 40분)

**S62 XML로 할 수 있는 세 가지** — **말** "타입만, QoS만, 둘 다에 Entity 트리까지. 실습 8이 이 순서로 코드에 있던 것을 XML로 옮깁니다. 하단의 자동 로드 규칙을 기억하세요. `DDS_QOS_PROFILE`에 파일 경로, `DDS_DEFAULT_QOS_PROFILE`에 `Library::Profile`을 두면 코드 수정 없이 QoS가 바뀝니다. JSON은 QoS만, 타입은 XML 전용입니다."

**S63 QoS 프로파일** — **말** "`qos_library` 안에 `qos_profile`, 그 안에 Entity별 `*_qos`입니다. `*_with_profile`은 프로파일에 그 Entity의 항목이 있어야 삽니다. 없으면 로드는 OK인데 생성에서 실패합니다." **정정 S63** 덱 XML의 `<publisher_qos>`·`<subscriber_qos>`의 partition `lab`은 실습 코드 `team_qos.xml`에 없다. 덱 XML을 그대로 쓰면 `lab9_sub_profile`(기본 프로파일 → Subscriber partition `lab`)과 `lab2_pub --profile`(Publisher는 NULL QoS → `""`)이 매칭되지 않는다. 덱에서 두 블록을 지우거나, 실습 코드의 XML을 정본으로 말한다. 오타 "Proflie"도 이 장과 S64 제목.

**S64 QoS Profile로 Entity 생성** — **말** "생성 전에 `load_profiles`, 그다음 `*_with_profile`에 이름을 줍니다. NULL QoS를 주면 기본 프로파일을 따르는데, 기본 프로파일은 첫 Participant 생성 전에 정해야 합니다." **정정 S64** dynamic 계열(`create_datareader_dynamic` 등)에는 `_with_profile`이 없다. 실습 8의 2단계는 `int2dds_env_set_default_qos_profile("Team::HardState")` 뒤 NULL QoS로 프로파일을 받는다.

**S65 타입을 XML로** — **말** "struct, member, type은 IDL과 같은 뜻이고 `key="true"`가 `@key`입니다. module 안에 두면 이름에 `lab::`이 붙습니다. 실습 8에서 이름 하나로 매칭이 사라지는 걸 봅니다."

**S66 XML 타입으로 Topic·Writer·Reader** — **말** "코드 생성 단계가 없습니다. 실행 시 XML을 읽고 타입 이름으로 TypeSupport를 얻어 `_dynamic` 계열로 만듭니다. 샘플은 DynamicData라 필드 이름으로 읽고 씁니다. 대가는 컴파일 오류가 반환값으로 바뀐다는 것입니다. 해제 순서는 DynamicData, Reader, TypeSupport, Registry입니다."

**S67·S68 한 파일로 · 구성 파일로 트리 생성** — **말** "네 섹션입니다. types, qos_library, domain_library, domain_participant_library. Participant 이름 하나로 트리 전체가 만들어지고 코드에는 이름만 남습니다. 배포 환경이 바뀌면 XML만 바꿉니다. C API는 트리의 Participant 핸들을 안 주니 트리 밖에 Entity를 더 만들 수 없고, 꺼낸 핸들은 트리를 지우기 전에 먼저 해제합니다."

**S69 실습 8 · XML 활용** — 런시트. pub는 실습 2의 `lab2_pub`(IDL 경로)다. **정정 S69** 덱 제목은 "실습 8"인데 코드 폴더와 README는 "실습 9"(`lab9_xml`, `lab9_sub_*`)다. 둘 중 하나로 맞춘다. 확인 포인트 "핸들 해제 순서를 바꾸면 무슨 일이 나는가"는 use-after-free라 결과가 정의되지 않으니 지운다.

| 단계 | sub 터미널 | pub 터미널 | 기대 출력 |
|---|---|---|---|
| 1 타입만 | `lab9_sub_xml` | `lab2_pub` | sub `matched writers=1`, `tile(r,c) p=… view=… inst=…`. IDL Writer ↔ XML Reader가 붙는다 |
| 1' 이름 불일치 | `lab9_sub_xml --xml lab9_xml/tile_lab.xml --type lab::TilePressure` | `lab2_pub` | `matched writers=0` 유지 |
| 2 QoS도 | `lab9_sub_profile` (나중) | `lab2_pub --profile Team::HardState --no-wait --hold` (먼저) | sub `effective reader qos: reliability=RELIABLE durability=TRANSIENT_LOCAL history=KEEP_LAST(1)` 뒤 늦게 붙었는데도 타일 4개의 최신값이 바로 온다 |
| 3 트리까지 | `lab9_sub_config` (나중) | (같음, 도메인 0) | `sub ready (config): reader=sub::tile reliability=RELIABLE durability=TRANSIENT_LOCAL`, `matched writers=1`, 타일 값 |
| 도전 | `tile.xml`의 `row`를 `rwo`로 바꿔 `lab9_sub_xml` | `lab2_pub` | 로드와 Topic·Reader 생성은 OK. 그 뒤는 둘 중 하나다(미실측, 시연 전 확인). 타입 비호환으로 `matched writers=0`에 머물거나, 매칭되면 첫 샘플의 `int2dds_dynamic_data_get_i16(sample, "row", …)`에서 `DYNAMIC_FIELD_NOT_FOUND`로 종료 |

`app.xml`은 domain 0 고정이라 3단계 pub는 `--domain 0`(기본)으로 띄운다. pub도 같은 프로파일이어야 한다. VOLATILE Writer는 TRANSIENT_LOCAL Reader와 매칭되지 않는다.

**말** 1' 뒤: "필드가 똑같아도 이름이 `lab::TilePressure`면 다른 타입입니다. 세션 4 표의 '타입 비호환' 행입니다." 2 뒤: "어제 실습 7 A를 XML 한 장으로 다시 한 겁니다. 코드에는 프로파일 이름 한 줄만 있습니다." 3 뒤: "이제 코드에 남은 건 파일 경로, `Apps::Monitor`, `sub::tile`, 필드 이름 셋입니다." 도전 뒤: "오타가 컴파일이 아니라 첫 take에서 터졌습니다. 동적 경로를 쓰면 반환값 검사가 선택이 아니라 필수입니다."

### 마무리 (20분, 덱에 장표 추가 예정)

권장 구성 세 장. (1) 이틀 요약 다섯 줄: Topic으로 만난다 · 키가 인스턴스를 만든다 · Discovery가 먼저다 · QoS는 Writer가 제공하고 Reader가 요구한다 · 코드 밖 XML로 옮길 수 있다. (2) "안 붙을 때" 체크리스트 한 장: Domain → Topic 이름 → Type → QoS(Incompatible 상태) → Partition. (3) 다음 단계: Python·C# 바인딩(같은 FFI), 다른 벤더와의 상호운용, 프로파일 배포. 남는 시간은 Day 1 실습 재시도로.

## 4. 부록

### A. 트러블슈팅

| 증상 | 확인 | 처치 |
|---|---|---|
| 아무것도 안 온다 | `lab3_sub`로 참가자가 보이나 → `lab6_sub`처럼 `incompatible_qos total`이 오르나 | Domain → Topic 이름 → Type → QoS → Partition 순서로 하나씩 |
| exe가 바로 죽는다 | `build\Release\int2dds_ffi.dll` 존재, `$env:Path`를 그 창에서 쳤나 | 재빌드 또는 dll 복사 |
| 빌드는 됐는데 실행이 조용히 실패 | 경로에 한글·공백 | ASCII 경로로 옮긴다 |
| 첫 실행 뒤로 안 붙는다 | 방화벽 팝업을 거부함 | 인바운드 UDP 7400–7500과 멀티캐스트 허용 |
| 강제 종료 뒤 NO_WRITERS가 안 온다 | 100초 안 지남 | 기다리거나 실습 7 B·C의 lease로 대체 |
| Wireshark에 아무것도 없다 | 같은 PC 캡처인데 Npcap loopback 어댑터가 아님 | 어댑터 선택, 필터 `rtps` |
| DEADLINE 콜백이 폭주 | 50/100ms 기한에 500ms 주기 | `--period-ms 100`, 기한 300/200ms |
| PowerShell이 `*`를 이상하게 처리하나 | 5.1은 `floor-*`를 그대로 넘긴다 | 따옴표 불필요 |
| `read_w_states(READ)`를 루프에 넣었더니 같은 샘플만 | read는 상태를 안 바꾼다 | 한 번만 부르거나 take |
| CMake가 lib를 못 찾는다 | `cargo build --release` 안 했거나 Debug로 configure | `-DCMAKE_BUILD_TYPE=Release`와 `--config Release` 둘 다 |
| 실습 7 D sub가 매칭 뒤 멈춤 | 드문 사례 | sub만 재실행 |

### B. 강의 전 덱 정정 목록

| 슬라이드 | 현재 | 실제 | 조치 |
|---|---|---|---|
| S2 표 · S69 제목 | 실습 8 · XML 활용 | 코드는 `lab9_xml`, README "실습 9" | 번호 하나로 통일 |
| S3 | "직접빌드 or dll제공" 스티커 | 미결정 | SDK 제공 권장, 스티커 제거 |
| S10 | 빈 슬라이드("ㅌ") | – | Wireshark 캡처로 채우거나 삭제 |
| S14 | 마법사 프롬프트 한글 | 영문 출력 | 표기 교체 |
| S23 | unregister → NO_WRITERS, Liveliness 회복 → ALIVE | autodispose=false 조건, 회복 경로 없음 | 라벨 수정, 오타 "Write가" |
| S27 | unregister된 타일: NO_WRITERS | 기본은 DISPOSED, `--autodispose 0`이면 NO_WRITERS | 확인 포인트 수정, 강제 종료 100초 명기 |
| S30 | "조회 루프 추가" 절차 | `lab3_sub`에 이미 있음 | "코드 확인"으로 |
| S33 | 8행 `'floor-*'`/`'floor-1'` = O | int2DDS는 X | X로 수정 |
| S35 | "실습 3의 pub/sub" | `lab4_pub`/`lab4_sub` | 이름 교체 |
| S41 | read 2회 → 같은 샘플, `w_states(NOT_READ)` 재시도 | 두 번째는 NO_DATA, 재시도는 READ/ANY | 두 항목 수정 |
| S43 | "'DDS 운영과 활용' 세션" 2곳 | 세션 9 = XML 활용 | 이름 교체 |
| S50 | "pub QoS를 고치면 재시작 없이" | DEADLINE만 | 조건 명기, 주기 100ms 권장 |
| S60 | A 첫 샘플부터, E 프로파일 | 양쪽 depth ≥ 10, E는 `recipes.h` 함수 | 조건 명기 |
| S63·S64 | XML에 partition `lab`, "Proflie" | labs XML에 없음, 오타 | 블록 삭제, 오타 수정 |
| S69 | 절차 2 `*_with_profile`, 확인 포인트 "해제 순서" | dynamic엔 없음(기본 프로파일), UB | 문구 교체, 항목 삭제 |
| 실습 8장 | "실습내용 확인 전" 스티커 | – | 제거 |
| 전체 | 푸터 쪽번호 리터럴 | 어긋남 | 유저가 마지막에 처리 |

### C. 명령 치트시트

| 실습 | sub | pub |
|---|---|---|
| 1 | `lab1_sub [--domain N] [--topic T]` | `lab1_pub [--domain N] [--no-wait] [--count N] [--hold] [--crash-after S]` |
| 2 | `lab2_sub [--events]` | `lab2_pub [--autodispose 0] [--hold] [--crash-after S] [--events] [--profile Team::HardState --no-wait]` |
| 3 | `lab3_sub` | `lab1_pub --count 3` / `--domain 1` / `--hold --crash-after 5` |
| 4 | `lab4_sub [--partition P] [--switch-after S --switch-to P]` | `lab4_pub --partition P[,P2]` |
| 5 | `lab5_sub_read` / `lab5_sub_listener [--slow-ms N]` / `lab5_sub_waitset [--timeout-ms N]` | `lab2_pub --hold` |
| 6 | `lab6_sub --reliability R --deadline-ms N --ownership O` | `lab6_pub` 같은 인자 `+ --fix-deadline-after S --fix-deadline-ms N` / `--try-immutable` |
| 7 | `lab7_sub --recipe hard_state\|health\|failover [--depth N] [--tbf-ms N --rate] [--gaps]` | `lab7_pub --recipe … [--name A --strength N] [--pause-after S --assert] [--period-ms N]` |
| 8 | `lab9_sub_xml [--xml F --type T]` / `lab9_sub_profile` / `lab9_sub_config` | `lab2_pub --profile Team::HardState --no-wait --hold` |

공통: `--domain N` · `--topic NAME` · `--count N`(0=무한) · `--period-ms N` · `--hold` · `--crash-after S` · `--no-wait`. 환경변수: `INT2DDS_META_PORT`(유니캐스트 메타 포트 고정), `DDS_QOS_PROFILE`, `DDS_DEFAULT_QOS_PROFILE`.
