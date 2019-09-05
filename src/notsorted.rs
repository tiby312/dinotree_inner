use crate::dinotree::*;
use crate::inner_prelude::*;


///The trait through which algorithms can use the not sorted version of the dinotree
pub trait NotSortedRefTrait where Self::Item:HasAabb<Num=Self::Num>{
    type Item:HasAabbMut<Num=Self::Num,Inner=Self::Inner>;
    type Axis:AxisTrait;
    type Num:NumTrait;
    type Inner;
    
    fn axis(&self)->Self::Axis;
    fn vistr(&self)->Vistr<Self::Item>;


    ///Return the height of the dinotree.
    #[inline]
    fn height(&self) -> usize;

    ///Return the number of nodes of the dinotree.
    #[inline]
    fn num_nodes(&self) -> usize;

    ///Return the number of bots in the tree.
    #[inline]
    fn num_bots(&self) -> usize;

}

///The mutable part of the not sorted trait.
pub trait NotSortedRefMutTrait:NotSortedRefTrait{
    fn vistr_mut(&mut self)->VistrMut<Self::Item>;
}




impl<K:NotSortedRefTrait> NotSortedRefTrait for &K{
    type Item=K::Item;
    type Axis=K::Axis;
    type Num=K::Num;
    type Inner=K::Inner;
    
    fn axis(&self)->Self::Axis{
        K::axis(self)
    }
    fn vistr(&self)->Vistr<Self::Item>{
        K::vistr(self)
    }

    ///Return the height of the dinotree.
    #[inline]
    fn height(&self) -> usize
    {
        K::height(self)
    }

    ///Return the number of nodes of the dinotree.
    #[inline]
    fn num_nodes(&self) -> usize
    {
        K::num_nodes(self)
    }

    ///Return the number of bots in the tree.
    #[inline]
    fn num_bots(&self) -> usize
    {
        K::num_bots(self)
    }

}

impl<K:NotSortedRefMutTrait> NotSortedRefTrait for &mut K{
    type Item=K::Item;
    type Axis=K::Axis;
    type Num=K::Num;
    type Inner=K::Inner;
    
    fn axis(&self)->Self::Axis{
        K::axis(self)
    }
    fn vistr(&self)->Vistr<Self::Item>{
        K::vistr(self)
    }

    ///Return the height of the dinotree.
    #[inline]
    fn height(&self) -> usize
    {
        K::height(self)
    }

    ///Return the number of nodes of the dinotree.
    #[inline]
    fn num_nodes(&self) -> usize
    {
        K::num_nodes(self)
    }

    ///Return the number of bots in the tree.
    #[inline]
    fn num_bots(&self) -> usize
    {
        K::num_bots(self)
    }

}

impl<K:NotSortedRefMutTrait> NotSortedRefMutTrait for &mut K{    
    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        K::vistr_mut(self)
    }
}

pub struct NotSorted<'a,A: AxisTrait,N:NumTrait, T>(pub DinoTree<'a,A,N,T>);



//TODO should really have own trait
impl<'a,A:AxisTrait,N:NumTrait,T> NotSortedRefTrait for NotSorted<'a,A,N,T>{
    type Item=BBoxMut<'a,N,T>;
    type Axis=A;
    type Num=N;
    type Inner=T;
    
    fn axis(&self)->Self::Axis{
        self.0.axis()
    }
    fn vistr(&self)->Vistr<Self::Item>{
        Vistr {
            inner: self.0.inner.tree.vistr(),
        }
    }

    ///Return the height of the dinotree.
    #[inline]
    fn height(&self) -> usize
    {
        self.0.height()
    }

    ///Return the number of nodes of the dinotree.
    #[inline]
    fn num_nodes(&self) -> usize
    {
        self.0.num_nodes()
    }

    ///Return the number of bots in the tree.
    #[inline]
    fn num_bots(&self) -> usize
    {
        self.0.num_bots()
    }

}


impl<'a,A:AxisTrait,N:NumTrait,T> NotSortedRefMutTrait for NotSorted<'a,A,N,T>{    
    fn vistr_mut(&mut self)->VistrMut<Self::Item>{
        VistrMut {
            inner: self.0.inner.tree.vistr_mut(),
        }
    }
}





///Builder for a DinoTree
/// # Examples
///
/// ```
/// use axgeom;
/// use dinotree::notsorted::NotSortedBuilder;
/// use dinotree_sample::SampleBuilder;
///
/// let builder = SampleBuilder::new();
/// let mut bots:Vec<_>= builder.build().take(1000).collect();
/// let mut tree=NotSortedBuilder::new(axgeom::XAXISS,&mut bots,|a|builder.create_aabb(a)).build_seq();
/// //Use tree
/// ```
pub struct NotSortedBuilder<'a, A: AxisTrait, T, Num: NumTrait, F: FnMut(&T) -> Rect<Num>> {
	inner:DinoTreeBuilder<'a,A,T,Num,F>
}


impl<'a, A: AxisTrait, T: Send+Sync, Num: NumTrait, F: FnMut(&T) -> Rect<Num>>
    NotSortedBuilder<'a, A, T, Num, F>
{

    ///Build not sorted in parallel
    pub fn build_par(&mut self) -> NotSorted<'a,A,Num,T> {
        let dlevel = compute_default_level_switch_sequential(self.inner.height_switch_seq, self.inner.height);
        

        let mut conts=self.inner.tree_prep();

        let cont_tree = create_tree_par(self.inner.axis,dlevel, &mut conts, NoSorter, &mut SplitterEmpty, self.inner.height, self.inner.rebal_strat);

        NotSorted(self.inner.tree_finish(conts,cont_tree))
        //NotSorted(self.inner.build_inner(dlevel, NoSorter, &mut SplitterEmpty))
    }
}
impl<'a, A: AxisTrait, T, Num: NumTrait, F: FnMut(&T) -> Rect<Num>>
    NotSortedBuilder<'a, A, T, Num, F>
{
    ///Create a dinotree builder.
    ///The user picks the axis along which the first divider will partition.
    ///If for example the user picks the x axis, then the first divider will be a line from top to bottom.
    ///The user also passes a function to create the bounding box of each bot in the slice passed.
    pub fn new(axis: A, bots: &mut [T], aabb_create: F) -> NotSortedBuilder<A, T, Num, F> {
        let rebal_strat = BinStrat::Checked;
        let height = compute_tree_height_heuristic(bots.len());
        let height_switch_seq = default_level_switch_sequential();

        let inner=DinoTreeBuilder {
            axis,
            bots,
            aabb_create,
            rebal_strat,
            height,
            height_switch_seq,
        };
        NotSortedBuilder{inner}
    }

    ///Choose a custom bin stratagy.
    pub fn with_bin_strat(&mut self, strat: BinStrat) -> &mut Self {
        self.inner.rebal_strat = strat;
        self
    }

    ///Choose a custom height for the tree.
    pub fn with_height(&mut self, height: usize) -> &mut Self {
        self.inner.height = height;
        self
        //TODO test corner cases of this
    }

    ///Choose the height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this argument is ignored.
    pub fn with_height_switch_seq(&mut self, height: usize) -> &mut Self {
        self.inner.height_switch_seq = height;
        self
    }

    

    ///Build a not sorted dinotree with a splitter.
    pub fn build_with_splitter_seq<S: Splitter>(
        &mut self,
        splitter: &mut S,
    ) -> NotSorted<'a,A,Num,T> {
        
        let mut conts=self.inner.tree_prep();

        let cont_tree = create_tree_seq(self.inner.axis, &mut conts, NoSorter, splitter, self.inner.height, self.inner.rebal_strat);

        NotSorted(self.inner.tree_finish(conts,cont_tree))
    }


    ///Build not sorted sequentially
    pub fn build_seq(&mut self) -> NotSorted<'a,A,Num,T> {

        let mut conts=self.inner.tree_prep();

        let cont_tree = create_tree_seq(self.inner.axis, &mut conts, NoSorter, &mut SplitterEmpty, self.inner.height, self.inner.rebal_strat);

        NotSorted(self.inner.tree_finish(conts,cont_tree))
    }


}
