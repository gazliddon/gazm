BIN := ../bin/osx

AS := $(BIN)/lwasm
LN := $(BIN)/lwlink
OUT := out

SRC := $(wildcard *.s)

all : dirs code
	@echo Done

code : $(OUT)/all.bin $(OUT)/all.syms makefile
	@echo Built code

dirs : $(OUT)
	@echo Made directories

$(OUT) :
	@mkdir -p $(OUT)

$(OUT)/all.bin $(OUT)/all.syms $(OUT)/all.map : $(SRC)
	@$(AS) -9 -r -o $(OUT)/all.bin all.s -s -l$(OUT)/all.syms --map=$(OUT)/all.map
	@ls -lt $(OUT)/all.bin
	@echo Compiled all.s

clean :
	@rm -rf $(OUT)
	@echo cleaned

