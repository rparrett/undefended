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
- nuke LastTile if a tower is built on it
- preserve player orientation when they fall off the world
- if player falls with a tower kit, respawn the kit
- show timers over spawn points
- show upcoming wave
