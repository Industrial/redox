use redox_ast::Param;

/// One parameter: a name, optionally `mut`, optionally typed.
pub(crate) fn param_text(param: &Param) -> String {
  let name = if param.is_mut {
    format!("mut {}", param.name.text)
  } else {
    param.name.text.clone()
  };
  match &param.ty {
    Some(ty) => format!("({name} : {})", ty.text),
    None => name,
  }
}
