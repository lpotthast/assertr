use assertr_derive::AssertrEq;

#[derive(AssertrEq)]
pub struct InvalidBounds {
    #[assertr_eq(compare_with = "compare", compare_bounds = "u32 + u64")]
    pub value: u32,
}

fn main() {}
