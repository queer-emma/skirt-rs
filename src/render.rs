use std::sync::Arc;

use parking_lot::RwLock;
use rust_decimal::{
    prelude::ToPrimitive,
    Decimal,
};
use svg::{
    Document,
    Node,
};

use crate::{
    aabb::{
        AsAABB,
        Rect,
        AABB,
    },
    error::Error,
};

#[derive(Debug, Default)]
struct Inner {
    /// note: this needs to be an option, so we can replace it easily. but the
    /// invariant should hold that it's always put back. the option is
    /// created empty, but the document will be lazily created when needed
    document: Option<Document>,

    /// this tracks how large the view box is.
    view_box: AABB<Decimal>,
}

/// thread-safe clonable wrapper around the document.
///
/// note: we also need to wrap it into an option to be able to replace the
/// value.
#[derive(Debug)]
pub struct Target {
    inner: Arc<RwLock<Inner>>,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl Target {
    pub fn add<T: Node>(&mut self, node: T) {
        let mut inner = self.inner.write();

        let document = inner.document.take().unwrap_or_else(Document::new);

        let document = document.add(node);

        inner.document = Some(document);
    }

    pub fn resize_for(&mut self, aabb: impl AsAABB<Decimal>) {
        let mut inner = self.inner.write();
        inner.view_box.insert(aabb);
    }

    fn build(self) -> Document {
        let mut inner = self.inner.write();

        let document = inner.document.take().unwrap_or_else(Document::new);

        let view_box = Rect::from(inner.view_box);

        document.set(
            "viewBox",
            (
                view_box.top_left().x.to_f64().unwrap(),
                view_box.top_left().y.to_f64().unwrap(),
                view_box.bottom_right().x.to_f64().unwrap(),
                view_box.bottom_right().y.to_f64().unwrap(),
            ),
        )
    }
}

/// todo: rename to distinguish from 3d renderer, e.g. `RenderPattern`.
pub trait Render {
    type Context;

    fn render(&self, target: &mut Target, context: &Self::Context) -> Result<(), Error>;
}
