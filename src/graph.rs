use crate::error::Error;
use crate::graph::MaybeProcessed::{Processed, Skip};
use crate::set::Set;
use derive_more::{From, Into};
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

#[derive(From, Into)]
pub struct StateWithWeightAndMetadata<S, W: Ord, M>(S, W, M);

impl<S, W: Ord, M> Eq for StateWithWeightAndMetadata<S, W, M> {}
impl<S, W: Ord, M> PartialEq<Self> for StateWithWeightAndMetadata<S, W, M> {
    fn eq(&self, other: &Self) -> bool {
        self.1.eq(&other.1)
    }
}

impl<S, W: Ord, M> PartialOrd<Self> for StateWithWeightAndMetadata<S, W, M> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S, W: Ord, M> Ord for StateWithWeightAndMetadata<S, W, M> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
    }
}

pub enum MaybeProcessed<T> {
    Processed(T),
    Skip(T),
}

pub struct Dijkstra<State, Weight, Metadata, VisitedStateSet, NeighborFnObj>
where
    Weight: Ord,
    VisitedStateSet: Set<State>,
{
    pub queue: BinaryHeap<Reverse<StateWithWeightAndMetadata<State, Weight, Metadata>>>,
    pub neighbor_fn: NeighborFnObj,
    pub visited: VisitedStateSet,
}

impl<State, Weight, Metadata, VisitedStateSet, NeighborFnObj> Iterator
    for Dijkstra<State, Weight, Metadata, VisitedStateSet, NeighborFnObj>
where
    State: Clone,
    Weight: Ord,
    VisitedStateSet: Set<State>,
    NeighborFnObj: NeighborFn<State, Weight, Metadata>,
{
    type Item = Result<MaybeProcessed<(State, Weight, Metadata)>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Reverse(state_weight_metadata)) = self.queue.pop() {
            let (state, weight, metadata) = state_weight_metadata.into();
            match self.visited.insert(state.clone()) {
                Err(e) => return Some(Err(e)),
                Ok(false) => return Some(Ok(Skip((state, weight, metadata)))),
                Ok(true) => self
                    .neighbor_fn
                    .get_neighbors(&state, &weight, &metadata)
                    .into_iter()
                    .map(StateWithWeightAndMetadata::from)
                    .map(Reverse)
                    .for_each(|swm| self.queue.push(swm)),
            }

            return Some(Ok(Processed((state, weight, metadata))));
        }

        None
    }
}

pub trait NeighborFn<State, Weight, Metadata> {
    fn get_neighbors(
        &mut self,
        state: &State,
        weight: &Weight,
        metadata: &Metadata,
    ) -> impl IntoIterator<Item = (State, Weight, Metadata)>;
}
