build:
	cargo build --release

run: build
	./target/release/tell <your_prompt> # Indentation is key here!

clean:
	cargo clean
	rm -rf ./target

deps:
	cargo add serde ollama-rs dirs
