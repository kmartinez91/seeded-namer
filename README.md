# seeded-namer

Answers one question: given a seed, what name does it produce?

I kept hitting this in small projects — a game needs NPC names, a test suite
needs fixture data, a script needs placeholder users — and wanting the names
to be random-looking but reproducible. Pulling in a full RNG crate plus a
name-list crate for that felt like overkill, and calling `rand::random()`
directly meant the names changed on every run, which made tests flaky and
made it impossible to say "seed 42 always gives you Kelra."

This is a small command-line tool and library that takes a `u64` seed and
turns it into a two- or three-syllable name. Same seed, same name, every
time, on any machine, forever (within the same version — see Compatibility
below).

## Usage

Build and run with a seed:

```
$ cargo run -- 42
Kelra
```

Ask for a sequence of names from one seed:

```
$ cargo run -- 42 5
Kelra
Thomir
Skoun
Vrail
Nozhar
```

No seed given falls back to the current time, so it's still useful as a
one-off name picker:

```
$ cargo run --
Bralon
```

Pick a different syllable set with `--style` (`default`, `harsh`, `soft`,
or `sci-fi`), in addition to seed and count:

```
$ cargo run -- --style harsh 42 3
Grokk
Dzarug
Kroth

$ cargo run -- --style sci-fi 42
Zirox
```

## As a library

The whole thing is built from pure functions — no RNG object to construct,
no hidden state:

```rust
use seeded_namer::generate_name;

let name = generate_name(42);
assert_eq!(name, generate_name(42)); // always true
```

To pick a syllable set other than the default, use `generate_name_with_style`
with a `Style`:

```rust
use seeded_namer::{generate_name_with_style, Style};

let name = generate_name_with_style(42, Style::Harsh);
```

`generate_name` and `name_sequence` (and their `_with_style` counterparts)
are deterministic in their arguments alone, so they're trivial to unit test:
pass a seed, check the output, no setup or mocking required.

## Compatibility

The seed-to-name mapping is considered part of this version's behavior, not
a stable API. If the syllable lists or the generator internals change in a
later version, the same seed may produce a different name. Don't rely on a
specific seed producing a specific name across upgrades.

## License

MIT, see LICENSE.
