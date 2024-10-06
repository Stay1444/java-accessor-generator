#[derive(Default, Debug, Clone, Copy)]
pub enum Visibility {
    #[default]
    Public,
    Private,
}

impl Visibility {
    pub fn to_string(self) -> String {
        match self {
            Visibility::Public => "public",
            Visibility::Private => "private",
        }
        .to_owned()
    }
}
