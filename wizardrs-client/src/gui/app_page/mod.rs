use strum::{Display, EnumIter};

pub(crate) mod host_page;
pub(crate) mod join_page;

#[derive(Debug, Display, EnumIter, Eq, PartialEq)]
pub enum AppPage {
    Host,
    Join,
}
