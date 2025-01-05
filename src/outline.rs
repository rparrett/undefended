use bevy::prelude::*;
use bevy_mod_outline::{
    AutoGenerateOutlineNormalsPlugin, OutlinePlugin as ActualOutlinePlugin, OutlineVolume,
};

use crate::{
    map::{ItemSpawner, MovingFloor, PlacedTower},
    GrabbedItem, Item, SelectedItem, SelectedTile,
};

pub struct OutlinePlugin;

impl Plugin for OutlinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ActualOutlinePlugin)
            .add_plugins(AutoGenerateOutlineNormalsPlugin::default())
            .add_systems(Update, update);
    }
}

fn update(
    selection_changed_query: Query<
        (&SelectedTile, &SelectedItem),
        Or<(Changed<SelectedTile>, Changed<SelectedItem>)>,
    >,
    grabbed_item_changed_query: Query<(), Changed<GrabbedItem>>,
    grabbed_item_removed: RemovedComponents<GrabbedItem>,
    grabbed_item_query: Query<&Item, With<GrabbedItem>>,
    invalid_tile_query: Query<(), Or<(With<MovingFloor>, With<PlacedTower>, With<ItemSpawner>)>>,
    placed_tower_query: Query<&PlacedTower>,
    mut outline_query: Query<&mut OutlineVolume>,
) {
    if selection_changed_query.is_empty()
        && grabbed_item_changed_query.is_empty()
        && grabbed_item_removed.is_empty()
    {
        return;
    }

    for mut outline in &mut outline_query {
        outline.visible = false;
    }

    let Ok((tile, item)) = selection_changed_query.get_single() else {
        return;
    };

    let grabbed_item = grabbed_item_query.get_single();

    match grabbed_item {
        Ok(Item::LaserAmmo) => {
            // Outline towers

            if let Some(entity) = tile.0 {
                if let Ok(placed_tower) = placed_tower_query.get(entity) {
                    if let Ok(mut outline) = outline_query.get_mut(placed_tower.0) {
                        outline.visible = true;
                    }
                }
            }
        }
        Ok(Item::TowerKit) => {
            // Outline valid tiles for tower placement

            if let Some(entity) = tile.0 {
                if invalid_tile_query.get(entity).is_err() {
                    if let Ok(mut outline) = outline_query.get_mut(entity) {
                        outline.visible = true;
                    }
                }
            }
        }
        _ => {
            // No item is grabbed.
            // Check selected item to see if it is
            // grabbable, and if so outline it.

            if let Some(entity) = item.0 {
                if let Ok(mut outline) = outline_query.get_mut(entity) {
                    outline.visible = true;
                }
            }
        }
    }
}
