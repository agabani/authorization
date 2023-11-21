# Bevy

```shell
cargo run --package examples-bevy
```

## Rules of the Game

`Players` and `Monsters` try to `attack` and `take` everything.

When anything is `killed`, the `Game Master` tries to spawn replacement `Players` and `Monsters`.

When anything is `killed`, the `Game Master` tries to spawn `Loot` for the `killer` to `take`.

If the `Loot` was not `taken` within a few seconds, everything is allowed to `take` that `Loot`.
