### Corolla-TCU
Piggyback TCU for Toyota Corolla ZZE122 (03-08)

### Fixed Development Stages
1. Theory
2. Analyzing
3. Prototyping
4. Testing
5. Finalizing

***Stage 1: Theory***
- ~~Research solenoid operation (in general)~~
- ~~A245E Transmission Resources~~
- ~~Design Initial Testing Development Hardware~~

***Stage: 2 - Analyzing***

Using the Development Hardware with ESP32:
- ~~Digital Inputs (Brakes, Gear Lever Position, Inhibitor Switch, OD Switch/Lamp)~~
- ~~PWM and Analog Inputs (Solenoid Operation, VSS, RPM, TPS)~~
- ~~Data logging and replay~~
  - ~~Determine PWM range~~
  - ~~Determine relationship between inputs and solenoid operation~~

**Current Stage: 3  Prototyping**

Using Kicad:
- Design a ESP32-S3 prototype board using the schematics from the [ESP32-S3-DevKitC-1 v1.1](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/hw-reference/esp32s3/user-guide-devkitc-1.html#esp32-s3-devkitc-1-v1-1)