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
    fn get_vec(components: &mut Box<dyn Components>) -> &mut Vec<Option<Self>>;
}

pub struct Ecs {
    entity_count: i64,
    pub entities: Vec<Entity>,
    pub components: Box<dyn Components>,
}

pub fn downcast_components<T: 'static>(c: &Box<dyn Components>) -> &T {
    c.as_any()
        .downcast_ref::<T>()
        .expect("wrong components type")
}

pub fn downcast_components_mut<T: 'static>(c: &mut Box<dyn Components>) -> &mut T {
    c.as_any_mut()
        .downcast_mut::<T>()
        .expect("wrong components type")
}

impl Ecs {
    pub fn new(components: Box<dyn Components>) -> Ecs {
        Ecs {
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
        self.get_component_option::<C>(entity).insert(component);
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
        &mut C::get_vec(&mut self.components)[index]
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Default)]
    struct TestComponents {
        ints: Vec<Option<i32>>,
        strs: Vec<Option<String>>,
    }

    impl Components for TestComponents {
        fn push_none(&mut self) {
            self.ints.push(None);
            self.strs.push(None);
        }
        fn remove(&mut self, index: usize) {
            self.ints.remove(index);
            self.strs.remove(index);
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    impl Component for i32 {
        fn get_vec(components: &mut Box<dyn Components>) -> &mut Vec<Option<Self>> {
            &mut downcast_components_mut::<TestComponents>(components).ints
        }
    }

    impl Component for String {
        fn get_vec(components: &mut Box<dyn Components>) -> &mut Vec<Option<Self>> {
            &mut downcast_components_mut::<TestComponents>(components).strs
        }
    }

    #[test]
    fn new_entity() {
        let components = Box::new(TestComponents::default());
        let mut ecs = Ecs::new(components);
        let e0 = ecs.new_entity();
        let e1 = ecs.new_entity();
        assert_ne!(e0, e1);
    }

    #[test]
    fn add_component() {
        let components = Box::new(TestComponents::default());
        let mut ecs = Ecs::new(components);
        let e0 = ecs.new_entity();

        let c = 21;
        ecs.add_component::<i32>(e0, c);

        let x = ecs.get_component::<i32>(e0);
        assert!(!x.is_none());
        assert_eq!(*x.unwrap(), 21);
    }

    #[test]
    fn add_existing_component() {
        let components = Box::new(TestComponents::default());
        let mut ecs = Ecs::new(components);
        let e0 = ecs.new_entity();

        let c = 42;
        ecs.add_component::<i32>(e0, c);

        let c = 17;
        ecs.add_component::<i32>(e0, c);

        let x = ecs.get_component::<i32>(e0);
        assert!(!x.is_none());
        assert_eq!(*x.unwrap(), 17);
    }

    #[test]
    fn remove_component() {
        let components = Box::new(TestComponents::default());
        let mut ecs = Ecs::new(components);
        let e0 = ecs.new_entity();

        let c = 0;
        ecs.add_component::<i32>(e0, c);

        {
            let c = ecs.get_component::<i32>(e0);
            assert!(!c.is_none());
        }

        ecs.remove_component::<i32>(e0);
        let c = ecs.get_component::<i32>(e0);
        assert!(c.is_none());
    }

    #[test]
    fn remove_entity() {
        let components = Box::new(TestComponents::default());
        let mut ecs = Ecs::new(components);
        let e0 = ecs.new_entity();

        let int_comp = 0;
        ecs.add_component::<i32>(e0, int_comp);
        let str_comp = "foo".to_owned();
        ecs.add_component::<String>(e0, str_comp);

        let e1 = ecs.new_entity();
        let int_comp = 42;
        ecs.add_component(e1, int_comp);

        {
            let int_comp = ecs.get_component::<i32>(e0);
            assert!(!int_comp.is_none());
            let str_comp = ecs.get_component::<String>(e0);
            assert!(!str_comp.is_none());
            let int_comp = ecs.get_component::<i32>(e1);
            assert!(!int_comp.is_none());
        }

        assert!(ecs.is_alive(e0));
        ecs.remove_entity(e0);
        assert!(!ecs.is_alive(e0));

        let int_comp = ecs.get_component::<i32>(e1);
        assert!(!int_comp.is_none());

        assert!(ecs.is_alive(e1));
        ecs.remove_entity(e1);
        assert!(!ecs.is_alive(e1));
    }

    #[test]
    fn multiple_entities() {
        let components = Box::new(TestComponents::default());
        let mut ecs = Ecs::new(components);
        let e0 = ecs.new_entity();
        let e1 = ecs.new_entity();

        let i0 = 10;
        ecs.add_component::<i32>(e0, i0);
        let i1 = 30;
        ecs.add_component::<i32>(e1, i1);

        {
            let i0 = ecs.get_component::<i32>(e0);
            assert!(!i0.is_none());
            assert_eq!(*i0.unwrap(), 10);

            let i1 = ecs.get_component::<i32>(e1);
            assert!(!i1.is_none());
            assert_eq!(*i1.unwrap(), 30);
        }

        let str_comp = "foo".to_owned();
        ecs.add_component::<String>(e1, str_comp);
        assert!(ecs.get_component::<String>(e0).is_none());
        assert!(!ecs.get_component::<String>(e1).is_none());

        assert!(ecs.is_alive(e1));
        ecs.remove_entity(e1);
        assert!(!ecs.is_alive(e1));
        assert!(!ecs.get_component::<i32>(e0).is_none());
    }

    #[test]
    #[should_panic]
    fn remove_missing_entity() {
        let components = Box::new(TestComponents::default());
        let mut ecs = Ecs::new(components);
        let entity = ecs.new_entity();
        ecs.remove_entity(entity);
        ecs.remove_entity(entity);
    }

    #[test]
    #[should_panic]
    fn get_component_on_missing_entity() {
        let components = Box::new(TestComponents::default());
        let mut ecs = Ecs::new(components);
        let entity = ecs.new_entity();
        ecs.add_component(entity, String::from("foo"));
        assert!(ecs.get_component::<String>(entity).is_some());
        ecs.remove_entity(entity);
        ecs.get_component::<String>(entity);
    }

    #[test]
    #[should_panic]
    fn remove_component_on_missing_entity() {
        let components = Box::new(TestComponents::default());
        let mut ecs = Ecs::new(components);
        let entity = ecs.new_entity();
        ecs.add_component(entity, String::from("foo"));
        assert!(ecs.get_component::<String>(entity).is_some());
        ecs.remove_entity(entity);
        ecs.remove_component::<String>(entity);
    }

    #[test]
    #[should_panic]
    fn add_component_to_missing_entity() {
        let components = Box::new(TestComponents::default());
        let mut ecs = Ecs::new(components);
        let entity = ecs.new_entity();
        ecs.add_component(entity, String::from("foo"));
        assert!(ecs.get_component::<String>(entity).is_some());
        ecs.remove_entity(entity);
        ecs.add_component::<i32>(entity, 42);
    }
}
