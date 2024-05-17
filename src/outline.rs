use bevy::prelude::*;
use bevy_mod_outline::{
    AsyncSceneInheritOutlinePlugin, AutoGenerateOutlineNormalsPlugin, InheritOutlineBundle,
    OutlineBundle, OutlinePlugin as ActualOutlinePlugin, OutlineVolume,
};

use crate::{
    map::{ItemSpawner, MovingFloor, PlacedTower},
    GrabbedItem, Item, SelectedItem, SelectedTile,
};

pub struct OutlinePlugin;

impl Plugin for OutlinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ActualOutlinePlugin)
            .add_plugins(AutoGenerateOutlineNormalsPlugin)
            .add_plugins(AsyncSceneInheritOutlinePlugin)
            .add_systems(Update, update_inner)
            .add_systems(Update, remove_inner)
            .add_systems(Update, update);
    }
}

#[derive(Component)]
pub struct InnerMeshOutline {
    pub color: Color,
    pub width: f32,
}

fn update_inner(
    mut commands: Commands,
    query: Query<(Entity, &InnerMeshOutline), Changed<InnerMeshOutline>>,
    children_query: Query<&Children>,
    mesh_query: Query<Entity, With<Handle<Mesh>>>,
) {
    // let mut added = false;

    // for (entity, inner_mesh_outline) in &query {
    //     for descendant in children_query.iter_descendants(entity) {
    //         if !added {
    //             if let Ok(mesh_entity) = mesh_query.get(descendant) {
    //                 // TODO we may be able to fix the "separate outline for tower head and body"
    //                 // problem by only adding the OutlineBundle to the toplevel mesh and then adding
    //                 // InheritOutlineBundle to every descendant of that mesh.

    //                 commands.entity(mesh_entity).insert(OutlineBundle {
    //                     outline: OutlineVolume {
    //                         width: inner_mesh_outline.width,
    //                         colour: inner_mesh_outline.color,
    //                         visible: true,
    //                     },
    //                     ..default()
    //                 });

    //                 added = true;
    //             }
    //         } else {
    //             commands
    //                 .entity(descendant)
    //                 .insert(InheritOutlineBundle::default());
    //         }
    //     }
    // }
}

fn remove_inner(
    mut commands: Commands,
    mut removed: RemovedComponents<InnerMeshOutline>,
    children_query: Query<&Children>,
    mesh_query: Query<Entity, With<Handle<Mesh>>>,
) {
    for entity in removed.read() {
        for descendant in children_query.iter_descendants(entity) {
            if let Ok(mesh_entity) = mesh_query.get(descendant) {
                commands.entity(mesh_entity).remove::<OutlineBundle>();
            }
        }
    }
}

fn update(
    mut commands: Commands,
    player_query: Query<
        (&SelectedTile, &SelectedItem),
        Or<(Changed<SelectedTile>, Changed<SelectedTile>)>,
    >,
    outlines_query: Query<Entity, With<InnerMeshOutline>>,
    grabbed_item_query: Query<&Item, With<GrabbedItem>>,
    invalid_tile_query: Query<(), Or<(With<MovingFloor>, With<PlacedTower>, With<ItemSpawner>)>>,
    placed_tower_query: Query<&PlacedTower>,
) {
    let Ok((tile, item)) = player_query.get_single() else {
        return;
    };

    for entity in &outlines_query {
        commands.entity(entity).remove::<InnerMeshOutline>();
    }

    let grabbed_item = grabbed_item_query.get_single();

    match grabbed_item {
        Ok(Item::LaserAmmo) => {
            // Outline towers

            // if let Some(entity) = tile.0 {
            //     if let Ok(placed_tower) = placed_tower_query.get(entity) {
            //         commands.entity(placed_tower.0).insert(InnerMeshOutline {
            //             width: 3.,
            //             color: Color::hsla(160., 0.9, 0.5, 1.0),
            //         });
            //     }
            // }
        }
        Ok(Item::TowerKit) => {
            // Outline valid tiles for tower placement

            if let Some(entity) = tile.0 {
                if invalid_tile_query.get(entity).is_err() {
                    commands.entity(entity).insert(InnerMeshOutline {
                        width: 3.,
                        color: Color::hsla(160., 0.9, 0.5, 1.0),
                    });
                }
            }
        }
        _ => {
            // Outline items that can be grabbed

            if let Some(entity) = item.0 {
                commands.entity(entity).insert(InnerMeshOutline {
                    width: 3.,
                    color: Color::hsla(160., 0.9, 0.5, 1.0),
                });
            }
        }
    }
}
