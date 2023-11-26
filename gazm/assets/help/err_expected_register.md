---
short: Expected to see a register
---

## Expected a Register

A valid register was expected. These are the valid registers for the 6809.

Gazm allows registers to be either lower or upper case 

| Register | Bits | Notes                    | Index? |
|:--------:|:----:|:-------------------------|:------:|
| A        | 8    | A accumulator            |        |
| B        | 8    | B accumulator            |        |
| D        | 16   | A + D registers combined |        |
| U        | 16   | User stack pointer       | ✅     |
| S        | 16   | System stack pointer     | ✅     |
| X        | 16   | X Index register         | ✅     |
| Y        | 16   | Y index register         | ✅     |
| CC       | 8    | Flags                    |        |
| PC       | 16   | Program counter          | ✅     |
