# 0003 Round the table

**Status:** implemented
**Date:** 2026-10-01

## Goal

Walk round the table and stoop down to it. A shot along the rail is a different
shot from the far end, and from one fixed place half the table is a shot you
cannot read.

## Behavior

**Holding the right button and dragging walks you round.** The left button is
already aiming and shooting, and the cursor is already pointing the shot, so the
walk takes the button nothing else is using. Across turns you round the table and
down raises and lowers you.

**You stay the same distance out.** Walking round is a circle rather than a
wander: the table is the subject and nothing about a shot wants the table bigger
or smaller. A zoom is a separate thing and this spec does not have one.

**You can neither lie flat on the cloth nor look straight down.** Flat, a cursor
ray never meets the table and there is nowhere to point; straight down, the beads
that show the shot are a dot. The height is clamped between the two.

**You start behind the head rail and well up**, which is where you break from and
the view that shows the whole table.

**The aim follows the walk without being told to.** Spec 0001 points the shot at
whatever the cursor is over on the cloth, and the cursor ray comes from the
camera, so turning the camera turns what the cursor means. Nothing in the aim
knows about the walk.

## Acceptance criteria

- It starts behind the head rail and above the cloth. — `view::tests::it_starts_behind_the_head_rail_and_above_the_cloth`
- Dragging across walks round rather than towards. — `view::tests::dragging_walks_round_the_table`
- And right round comes back where it started. — `view::tests::walking_right_round_comes_back`
- Dragging down and up changes the height. — `view::tests::dragging_up_and_down_changes_the_height`
- It never lies flat on the cloth or gets under it. — `view::tests::it_never_lies_flat_or_looks_straight_down`
- The right button walks and the left still shoots. — `poolhall_game::tests::the_right_button_walks_and_the_left_shoots`
- Walking moves the eye and leaves the table where it is. — `poolhall_game::tests::walking_moves_the_eye_and_not_the_table`

### Verified by hand

- A shot down the rail is readable from the end of the table and from the side,
  and they are different shots to read.
- Stooping to the cloth makes a thin cut legible in a way the high view does not.
- The band on a striped ball is clearer from low down than from high up, which is
  what a real table is like and what spec 0002's band width is widened against.

## Out of scope

Zoom. A camera that follows the cue ball, or swings to where the action is after
a shot. Snapping to a rail or to behind the cue ball. Keys that do the same job
as the drag.
