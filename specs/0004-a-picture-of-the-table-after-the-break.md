# 0004 A picture of the table after the break

**Status:** implemented
**Date:** 2026-11-02

## Goal

The frame this game shows the world is held against a picture of what it
should be, so that a change which breaks it fails a test rather than waiting
for somebody to look at the readme.

## Why

`--screenshot` has taken the break and settled it since the readme needed a
picture, because an opening rack is a photograph of a table nobody has played
on. That staging is the one thing here worth photographing, and nothing has
ever checked that it still comes out.

The engine's spec 0076 made the check one call. Before it, taking a picture
meant assembling an offscreen renderer, an empty overlay, a `block_on` and an
`Err` turned into a panic, which is why fourteen games were built on that
engine and one of them had pictures.

## Behavior

**The picture is the staged break and not a pose of its own.** A game built
with the staging on, loaded, updated once and drawn: the same state
`refresh-screenshots` photographs. So what this guards is the picture already
on show rather than a second arrangement that could drift from it.

**One update and no more.** The pose takes the shot and settles it on its own
clock inside that call, so the frame is whole by the end of it, and a second
call would find the table already broken and do nothing. The dt is nought, so
nothing else moves between the staging and the shutter.

**It is whatever size the engine says**, `blitzkit::picture::SIZE`, because
this game has no reason of its own. It was given twice that at first on the
argument that the number on a ball and the band round it are a pixel or two
at the engine's size, so everything spec 0002 put on them could come off
without the comparison moving. Measured rather than assumed, by taking the
faces off and asking how far the picture went: 0.3% at the engine's size
against 0.2% at twice it, where the tolerance is 0.2%. The bigger picture is
the one that nearly fails to notice.

**And 0.3% is thin.** That is this shot rather than its size: fifteen balls
across a table seen from the baulk end are a small part of the frame, so what
this holds well is the table, the spread of the pack and the light, and what
it only just holds is the faces. A shot from closer in would hold them
properly and is not the picture the readme shows, so the two cannot be the
same frame. Named here rather than left for somebody to find when a lost
band goes through.

**The reference lives in `tests/pictures/` and is looked at before it is
committed**, which is the engine's rule and is one no code can enforce.

**A machine with no adapter checks nothing and says so.** The engine writes
which picture went unchecked and hands back that it skipped, and nothing here
turns that into a failure.

## Acceptance criteria

- A table that has been broken matches the picture of
  one. — `poolhall_game::tests::the_table_after_the_break_looks_right`

### Verified by hand

- **The control**, which is the same shot with the staging off: a rack
  instead of a break. It differs by 3.1% of its pixels against a tolerance of
  0.2%, so the reference is of the broken table rather than of anything at
  all. Set `PoolhallGame::new(false)` in the test to see it.
- **What a lost face costs**, which is what the size was settled on: clear
  `faces` after `load` and the picture moves 0.3%, which passes and is the
  thin margin above.
- **The reference itself**, looked at before it was committed: the cue ball
  and the stick near the baulk cushion, the pack loose around the apex, one
  ball out by the top rail.

## Out of scope

**Failing a run that drew nothing.** A machine with no adapter skips, and
deciding that a run with no pictures in it is a failed run belongs to
whatever runs the gate rather than to this game.

**A picture per state.** One frame, and the one already staged. Aiming, the
tip and a pocketed ball would each need a pose invented for them, and an
invented pose is a second account of what this game looks like.
