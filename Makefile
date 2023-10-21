all: fuzzy


fuzzy: 
	@RUST_LOG=info cargo run
