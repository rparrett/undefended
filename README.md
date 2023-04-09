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
- !! add a win/loss condition
- !! set up GH/CI
