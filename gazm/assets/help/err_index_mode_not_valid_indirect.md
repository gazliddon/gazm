---
short: This indexing mode is incompatible with indirect addressing
---

## Indexing Mode incompatible With Indirect Address

Not all 6809 addressing modes are compatible with indirect addressing.

Single post increment and single Pre-decrement are both disallowed

```
    lda [,y+]           ; Error, this indexing mode is illegal with indirect addressing
    lda [,y++]          ; Ok, double post increment is valid
```

| **Addressing Mode**   | **Example** | **Indirect** |
|:----------------------|:------------|:------------|
| Constant offset       | lda 100,x   | lda [100,x]  |
| Post increment        | lda ,y+     | ❌           |
| Double post increment | lda ,y++    | lda [,y++]   |
| Pre-decrement         | lda ,-x     | ❌           |
| Double Pre-decrement  | lda ,--x    | lda [,--x]   |
| Zero                  | lda ,x      | lda [,x]     |
| Add B                 | lda b,x     | lda [b,x]    |
| Add A                 | lda a,x     | lda [a,x]    |
| Add D                 | lda d,x     | lda [d,x]    |
| PC Offset             | lda 10,pc   | lda [10,pc]  |

