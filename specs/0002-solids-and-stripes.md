# 0002 Solids and stripes

**Status:** implemented
**Date:** 2026-10-01

## Goal

Nine through fifteen wear a band of colour round the middle, the way a real
striped ball does, and the band turns with the ball.

## What this is not

It is not a rule. Eight ball's groups, taking solids or stripes and clearing
yours before the eight, are an ordering, and spec 0001 took the ordering out on
purpose. Jake looked at putting one back and said it was too much for this.

So the stripes are what a ball looks like and nothing else. Any ball may still be
hit first and any may be potted, and the run still ends when the table is clear.
A practice table is a table you can practise on, not a table that marks you.

## Behavior

**One through seven are solid**, a flat colour each, as they already are.

**The eight is black**, and solid.

**Nine through fifteen are white with a band**, each band the colour of the ball
seven below it: nine takes the one's yellow, ten the two's blue, and so on. That
is how a real set is made and it is why a glance tells you which is which.

**The band is drawn, not loaded.** A texture of white with a stripe across the
middle of it, one per hue, generated the way marble's checker is. Seven textures
rather than fifteen: a solid needs no texture at all.

**The band runs round the ball's equator**, which on blitzkit's sphere is the
middle of the texture's v. Where that equator points depends on which way the
ball has turned, and nothing sets it: a racked ball's band points wherever the
rack left it, exactly as a real one does.

**A ball's band turns with it.** Spec 0030 gives every body a spin, and spec 0031
takes it away again as the ball rolls. Adding that spin up gives which way round
the ball has got to, the same way marble's `facing` does, and the band follows.
Without it a rolling ball would look like a photograph of a rolling ball.

**The cue ball stays white**, with nothing on it.

## Acceptance criteria

- A stripe texture is white with a band across its middle. — `paint::tests::a_stripe_is_white_with_a_band`
- The band is the colour it was asked for. — `paint::tests::the_band_is_the_colour_asked_for`
- And it is opaque, top to bottom of the band. — `paint::tests::a_stripe_is_opaque`
- A ball that has rolled has turned. — `rules::tests::a_rolling_ball_turns`
- It turns the way it rolls. — `rules::tests::a_ball_turns_the_way_it_rolls`
- A ball that has not moved has not turned. — `rules::tests::a_still_ball_does_not_turn`
- A new rack has every ball upright and untouched. — `rules::tests::a_new_rack_has_not_turned`
- Seven of the fifteen are striped, the eight is not one of them, and nine takes the one's hue. — `paint::tests::seven_are_striped_and_the_eight_is_not`
- The band goes all the way round rather than being a patch. — `paint::tests::the_band_goes_all_the_way_round`
- A stripe is the size it says. — `paint::tests::it_is_the_size_it_says`

### Verified by hand

- A striped ball reads as striped at a glance, from across the table and when it
  is sitting against a cushion.
- The band turns as the ball rolls, and a ball that has stopped keeps the band
  where it stopped rather than snapping back.
- The rack reads as a rack: solids and stripes mixed, the eight black in the
  middle.

## Out of scope

Numbers on the balls, which want a digit per ball and a patch of white to put it
on. Groups, and any rule that reads one. A cue ball with a spot on it. Shadows or
reflections of the band. Anything that reads which group a ball is in other than
the player's eye.
