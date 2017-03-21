pub enum CompareType {
    KNearest(usize),
    Dist(u64),
    DistRatio(f64),
}
