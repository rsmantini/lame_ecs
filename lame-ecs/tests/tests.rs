#[derive(Debug, PartialEq)]
pub struct Foo {
    m: i32,
}

#[derive(Debug, PartialEq)]
pub struct Bar {
    m: String,
}

lame_ecs::create_component_collection!(Foo, Bar);

#[test]
fn new_entity() {
    let mut world = lame_ecs::create_world!();
    let e0 = world.new_entity();
    let e1 = world.new_entity();
    assert_ne!(e0, e1);
}

#[test]
fn add_component() {
    let mut world = lame_ecs::create_world!();
    let e0 = world.new_entity();

    let c = Foo { m: 21 };
    world.add_component::<Foo>(e0, c);

    let x = world.get_component::<Foo>(e0);
    assert!(!x.is_none());
    assert_eq!(x.unwrap().m, 21);
}

#[test]
fn add_existing_component() {
    let mut world = lame_ecs::create_world!();
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
    let mut world = lame_ecs::create_world!();
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
    let mut world = lame_ecs::create_world!();
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
    let mut world = lame_ecs::create_world!();
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
    let mut world = lame_ecs::create_world!();
    let entity = world.new_entity();
    world.remove_entity(entity);
    world.remove_entity(entity);
}

#[test]
#[should_panic]
fn get_component_on_missing_entity() {
    let mut world = lame_ecs::create_world!();
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
    let mut world = lame_ecs::create_world!();
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
    let mut world = lame_ecs::create_world!();
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
    let mut world = lame_ecs::create_world!();
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

    let mut iter = lame_ecs::component_iter!(world, Foo, Bar);
    assert_eq!(
        iter.next().unwrap(),
        (
            &Foo { m: 42 },
            &Bar {
                m: "bar0".to_owned()
            },
            &e0
        )
    );
    assert_eq!(iter.next(), None);

    let mut iter = lame_ecs::component_iter_mut!(world, Foo);
    assert_eq!(iter.next().unwrap(), (&mut Foo { m: 42 }, &e0));
    assert_eq!(iter.next().unwrap(), (&mut Foo { m: 84 }, &e1));
    assert_eq!(iter.next(), None);

    let mut iter = lame_ecs::component_iter!(world, Bar);
    assert_eq!(
        iter.next().unwrap(),
        (
            &Bar {
                m: "bar0".to_owned()
            },
            &e0
        )
    );
    assert_eq!(iter.next(), None);
}
