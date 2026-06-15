PACKAGE := pactrl
BIN_NAME ?= pctrl
PROFILE ?= release
TARGET_DIR := target/$(PROFILE)
CARGO_BUILD_FLAGS :=

ifeq ($(PROFILE),release)
CARGO_BUILD_FLAGS := --release
endif

ifeq ($(OS),Windows_NT)
EXE_EXT := .exe
DEFAULT_PREFIX := $(USERPROFILE)/.cargo/bin
MKDIR := if not exist "$(subst /,\,$(DESTDIR)$(PREFIX))" mkdir "$(subst /,\,$(DESTDIR)$(PREFIX))"
COPY := copy /Y "$(subst /,\,$(TARGET_DIR)/$(PACKAGE)$(EXE_EXT))" "$(subst /,\,$(DESTDIR)$(PREFIX)/$(BIN_NAME)$(EXE_EXT))"
REMOVE := if exist "$(subst /,\,$(DESTDIR)$(PREFIX)/$(BIN_NAME)$(EXE_EXT))" del /F /Q "$(subst /,\,$(DESTDIR)$(PREFIX)/$(BIN_NAME)$(EXE_EXT))"
else
EXE_EXT :=
DEFAULT_PREFIX := $(HOME)/.cargo/bin
MKDIR := mkdir -p "$(DESTDIR)$(PREFIX)"
COPY := cp "$(TARGET_DIR)/$(PACKAGE)$(EXE_EXT)" "$(DESTDIR)$(PREFIX)/$(BIN_NAME)$(EXE_EXT)"
REMOVE := rm -f "$(DESTDIR)$(PREFIX)/$(BIN_NAME)$(EXE_EXT)"
endif

PREFIX ?= $(DEFAULT_PREFIX)

.PHONY: all build check fmt test install uninstall clean help

all: build

build:
	cargo build $(CARGO_BUILD_FLAGS)

check:
	cargo check

fmt:
	cargo fmt

test:
	cargo test

install: build
	$(MKDIR)
	$(COPY)

uninstall:
	$(REMOVE)

clean:
	cargo clean

help:
	@echo Targets:
	@echo   make build      Build release binary
	@echo   make check      Run cargo check
	@echo   make fmt        Format Rust code
	@echo   make test       Run tests
	@echo   make install    Install binary as $(BIN_NAME)
	@echo   make uninstall  Remove installed binary
	@echo   make clean      Remove Cargo build output
	@echo Variables:
	@echo   PREFIX=path     Install directory, default: $(DEFAULT_PREFIX)
	@echo   BIN_NAME=name   Installed command name, default: pctrl
	@echo   PROFILE=release Cargo profile, default: release
