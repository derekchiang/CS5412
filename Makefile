all: messenger paxos

messenger:
	rustc src/messenger/lib.rs --out-dir ./build

paxos:
	rustc src/paxos/lib.rs -L ./build --out-dir ./build

test:
	rustc src/paxos/lib.rs -L ./build --test --out-dir ./build
	./build/paxos

clean:
	rm -rf ./build
	mkdir ./build

.PHONY: all messenger paxos test clean