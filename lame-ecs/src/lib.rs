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

pub trait Components {
    fn push_none(&mut self);
    fn remove(&mut self, index: usize);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait Component: Sized {
    fn get_vec(components: &mut dyn Components) -> &mut Vec<Option<Self>>;
}

pub struct World {
    entity_count: i64,
    pub entities: Vec<Entity>,
    pub components: Box<dyn Components>,
}

pub fn downcast_components<T: 'static>(c: &dyn Components) -> &T {
    c.as_any()
        .downcast_ref::<T>()
        .expect("wrong components type")
}

pub fn downcast_components_mut<T: 'static>(c: &mut dyn Components) -> &mut T {
    c.as_any_mut()
        .downcast_mut::<T>()
        .expect("wrong components type")
}

impl World {
    pub fn new(components: Box<dyn Components>) -> World {
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

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Debug, PartialEq)]
    struct Foo {
        m: i32,
    }

    #[derive(Debug, PartialEq)]
    struct Bar {
        m: String,
    }

    #[component_collection(Foo, Bar)]
    struct TestComponents {}

    #[test]
    fn new_entity() {
        let mut world = create_world!(TestComponents);
        let e0 = world.new_entity();
        let e1 = world.new_entity();
        assert_ne!(e0, e1);
    }

    #[test]
    fn add_component() {
        let mut world = create_world!(TestComponents);
        let e0 = world.new_entity();

        let c = Foo { m: 21 };
        world.add_component::<Foo>(e0, c);

        let x = world.get_component::<Foo>(e0);
        assert!(!x.is_none());
        assert_eq!(x.unwrap().m, 21);
    }

    #[test]
    fn add_existing_component() {
        let mut world = create_world!(TestComponents);
        let e0 = world.new_entity();

        let c = Foo { m: 42 };
        world.add_component::<Foo>(e0, c);

        let c = Foo { m: 17 };
        world.add_component::<Foo>(e0, c);

        let x = world.get_component::<Foo>(e0);
        assert!(!x.is_none());
        assert_eq!(x.unwrap().m, 17);
    }

    #[test]
    fn remove_component() {
        let mut world = create_world!(TestComponents);
        let e0 = world.new_entity();

        let c = Foo { m: 0 };
        world.add_component::<Foo>(e0, c);

        {
            let c = world.get_component::<Foo>(e0);
            assert!(!c.is_none());
        }

        world.remove_component::<Foo>(e0);
        let c = world.get_component::<Foo>(e0);
        assert!(c.is_none());
    }

    #[test]
    fn remove_entity() {
        let mut world = create_world!(TestComponents);
        let e0 = world.new_entity();

        let foo_comp = Foo { m: 0 };
        world.add_component::<Foo>(e0, foo_comp);
        let bar_comp = Bar {
            m: "bar".to_owned(),
        };
        world.add_component::<Bar>(e0, bar_comp);

        let e1 = world.new_entity();
        let foo_comp = Foo { m: 42 };
        world.add_component(e1, foo_comp);

        {
            let int_comp = world.get_component::<Foo>(e0);
            assert!(!int_comp.is_none());
            let str_comp = world.get_component::<Bar>(e0);
            assert!(!str_comp.is_none());
            let int_comp = world.get_component::<Foo>(e1);
            assert!(!int_comp.is_none());
        }

        assert!(world.is_alive(e0));
        world.remove_entity(e0);
        assert!(!world.is_alive(e0));

        let int_comp = world.get_component::<Foo>(e1);
        assert!(!int_comp.is_none());

        assert!(world.is_alive(e1));
        world.remove_entity(e1);
        assert!(!world.is_alive(e1));
    }

    #[test]
    fn multiple_entities() {
        let mut world = create_world!(TestComponents);
        let e0 = world.new_entity();
        let e1 = world.new_entity();

        let i0 = Foo { m: 10 };
        world.add_component::<Foo>(e0, i0);
        let i1 = Foo { m: 30 };
        world.add_component::<Foo>(e1, i1);

        {
            let i0 = world.get_component::<Foo>(e0);
            assert!(!i0.is_none());
            assert_eq!(i0.unwrap().m, 10);

            let i1 = world.get_component::<Foo>(e1);
            assert!(!i1.is_none());
            assert_eq!(i1.unwrap().m, 30);
        }

        let bar_comp = Bar {
            m: "bar".to_owned(),
        };
        world.add_component::<Bar>(e1, bar_comp);
        assert!(world.get_component::<Bar>(e0).is_none());
        assert!(!world.get_component::<Bar>(e1).is_none());

        assert!(world.is_alive(e1));
        world.remove_entity(e1);
        assert!(!world.is_alive(e1));
        assert!(!world.get_component::<Foo>(e0).is_none());
    }

    #[test]
    #[should_panic]
    fn remove_missing_entity() {
        let mut world = create_world!(TestComponents);
        let entity = world.new_entity();
        world.remove_entity(entity);
        world.remove_entity(entity);
    }

    #[test]
    #[should_panic]
    fn get_component_on_missing_entity() {
        let mut world = create_world!(TestComponents);
        let entity = world.new_entity();
        world.add_component(
            entity,
            Bar {
                m: String::from("foo"),
            },
        );
        assert!(world.get_component::<Bar>(entity).is_some());
        world.remove_entity(entity);
        world.get_component::<Bar>(entity);
    }

    #[test]
    #[should_panic]
    fn remove_component_on_missing_entity() {
        let mut world = create_world!(TestComponents);
        let entity = world.new_entity();
        world.add_component(
            entity,
            Bar {
                m: String::from("foo"),
            },
        );
        assert!(world.get_component::<Bar>(entity).is_some());
        world.remove_entity(entity);
        world.remove_component::<Bar>(entity);
    }

    #[test]
    #[should_panic]
    fn add_component_to_missing_entity() {
        let mut world = create_world!(TestComponents);
        let entity = world.new_entity();
        world.add_component(
            entity,
            Bar {
                m: String::from("foo"),
            },
        );
        assert!(world.get_component::<Bar>(entity).is_some());
        world.remove_entity(entity);
        world.add_component::<Foo>(entity, Foo { m: 42 });
    }

    #[test]
    fn test_component_iter_macro() {
        let mut world = create_world!(TestComponents);
        let e0 = world.new_entity();
        let e1 = world.new_entity();
        world.add_component(e0, Foo { m: 42 });
        world.add_component(
            e0,
            Bar {
                m: "bar0".to_owned(),
            },
        );
        world.add_component(e1, Foo { m: 84 });
        let components = get_component_collection!(world, TestComponents);

        let mut iter = component_iter!(world, components, Foo, Bar);
        assert_eq!(
            iter.next().unwrap(),
            (
                &mut Foo { m: 42 },
                &mut Bar {
                    m: "bar0".to_owned()
                },
                &e0
            )
        );
        assert_eq!(iter.next(), None);

        let mut iter = component_iter!(world, components, Foo);
        assert_eq!(iter.next().unwrap(), (&mut Foo { m: 42 }, &e0));
        assert_eq!(iter.next().unwrap(), (&mut Foo { m: 84 }, &e1));
        assert_eq!(iter.next(), None);

        let mut iter = component_iter!(world, components, Bar);
        assert_eq!(
            iter.next().unwrap(),
            (
                &mut Bar {
                    m: "bar0".to_owned()
                },
                &e0
            )
        );
        assert_eq!(iter.next(), None);
    }
}
