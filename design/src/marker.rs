/// Type that can be relocated in memory.
///
/// <div class="warning">
/// This trait doesn't actually work: a type that is `!Move` can still be moved,
/// since denying that needs compiler integration. This trait is only included
/// for being able to show how much cleaner the [`Pin`] story looks like if we
/// had this trait.
/// </div>
///
/// [`Pin`]: std::pin::Pin
#[cfg(feature = "move_trait")]
pub auto trait Move {}
