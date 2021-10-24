/*
 * Based on Obdev's AVRUSB code and under the same license.
 *
 * TODO: Make a proper file header. :-)
 */
#ifndef __DigiPin_h__
#define __DigiPin_h__

#include <avr/pgmspace.h>
#include <avr/interrupt.h>
#include <string.h>
#include "usbdrv.h"

typedef uint8_t byte;

#include <util/delay.h>     /* for _delay_ms() */

class DigiPinDevice {
 public:
  DigiPinDevice ();

  void begin();
  void poll();
};

extern DigiPinDevice DigiPin;

#endif // __DigiPin_h__
