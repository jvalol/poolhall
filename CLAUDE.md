# diamond

Fifteen balls in a triangle, and a cue ball to break them with. The tenth game on
`blitzkit`. The dependency is the published crate, overridden by the engine
checkout at `../blitzkit` when built inside this project folder.

The rack is a triangle with the eight buried in the middle. The name is the
sights on the rails, the diamonds you aim off, which outlive any one rack: this
started as nine ball and its diamond of nine before Jake said he meant the
triangle.

## Build and test

Requires Rust 1.87 or newer, the engine's MSRV.

```
cargo build
cargo test
cargo run
cargo clippy
cargo fmt
```

## How work happens here

Behavior changes are spec driven:

1. **Write the spec first.** Copy `specs/TEMPLATE.md` to `specs/NNNN-short-name.md`
   and fill it in. Say what the game does, not how the code does it.
2. **Make the acceptance criteria testable.** Each one names the test that proves
   it, or goes under "Verified by hand" when it needs a window.
3. **Write the tests, then the code.** `cargo test` passes before a commit.
4. **Update the spec when behavior changes.** A spec that disagrees with the game
   is a bug in the spec.

## Layout

- `src/main.rs` — nothing yet. The game is a spec so far.

## Why this game exists

carom put one sphere against another and asked where it would go. This asks the
harder half of the same question: where does the thing you hit it with go, and
can you choose.

blitzkit spec 0032 made that possible, by letting a game strike a body off its
middle. Spec 0030's contact friction turns that into a cue ball that comes back,
stops dead, or runs on, and choosing between those is most of what a pool player
is doing. Without 0032 this is a geometry puzzle where the cue ball always stops
in the same place.

## Why the rails bounce too hard

Restitution and friction live on the body, and blitzkit's static world has
neither, so a cushion returns exactly what another ball does. Real cushions give
back about three quarters and ball on ball about nineteen twentieths.

Ball on ball is what the player aims with, so the realistic number goes there and
the rails are wrong until the engine can give them a material of their own. That
is written in the spec's hand-verified list rather than hidden by detuning the
balls, because a game that quietly compensates for an engine gap is a game nobody
will remember to fix.

## Why every number is measured

carom's table was sized by working out how far a shot travels. The working was
wrong by more than a factor of two, a ball rolled off the edge of the world, and
every stray shot cost half a minute until somebody played it.

So the numbers here come from tests that measure: how far a ball runs at full
speed, how long the longest shot takes, how much draw a given cloth leaves by the
time the balls meet. Anything that could be reasoned out will be measured
instead.
