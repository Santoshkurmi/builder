pub enum Status {
    Error,
    Success,
    Pending,
    Building,
    Full,
    AlreadyBuilding,
    Aborted,
    NotFound,
    SomethingWentWrong,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Error => "error",
            Status::Success => "success",
            Status::Pending => "pending",
            Status::Building => "building",
            Status::Full => "full",
            Status::AlreadyBuilding => "already_building",
            Status::Aborted => "aborted",
            Status::NotFound => "not_found",
            Status::SomethingWentWrong => "something_went_wrong",
        }
    } //as_str
}
