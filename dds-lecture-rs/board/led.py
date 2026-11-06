#!/usr/bin/env python3
"""board_sub 출력에서 'led N on 밝기|off|blink|fast' 줄만 LED로 옮기고 나머지 줄은 그대로 보여 준다.
DDS는 하지 않는다. 받는 것 · QoS는 전부 board_sub(Rust)와 board.xml이 정한다.

  Pi:  ./board_sub | python3 led.py --pins 17,27,22,23
  PC:  board_sub | python led.py --text
"""
import argparse
import sys


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pins", default="17,27,22,23", help="LED 0~3의 BCM 핀 번호")
    ap.add_argument("--text", action="store_true", help="GPIO 없이 LED 상태를 글자로 표시")
    a = ap.parse_args()
    pins = [int(p) for p in a.pins.split(",")]

    leds = None
    if not a.text:
        try:
            from gpiozero import PWMLED

            leds = [PWMLED(p) for p in pins]
        except Exception as e:
            print(f"GPIO를 쓸 수 없어 글자로 표시한다: {e}", flush=True)

    state = ["off"] * len(pins)
    for line in sys.stdin:
        parts = line.split()
        if len(parts) < 3 or parts[0] != "led":
            print(line, end="", flush=True)
            continue
        i, mode = int(parts[1]), parts[2]
        if not 0 <= i < len(pins):
            continue
        level = float(parts[3]) if len(parts) > 3 else 1.0
        new = f"on {level:.2f}" if mode == "on" else mode
        if new == state[i]:
            continue
        state[i] = new
        if leds:
            led = leds[i]
            if mode == "on":
                led.value = level
            elif mode == "off":
                led.off()
            elif mode == "blink":
                led.blink(on_time=0.5, off_time=0.5)
            elif mode == "fast":
                led.blink(on_time=0.1, off_time=0.1)
        print("LED  " + "  ".join(f"[{n}] {s:<8}" for n, s in enumerate(state)), flush=True)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
