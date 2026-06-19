PACKAGE := pathctrl
BIN_NAME ?= pathctrl
PROFILE ?= release
TARGET_DIR := target/$(PROFILE)
CARGO ?= cargo
CARGO_BUILD_FLAGS :=

ifeq ($(PROFILE),release)
CARGO_BUILD_FLAGS := --release
endif

ifeq ($(OS),Windows_NT)
EXE_EXT := .exe
DEFAULT_PREFIX := $(USERPROFILE)/.cargo/bin
PREFIX ?= $(DEFAULT_PREFIX)
MKDIR := if not exist "$(subst /,\,$(DESTDIR)$(PREFIX))" mkdir "$(subst /,\,$(DESTDIR)$(PREFIX))"
COPY := copy /Y "$(subst /,\,$(TARGET_DIR)/$(PACKAGE)$(EXE_EXT))" "$(subst /,\,$(DESTDIR)$(PREFIX)/$(BIN_NAME)$(EXE_EXT))"
REMOVE := if exist "$(subst /,\,$(DESTDIR)$(PREFIX)/$(BIN_NAME)$(EXE_EXT))" del /F /Q "$(subst /,\,$(DESTDIR)$(PREFIX)/$(BIN_NAME)$(EXE_EXT))"
else
EXE_EXT :=
DEFAULT_APPDIR := /opt/$(PACKAGE)
DEFAULT_BINDIR := /usr/local/bin
APPDIR ?= $(DEFAULT_APPDIR)
BINDIR ?= $(DEFAULT_BINDIR)
LEGACY_APPDIR := /opt/pactrl
LEGACY_TARGET := $(LEGACY_APPDIR)/pactrl
LEGACY_BIN_NAMES := pctrl pactrl
MKDIR = mkdir -p "$(DESTDIR)$(APPDIR)" "$(DESTDIR)$(BINDIR)"
COPY = install -m 755 "$(TARGET_DIR)/$(PACKAGE)$(EXE_EXT)" "$(DESTDIR)$(APPDIR)/$(PACKAGE)$(EXE_EXT)"
LINK = ln -sfn "$(APPDIR)/$(PACKAGE)$(EXE_EXT)" "$(DESTDIR)$(BINDIR)/$(BIN_NAME)$(EXE_EXT)"
REMOVE = rm -f "$(DESTDIR)$(BINDIR)/$(BIN_NAME)$(EXE_EXT)" "$(DESTDIR)$(APPDIR)/$(PACKAGE)$(EXE_EXT)"
RMDIR = rmdir "$(DESTDIR)$(APPDIR)" 2>/dev/null || true
endif

.PHONY: all build check fmt test install uninstall uninstall-legacy clean help

all: build

build:
	$(CARGO) build $(CARGO_BUILD_FLAGS)

check:
	$(CARGO) check

fmt:
	$(CARGO) fmt

test:
	$(CARGO) test

ifeq ($(OS),Windows_NT)
install: build
else
install: build-for-install
endif
	$(MKDIR)
	$(COPY)
ifneq ($(OS),Windows_NT)
	$(LINK)
endif

ifneq ($(OS),Windows_NT)
build-for-install:
	@if command -v "$(CARGO)" >/dev/null 2>&1; then \
		$(CARGO) build $(CARGO_BUILD_FLAGS); \
	elif [ -x "$(TARGET_DIR)/$(PACKAGE)$(EXE_EXT)" ]; then \
		echo "cargo not found; installing existing $(TARGET_DIR)/$(PACKAGE)$(EXE_EXT)"; \
	else \
		echo "cargo not found and $(TARGET_DIR)/$(PACKAGE)$(EXE_EXT) does not exist."; \
		echo "Run: make build"; \
		echo "Then: sudo make install"; \
		exit 1; \
	fi
endif

uninstall:
	$(REMOVE)
ifneq ($(OS),Windows_NT)
	$(RMDIR)
endif

ifneq ($(OS),Windows_NT)
uninstall-legacy:
	@for name in $(LEGACY_BIN_NAMES); do \
		link="$(DESTDIR)$(BINDIR)/$$name"; \
		if [ -L "$$link" ] && [ "$$(readlink "$$link")" = "$(LEGACY_TARGET)" ]; then \
			rm -f "$$link"; \
			echo "Removed legacy link $$link"; \
		fi; \
	done
	@rm -f "$(DESTDIR)$(LEGACY_TARGET)"
	@rmdir "$(DESTDIR)$(LEGACY_APPDIR)" 2>/dev/null || true
endif

clean:
	$(CARGO) clean

help:
	@echo Targets:
	@echo   make build      Build release binary
	@echo   make check      Run cargo check
	@echo   make fmt        Format Rust code
	@echo   make test       Run tests
	@echo   make install    Install binary as $(BIN_NAME)
	@echo   make uninstall  Remove installed binary
ifneq ($(OS),Windows_NT)
	@echo   make uninstall-legacy  Remove old pctrl/pactrl links from this project
endif
	@echo   make clean      Remove Cargo build output
	@echo Variables:
ifeq ($(OS),Windows_NT)
	@echo   PREFIX=path     Install directory, default: $(DEFAULT_PREFIX)
else
	@echo   APPDIR=path     Application directory, default: $(DEFAULT_APPDIR)
	@echo   BINDIR=path     Symlink directory, default: $(DEFAULT_BINDIR)
endif
	@echo   BIN_NAME=name   Installed command name, default: pathctrl
	@echo   CARGO=path      Cargo command, default: cargo
	@echo   PROFILE=release Cargo profile, default: release
ifneq ($(OS),Windows_NT)
	@echo Usage:
	@echo   make build '&&' sudo make install
endif
