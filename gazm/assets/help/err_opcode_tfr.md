## TFR

Transfer register to register

r0 → r1

| **Source Form** | **Addressing Mode** | **Opcode** | **Cycles** | **Byte Count** |
|:---------------:|:-------------------:|:----------:|:----------:|:--------------:|
| TFR r0,r1       | IMMEDIATE           | 1F         | 6          | 2              |

TFR copies the contents of a source register into a destination register. None of the
Condition Code flags are affected unless CC is specified as the destination register.
The TFR instruction can be used to alter the flow of execution by specifying PC as the
destination register.

Any of the 6809 registers may be specified as either the source, destination or both.
Specifying the same register for both the source and destination produces an instruction
which, like NOP, has no effect.

The table below explains how the destination register is affected when the source and
destination sizes are different. This behavior differs from the 6309 implementation.

---

| **Source Form** | **Addressing Mode** | **Opcode** | **Cycles** | **Byte Count** |
|:---------------:|:-------------------:|:----------:|:----------:|:--------------:|
| TSTA            | INHERENT            | 4D         | 2 / 1      | 1              |
| TSTB            | INHERENT            | 5D         | 2 / 1      | 1              |
| TSTD            | INHERENT            | 104D       | 3 / 2      | 2              |
| TSTE            | INHERENT            | 114D       | 3 / 2      | 2              |
| TSTF            | INHERENT            | 115D       | 3 / 2      | 2              |
| TSTW            | INHERENT            | 105D       | 3 / 2      | 2              |

