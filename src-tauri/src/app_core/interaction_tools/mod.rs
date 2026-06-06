mod element_queries;
mod click_focus;
mod text_entry;

pub(crate) use text_entry::resolve_typeable_element;

#[cfg(test)]
pub(crate) use click_focus::resolve_clickable_element;
#[cfg(test)]
pub(crate) use text_entry::resolve_form_element;
