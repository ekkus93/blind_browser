mod click_focus;
mod element_queries;
mod text_entry;

pub(crate) use text_entry::resolve_typeable_element;

pub(crate) use click_focus::resolve_clickable_element;
#[cfg(test)]
pub(crate) use text_entry::resolve_form_element;
