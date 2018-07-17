use inner_prelude::*;
use base_kdtree::KdTree;
use HasAabb;
use tree_alloc::NdIterMut;
use tree_alloc::NdIter;
use compt::CTreeIterator;
use axgeom::*;


#[derive(Copy,Clone)]
pub struct BBox<N:NumTrait,T>{
    rect:Rect<N>,
    pub inner:T
}
impl<N:NumTrait,T> HasAabb for BBox<N,T>{
    type Num=N;
    fn get(&self)->&Rect<Self::Num>{
        &self.rect
    }
}

pub mod fast_alloc{
    use super::*;
    pub fn new<JJ:par::Joiner,K:TreeTimerTrait,F:Fn(&T)->Rect<Num>,A:AxisTrait,N:Copy,T:Copy,Num:NumTrait>(axis:A,n:N,bots:&[T],mut aabb_create:F)->(DynTree<A,N,BBox<Num,T>>,K::Bag){   
        let height=compute_tree_height_heuristic(bots.len());

        pub struct Cont2<N:NumTrait>{
            rect:Rect<N>,
            pub index:u32
        }
        impl<N:NumTrait> HasAabb for Cont2<N>{
            type Num=N;
            fn get(&self)->&Rect<N>{
                &self.rect
            }
        }

        let num_bots=bots.len();
        let max=std::u32::MAX;
        assert!(num_bots < max as usize,"problems of size {} are bigger are not supported");


        let mut conts:Vec<Cont2<Num>>=bots.iter().enumerate().map(|(index,k)|{
            Cont2{rect:aabb_create(k),index:index as u32}
        }).collect();
        
        {
            let (mut tree2,_bag)=KdTree::new::<JJ,K>(axis,&mut conts,height);
            
            
            let mover={

                let kk:Vec<u32>=tree2.get_tree().create_down().dfs_preorder_iter().flat_map(|(node,_extra)|{
                    node.range.iter()
                }).map(|a|a.index).collect();

                Mover(kk)
            };
            

            let height=tree2.get_tree().get_height();                
            let num_nodes=tree2.get_tree().get_nodes().len();


            let tree={
                let ii=tree2.get_tree_mut().create_down_mut().map(|node,eextra|{
                    let l=tree_alloc::LeafConstructor{misc:n,it:node.range.iter_mut().map(|b|{
                        BBox{rect:b.rect,inner:bots[b.index as usize]}
                    })};

                    let extra=match eextra{
                        Some(())=>{
                            Some(tree_alloc::ExtraConstructor{
                                comp:Some(node.div)
                            })
                        },
                        None=>{
                            None
                        }
                    };

                    (l,extra)
                });

                TreeAllocDstDfsOrder::new(ii,num_nodes,num_bots)
            };

            let fb=DynTreeRaw{axis,height,num_nodes,num_bots,alloc:tree};
            (DynTree{mover,tree:fb},_bag)
        }
    }
}



/// The tree this crate revoles around.
pub struct DynTree<A:AxisTrait,N,T:HasAabb>{
    mover:Mover,
    tree:DynTreeRaw<A,N,T>,
}

impl<A:AxisTrait,N:Copy,T:Copy,Num:NumTrait> DynTree<A,N,BBox<Num,T>>{
    pub fn new(axis:A,n:N,bots:&[T],aabb_create:impl Fn(&T)->Rect<Num>)->DynTree<A,N,BBox<Num,T>>{   
        fast_alloc::new::<par::Parallel,treetimer::TreeTimerEmpty,_,_,_,_,_>(axis,n,bots,aabb_create).0
    }
    pub fn new_seq(axis:A,n:N,bots:&[T],aabb_create:impl Fn(&T)->Rect<Num>)->DynTree<A,N,BBox<Num,T>>{   
        fast_alloc::new::<par::Sequential,treetimer::TreeTimerEmpty,_,_,_,_,_>(axis,n,bots,aabb_create).0
    }

    pub fn with_debug(axis:A,n:N,bots:&[T],aabb_create:impl Fn(&T)->Rect<Num>)->DynTree<A,N,BBox<Num,T>>{   
        fast_alloc::new::<par::Parallel,treetimer::TreeTimer2,_,_,_,_,_>(axis,n,bots,aabb_create).0
    }

    pub fn with_debug_seq(axis:A,n:N,bots:&[T],aabb_create:impl Fn(&T)->Rect<Num>)->DynTree<A,N,BBox<Num,T>>{   
        fast_alloc::new::<par::Parallel,treetimer::TreeTimer2,_,_,_,_,_>(axis,n,bots,aabb_create).0
    }
}

impl<A:AxisTrait,N:Copy,T:HasAabb+Copy> DynTree<A,N,T>{
    
    ///Returns the bots to their original ordering.
    pub fn into_iter_orig_order(self)->impl ExactSizeIterator<Item=T>{
        let mut ret=Vec::with_capacity(self.mover.0.len());
        unsafe{ret.set_len(self.mover.0.len())};

        for ((node,_),mov) in self.tree.get_iter().dfs_preorder_iter().zip(self.mover.0.iter()){
            for bot in node.range.iter(){
                ret[*mov as usize]=*bot;
            }
        }
        ret.into_iter()
    }

    ///Transform the current tree to have a different extra component to each node.
    pub fn with_extra<N2:Copy>(self,n2:N2)->DynTree<A,N2,T>{
        let (mover,fb)={
            let axis=self.tree.get_axis();
            

            let height=self.get_height();
            let num_nodes=self.tree.get_num_nodes();
            let num_bots=self.tree.get_num_bots();

            let mover=self.mover.clone();
            let ii=self.get_iter().map(|node,eextra|{
                let l=tree_alloc::LeafConstructor{misc:n2,it:node.range.iter().map(|b|*b)};

                let extra=match eextra{
                    Some(extra)=>{
                        Some(tree_alloc::ExtraConstructor{
                            comp:extra.map(|a|*a)
                        })
                    },
                    None=>{
                        None
                    }
                };

                (l,extra)
            });
            
            let tree=TreeAllocDstDfsOrder::new(ii,num_nodes,num_bots);
            (mover,DynTreeRaw{axis,height,num_nodes,num_bots,alloc:tree})
        };

        DynTree{mover,tree:fb}
    }
}
impl<A:AxisTrait,N:Copy,T:HasAabb> DynTree<A,N,T>{

    ///Think twice before using this as this data structure is not optimal for linear traversal of the bots.
    ///Instead, prefer to iterate through all the bots before the tree is constructed.
    pub fn iter_every_bot_mut<'a>(&'a mut self)->impl Iterator<Item=&'a mut T>{
        self.get_iter_mut().dfs_preorder_iter().flat_map(|(a,_)|a.range.iter_mut())
    }

    ///Think twice before using this as this data structure is not optimal for linear traversal of the bots.
    ///Instead, prefer to iterate through all the bots before the tree is constructed.
    pub fn iter_every_bot<'a>(&'a self)->impl Iterator<Item=&'a T>{
        self.get_iter().dfs_preorder_iter().flat_map(|(a,_)|a.range.iter())
    }
    
    
    ///Compute a metric to determine how healthy the tree is based on how many bots
    ///live in higher nodes verses lower nodes. Ideally all bots would live in leaves.
    pub fn compute_tree_health(&self)->f64{
        unimplemented!();
    }

    ///If this function does not panic, then this tree's invariants are being met.
    pub fn debug_assert_invariants(&self){
        unimplemented!();
    }
    ///Get the axis of the starting divider.
    ///If this were the x axis, for example, the first dividing line would be from top to bottom,
    ///partitioning the bots by their x values.
    pub fn get_axis(&self)->A{
        self.tree.get_axis()
    }

    ///Get the height of the tree.
    pub fn get_height(&self)->usize{
        self.tree.get_height()
    }

    ///Create a mutable tree visitor.
    pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
        self.tree.get_iter_mut()
    }

    ///Create an immutable tree visitor.
    pub fn get_iter<'b>(&'b self)->NdIter<'b,N,T>{
        self.tree.get_iter()
    }


    ///Returns the number of bots that are in the tree.
    pub fn get_num_bots(&self)->usize{
        self.tree.num_bots
    }
}

//TODO get rid of this layer. It doesnt add anything.
use tree_alloc::TreeAllocDstDfsOrder;

pub struct DynTreeRaw<A:AxisTrait,N,T:HasAabb>{
    height:usize,
    num_nodes:usize,
    num_bots:usize,
    alloc:TreeAllocDstDfsOrder<N,T>,
    axis:A
}

impl<A:AxisTrait,N,T:HasAabb> DynTreeRaw<A,N,T>{
   

    pub fn get_axis(&self)->A{
        self.axis
    }
    pub fn get_num_nodes(&self)->usize{
        self.num_nodes
    }
    pub fn get_num_bots(&self)->usize{
        self.num_bots
    }
    pub fn get_height(&self)->usize{
        self.height
    }
    pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'b,N,T>{
        self.alloc.get_iter_mut()
    }
    pub fn get_iter<'b>(&'b self)->NdIter<'b,N,T>{
        self.alloc.get_iter()
    }
}



#[derive(Clone)]
pub struct Mover(
    pub Vec<u32>
);
