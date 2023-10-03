# Changelog

## 0.2.1 (2023-10-03)

* Upgraded to Bevy 0.11.
* Fixed the player getting jostled slightly while riding the moving platform.
* Fixed a crash on MacOS 14.
* Fixed an issue where some render pipelines were not preloaded properly, causing a hiccup when starting the game.
* Disabled the UI for the 2d background camera.

## 0.2.0 (2023-04-23)

* Preload pipelines to prevent stutter after pressing play.
* Fix tower feeding not working at apex of jump.
* Preserve the player's orientation when they fall off the map.
* Make item collider slightly more forgiving.
* Align item probe z height with item center of mass, making them even more forgiving.
* Increase parallax effect in starfield.
* Fix ammo counter and spawn timer labels appearing on top of other UI.
* Readability tweaks for upcoming wave UI.
* Use "--" for wave timer display when no more enemies are coming.
* Don't allow right click on canvas to prevent sticky inputs when showing context menu.
* Fix "bad sound" playing when pressing interact while not carrying an item.
* Fix excessive speed when moving diagonally with keyboard.
* Adjust max speed / jump height to make up for above.
* Fix player being able to jump on top of item pickups.
* Show item name on label for first spawn.
* Switch to unreleased bevy_rapier with working debug lines.
* Lots of cleanup / refactoring with the intention of supporting local multiplayer.

## 0.1.0 (2023-04-10)

* This version was submitted to Bevy Jam 3.
