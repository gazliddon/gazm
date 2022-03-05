BIN_DIR := bin
DEFENDER_DIR := ../../defender
DEFENDER_BIN_DIR := $(DEFENDER_DIR)/bin
DEFENDER_SRC_DIR := $(DEFENDER_DIR)/gasmsrc
RED_LABEL := $(DEFENDER_DIR)/redlabel

ASM := ../target/debug/gasm --star-comments --max-errors 50 --mem-size 94208\
	   $(DEFENDER_SRC_DIR)/macros.68 \
	   $(DEFENDER_SRC_DIR)/memory.src


all: gasm directories roms
	@echo done

gasm:
	cargo build

directories:
	@mkdir -p $(BIN_DIR)
	@echo Made out dir

roms : intermediates
	@echo TODO build ROMS

intermediates : $(BIN_DIR)/defa7-defb6-amode1.o  $(BIN_DIR)/roms.o \
	$(BIN_DIR)/defend.11 \
	$(BIN_DIR)/defend.6
	@echo Built intermediates

DEFA7_DEFB6_AMODE1_SRC = phr6.src defa7.src defb6.src samexap7.src amode1.src
DEFA7_DEFB6_AMODE1_SRC_FULL := $(foreach file,$(DEFA7_DEFB6_AMODE1_SRC), $(DEFENDER_SRC_DIR)/$(file))
$(BIN_DIR)/defa7-defb6-amode1.o : $(DEFA7_DEFB6_AMODE1_SRC_FULL)
	@echo Building $@ from $(DEFA7_DEFB6_AMODE1_SRC_FULL)
	@$(ASM) $(DEFA7_DEFB6_AMODE1_SRC_FULL) \
	--as6809-sym $(DEFENDER_BIN_DIR)/$(notdir $(basename $@)).sym \
	-s $(BIN_DIR)/$(notdir $(basename $@)).sym \
	-v \
	\
	--bin-ref $(RED_LABEL)/defend.1 0000 800 d000 \
	--bin-ref $(RED_LABEL)/defend.4 0000 800 d800 \
	--bin-ref $(RED_LABEL)/defend.2 0000 1000 e000 \
	--bin-ref $(RED_LABEL)/defend.3 0000 1000 f000 \
	--bin-ref $(RED_LABEL)/defend.9 0000 800 10000 \
	--bin-ref $(RED_LABEL)/defend.12 0000 800 10800 \
	\
	--write-bin $(BIN_DIR)/defend.1 d000 800 \
	--write-bin $(BIN_DIR)/defend.4 d800 800 \
	--write-bin $(BIN_DIR)/defend.2 e000 1000 \
	--write-bin $(BIN_DIR)/defend.3 f000 1000 \
	--write-bin $(BIN_DIR)/defend.9 10000 800 \
	--write-bin $(BIN_DIR)/defend.12 10800 800 \
	--write-bin $@ 0 17000

$(BIN_DIR)/defend.6 : $(DEFENDER_SRC_DIR)/defend6.src
	@$(ASM) $? \
	--write-bin $@ 16000 800

$(BIN_DIR)/defend.11 : $(DEFENDER_SRC_DIR)/defend11.src
	@$(ASM) $? \
	--bin-ref $(RED_LABEL)/defend.11 0 800 11800 \
	--write-bin $@ 11800 800

ROMS_SRC := phr6.src mess0.src romc0.src romc8.src blk71.src
ROMS_SRC_FULL := $(foreach file,$(ROMS_SRC), $(DEFENDER_SRC_DIR)/$(file))
$(BIN_DIR)/roms.o : $(ROMS_SRC_FULL)
	@echo Building $@ from $?
	@$(ASM) $? \
	--bin-ref $(RED_LABEL)/defend.8 0 800 11000 \
	--bin-ref $(RED_LABEL)/defend.11 0 800 11800 \
	--bin-ref $(RED_LABEL)/defend.7 0 800 12000 \
	--bin-ref $(RED_LABEL)/defend.10  0 800 12800 \
	--bin-ref $(RED_LABEL)/defend.6  0 800 16000 \
	--as6809-sym $(DEFENDER_BIN_DIR)/$(notdir $(basename $@)).sym \
	--write-bin $@ 0 17000 \
	--write-bin $(BIN_DIR)/defend.8 11000 800 \
	--write-bin $(BIN_DIR)/defend.7 12000 800 \
	--write-bin $(BIN_DIR)/defend.10 12800 800

clean:
	@rm -rf $(BIN_DIR)
	@echo Cleaned

cmp:
	cmp  $(RED_LABEL)/defend.1 $(BIN_DIR)/defend.1
	cmp  $(RED_LABEL)/defend.2 $(BIN_DIR)/defend.2
	cmp  $(RED_LABEL)/defend.3 $(BIN_DIR)/defend.3
	cmp  $(RED_LABEL)/defend.4 $(BIN_DIR)/defend.4
	cmp  $(RED_LABEL)/defend.7 $(BIN_DIR)/defend.7
	cmp  $(RED_LABEL)/defend.8 $(BIN_DIR)/defend.8
	cmp  $(RED_LABEL)/defend.9 $(BIN_DIR)/defend.9
	cmp  $(RED_LABEL)/defend.10 $(BIN_DIR)/defend.10
	cmp  $(RED_LABEL)/defend.12 $(BIN_DIR)/defend.12
	cmp  $(RED_LABEL)/defend.11 $(BIN_DIR)/defend.11
	cmp  $(RED_LABEL)/defend.6 $(BIN_DIR)/defend.6

ifeq ("x", "y")

defend.6
	blk71


defend.1
	bin/defa7-defb6-amode1.o 0xb000,0x000,0x800

defend.2
	bin/defa7-defb6-amode1.o 0xc000,0x000,0x1000

defend.3
	bin/defa7-defb6-amode1.o,0xd000,0x0000,0x0c60
	bin/samexap7.o,0x0000,0x0c60,0x02f8
	bin/defa7-defb6-amode1.o,0xdf57,0x0f58,0x0230

	org 0
	incbin "bin/defa7-defb6-amode1.o",0xd000,0x0c60
	org $c60
	incbin "bin/samexap7.o",0x0000,0x02f8
	org $f58
	incbin "bin/defa7-defb6-amode1.o",0xdf57,0x0230


defend.4
	bin/defa7-defb6-amode1.o,0xb800,0x0000,0x0800

defend.6
	bin/blk71.o,0x0001,0x0000,0x0772,\
	bin/roms.o,0xa779,0x0778,0x0088,

defend.7
	bin/roms.o,0xa000,0x0000,0x0800,"roms"
defend.8
	bin/roms.o,0x0009,0x0000,0x0800,"roms"

defend.9
	bin/defa7-defb6-amode1.o,0x0000,0x0000,0x0800

defend.10
	bin/roms.o,0xa801,0x0000,0x0800,

defend.11
	bin/roms.o,0x0801,0x0000,0x0800,
	src/unknown.bin,0x0001,0x0450,0x0800,

defend.12
	bin/defa7-defb6-amode1.o,0x0800,0x0000,0x0800,
	bin/defa7-defb6-amode1.o,0xaeeb,0x06e9,0x0117,"amode tail"

endif
