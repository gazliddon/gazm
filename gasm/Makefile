BIN_DIR := bin
DEFENDER_DIR := ../../defender
DEFENDER_BIN_DIR := $(DEFENDER_DIR)/bin
DEFENDER_SRC_DIR := $(DEFENDER_DIR)/gasmsrc
RED_LABEL := $(DEFENDER_DIR)/redlabel

ASM := ../target/release/gasm --star-comments --max-errors 50 $(DEFENDER_SRC_DIR)/macros.68

all: gasm directories roms
	@echo done

gasm:
	cargo build --release

directories:
	@mkdir -p $(BIN_DIR)
	@echo Made out dir

roms : intermediates
	@echo TODO build ROMS

intermediates : $(BIN_DIR)/defa7-defb6-amode1.o $(BIN_DIR)/samexap7.o $(BIN_DIR)/blk71.o $(BIN_DIR)/roms.o
	@echo Built intermediates

# Build amode1 # The equivalent of: RASM PHR2,DEFA2,DEFB2,AMODE0;-X (ELSE CREF SYMBOL OVERFLOW)
DEFA7_DEFB6_AMODE1_SRC = phr6.src defa7.src defb6.src amode1.src
DEFA7_DEFB6_AMODE1_SRC_FULL := $(foreach file,$(DEFA7_DEFB6_AMODE1_SRC), $(DEFENDER_SRC_DIR)/$(file))
$(BIN_DIR)/defa7-defb6-amode1.o : $(DEFA7_DEFB6_AMODE1_SRC_FULL)
	@echo Building $@ from $(DEFA7_DEFB6_AMODE1_SRC_FULL)
	@$(ASM) $(DEFA7_DEFB6_AMODE1_SRC_FULL) \
	--as6809-sym $(DEFENDER_BIN_DIR)/$(notdir $(basename $@)).sym \
	--bin-ref $(DEFENDER_BIN_DIR)/$(notdir $(basename $@)).o 0000 e000 2000 \
	--bin-ref $(RED_LABEL)/defend.1 0000 800 d000 \
	--bin-ref $(RED_LABEL)/defend.2 0000 1000 e000 \
	--bin-ref $(RED_LABEL)/defend.4 0000 800 d800 \
	--bin-ref $(RED_LABEL)/defend.9 0000 800 2000 \
	--bin-ref $(RED_LABEL)/defend.12 0000 800 2800 \
	--bin-ref $(DEFENDER_BIN_DIR)/$(notdir $(basename $@)).o aee9 117 2ee9 \
	--write-bin $@ 2000 e000 \
	--write-bin $(BIN_DIR)/defend.1 d000 800 \
	--write-bin $(BIN_DIR)/defend.2 e000 1000 \
	--write-bin $(BIN_DIR)/defend.4 d800 800 \
	--write-bin $(BIN_DIR)/defend.9 2000 800 \
	--write-bin $(BIN_DIR)/defend.12 2800 800 \
	--write-bin x.bin 2ee9 117

SAMEXAP7_SRC = phr6.src samexap7.src 
SAMEXAP7_SRC_FULL := $(foreach file,$(SAMEXAP7_SRC), $(DEFENDER_SRC_DIR)/$(file))
$(BIN_DIR)/samexap7.o : $(SAMEXAP7_SRC_FULL)
	@echo Building $@ from $?
	@$(ASM) $? \
	--as6809-sym $(DEFENDER_BIN_DIR)/$(notdir $(basename $@)).sym \
	--bin-ref $(DEFENDER_BIN_DIR)/$(notdir $(basename $@)).o 0 2f8 fc60 \
	--write-bin $@ fc60 2f8


# Build amode1 # The equivalent of: RASM PHR2,DEFA2,DEFB2,AMODE0;-X (ELSE CREF SYMBOL OVERFLOW)
BLK71_SRC = blk71.src
BLK71_SRC_FULL := $(foreach file,$(BLK71_SRC), $(DEFENDER_SRC_DIR)/$(file))
$(BIN_DIR)/blk71.o : $(BLK71_SRC_FULL)
	@echo Building $@ from $?
	@$(ASM) $? \
	--as6809-sym $(DEFENDER_BIN_DIR)/$(notdir $(basename $@)).sym \
	--bin-ref $(DEFENDER_BIN_DIR)/$(notdir $(basename $@)).o 0000 772 c000 \
	--write-bin $@ c000 772

ROMS_SRC := mess0.src romf8.src romc0.src romc8.src
ROMS_SRC_FULL := $(foreach file,$(ROMS_SRC), $(DEFENDER_SRC_DIR)/$(file))

$(BIN_DIR)/roms.o : $(ROMS_SRC_FULL)
	@echo Building $@ from $?
	@$(ASM) $? \
	--as6809-sym $(DEFENDER_BIN_DIR)/$(notdir $(basename $@)).sym \
	--bin-ref $(DEFENDER_BIN_DIR)/$(notdir $(basename $@)).o 0000 e000 2000 \
	--bin-ref $(RED_LABEL)/defend.8 0000 800 2000 \
	--bin-ref $(RED_LABEL)/defend.10 0000 800 c800 \
	--bin-ref $(RED_LABEL)/defend.7 0000 800 c000 \
	--bin-ref $(RED_LABEL)/defend.11 0000 800 2800 \
	--write-bin $@ 2000 e000 \
	--write-bin $(BIN_DIR)/defend.8 2000 800 \
	--write-bin $(BIN_DIR)/defend.10 c800 800 \
	--write-bin $(BIN_DIR)/defend.7 c000 800 \
	--write-bin $(BIN_DIR)/defend.11.a 2800 800 

clean:
	@rm -rf $(BIN_DIR)
	@echo Cleaned

ifeq ("x", "y")
defend.1
	bin/defa7-defb6-amode1.o 0xb000,0x000,0x800

defend.2
	bin/defa7-defb6-amode1.o 0xc000,0x000,0x1000

defend.3
	bin/defa7-defb6-amode1.o,0xd000,0x0000,0x0c60
	bin/samexap7.o,0x0000,0x0c60,0x02f8
	bin/defa7-defb6-amode1.o,0xdf57,0x0f58,0x0230

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
