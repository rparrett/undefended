# UNDEFENDED

For Bevy Jam #3. [v0.1.4](https://github.com/rparrett/undefended/tree/v0.1.4) is the version that was submitted.

[Play on itch.io](https://euclidean-whale.itch.io/undefended)

## Acknowledgements

`Orbitron-Medium.ttf` is licensed under the SIL Open Font License.

PromptFont by Yukari "Shinmera" Hafner, available at <https://shinmera.com/promptfont>.

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

- animate spawned items up from inside spawner
- if player falls with a tower kit, respawn the kit
- add pause/restart screen that also shows controls
- better keyboard controls
  - turning in place?
- make LastTile a small FIFO in case the user places a tower on their own tile and then falls
- add loading bar for assets and pipelines
- upgrade towers by feeding them a tower kit?
