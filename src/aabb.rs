use std::ops::Sub;

use derive_more::{
    AsRef,
    From,
    Into,
};
use nalgebra::{
    Point2,
    Scalar,
    Vector2,
};
use rust_decimal::prelude::Zero;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Rect<T>
where
    T: Clone + Scalar + Zero,
{
    // x: left (<0) to right (>0)
    // y: top (<0) to bottom (>0)
    top_left: Point2<T>,
    bottom_right: Point2<T>,
}

impl<T> Rect<T>
where
    T: Clone + Scalar + Zero,
{
    pub fn new(top_left: Point2<T>, bottom_right: Point2<T>) -> Self {
        Self {
            top_left,
            bottom_right,
        }
    }

    pub fn top_left(&self) -> &Point2<T> {
        &self.top_left
    }

    pub fn bottom_right(&self) -> &Point2<T> {
        &self.bottom_right
    }
}

impl<T> Rect<T>
where
    T: Copy + Clone + Scalar + Zero + Sub<T, Output = T>,
{
    pub fn width(&self) -> T {
        self.bottom_right.x - self.top_left.x
    }

    pub fn height(&self) -> T {
        self.bottom_right.y - self.top_left.y
    }

    pub fn size(&self) -> Vector2<T> {
        Vector2::from([self.width(), self.height()])
    }
}

#[derive(Copy, Clone, Debug, Default, From, Into, AsRef)]
pub struct AABB<T>(Rect<T>)
where
    T: Clone + Scalar + Zero;

impl<T> AABB<T>
where
    T: Copy + Clone + Scalar + Zero + PartialOrd,
{
    pub fn insert_point(&mut self, point: Point2<T>) {
        if point.x < self.0.top_left.x {
            self.0.top_left.x = point.x
        }
        if point.y < self.0.top_left.y {
            self.0.top_left.y = point.y
        }

        if point.x > self.0.bottom_right.x {
            self.0.top_left.x = point.x
        }
        if point.y > self.0.bottom_right.y {
            self.0.top_left.y = point.y
        }
    }

    pub fn insert(&mut self, aabb: impl AsAABB<T>) {
        let aabb = aabb.as_aabb();
        self.insert_point(aabb.0.top_left);
        self.insert_point(aabb.0.bottom_right);
    }
}

pub trait AsAABB<T>
where
    T: Clone + Scalar + Zero + PartialOrd,
{
    fn as_aabb(&self) -> AABB<T>;
}

impl<T> AsAABB<T> for Point2<T>
where
    T: Copy + Clone + Scalar + Zero + PartialOrd,
{
    fn as_aabb(&self) -> AABB<T> {
        AABB::from(Rect::new(*self, *self))
    }
}

impl<T> AsAABB<T> for Rect<T>
where
    T: Copy + Clone + Scalar + Zero + PartialOrd,
{
    fn as_aabb(&self) -> AABB<T> {
        AABB::from(*self)
    }
}
