# NES Dragon Quest (J) password library

## CLI usage

```sh
# decode password to game state
cargo run --release --example decode -- 'まるかつはやつはりせかいいちだつたのだよ'

# encode game state to password
cargo run --release --example encode -- examples/sample.json

# generate passwords by pattern (up to 10)
cargo run --release --example generate -- 'ゆうていみやおうきむこうほりいゆうじ??' 10
```

## Notes

`generate()` function is faster than naive algorithm thanks to dynamic programming.
But, if your pattern starts with "??", it might take some time.
