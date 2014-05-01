RUSTC ?= rustc

all: paxos

paxos:
	mkdir -p lib bin
	$(RUSTC) -O -L ../rust-busybee/build --out-dir lib  src/paxos/lib.rs

test: paxos
	$(RUSTC) src/paxos/lib.rs -L lib -L ../rust-busybee/build --test -o bin/test
	./bin/test

clean:
	rm -rf lib bin

.PHONY: all paxos test clean