# UNDEFENDED

For Bevy Jam #3. TODO is the version that was submitted.

[Play on itch.io](https://euclidean-whale.itch.io/undefended)

## Acknowledgements

`Orbitron-Medium.ttf` is licensed under the SIL Open Font License.

All other assets are original creations by me for this project.

## 3d workflow

### Tiles

- 16x16x4 magicavoxel
- export to obj
- import into blender
- change transform pivot point to bounding box center
- select object, geometry to origin
- scale by 1.25 in all dimensions
- export as gltf (check y-up)

## TODO

- turning in place for keyboard users
- configure directional light
- animate spawned items up from inside spawner
- preserve player orientation when they fall off the world
- if player falls with a tower kit, respawn the kit
- play bad sound when attempting to place something in empty space
