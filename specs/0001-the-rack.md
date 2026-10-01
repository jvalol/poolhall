# 0001 The rack

**Status:** implemented
**Date:** 2026-10-01

## Goal

Fifteen balls in a triangle, and a cue ball you hit them with. Pot them all, and
the shot that matters is rarely the one you are taking: it is where the cue ball
stops for the next one.

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

**Fifteen balls, one through fifteen**, racked in a triangle with the one at the
apex on the foot spot and the eight buried in the middle of the third row. The
others go where they fall; a real rack randomises them and so does this.

This spec had nine in a diamond, which is nine ball's rack. Jake meant the
triangle, which is every other game played on a pool table and the one anybody
pictures.

**There is no order.** Hit any ball first and pot any ball, and the run ends when
the table is clear.

This spec had nine ball's ordering rule, lowest ball first with the nine winning
whenever it dropped, and Jake took it out. What is left is fifteen balls and as few
shots as you can manage, which is the same game of position without a rule to
read. The wrong ball foul went with it.

**A shot is a direction, a speed, and a place on the cue ball.** The cursor aims
it, as in carom, through blitzkit spec 0025. Where on the ball it is struck is a
second input and the one this game is about: low, high, or off to one side.

**Putting the cue ball down is what ends being in hand.** One click places it,
and the next press and hold is a shot. Without that every click put the ball
somewhere again and nothing ever got as far as shooting, which is how it shipped
the first time.

**A cursor on top of the cue ball points nowhere**, and the shot keeps the
direction it had. The direction from a ball to itself is nothing at all, and
putting the ball down under the cursor does exactly that, so a shot taken
straight afterwards was refused in silence.

**A foul costs a shot and gives you the cue ball in hand.** The fouls are the cue
ball potted, no contact with the lowest ball, and no ball reaching a cushion after
contact. In hand means you place it anywhere before shooting again.

**The run ends when the table is clear**, and the score is the shots it took,
fouls included. One player, as carom is. An opponent is a different spec.

**A ball potted is out of play.** It leaves the table and stops being simulated,
which carom learned the slow way: a ball left in play gets walked further out by
later shots until it leaves the world, and a falling body is a body still moving.

## The table

**A rectangle twice as long as it is wide**, with six pockets: one at each corner
and one at the middle of each long rail. Real proportions, in units where a ball
is one across.

**The cushions are gaps. The cloth is not.** Six lengths of rail with a space at
every pocket, so a ball on its way to one goes between real geometry rather than
through a special case in the rules. The cloth under them is one piece with an
apron past the rails, and a ball is potted by reaching the jaws rather than by
falling through a hole.

That is a change from what this spec first said, which was holes in the cloth and
a ball caught on the way down. Prettier, and it is also exactly how carom lost
half a minute a shot: a body with nothing under it falls for ever, a falling body
is a body still moving, and the shot never ends. There is no sense building the
same trap twice to see if it still works.

**A ball meets a cushion if its path meets one**, swept along where it was going
rather than looked for where it ended up. Two wrong answers came first. Asking
whether a ball is near a rail misses it, because a ball bounces inside the step
it touches and never ends one at touching distance: the closest a measured one
got was 0.536 against a ball radius of 0.5. Sweeping from where it was to where
it ended up misses it too, because a step containing a bounce has a net
displacement pointing away from the rail. Where it was and where it was going is
the path `through_the_world` sweeps, and `sweep_sphere` is what it sweeps with.

**A potted ball comes off the table in the step it reaches the jaws**, not when
the shot ends. Past the cushions there is apron and then nothing, so a ball left
rolling after it has been potted finds the edge of the world. This was still
wrong when the rules were first written, and sixteen test breaks found it.

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

- The rack is fifteen balls in a triangle, inside the table and clear of each other. — `table::tests::the_rack_is_a_triangle`
- The one is at the apex and the eight in the middle of the third row. — `table::tests::the_one_leads_and_the_eight_is_buried`
- A ball reaching a pocket is potted. — `table::tests::a_ball_in_the_jaws_is_potted`
- And one on a rail or in the middle of the table is not. — `table::tests::the_rails_are_not_pockets`
- No rail covers a pocket, or nothing could be potted in it. — `table::tests::a_ball_can_reach_every_pocket`
- A new run is a full rack, no shots, and the cue ball in hand. — `rules::tests::a_new_run_is_a_full_rack`
- Any ball may be hit first. — `rules::tests::any_ball_first_is_legal`
- Hitting nothing at all is a foul. — `rules::tests::missing_everything_is_a_foul`
- Potting the cue ball is a foul, and it comes back. — `rules::tests::potting_the_cue_ball_is_a_foul`
- No cushion after contact is a foul. — `rules::tests::no_rail_after_contact_is_a_foul`
- A foul costs a shot and gives ball in hand. — `rules::tests::a_foul_costs_a_shot_and_the_cue_ball`
- And the cue ball cannot be moved at any other time. — `rules::tests::the_cue_ball_cannot_be_moved_unless_it_is_in_hand`
- An empty table ends the run. — `rules::tests::an_empty_table_ends_it`
- And one with a ball still on it does not. — `rules::tests::a_table_with_one_ball_left_is_not_over`
- A potted ball leaves play and no later shot moves it. — `rules::tests::a_potted_ball_is_out_of_play`
- Every break ends, from any aim, at everything there is. — `rules::tests::no_shot_runs_for_ever`
- And nothing still in play ever leaves the table. — `rules::tests::nothing_in_play_leaves_the_table`
- The hardest shot can run the table and come back. — `shot::tests::a_ball_can_run_the_table`
- A ball struck low leaves the cue ball behind where they met. — `shot::tests::low_draws_the_cue_ball_back`
- And struck high, well past it. — `shot::tests::high_runs_the_cue_ball_on`
- With every height between those in order. — `shot::tests::the_whole_range_is_in_order`
- Side changes the angle off a cushion. — `shot::tests::side_changes_the_angle_off_a_rail`
- The tip lands on the ball, wherever it is aimed. — `shot::tests::the_tip_lands_on_the_ball`
- The cursor points the shot. — `poolhall_game::tests::the_cursor_points_the_shot`
- A cursor off the window does not move it. — `poolhall_game::tests::a_cursor_off_the_window_does_not_move_the_shot`
- The arrow keys move the tip and keep it on the ball. — `poolhall_game::tests::the_arrow_keys_move_the_tip_and_stay_on_the_ball`
- Space puts it back in the middle. — `poolhall_game::tests::space_puts_the_tip_back_in_the_middle`
- In hand, a click places the cue ball rather than shooting. — `poolhall_game::tests::in_hand_a_click_places_the_cue_ball_rather_than_shooting`
- And the click after that winds up a shot and takes it. — `poolhall_game::tests::placing_the_cue_ball_lets_the_next_click_shoot`
- Putting it down ends being in hand, and a later click does not pick it up. — `rules::tests::putting_it_down_ends_being_in_hand`
- The readout is cleared each frame rather than piling up. — `poolhall_game::tests::the_readout_does_not_pile_up`
- And its lines are evenly spaced. — `poolhall_game::tests::the_readout_lines_are_evenly_spaced`
- One shot reads as one shot, and one foul as one foul. — `poolhall_game::tests::one_shot_is_not_one_shots`

### Verified by hand

- The break scatters fifteen balls and none of them ends up inside another. This is
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
- The pockets are round, and read as holes rather than as anything shiny. The specular in
  blitzkit spec 0012 is not tinted by the material colour, so a black slab still
  takes a full white highlight, and the lobe is `pow(dot, shininess)`: a low
  shininess is a huge one. A squashed sphere at the default came out as a white
  blob and a slab at a shininess of one came out whiter, which is why they are
  drawn flat at 256.

## Out of scope

An opponent, turns, safeties played against one, and every other rule that needs
two people. Push out after the break. Call shot. Three foul. Spotting a ball.
A cue drawn on screen, and any animation of the strike: the shot is the cursor
and the place on the ball, and the ball simply goes.
