RUSTC ?= rustc
examples = src/examples/lock.rs src/examples/counter.rs src/examples/fail_counter.rs

all: paxos examples

paxos:
	mkdir -p lib bin
	$(RUSTC) -g -O -L ../rust-busybee/build --out-dir lib src/paxos/lib.rs

examples:
	mkdir -p lib bin
	$(foreach example, $(examples), $(RUSTC) -L ../rust-busybee/build -L lib --out-dir bin $(example);)

test: paxos
	$(RUSTC) src/paxos/lib.rs -L lib -L ../rust-busybee/build --test -o bin/test
	./bin/test

clean:
	rm -rf lib bin

.PHONY: all paxos test clean