use shipyard::*;
use vermarine_lib::components::*;
use vermarine_lib::starter::components::*;

pub fn new_game(
    mut entities: EntitiesViewMut, 
    mut transforms: ViewMut<Transform>, 
    mut sprites: ViewMut<Sprite>
) {
    entities.add_entity((&mut transforms, &mut sprites), (
        Transform::new(640f64, 360f64, 16f64),
        Sprite { texture: "circle" }
    ));
}