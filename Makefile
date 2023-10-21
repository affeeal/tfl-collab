all: fuzzy


fuzzy: 
	@RUST_LOG=info cargo run -- --regex-count $(rc) --string-count $(sc)

test_regex:
	@RUST_LOG=info cargo run -- --regex "$(r)" --string-count $(sc)
