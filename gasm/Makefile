BIN_DIR := bin
DEFENDER_DIR := ../../defender
DEFENDER_BIN_DIR := $(DEFENDER_DIR)/bin
DEFENDER_SRC_DIR := $(DEFENDER_DIR)/gasmsrc

ASM := cargo run --bin gasm -- -v --star-comments --max-errors 50 $(DEFENDER_SRC_DIR)/macros.68

all: directories roms
	@echo done

directories:
	@mkdir -p $(BIN_DIR)
	@echo Made out dir

defender_1 = phr6.src defa7.src defb6.src amode1.src

defender_1_src = $(foreach file,$(defender_1), $(DEFENDER_SRC_DIR)/$(file))

$(BIN_DIR)/defender.1 : $(defender_1_src)
	$(ASM) $(defender_1_src) \
	--as6809-sym $(DEFENDER_BIN_DIR)/defa7-defb6-amode1.sym \
	--as6809-bin $(DEFENDER_BIN_DIR)/defa7-defb6-amode1.o

roms : $(BIN_DIR)/defender.1


