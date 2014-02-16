all: russenger paxos

russenger:
	# cd ../russenger && make

paxos:
	mkdir -p lib bin
	rustc src/paxos/lib.rs --out-dir lib

test:
	rustc src/paxos/lib.rs -L lib --test -o bin/test
	./bin/test

clean:
	rm -rf lib bin

.PHONY: all russenger paxos test clean