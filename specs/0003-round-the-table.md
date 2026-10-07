# 0003 Round the table

**Status:** implemented
**Date:** 2026-10-01

## Goal

Walk round the table, stoop down to it, and lean in. A shot along the rail is a
different shot from the far end, and from one fixed place half the table is a
shot you cannot read.

## Behavior

**Holding the right button and dragging walks you round.** The left button is
already aiming and shooting, and the cursor is already pointing the shot, so the
walk takes the button nothing else is using. Across turns you round the table and
down raises and lowers you.

**Walking round is a circle rather than a wander.** Dragging turns you and
raises you and nothing else: the table is the subject and the drag never carries
you towards it.

**The wheel leans you in and out.** This spec said a zoom was a separate thing
and did not have one, and then there was no way to get a close eye on a shot. A
notch is a share of where you already are rather than a fixed distance, so it
moves you as much from close up as from far off: a fixed step crawls at one end
and jumps at the other. It stops near enough to read a thin cut and far enough
that the whole table is still in the window.

The share is small. At a seventeenth a flick of the wheel crossed the whole
range, which is a zoom you fight rather than one you use.

**The height runs from nearly flat to nearly overhead**, a hair under five
degrees to a hair under ninety. Both ends are worth having: a plan view is the
clearest look at an angle there is, and a low one is the only way to read a thin
cut.

Not flat, because a view along the cloth never meets it and there is nowhere to
point. Not exactly straight down either, and the reason this spec first gave for
that was wrong. It said the beads that show the shot would be a dot, when the
beads lie on the cloth and are at their clearest from above. The real reason is
the arithmetic: the eye looks along the up axis from directly over the table, and
a view matrix built from two parallel vectors is nothing at all.

**You start behind the head rail and well up**, which is where you break from and
the view that shows the whole table.

It did not show the whole table. The starting distance was close enough that the
near end ran off the bottom of the window while a third of the frame above it
was empty, which is the one thing this spec asks of the opening view and the one
thing nothing checks. It is far enough back now for the near cushion to be in
frame, and no further: a whole table with a wide black border round it is the
other failure, and that one is easy to arrive at while fixing this one.

**The aim follows the walk without being told to.** Spec 0001 points the shot at
whatever the cursor is over on the cloth, and the cursor ray comes from the
camera, so turning the camera turns what the cursor means. Nothing in the aim
knows about the walk.

## Acceptance criteria

- It starts behind the head rail and above the cloth. — `view::tests::it_starts_behind_the_head_rail_and_above_the_cloth`
- Dragging across walks round rather than towards. — `view::tests::dragging_walks_round_the_table`
- The wheel leans in and out without walking round. — `view::tests::the_wheel_leans_in_and_out`
- And stops short of the table and of the next room. — `view::tests::it_cannot_lean_past_the_table_or_into_a_ball`
- A notch is worth the same share from anywhere. — `view::tests::a_notch_is_worth_the_same_from_anywhere`
- And right round comes back where it started. — `view::tests::walking_right_round_comes_back`
- Dragging down and up changes the height. — `view::tests::dragging_up_and_down_changes_the_height`
- It stops a hair short of flat and of straight down. — `view::tests::it_stops_a_hair_short_of_flat_and_of_straight_down`
- And reaches from nearly flat to nearly overhead. — `view::tests::it_reaches_from_nearly_flat_to_nearly_overhead`
- The right button walks and the left still shoots. — `poolhall_game::tests::the_right_button_walks_and_the_left_shoots`
- Walking moves the eye and leaves the table where it is. — `poolhall_game::tests::walking_moves_the_eye_and_not_the_table`

### Verified by hand

- A shot down the rail is readable from the end of the table and from the side,
  and they are different shots to read.
- Stooping to the cloth makes a thin cut legible in a way the high view does not,
  and going right overhead makes the angle between two balls obvious in a way no
  other height does.
- Leaning in gets a close enough eye on a shot to pick the contact point, and
  leaning out still shows the whole table.
- The band on a striped ball is clearer from low down than from high up, which is
  what a real table is like and what spec 0002's band width is widened against.

## Out of scope

A camera that follows the cue ball, or swings to where the action is after
a shot. Snapping to a rail or to behind the cue ball. Keys that do the same job
as the drag.
