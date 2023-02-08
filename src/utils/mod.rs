#[cfg(feature = "adaptive")]
mod best_ordered_list;
#[cfg(any(feature = "adaptive", feature = "brute-force"))]
mod float_range;

#[cfg(feature = "adaptive")]
pub use best_ordered_list::BestOrderedList;
#[cfg(any(feature = "adaptive", feature = "brute-force"))]
pub use float_range::FloatRange;
