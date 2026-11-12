# Leica DISTO BLE

The native implementation follows the `leica-disto-ble` skill, kept outside
this repository. That skill records measurements
from the owner's E7500i/D510 on 2026-10-23. The Rust connection itself has not
been tested with hardware.

## Connect and draw

1. Enable Bluetooth on the DISTO and keep it awake. Windows pairing and a COM
   port are unnecessary for the E7500i/D510 BLE connection.
2. Choose Tools > Connect, the Bluetooth status button, or the Bluetooth command
   in command search. Automatic discovery shows devices advertising the Leica
   service or a DISTO name. Options > Hardware > Manual lists all nearby BLE
   devices; its legacy COM Port field does not select a serial connection.
3. Select the DISTO and Connect. The chip changes only after service discovery
   and subscription succeed. An unsupported device reports the reason.
4. Start an exterior or interior wall at an anchor, then press Measure on the
   DISTO. Standard mode replaces the numeric drawing buffer; use a direction key
   and Enter just as with a typed length. Continuous mode commits each reading
   along the previous wall's heading (the existing Enter default for a first
   wall), with one ordinary undo step per reading.
5. Choose Connect again to disconnect. Cancel/Escape dismisses a pending
   connection silently. Physical disconnection clears the chip. Old scan,
   connection and measurement events cannot reactivate a previous session.

Rounding applies in the sketch's selected units, then converts to the drawing's
internal feet. Measurements do not edit a drawing while a dialog is open, while
editing tracked numeric fields, outside wall-drawing mode, or before an anchor
has been set. The instrument's own unit setting does not change the wire format.

## Protocol and validation

- Service: `3ab10100-f831-4395-b29d-570977d5bf94`.
- Distance: `3ab10101-f831-4395-b29d-570977d5bf94`, read/indicate on E7500i.
- Subscribe to indications or notifications; consume only the distance UUID.
  Do not interpret unit/tilt/temperature values as distances or write commands.
- Payload: first four bytes, little-endian float32 metres, positive and finite.
  Captured `cf f7 db 3f` is approximately 1.7185 m / 5.6381 ft.
- Tests cover captured-byte decoding, invalid payloads, all rounding steps,
  numeric-buffer formatting, Standard and Continuous drawing, metric rounding,
  undo, modal guards, cancellation, disconnect and stale session events. Test
  builds never access the Bluetooth radio.

Before claiming hardware acceptance, connect the native executable to the
E7500i, measure a known length, verify Standard and Continuous input and rounding,
then disconnect/reconnect and power off the instrument. Repeat on other models
before claiming compatibility. A weak adapter signal can cause long connection
times; the skill documents using a USB Bluetooth adapter on a short cable.
Legacy classic-Bluetooth SPP/COM instruments need a separate serial transport.
