# tfl-collab
Joint lab works on the TFL course, BMSTU, 5th term

## Usage


### Run fuzzy test with custom params

```
$ RUST_LOG=info cargo run -- --regex-count <REGEX_COUNT> --string-count <STRING_COUNT> --lookahead-count <LOOKAHEAD_COUNT>  --star-height <STAR_HEIGHT> --letter-count <LETTER_COUNT>
```

### Run test for given regex

```
$ RUST_LOG=info cargo run -- --regex "<REGEX>" --string-count <STRING_COUNT>
```


