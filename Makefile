RUSTC ?= rustc

all: paxos examples

paxos:
	mkdir -p lib bin
	$(RUSTC) -g -O -L ../rust-busybee/build --out-dir lib src/paxos/lib.rs

examples:
	mkdir -p lib bin
	$(RUSTC) -L ../rust-busybee/build -L lib --out-dir bin src/examples/counter.rs

test: paxos
	$(RUSTC) src/paxos/lib.rs -L lib -L ../rust-busybee/build --test -o bin/test
	./bin/test

clean:
	rm -rf lib bin

.PHONY: all paxos test clean