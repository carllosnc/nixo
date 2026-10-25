//! Biblioteca de componentes do menu de configurações.
//!
//! Cada elemento visual existe em um único lugar. As telas (`tabs`,
//! `subtitle_style`) apenas compõem estes componentes; o visual é controlado
//! pelos tokens em [`crate::popovers::style`].

mod blocks;
mod controls;
mod icons;
mod primitives;
mod rows;

pub(crate) use blocks::{
    Choice, EmptyState, PopoverFrame, WarningBanner, choice_list, submenu_header,
};
#[allow(unused_imports)]
pub(crate) use controls::MenuToggle;
pub(crate) use controls::{delay_stepper, menu_slider};
pub(crate) use icons::LeadingIcon;
pub(crate) use rows::{MenuButton, MenuDivider, MenuOption, NavRow, SectionTitle};
