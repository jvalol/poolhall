# 0001 The rack

**Status:** draft
**Date:** 2026-10-01

## Goal

Nine balls in a diamond, and a cue ball you hit them with. Pot them in order, one
through nine, and the shot that matters is rarely the one you are taking: it is
where the cue ball stops for the next one.

The tenth game on blitzkit, and the first where the player is aiming at the ball
after this one.

## Why this game exists

carom put one sphere against another and asked where it would go. This asks the
harder half of the same question: where does the thing you hit it *with* go, and
can you choose.

That became possible an hour ago. blitzkit spec 0032 lets a game strike a body
off its middle, so a ball can be sent with backspin or topspin, and spec 0030's
contact friction turns that into a cue ball that comes back, stops dead, or runs
on. Measured in 0032: struck at the bottom a ball ends up behind where it met the
one it hit, dead centre it ends well past, and every height between falls in
order. That range is the game.

Without 0032 this is a geometry puzzle where the cue ball always stops in the
same place. With it, every shot has a second half.

## Behavior

**Nine balls, one through nine**, racked in a diamond with the one at the apex on
the foot spot and the nine in the middle. The others go where they fall; a real
rack randomises them and so does this.

**You must hit the lowest ball on the table first.** That is the whole of nine
ball's ordering rule. Anything may be potted after that contact, including by
carom off the lowest ball, and potting the nine wins at any point: on the break,
off a combination, or last of all.

**A shot is a direction, a speed, and a place on the cue ball.** The cursor aims
it, as in carom, through blitzkit spec 0025. Where on the ball it is struck is a
second input and the one this game is about: low, high, or off to one side.

**A foul costs a shot and gives you the cue ball in hand.** The fouls are the cue
ball potted, no contact with the lowest ball, and no ball reaching a cushion after
contact. In hand means you place it anywhere before shooting again.

**The run ends when the nine is potted**, and the score is the shots it took,
fouls included. One player, as carom is. An opponent is a different spec.

**A ball potted is out of play.** It leaves the table and stops being simulated,
which carom learned the slow way: a ball left in play gets walked further out by
later shots until it leaves the world, and a falling body is a body still moving.

## The table

**A rectangle twice as long as it is wide**, with six pockets: one at each corner
and one at the middle of each long rail. Real proportions, in units where a ball
is one across.

**Pockets are gaps rather than holes.** The cushions are boxes with spaces
between them and the cloth is a box with the pockets cut out of it, so a ball
that reaches one has nothing under it and falls. The game catches it on the way
down and takes it out of play. This is what the engine's static world can
express, and it happens to be what a pocket is.

**A ball that falls has to be caught in the same step it starts falling.** Not
eventually. carom waited twenty seconds for a falling marble because nothing was
watching.

## What it asks of blitzkit

**Cushions that are not made of ball.** Restitution and friction live on the
body, and the static world has none, so a rail returns exactly as much as another
ball does. Real cushions return around three quarters and ball on ball around
nineteen twentieths, which is not a small difference: it is the difference
between a cue ball that dies on the rail and one that comes back off it across
the table.

The workaround is to pick one and live with it, and this spec says up front which
way it will lean: ball on ball is what the player is aiming with, so the number
goes there, and the rails bounce too hard until the engine can say otherwise.
That is a spec of its own, materials on the static world, and this game is the
one asking for it.

**Nothing else.** Striking, rolling resistance, cursor rays, sphere against
sphere and the swept world are all there and all released.

## What it will not have

**Throw**, where a spinning cue ball drags the ball it hits off the line of their
middles. It needs a contact with area and nothing in spec 0030 has any. On thin
cuts a real player aims around it; here the line of middles is exactly where the
object ball goes.

**Swerve on the cloth.** Side spin does nothing until the ball meets a cushion,
where it works and was measured working. A real ball struck with a level cue runs
nearly straight too, so this is closer to right than it sounds, but masse is out.

Both are written in blitzkit spec 0032 and neither is a surprise.

## The numbers are measured, not reasoned

carom's table was sized by working out how far a shot travels, and the working
was wrong by more than a factor of two, which cost half a minute a shot until
somebody played it. Every number here that could be worked out will be measured
by a test instead: how far a ball runs at full speed, how long the longest shot
takes, how much draw a given cloth leaves by the time the balls meet.

## Acceptance criteria

- The rack is nine balls in a diamond, inside the table and clear of each other. — `table::tests::the_rack_is_a_diamond`
- The one is at the apex and the nine in the middle. — `table::tests::the_one_leads_and_the_nine_is_buried`
- A ball reaching a pocket is potted. — `table::tests::a_ball_in_the_jaws_is_potted`
- And one that passes over a rail is not. — `table::tests::the_rails_are_not_pockets`
- Nothing can reach the edge of the world. — `table::tests::nothing_leaves_the_table`
- Hitting the lowest ball first is legal. — `rules::tests::the_lowest_ball_first_is_legal`
- Hitting anything else first is a foul. — `rules::tests::any_other_ball_first_is_a_foul`
- Hitting nothing at all is a foul. — `rules::tests::missing_everything_is_a_foul`
- Potting the cue ball is a foul. — `rules::tests::potting_the_cue_ball_is_a_foul`
- No cushion after contact is a foul. — `rules::tests::no_rail_after_contact_is_a_foul`
- A foul costs a shot and gives ball in hand. — `rules::tests::a_foul_costs_a_shot_and_the_cue_ball`
- Potting the nine ends the run, whenever it happens. — `rules::tests::the_nine_ends_it`
- Including on the break. — `rules::tests::the_nine_on_the_break_ends_it`
- A potted ball leaves play and no later shot moves it. — `rules::tests::a_potted_ball_is_out_of_play`
- Every shot ends, from any aim, speed and strike. — `shot::tests::no_shot_runs_for_ever`
- A ball struck low leaves the cue ball behind where they met. — `shot::tests::low_draws_the_cue_ball_back`
- And struck high, well past it. — `shot::tests::high_runs_the_cue_ball_on`
- Side changes the angle off a cushion. — `shot::tests::side_changes_the_angle_off_a_rail`

### Verified by hand

- The break scatters nine balls and none of them ends up inside another. This is
  the harder test of the engine's one pass over the pairs than carom's cross: a
  rack is touching where a cross is a tenth of a ball apart.
- A ball potted drops and is gone, and nothing about it stutters on the way.
- Draw, stun and follow are three visibly different shots from the same aim at
  the same speed, and choosing between them changes where the next shot is from.
- A cue ball played off a rail with side comes back at a different angle than one
  played without. This is the only check that spec 0032's measurement means
  something to a player.
- The rails bounce too hard. Say so here until the engine can give them a
  material of their own, rather than quietly tuning the balls to hide it.

## Out of scope

An opponent, turns, safeties played against one, and every other rule that needs
two people. Push out after the break. Call shot. Three foul. Spotting a ball.
A cue drawn on screen, and any animation of the strike: the shot is the cursor
and the place on the ball, and the ball simply goes.
