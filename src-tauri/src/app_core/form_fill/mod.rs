mod field_focus;
mod field_fill;
mod form_submit;

pub(crate) use field_focus::resolve_direct_focus_field_command;
pub(crate) use field_fill::resolve_direct_fill_command_internal;
pub(crate) use form_submit::resolve_direct_submit_form_command;

#[cfg(test)]
pub(crate) use field_fill::resolve_direct_fill_field_command;
#[cfg(test)]
pub(crate) use field_fill::resolve_direct_fill_and_submit_command;
