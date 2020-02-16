BIN := ../bin/osx
AS := $(BIN)/lwasm
OUT := out

all : dirs code
	@echo Done

code : $(OUT)/demo.syms $(OUT)/test.syms makefile
	@echo Built code

$(OUT)/demo.syms : $(wildcard demo/*.68)
$(OUT)/test.syms : $(wildcard test/*.68)

dirs : $(OUT)
	@echo Made directories

$(OUT) :
	@mkdir -p $(OUT)

$(OUT)/%.syms  : %/all.68
	$(AS) -9 -r $*/all.68 -s -l$@ -I.
	@echo Created $@

clean :
	@rm -rf $(OUT)
	@echo cleaned

