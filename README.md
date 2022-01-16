# lame_ecs
Minimal ecs library that I did to learn a bit of rust

**features (or lack thereof)**
* static ecs: all supported components must known at compile time
* operations on non-existent/deleted entities panic

**usage**

Call the macro `create_component_collection_struct` passing the type of the used components:
```rs
use lame_ecs::*

#[derive(Debug, PartialEq)]
struct Foo {
    m: i32,
}

#[derive(Debug, PartialEq)]
struct Bar {
    m: String,
}

#[component_collection(Foo, Bar)]
struct TestComponents{}
```
Use it as following:
```rs
let mut world = create_world!(TestComponents);
let e0 = world.new_entity();
let e1 = world.new_entity();

let foo0 = Foo{m: 10};
world.add_component::<Foo>(e0, foo0);

let foo1 = Foo{m: 30};
world.add_component::<Foo>(e1, foo1);

let bar = Bar{m: "bar".to_owned()};
world.add_component::<Bar>(e1, bar);

let bar = world.get_component::<Bar>(e1);

world.remove_component::<Foo>(e0);
world.remove_entity(e1);
assert!(!world.is_alive(e1));
```

**componen iteration**
Iteration on components is acomplished via the `component_iter` macro
Use the `get_component_collection` macro passing the world instance and the type of the component collection:

```rs
let components = get_component_collection!(world, TestComponents);
```
then get use the `component_iter` macro passing the world instance, component collection instance and list of components types to be iterated on:

```rs
let mut iter = component_iter!(world, components, Foo, Bar);
```

The iterator will yield a tuple with the requested components and the entity that owns then. It only yield entities that contains *ALL* of the request components


Example:

```rs
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
```
