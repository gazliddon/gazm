# Gazm - 6809 Assembler Release Notes

## 0.9.2

### Done
* Remove stargate source and binaries
* Add CSUM checks for SG compilation
* Add sound rom compilation to stargate

### Todo
* Make tokenize CPU agnostic
* Endian agnosticism in Binary struct (can I use a memblock?)
* Add 6502
* Find a way to separate and persist CPU specific data in the Assembler class
    * Specifically for SetDp - Dp storage on 6809
    * Remove SetDp warnings
* Collect warnings for binary mismatches for non opocode binary writing
* ast.rs TODO trying to import non existant labels should be an error
* tokenize.rs TODO: BUG Replace parent with incstack
* error.rs // TODO: Remove6809

## 0.9.16
* Added 6800 assembly
* Can assembly stargate sound rom
* Midway through refactoring binary errors
* Still assembles stargate
* LSP removed (for now)
    * Needs to be able to work multi CPU

## 0.9.15
* Replacement front end
* Takes a step back in handling errors

## 0.9.13
* Work started updating LSP server
* Added cpu as a config item

## 0.9.12
* Fixed: `bsz args` were parsed as `bsz` {val} {count} rather than `bsz` {count} {val}

## 0.9.11
* Fixed failure to assemble opcodes with operands containing references to current PC "*"

