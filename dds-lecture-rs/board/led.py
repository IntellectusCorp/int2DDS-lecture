#!/usr/bin/env python3
"""board_sub 출력에서 'led N on 밝기|off|blink|fast' 줄만 LED로 옮기고 나머지 줄은 그대로 보여 준다.
DDS는 하지 않는다. 받는 것 · QoS는 전부 board_sub(Rust)와 board.xml이 정한다.

  UNO Q:  ./board_sub | python3 led.py        내장 LED 매트릭스(8x13)의 네 구역 + 글자
  PC:     board_sub | python led.py --text    글자로만

매트릭스는 MCU(STM32)가 제어한다. unoq_led 앱의 스케치가 내놓는 draw(프레임 104바이트)를
arduino-router 소켓으로 부른다(MessagePack-RPC). 깜빡임은 여기서 프레임을 바꿔 보내 만든다.
Ctrl+C는 board_sub가 받아 정리하고, 이 프로그램은 입력이 끝나면 매트릭스를 끄고 끝난다.
"""
import argparse
import os
import signal
import socket
import struct
import sys
import threading
import time

ROWS, COLS = 8, 13
# LED 0~3 = 타일 (1,1)(1,2)(2,1)(2,2) → 왼쪽 위 · 오른쪽 위 · 왼쪽 아래 · 오른쪽 아래. 구역 사이는 비워 둔다
BLOCKS = [
    (range(0, 3), range(0, 6)),
    (range(0, 3), range(7, 13)),
    (range(5, 8), range(0, 6)),
    (range(5, 8), range(7, 13)),
]
BLINK = {"blink": 0.5, "fast": 0.1}  # 켜짐 · 꺼짐 시간(초)


def render(tiles, now):
    """LED 4개의 (mode, 밝기)를 매트릭스 프레임(행 우선, 0~255)으로 바꾼다."""
    frame = bytearray(ROWS * COLS)
    for (rows, cols), (mode, level) in zip(BLOCKS, tiles):
        if mode == "on":
            # 눈은 어두운 쪽 차이에 민감해서 제곱으로 벌린다 (0.30 → 23, 0.80 → 163, 1.00 → 255)
            v = max(8, round(255 * level * level)) if level > 0 else 0
        elif mode in BLINK:
            v = 255 if int(now / BLINK[mode]) % 2 == 0 else 0
        else:
            v = 0
        for r in rows:
            for c in cols:
                frame[r * COLS + c] = v
    return bytes(frame)


def pack_call(msgid, method, frame):
    """MessagePack-RPC 요청 [0, msgid, method, [frame(bin)]]"""
    m = method.encode()
    head = b"\x94\x00\xce" + struct.pack(">I", msgid) + bytes([0xA0 | len(m)]) + m
    return head + b"\x91\xc4" + bytes([len(frame)]) + frame


def unpack(f):
    """스트림에서 MessagePack 값 하나를 읽는다 (응답 [1, msgid, error, result]를 읽는 데 쓴다)."""

    def rd(n):
        b = f.read(n)
        if len(b) != n:
            raise ConnectionError("라우터가 연결을 끊었다")
        return b

    def num(fmt):
        return struct.unpack(">" + fmt, rd(struct.calcsize(fmt)))[0]

    def seq(n):
        return [unpack(f) for _ in range(n)]

    def pairs(n):
        return {str(unpack(f)): unpack(f) for _ in range(n)}

    t = rd(1)[0]
    if t <= 0x7F:
        return t
    if t >= 0xE0:
        return t - 0x100
    if t <= 0x8F:
        return pairs(t & 0x0F)
    if t <= 0x9F:
        return seq(t & 0x0F)
    if t <= 0xBF:
        return rd(t & 0x1F).decode("utf-8", "replace")
    if t == 0xC0:
        return None
    if t in (0xC2, 0xC3):
        return t == 0xC3
    if t in (0xC4, 0xC5, 0xC6):  # bin
        return rd(num("BHI"[t - 0xC4]))
    if t in (0xC7, 0xC8, 0xC9):  # ext: 길이 + 타입 1바이트 + 데이터
        return rd(num("BHI"[t - 0xC7]) + 1)
    if t in (0xCA, 0xCB):
        return num("fd"[t - 0xCA])
    if 0xCC <= t <= 0xCF:
        return num("BHIQ"[t - 0xCC])
    if 0xD0 <= t <= 0xD3:
        return num("bhiq"[t - 0xD0])
    if 0xD4 <= t <= 0xD8:  # fixext
        return rd((1 << (t - 0xD4)) + 1)
    if t in (0xD9, 0xDA, 0xDB):
        return rd(num("BHI"[t - 0xD9])).decode("utf-8", "replace")
    if t in (0xDC, 0xDD):
        return seq(num("HI"[t - 0xDC]))
    if t in (0xDE, 0xDF):
        return pairs(num("HI"[t - 0xDE]))
    raise ValueError(f"알 수 없는 MessagePack 타입 0x{t:02x}")


class Matrix:
    """arduino-router에 붙어 스케치의 draw(frame)를 부른다. 안 되면 이유를 한 번 알리고 계속 다시 시도한다."""

    def __init__(self, path):
        self.path = path
        self.sock = self.f = None
        self.msgid = 0
        self.said = None

    def say(self, msg):
        if msg != self.said:
            self.said = msg
            print(f"매트릭스: {msg}", flush=True)

    def close(self):
        if self.sock:
            self.sock.close()
        self.sock = self.f = None

    def draw(self, frame):
        try:
            if self.sock is None:
                s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
                s.settimeout(2.0)
                try:
                    s.connect(self.path)
                except OSError:
                    s.close()
                    raise
                self.sock, self.f = s, s.makefile("rb")
            self.msgid = (self.msgid + 1) & 0xFFFFFFFF
            self.sock.sendall(pack_call(self.msgid, "draw", frame))
            while True:  # 내 요청의 응답이 올 때까지 읽는다
                msg = unpack(self.f)
                if isinstance(msg, list) and len(msg) == 4 and msg[0] == 1 and msg[1] == self.msgid:
                    break
        except PermissionError:
            self.say("소켓 권한이 없다. sudo usermod -aG arduino-router $USER 뒤 다시 로그인한다")
        except (OSError, ValueError) as e:
            self.say(f"{self.path}에 연결하지 못했다 ({e})")
        else:
            if msg[2]:
                self.say(f"draw 호출 실패 {msg[2]}. unoq_led 앱(스케치)이 실행 중인지 확인한다")
                return False
            self.say("표시 중")
            return True
        self.close()
        return False


def drive(matrix, tiles, lock, stop):
    """프레임이 바뀔 때만 보낸다. 2초마다 한 번은 다시 보내서 스케치가 다시 시작해도 복구된다."""
    sent, sent_at = None, 0.0
    while not stop.is_set():
        now = time.monotonic()
        with lock:
            frame = render(tiles, now)
        if frame != sent or now - sent_at >= 2.0:
            if matrix.draw(frame):
                sent, sent_at = frame, now
            else:
                sent = None
                stop.wait(1.0)
        stop.wait(0.05)
    matrix.draw(bytes(ROWS * COLS))  # 끝날 때 끈다
    matrix.close()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--text", action="store_true", help="매트릭스 없이 LED 상태를 글자로만 표시")
    ap.add_argument("--socket", default="/var/run/arduino-router.sock", help="arduino-router 소켓")
    a = ap.parse_args()

    n = len(BLOCKS)
    state = ["off"] * n  # 글자 표시용
    tiles = [("off", 0.0)] * n  # 매트릭스용 (mode, 밝기)
    lock, stop, worker = threading.Lock(), threading.Event(), None
    if not a.text:
        if hasattr(socket, "AF_UNIX") and os.path.exists(a.socket):
            worker = threading.Thread(target=drive, args=(Matrix(a.socket), tiles, lock, stop))
            worker.start()
        else:
            print(f"arduino-router 소켓({a.socket})이 없어 글자로만 표시한다", flush=True)

    try:
        for line in sys.stdin:
            parts = line.split()
            if len(parts) < 3 or parts[0] != "led":
                print(line, end="", flush=True)
                continue
            i, mode = int(parts[1]), parts[2]
            if not 0 <= i < n:
                continue
            level = float(parts[3]) if len(parts) > 3 else 1.0
            new = f"on {level:.2f}" if mode == "on" else mode
            if new == state[i]:
                continue
            state[i] = new
            with lock:
                tiles[i] = (mode, level)
            print("LED  " + "  ".join(f"[{k}] {s:<8}" for k, s in enumerate(state)), flush=True)
    finally:
        stop.set()
        if worker:
            worker.join()


if __name__ == "__main__":
    # 파이프로 묶여 있어 Ctrl+C가 양쪽에 간다. board_sub가 정리하며 찍는 줄까지 받고 끝내려고 무시한다
    signal.signal(signal.SIGINT, signal.SIG_IGN)
    sys.stdout.reconfigure(errors="replace")  # 한글을 못 찍는 터미널 · 리다이렉트에서도 죽지 않게
    main()
