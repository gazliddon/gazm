---
short: Expected to see an index register
---

## Expected An Index Register

This addressing mode requires a valid register and that register must be usable as an index register

```
    lda 100,z     ; Illegal, z isn't a valid 6809 register
    lda 100,a     ; Illegal, a cannot be used as an index register
    lda 100,U     ; Legal, you can use u as an index register
```

These are the regitsers you can use as index registers

| Register | Bits | Notes                    | Index? |
|:--------:|:----:|:-------------------------|:------:|
| U        | 16   | User stack pointer       | ✅     |
| S        | 16   | System stack pointer     | ✅     |
| X        | 16   | X Index register         | ✅     |
| Y        | 16   | Y index register         | ✅     |
| PC       | 16   | Program counter          | ✅     |
