# COBALT Engine - GnuCOBOL Build
COBC = cobc
COBC_FLAGS = -x -free -I copybooks
SRC = $(wildcard src/*.cbl)
TARGET = dist/cobalt

.PHONY: all clean

all: $(TARGET)

$(TARGET): $(SRC) $(wildcard copybooks/*.cpy)
	@mkdir -p dist
	$(COBC) $(COBC_FLAGS) -o $(TARGET) $(SRC)

clean:
	rm -rf dist/
