use crate::inner_prelude::*;
use axgeom::AxisTrait;
use compt::Visitor;

use crate::tree::Vistr;


#[must_use]
///Returns false if the tree's invariants are not met.
pub fn assert_invariants<K:DinoTreeRefTrait>(tree:K)->bool{
    inner(tree.axis(), tree.vistr().with_depth(compt::Depth(0))).is_ok()
}
fn a_bot_has_value<N: NumTrait>(it: impl Iterator<Item = N>, val: N) -> bool {
    for b in it {
        if b == val {
            return true;
        }
    }
    false
}

fn inner<'a,A: AxisTrait, T: HasAabbMut>(
    axis: A,
    iter: compt::LevelIter<Vistr<T>>,
) -> Result<(), ()> {
    macro_rules! assert2 {
        ($bla:expr) => {
            if !$bla {
                return Err(());
            }
        };
    }

    let ((_depth, nn), rest) = iter.next();

    let axis_next = axis.next();

    let f = |a: &BBoxRef<T::Num,T::Inner>, b: &BBoxRef<T::Num,T::Inner>| -> Option<core::cmp::Ordering> {
        let j=a.rect
            .get_range(axis_next)
            .left
            .cmp(&b.rect.get_range(axis_next).left);
        Some(j)
    };

    {
        use is_sorted::IsSorted;
        assert2!(IsSorted::is_sorted_by(&mut nn.bots.iter(),f));
        //assert2!(nn.bots.iter().is_sorted_by(f));
    }

    if let Some([left, right]) = rest {
        match nn.div {
            Some(div) => {
                match nn.cont {
                    Some(cont) => {
                        for bot in nn.bots.iter() {
                            assert2!(bot.rect.get_range(axis).contains(*div));
                        }

                        assert2!(a_bot_has_value(
                            nn.bots.iter().map(|b| b.rect.get_range(axis).left),
                            *div
                        ));

                        for bot in nn.bots.iter() {
                            assert2!(cont.contains_range(bot.rect.get_range(axis)));
                        }

                        assert2!(a_bot_has_value(
                            nn.bots.iter().map(|b| b.rect.get_range(axis).left),
                            cont.left
                        ));
                        assert2!(a_bot_has_value(
                            nn.bots.iter().map(|b| b.rect.get_range(axis).right),
                            cont.right
                        ));
                    }
                    None => assert2!(nn.bots.is_empty()),
                }

                inner(axis_next, left)?;
                inner(axis_next, right)?;
            }
            None => {
                for (_depth, n) in left.dfs_preorder_iter().chain(right.dfs_preorder_iter()) {
                    assert2!(n.bots.is_empty());
                    assert2!(n.cont.is_none());
                    assert2!(n.div.is_none());
                }
            }
        }
    }
    Ok(())
}
