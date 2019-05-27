//!
//! # Overview
//!
//! Provides the dinotree data structure and ways to traverse it. No actual query algorithms are provided in this crate.
//! Only the data structure and a way to construct and traverse it are provided in this crate.
//! 
//! The tree is comprised of copies of objects (rather than references) sorted to improve cache coherency. 
//! There is an alternative NoCopyDinoTree that does not allocate more space,
//! but instead rearranges the bots in a user provided slice for better cache coherency. 
//!
//! ~~~~text
//! 2d Tree Divider Representation:
//!
//!
//!    o   ┆┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┃         ┆         o
//!  ┈┈┈┈┈┈┆     o      o     ┃     o   ┆   o                 o
//!  ───────o─────────────────┃         o┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈
//!                ┆       o  o   o     ┆
//!        o       ┆    o     ┃┈┈┈┈┈o┈┈┈┆       o
//!                ┆   o      ┃         o             o
//!                ┆┈┈┈┈┈┈┈┈┈┈┃         ┆                   o
//!      o         o    o     ┃───────o────────────────────────
//!                ┆          ┃                ┆   o
//!  ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆      o   o   o            ┆┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈
//!     o          ┆          ┃┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆         o
//!          o     ┆   o      ┃        o       ┆   o
//!                ┆          ┃                ┆
//!
//! Axis alternates every level.
//! Divider placement is placed at the median at each level.
//! Objects that intersect a divider belong to that node.
//! Every divider keeps track of how thick a line would have to be
//! to cover all the bots it owns.
//! All the objects in a node are sorted along that node's axis.
//! 
//! ~~~~
//!
//! # Unsafety
//!
//! The HasAabb trait is marked as unsafe. See its description.
//! Unsafety used to have slices of bots in the tree, but also a slice of all the bots
//! so that we can efficiently return a slice of all the bots.
//! Unsafety is used to reuse code between sequential and parallel build algorithms.
//!


#[cfg(all(feature = "unstable", test))]
extern crate test;

extern crate axgeom;
extern crate compt;
extern crate is_sorted;
extern crate itertools;
extern crate pdqselect;
extern crate rayon;
extern crate reorder;

mod inner_prelude {
    pub use axgeom::*;
    pub(crate) use compt;
    pub use itertools::Itertools;
    pub use std::iter::*;
    pub use std::marker::PhantomData;
    pub use std::mem::*;
    pub use std::time::Instant;

    pub(crate) use super::*;
    pub(crate) use crate::advanced::Splitter;
    pub(crate) use crate::compt::Depth;
    pub(crate) use crate::compt::Visitor;
    pub(crate) use crate::advanced::par;
    pub(crate) use crate::tree;
    pub(crate) use crate::tree::*;
}


pub use assert_invariants::assert_invariants;
mod assert_invariants;

mod tree;

pub use crate::tree::DinoTreeRefTrait;
pub use crate::tree::DinoTreeRefMutTrait;
pub use crate::tree::NodeRef;
pub use crate::tree::NodeRefMut;
pub use crate::tree::Vistr;
pub use crate::tree::VistrMut;
pub use crate::tree::Node;

///Contains code to construct the dyntree.
///Main property is that the nodes and the bots are all copied into one
///segment of memory.
mod tools;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;



///A version of dinotree where the elements are copied directly into the tree.
pub mod copy;
///A version where the bots are not copied. This means that the slice borrowed from the user
///must remain borrowed for the entire lifetime of the tree.
pub mod nocopy;
///A version of a dinotree where the bots that belong to a node are not
///sorted along an axis. So this is really a regular kd-tree.
pub mod notsorted;

///Provies some debugging and misc functions.
pub mod advanced;

///The underlying number type used for the dinotree.
///It is auto implemented by all types that satisfy the type constraints.
///Notice that no arithmatic is possible. The tree is constructed
///using only comparisons and copying.
pub trait NumTrait: Ord + Copy + Send + Sync + std::fmt::Debug {}

impl<T> NumTrait for T where T: Ord + Copy + Send + Sync + std::fmt::Debug {}

///Marker trait to signify that this object has an axis aligned bounding box.
///If two HasAabb objects have aabb's that do not intersect, then it must be safe to have a mutable reference
///to each simultaneously.
///
///The aabb must not change while the object is contained in the tree, even though
///many query algorithms will give mutable references to the elements in the tree.
///So multiple calls to get() must return the same bounding box region while the object is in the tree.
///Not doing so would violate invariants of the tree, and would thus make all the
///query algorithms performed on the tree would not be correct.
///
///Not only will the algorithms not be correct, but undefined behavior may be introduced.
///Some algorithms rely on the positions of the bounding boxes to determined if two aabbs can
///be mutably borrowed at the same time. For example the multirect algorithm makes this assumption.
pub unsafe trait HasAabb {
    type Num: NumTrait;
    fn get(&self) -> &axgeom::Rect<Self::Num>;
}

///A wrapper type around a type T and bounding box where the bounding box is hidden.
///This is what is inserted into the tree. This way the user
///cannot modify the bounding box since it is hidden, with only read access.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BBox<N: NumTrait, T> {
    rect: axgeom::Rect<N>,
    pub inner: T,
}

use std::fmt::Debug;
use std::fmt::Formatter;

impl<N: NumTrait + Debug, T: Debug> Debug for BBox<N, T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.rect.fmt(f)?;
        self.inner.fmt(f)
    }
}

impl<N: NumTrait, T> BBox<N, T> {
    ///Unsafe since user could create a new BBox with a different aabb
    ///inside of a callback function and assign it to the mutable reference.
    #[inline]
    pub unsafe fn new(rect: axgeom::Rect<N>, inner: T) -> BBox<N, T> {
        BBox { rect, inner }
    }

    ///Unsafe since user could call this function
    ///using a mutable reference from inside of a callback function
    ///of a dinotree query function.
    #[inline]
    pub unsafe fn set_aabb(&mut self,aabb:axgeom::Rect<N>){
        self.rect=aabb;
    }
}

unsafe impl<N: NumTrait, T> HasAabb for BBox<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &axgeom::Rect<Self::Num> {
        &self.rect
    }
}
