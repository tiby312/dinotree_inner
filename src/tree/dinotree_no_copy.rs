
use super::*;




pub struct DinoTreeNoCopyBuilder<'a,A:AxisTrait,N:Copy,T:HasAabb+Copy>{
    axis:A,
    n:N,
    bots:&'a mut [T],
    rebal_strat:RebalStrat,
    height:usize,
    height_switch_seq:usize
}
impl<'a,A:AxisTrait,N:Copy,T:HasAabb+Copy> DinoTreeNoCopyBuilder<'a,A,N,T>{

    fn new(axis:A,n:N,bots:&'a mut [T])->DinoTreeNoCopyBuilder<'a,A,N,T>{
        let rebal_strat=RebalStrat::First;
        let height=compute_tree_height_heuristic(bots.len());
        let height_switch_seq=default_level_switch_sequential();

        DinoTreeNoCopyBuilder{axis,n,bots,rebal_strat,height,height_switch_seq}
    }


    pub fn build_seq(self)->DinoTreeNoCopy<'a,A,N,T>{
        self.build_inner(par::Sequential,DefaultSorter,&mut crate::advanced::SplitterEmpty)
    }

    pub fn build_par(self)->DinoTreeNoCopy<'a,A,N,T>{
        let dlevel=compute_default_level_switch_sequential(self.height_switch_seq,self.height);
        self.build_inner(dlevel,DefaultSorter,&mut crate::advanced::SplitterEmpty)
    }


    fn build_inner<JJ:par::Joiner,K:Splitter+Send>
        (self,par:JJ,sorter:impl Sorter,ka:&mut K)->DinoTreeNoCopy<'a,A,N,T>
    {   
        let axis=self.axis;
        let n=self.n;
        let bots=self.bots;

        let height=self.height;
        let rebal_type=self.rebal_strat;



        let bots2=unsafe{&mut *(bots as *mut [_])};
        use crate::tree::cont_tree::*;
        

        let num_bots=bots.len();
        let max=std::u32::MAX;
        
        assert!(num_bots < max as usize,"problems of size {} are bigger are not supported",max);


        let mut conts:Vec<_>=bots.iter().enumerate().map(|(index,k)|{
            Cont2{rect:*k.get(),index:index as u32}
        }).collect();


    
        
        let binstrat=match rebal_type{
            RebalStrat::First=>{
                BinStrat::LeftMidRight
            },
            RebalStrat::Second=>{
                BinStrat::MidLeftRight
            },
            RebalStrat::Third=>{
                BinStrat::LeftRightMid
            }
        };

        let mut cont_tree=ContTree::new(axis,par,&mut conts,sorter,ka,height,binstrat);




        let new_bots={
            impl<Num:NumTrait> reorder::HasIndex for Cont2<Num>{
                fn get(&self)->usize{
                    self.index as usize
                }
                fn set(&mut self,index:usize){
                    self.index=index as u32;
                }
            }
            //bots
            reorder::reorder(bots,cont_tree.get_conts_mut())
        };


        let new_tree={
            let new_nodes={
                let mut rest:Option<&mut [T]>=Some(new_bots);
                let mut new_nodes=Vec::with_capacity(cont_tree.get_tree().get_nodes().len());
                for node in cont_tree.get_tree_mut().dfs_inorder_iter(){
                    let (b,rest2)=rest.take().unwrap().split_at_mut(node.mid.len());
                    rest=Some(rest2);
                    new_nodes.push(Node3{n,fullcomp:node.fullcomp,mid:unsafe{std::ptr::Unique::new_unchecked(b as *mut [_])}});
                }
                new_nodes
            };

            compt::dfs_order::CompleteTreeContainer::from_vec(new_nodes).unwrap()
        };

        let mover=cont_tree.get_conts().iter().map(|a|crate::tree::dinotree_no_copy::Index(a.index)).collect();


        DinoTreeNoCopy{mover,axis,bots:bots2,nodes:new_tree}

    }
}


pub struct Index(pub u32);
impl reorder::HasIndex for Index{
    fn get(&self)->usize{
        self.0 as usize
    }
    fn set(&mut self,index:usize){
        self.0=index as u32;
    }
}


///A version where the bots are not copied. This means that the slice borrowed from the user
///must remain borrowed for the entire lifetime of the tree.
pub struct DinoTreeNoCopy<'a,A:AxisTrait,N,T:HasAabb>{
    axis:A,
    bots:&'a mut [T],
    nodes:compt::dfs_order::CompleteTreeContainer<Node3<N,T>,compt::dfs_order::InOrder>,
    mover:Vec<Index>
}

impl<'a,A:AxisTrait,N:Copy,T:HasAabb+Copy> DinoTreeNoCopy<'a,A,N,T>{

    ///Safe to assume aabb_create is called for each bot in the slice in order.
    ///Parallelization is done using rayon crate.
    #[inline]
    pub fn new(axis:A,n:N,bots:&'a mut [T])->DinoTreeNoCopy<'a,A,N,T>{  
        DinoTreeNoCopyBuilder::new(axis,n,bots).build_par() 
    }

    pub fn new_seq(axis:A,n:N,bots:&'a mut [T])->DinoTreeNoCopy<'a,A,N,T>{ 
        DinoTreeNoCopyBuilder::new(axis,n,bots).build_seq() 
    }

    ///Returns the bots to their original ordering. This is what you would call after you used this tree
    ///to make the changes you made while querying the tree (through use of vistr_mut) be copied back into the original list.
    pub fn into_original(mut self)->&'a mut [T]{
        reorder::reorder(self.bots,&mut self.mover)
    }

    pub fn as_ref_mut(&mut self)->DinoTreeRefMut<A,N,T>{
        DinoTreeRefMut{axis:self.axis,bots:self.bots,tree:&mut self.nodes}
    }
    pub fn as_ref(&self)->DinoTreeRef<A,N,T>{
        DinoTreeRef{axis:self.axis,bots:self.bots,tree:&self.nodes}
    }

}