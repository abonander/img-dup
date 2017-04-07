/// Struct for types which are safe to clone from a concurrently shared reference,
/// but which opt out of `Sync` for other reasons.
#[derive(Clone)]
pub struct CloneCell<T>(T);

impl<T> CloneCell<T> {
    /// ### Safety
    /// The user must be sure that `T` is safe to clone from a concurrently shared reference.
    pub unsafe fn new(val: T) -> Self {
        CloneCell(val)
    }

    pub fn clone_inner(&self) -> T where T: Clone {
        self.0.clone()
    }
}

unsafe impl<T> Sync for CloneCell<T> {}

pub unsafe trait SyncClone: Clone {}
unsafe impl<T: Clone + Sync> SyncClone for T {}
