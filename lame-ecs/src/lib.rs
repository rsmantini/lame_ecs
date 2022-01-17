pub use itertools;
pub use lame_ecs_macro::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Entity {
    id: i64,
}

impl Entity {
    pub fn new(id: i64) -> Entity {
        Entity { id }
    }

    pub fn id(&self) -> i64 {
        self.id
    }
}

pub trait ComponentCollection {
    fn push_none(&mut self);
    fn remove(&mut self, index: usize);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait Component: Sized {
    fn get_vec(components: &mut dyn ComponentCollection) -> &mut Vec<Option<Self>>;
}

pub struct World {
    entity_count: i64,
    pub entities: Vec<Entity>,
    pub components: Box<dyn ComponentCollection>,
}

pub fn downcast_components<T: 'static>(c: &dyn ComponentCollection) -> &T {
    c.as_any()
        .downcast_ref::<T>()
        .expect("wrong components type")
}

pub fn downcast_components_mut<T: 'static>(c: &mut dyn ComponentCollection) -> &mut T {
    c.as_any_mut()
        .downcast_mut::<T>()
        .expect("wrong components type")
}

impl World {
    pub fn new(components: Box<dyn ComponentCollection>) -> World {
        World {
            entity_count: 0,
            entities: Vec::new(),
            components,
        }
    }

    pub fn new_entity(&mut self) -> Entity {
        if self.entity_count == core::i64::MAX {
            panic!("maximum number of entities reached");
        }
        let id = self.entity_count;
        self.entity_count += 1;
        self.components.push_none();
        self.entities.push(Entity { id });
        Entity { id }
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        let index = self
            .get_index(entity)
            .expect("trying to remove invalid entity");
        self.entities.remove(index);
        self.components.remove(index);
    }

    pub fn get_component<C: Component>(&mut self, entity: Entity) -> Option<&mut C> {
        self.get_component_option(entity).as_mut()
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) {
        *self.get_component_option::<C>(entity) = Some(component);
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) {
        self.get_component_option::<C>(entity).take();
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.get_index(entity).is_some()
    }

    fn get_index(&self, entity: Entity) -> Option<usize> {
        self.entities.iter().position(|x| x.id == entity.id)
    }

    fn get_component_option<C: Component>(&mut self, entity: Entity) -> &mut Option<C> {
        let index = self.get_index(entity).expect("operation on invalid entity");
        &mut C::get_vec(self.components.as_mut())[index]
    }
}
