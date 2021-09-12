# lame_ecs
Minimal ecs library that I did to learn a bit of rust

**features (or lack thereof)**
* static ecs: all supported components must known at compile time
* operations on non-existent/deleted entities panic

**usage**

Define a struct with a vector of option for every component the application needs:
```rs
#[derive(Default)]
struct TestComponents {
    ints: Vec<Option<i32>>,
    strs: Vec<Option<String>>,
}
```

Implement the trait ```lame_ecs::Components``` for this struct as following:
1. ```push_none``` should push ```None``` to every vector in the struct
2. ```remove``` should call ```remove(index)``` in every vector 

```rs
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
```
Implement the trait ```lame_ecs::Component``` for every component type that is in use

It is used to get the vector of components given an ```lame_ecs::Components``` and a component type

```rs
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
```

Finally it can be used as following:
```rs
let components = Box::new(TestComponents::default());
let mut ecs = Ecs::new(components);
let e0 = ecs.new_entity();
let e1 = ecs.new_entity();

let i0 = 10;
ecs.add_component::<i32>(e0, i0);

let i1 = 30;
ecs.add_component::<i32>(e1, i1);

let str_comp = "foo".to_owned();
ecs.add_component::<String>(e1, str_comp);

let str_comp = ecs.get_component::<String>(e1);

ecs.remove_component::<i32>(e0);
ecs.remove_entity(e1);
assert!(!ecs.is_alive(e1));
```
