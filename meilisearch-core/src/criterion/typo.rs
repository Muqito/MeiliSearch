use std::cmp::Ordering;

use compact_arena::SmallArena;

use crate::automaton::QueryEnhancer;
use crate::bucket_sort::{PostingsListView, QueryWordAutomaton};
use crate::RawDocument;

use super::{Criterion, Context, ContextMut, prepare_query_distances};

pub struct Typo;

impl Criterion for Typo {
    fn name(&self) -> &str { "typo" }

    fn prepare<'p, 'tag, 'txn, 'q, 'a, 'r>(
        &self,
        ctx: ContextMut<'p, 'tag, 'txn, 'q, 'a>,
        documents: &mut [RawDocument<'r, 'tag>],
    ) {
        prepare_query_distances(documents, ctx.query_enhancer, ctx.automatons, ctx.postings_lists);
    }

    fn evaluate(&self, _ctx: &Context, lhs: &RawDocument, rhs: &RawDocument) -> Ordering {
        // This function is a wrong logarithmic 10 function.
        // It is safe to panic on input number higher than 3,
        // the number of typos is never bigger than that.
        #[inline]
        fn custom_log10(n: u8) -> f32 {
            match n {
                0 => 0.0,     // log(1)
                1 => 0.30102, // log(2)
                2 => 0.47712, // log(3)
                3 => 0.60205, // log(4)
                _ => panic!("invalid number"),
            }
        }

        #[inline]
        fn compute_typos(distances: &[Option<u8>]) -> usize {
            let mut number_words: usize = 0;
            let mut sum_typos = 0.0;

            for distance in distances {
                if let Some(distance) = distance {
                    sum_typos += custom_log10(*distance);
                    number_words += 1;
                }
            }

            (number_words as f32 / (sum_typos + 1.0) * 1000.0) as usize
        }

        let lhs = compute_typos(&lhs.processed_distances);
        let rhs = compute_typos(&rhs.processed_distances);

        lhs.cmp(&rhs).reverse()
    }
}