// 보드 실습 · Display의 LED: UNO Q 내장 LED 매트릭스(8x13)에 led.py가 보낸 프레임을 그대로 그린다.
//   led.py → arduino-router → draw(프레임 104바이트, 행 우선, 밝기 0~255)
//   무엇을 그릴지(타일 구역 · 밝기 · 깜빡임)는 전부 led.py가 정한다. 이 스케치는 고칠 일이 없다.
#include <Arduino_RouterBridge.h>
#include <Arduino_LED_Matrix.h>
#include <vector>

Arduino_LED_Matrix matrix;

const size_t FRAME_SIZE = 8 * 13;
uint8_t frame[FRAME_SIZE] = {0};

// led.py가 부른다. Bridge 콜백은 loop와 다른 스레드라서 여기서는 버퍼만 바꾼다
void draw(std::vector<uint8_t> next) {
  memcpy(frame, next.data(), min(next.size(), FRAME_SIZE));
}

void setup() {
  matrix.begin();
  matrix.setGrayscaleBits(8);  // 밝기 0~255
  matrix.clear();

  Bridge.begin();
  Bridge.provide("draw", draw);
}

void loop() {
  matrix.draw(frame);
  delay(10);
}
