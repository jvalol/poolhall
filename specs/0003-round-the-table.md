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

**You start behind the head rail, round to one side, and low**, which is where
you break from and the view that shows the table as a shape.

Square behind the rail at fifty four degrees up is the view that tells you
least. The table is a corridor, every ball lines up with every other, and
nothing about the lie reads. A third of a radian round from square and nearer
the cloth, it is a table with balls spread across it. The three numbers were
matched by eye against a view Jake found by playing, which is the only way
anyone was going to find them.

It did not show the whole table. The starting distance was close enough that the
near end ran off the bottom of the window while a third of the frame above it
was empty, which is the one thing this spec asks of the opening view and the one
thing nothing checks. It is far enough back now for the near cushion to be in
frame, and no further: a whole table with a wide black border round it is the
other failure, and that one is easy to arrive at while fixing this one.

**The keys move you and the mouse takes the shot.** Walking round, leaning in
and sliding across are all on the keyboard, and the mouse does nothing but
point at the cloth and pull the cue back. A drag that walked you round meant
the one pointing device was doing both jobs, and the hand that aims kept
moving the room.

W and S lean in and out, A and D walk round, and the arrows slide what you are
looking at. That is the arcade's own split, which is the building this engine
was shown off in: the letters are the feet and the arrows are the head.

Shift gives each key its second job. With it held the arrows put the cue tip,
which is the fine control and the one that wants a modifier rather than a key
of its own, and W and S stand you up and stoop you. Standing up had nowhere
else to go once the drag was gone, and it is half of where you are.

Everything the keys do is a rate a second and not a step a press, because a key
that moves the eye a fixed amount per press is a key you hammer.

D walks you to your right and A to your left, which is the opposite sign to the
drag this replaced: dragging right turns the world right, which walks you left,
while pressing D means move me right. One sign served both and sent the keys
the wrong way round. Nothing caught it, because every other test of walking
asks only that the eye moved and stayed the same distance out, which a wrong
sign passes perfectly.

**The eye may go inside the table, and that is not a fault.** It is `back` out
and `above` up, and neither of those knows where the table is, so leaning right
in at the flattest angle puts it below the cushions and sliding the view up the
table carries it in over the cloth.

That was read as a bug and fixed, by raising the eye until it cleared the
cushions. Played rather than reasoned about, it is better the other way: down
among the balls at cloth height is the view a player actually takes to read a
thin cut, and the cushions passing through the frame are no worse than the rail
you would really be looking over. The fix was taken out again.

What is left is the rule that a view along the cloth never meets it, which is
what `LOWEST` is for and is a different thing: that one is arithmetic and this
one was taste.

**And it slides across, with shift held.** Walking round and leaning in are two
of the three things a person wants from a camera. The third is to put a
particular corner in the middle of the window, and without it the eye orbited
one fixed point for ever: every view was the table's middle seen from somewhere
else.

Shift and the same right drag, rather than a button of its own. There is no
third button on a trackpad and the arrow keys are already the cue tip, so the
one drag does two things depending on a modifier. It slides along the cloth in
the eye's own directions, so dragging right moves the table right whichever
side you are standing on, and dragging up sends the ground away under you
rather than carrying the camera over it, which is the one that reads as moving
the view and not shoving the furniture.

How far a pixel slides is a share of how far out the eye is, the same shape as
a zoom notch and for the same reason: one fixed distance crawls from across the
room and jumps from up close.

It stops a little past the cushion. Panning with nothing to stop it is a camera
lost in the black with no way back, and there is no key spare to recentre with.

**The aim follows the walk without being told to.** Spec 0001 points the shot at
whatever the cursor is over on the cloth, and the cursor ray comes from the
camera, so turning the camera turns what the cursor means. Nothing in the aim
knows about the walk.

## Acceptance criteria

- It starts behind the head rail, off to one side, above the cloth. — `view::tests::it_starts_behind_the_head_rail_and_off_to_one_side`
- D walks you to your right and A to your left. — `view::tests::d_walks_you_to_your_right`
- Walking across walks round rather than towards. — `view::tests::dragging_walks_round_the_table`
- The keys walk you and the mouse is left for the shot. — `poolhall_game::tests::the_keys_walk_and_the_mouse_is_for_the_shot`
- The arrows slide what you are looking at, and leave the cue tip alone. — `poolhall_game::tests::the_arrows_slide_what_you_are_looking_at`
- Shift and the arrows slide what the eye looks at along the cloth. — `view::tests::panning_slides_what_it_looks_at`
- It slides the way the eye faces, not the way the world is laid out. — `view::tests::panning_follows_where_you_are_standing`
- And it cannot be slid off into the black. — `view::tests::panning_stops_at_the_edge_of_the_table`
- The wheel leans in and out without walking round. — `view::tests::the_wheel_leans_in_and_out`
- And stops short of the table and of the next room. — `view::tests::it_cannot_lean_past_the_table_or_into_a_ball`
- A notch is worth the same share from anywhere. — `view::tests::a_notch_is_worth_the_same_from_anywhere`
- And right round comes back where it started. — `view::tests::walking_right_round_comes_back`
- Shift with W and S stands you up and stoops you. — `view::tests::standing_up_and_stooping_changes_the_height`
- It stops a hair short of flat and of straight down. — `view::tests::it_stops_a_hair_short_of_flat_and_of_straight_down`
- And reaches from nearly flat to nearly overhead. — `view::tests::it_reaches_from_nearly_flat_to_nearly_overhead`
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
