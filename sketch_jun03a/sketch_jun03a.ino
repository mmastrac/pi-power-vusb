#include <DigiPin.h>

void setup() {
  DigiPin.begin();
}

void loop() {
  DigiPin.poll();
}
