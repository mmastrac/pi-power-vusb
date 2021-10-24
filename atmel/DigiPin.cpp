/* Name: DigiPin.c
 * Based on V-USB Arduino Examples by Philip J. Lindsay
 * Modification for the Digispark by Erik Kettenburg, Digistump LLC
 * VID/PID changed to pair owned by Digistump LLC, code modified to use pinchange int for attiny85
 * Original notice below:
 * Based on project: hid-data, example how to use HID for data transfer
 * (Uses modified descriptor and usbFunctionSetup from it.)
 * Original author: Christian Starkjohann
 * Arduino modifications by: Philip J. Lindsay
 * Creation Date: 2008-04-11
 * Tabsize: 4
 * Copyright: (c) 2008 by OBJECTIVE DEVELOPMENT Software GmbH
 * License: GNU GPL v2 (see License.txt), GNU GPL v3 or proprietary (CommercialLicense.txt)
 * This Revision: $Id: main.c 692 2008-11-07 15:07:40Z cs $
 */

/*
This example should run on most AVRs with only little changes. No special
hardware resources except INT0 are used. You may have to change usbconfig.h for
different I/O pins for USB. Please note that USB D+ must be the INT0 pin, or
at least be connected to INT0 as well.
*/

#include <Arduino.h>
#include <wiring.h>
#include <avr/eeprom.h>
#include <avr/pgmspace.h>   /* required by usbdrv.h */
#include "usbdrv.h"

#include "DigiPin.h"

bool eeprom_good = false;
uchar eeprom_poweron_delay = 0;
uchar eeprom_sequence_delay = 0;
uchar eeprom_pin_modes = 0;
uchar eeprom_pin_levels = 0;

int portMap(int port) {
  switch (port) {
    case 0:
    case 1:
    case 2:
      return port;
    case 3:
      return 5;
    default:
      return -1;
  }
}

void reset() {
  for (int i = 0; i < 6; i++) {
    if (i != 3 && i != 4) {
      pinMode(i, INPUT);
      digitalWrite(i, LOW);
    }
  }
}

void reboot(void) {
  noInterrupts(); // disable interrupts which could mess with changing prescaler
  usbDeviceDisconnect();
  _delay_ms(250);
  CLKPR = 0b10000000; // enable prescaler speed change
  CLKPR = 0; // set prescaler to default (16mhz) mode required by bootloader
  void (*ptrToFunction)(); // allocate a function pointer
  ptrToFunction = 0x0000; // set function pointer to bootloader reset vector
  (*ptrToFunction)(); // jump to reset, which bounces in to bootloader
}

void reread_eeprom() {
  eeprom_good = false;

  eeprom_poweron_delay = 0; // 0 * 250ms
  eeprom_sequence_delay = 0; // 0 * 250ms

  uchar eeprom_data[5];
  eeprom_data[0] = eeprom_read_byte((uint8_t*)0);
  eeprom_data[1] = eeprom_read_byte((uint8_t*)1);
  eeprom_data[2] = eeprom_read_byte((uint8_t*)2);
  eeprom_data[3] = eeprom_read_byte((uint8_t*)3);
  eeprom_data[4] = eeprom_read_byte((uint8_t*)4);
  eeprom_data[5] = eeprom_read_byte((uint8_t*)5);

  uchar checksum1 = eeprom_data[0] ^ eeprom_data[1] ^ eeprom_data[2] ^ eeprom_data[3];
  uchar checksum2 = eeprom_data[0] + eeprom_data[1] + eeprom_data[2] + eeprom_data[3];

  if (checksum1 != eeprom_data[4] || checksum2 != eeprom_data[5]) {
    // Bad values
    return;
  }

  eeprom_poweron_delay = eeprom_data[0];
  eeprom_sequence_delay = eeprom_data[1];
  eeprom_pin_modes = eeprom_data[2];
  eeprom_pin_levels = eeprom_data[3];
  eeprom_good = true;
}

DigiPinDevice::DigiPinDevice() {
}

void DigiPinDevice::begin() {
    cli();

    reset();
    reread_eeprom();

    usbInit();

    usbDeviceDisconnect();
    uchar   i;
    i = 0;
    while(--i){             
        _delay_ms(10);
    }

    for (uchar i = 0; i < eeprom_poweron_delay; i++) {
      _delay_ms(250);
    }

    for (int i = 0; i < 4; i++) {
      int port = portMap(i);
      if (eeprom_pin_modes & (1 << i)) {
        pinMode(port, OUTPUT);
        digitalWrite(port, (eeprom_pin_levels & (1 << i)) ? HIGH : LOW);
        for (uchar j = 0; j < eeprom_sequence_delay; j++) {
          _delay_ms(250);
        }
      } else {
        pinMode(port, INPUT);
        digitalWrite(port, (eeprom_pin_levels & (1 << i)) ? HIGH : LOW);
      }
    }

    usbDeviceConnect();

    sei();
}

void DigiPinDevice::poll() {
  usbPoll();
}

/* ------------------------------------------------------------------------- */
/* ----------------------------- USB interface ----------------------------- */
/* ------------------------------------------------------------------------- */

#ifdef __cplusplus
extern "C"{
#endif 

/* ------------------------------------------------------------------------- */

enum {
  VENDOR_RQ_SET_PINMODE = 1,
  VENDOR_RQ_DIGITAL_WRITE = 2,
  VENDOR_RQ_DIGITAL_READ = 3,
  VENDOR_RQ_ANALOG_READ = 4,
  VENDOR_RQ_EEPROM_READ = 5,
  VENDOR_RQ_EEPROM_WRITE = 6,
  VENDOR_RQ_DEBUG_SOF_COUNT = 250,
  VENDOR_RQ_DEBUG_OSCCAL = 251,
  VENDOR_RQ_DEBUG_REREAD_EEPROM = 252,
  VENDOR_RQ_DEBUG_BOOTLOADER = 253,
};

int buffer = 0;

usbMsgLen_t usbFunctionSetup(uchar data[8])
{
    usbRequest_t *rq = (usbRequest_t *)data;   // cast to structured data for parsing
    int port;
    switch(rq->bRequest){
    case VENDOR_RQ_SET_PINMODE:
      port = portMap(rq->wIndex.bytes[0]);
      if (port == -1) {
        return 0;
      }
      pinMode(port, rq->wValue.bytes[0] ? OUTPUT : INPUT);
      return 0;
    case VENDOR_RQ_DIGITAL_WRITE:
      port = portMap(rq->wIndex.bytes[0]);
      if (port == -1) {
        return 0;
      }
      digitalWrite(port, rq->wValue.bytes[0] ? HIGH : LOW);
      return 0;
    case VENDOR_RQ_DIGITAL_READ:
      port = portMap(rq->wIndex.bytes[0]);
      if (port == -1) {
        return 0;
      }
      buffer = digitalRead(port);
      usbMsgPtr = (uchar*)(void*)&buffer;
      return 4;
    case VENDOR_RQ_ANALOG_READ:
      port = portMap(rq->wIndex.bytes[0]);
      if (port == -1) {
        return 0;
      }
      buffer = analogRead(port);
      usbMsgPtr = (uchar*)(void*)&buffer;
      return 4; 
    case VENDOR_RQ_EEPROM_READ:
      buffer = eeprom_read_byte((uint8_t*)rq->wIndex.bytes[0]);
      usbMsgPtr = (uchar*)(void*)&buffer;
      return 1;
    case VENDOR_RQ_EEPROM_WRITE:
      eeprom_write_byte((uint8_t*)rq->wIndex.bytes[0], rq->wValue.bytes[0]);
      return 0;
    case VENDOR_RQ_DEBUG_SOF_COUNT:
      buffer = usbSofCount;
      usbMsgPtr = (uchar*)(void*)&buffer;
      return 1;
    case VENDOR_RQ_DEBUG_OSCCAL:
      buffer = OSCCAL;
      usbMsgPtr = (uchar*)(void*)&buffer;
      return 1;
    case VENDOR_RQ_DEBUG_REREAD_EEPROM:
      reread_eeprom();
      buffer = eeprom_good;
      usbMsgPtr = (uchar*)(void*)&buffer;
      return 1;
    case VENDOR_RQ_DEBUG_BOOTLOADER:
      cli();
      reset();
      reboot();
      return 0;
    }
    // Ignore unknown requests
    return 0;
}
#ifdef __cplusplus
} // extern "C"
#endif

uchar usbFunctionRead(uchar *data, uchar len) {
  return 0;
}

uchar usbFunctionWrite(uchar *data, uchar len) {
  return 0;
}

DigiPinDevice DigiPin = DigiPinDevice();

