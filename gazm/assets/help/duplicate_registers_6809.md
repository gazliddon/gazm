---
short: You can't have duplicate registers in a register list
---

## Duplicate Registers in Register List

For opcodes that take a register list as an argument, each register must only appear once.

```
    pshs x,x,y    ; Illegal, the x register appears twice
    pshs x,y      ; Legal, each register is unique
```

