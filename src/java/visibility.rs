use std::fmt::Display;

#[derive(Default, Debug, Clone, Copy)]
pub enum Visibility {
    #[default]
    Public,
    Private,
}

impl Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Public => write!(f, "public"),
            Visibility::Private => write!(f, "private"),
        }
    }
}
