use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct Scope {
    sub_scopes: Vec<Rc<RefCell<Scope>>>,
    parent_scope: Option<Weak<RefCell<Scope>>>,
    labels: HashMap<String, i32>,
}

pub fn get_base_scope() -> Rc<RefCell<Scope>> {
    let root: Rc<RefCell<Scope>> = Rc::new(RefCell::new(Scope {
        sub_scopes: Vec::new(),
        parent_scope: None,
        labels: HashMap::new(),
    }));
    return root;
}

pub fn add_sub_scope(parent: &Rc<RefCell<Scope>>) -> Rc<RefCell<Scope>> {
    let child = Rc::new(RefCell::new(Scope {
        sub_scopes: Vec::new(),
        parent_scope: Some(Rc::downgrade(&parent)),
        labels: HashMap::new(),
    }));
    parent.borrow_mut().sub_scopes.push(Rc::clone(&child));
    return child;
}
