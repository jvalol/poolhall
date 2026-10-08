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

So the stripes are what a ball looks like and nothing else. Any ball may be
potted in any order, and the run still ends when the table is clear.

One piece of the ordering came back later: the eight may not be hit first until
it is the only ball left, per spec 0001. That is a rule about the eight and not
about groups, and it needs none of this spec: the eight is black whether or not
anything is striped.
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

**Every numbered ball wears its number**, in a white circle, the eight an 8 and
the fifteen a 15. Two circles, one over each pole, which is the count a real
ball has.

A real ball wears them on its equator and this one cannot. From a camera above a
table you are looking at the top of a ball: an equator is edge on, and the first
try at this put the circle there and got a white sliver at the rim with the
numeral turned away. Up is where you are looking from, so up is where the number
goes. The same trade this spec already makes by drawing the band wider than a
real ball's.

**The numeral is drawn in the sphere's stretch rather than against it.** A pole
is the one place the texture is worst behaved: every column of it meets at that
one point, so the cap is a strip across the whole width and anything drawn
square into that strip is smeared into a ring. Each texel of the cap is turned
back into the place it sits on the flat the pole touches, so far out at so many
degrees round, and the glyph is read there. What comes out is bent in the
texture and straight on the ball, which is the only way round that works.

**The numerals are drawn too**, like the band, from a ten glyph font five across
and seven down. A font file for ten shapes that are never set in a line is more
to ship and more to go wrong.

**Every ball is a texture now**, solids included, because a number is a texture
and a solid ball has one. What a ball is drawn with is near enough white and the
whole of its paint comes from the picture.

**The cloth takes the turn out of a ball that is going nowhere.** A pool ball
that has stopped travelling has stopped turning, and one that sits rotating on
the spot is a thing a table does not do.

The engine spends a spin by rolling the ball along, which is right for a ball
with room to roll into and wrong for one held by its neighbours. Wedged in a
cluster it has nowhere to go, keeps a velocity it cannot spend, and the contact
turns that velocity back into spin every step as fast as anything takes it
away. Two balls off an ordinary break sat turning for most of a second, which
is invisible on a plain coloured ball and obvious the moment there is a number
on it.

So what a ball did is what counts, not what it meant to do: a ball that has not
moved this step has its spin and its speed taken down together, because either
one alone feeds the other back. Only below the speed a shot ends at, so it
never touches a ball that is still travelling and never eats a draw.

**And a shot is not over while something is still turning.** The test was on
speed alone, so a shot could end with a ball mid turn and freeze it there.

**The cue ball stays white**, with nothing on it.

## Acceptance criteria

- A stripe texture is white with a band across its middle. — `paint::tests::a_stripe_is_white_with_a_band`
- The band is the colour it was asked for. — `paint::tests::the_band_is_the_colour_asked_for`
- And it is opaque, top to bottom of the band. — `paint::tests::a_stripe_is_opaque`
- A ball that has rolled has turned. — `rules::tests::a_rolling_ball_turns`
- It turns the way it rolls. — `rules::tests::a_ball_turns_the_way_it_rolls`
- A ball that has not moved has not turned. — `rules::tests::a_still_ball_does_not_turn`
- A new rack has every ball upright and untouched. — `rules::tests::a_new_rack_has_not_turned`
- And no ball turns on the spot. — `rules::tests::nothing_turns_on_the_spot`
- Seven of the fifteen are striped, the eight is not one of them, and nine takes the one's hue. — `paint::tests::seven_are_striped_and_the_eight_is_not`
- The band goes all the way round rather than being a patch. — `paint::tests::the_band_goes_all_the_way_round`
- A stripe is the size it says. — `paint::tests::it_is_the_size_it_says`
- Every ball carries its own number, over each pole. — `paint::tests::every_ball_wears_its_number`
- The circle stops short of the middle rather than whitening the ball. — `paint::tests::the_circle_stops_short_of_the_middle`
- The numeral is drawn in polar coordinates, so a pole does not smear it into a ring. — `paint::tests::the_numeral_is_not_smeared_round_the_pole`
- No two balls look alike. — `paint::tests::no_two_balls_look_alike`

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
