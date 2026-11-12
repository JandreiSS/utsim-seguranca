//! Os textos da janela, nas cinco línguas da família: as chaves de
//! `idiomas/<código>/seguranca.ftl` (`docs/traducao.md`). As telas não escrevem texto
//! literal: leem `tx("chave")` ou `fx("chave", &[("nome", vx(&valor))])`.

use std::fmt::Display;

use utsim_idiomas::{Catalogo, Valor};

/// Os textos do app nas cinco línguas, embutidos de `idiomas/`.
pub static TEXTOS: Catalogo = utsim_idiomas::catalogo!("seguranca");

/// O texto de uma chave na língua de agora.
pub fn tx(chave: &'static str) -> &'static str {
    TEXTOS.t(chave)
}

/// O texto de uma chave com valores com nome.
pub fn fx(chave: &str, args: &[(&str, Valor)]) -> String {
    TEXTOS.f(chave, args)
}

/// Um valor: número quando se lê de volta igual (o plural escolhe por ele, e ele sai com a
/// vírgula ou o ponto da língua), senão texto.
pub fn vx(x: &impl Display) -> Valor<'static> {
    let s = x.to_string();
    if let Ok(n) = s.parse::<i64>()
        && n.to_string() == s
    {
        return n.into();
    }
    if let Ok(f) = s.parse::<f64>()
        && f.is_finite()
        && f.to_string() == s
    {
        return f.into();
    }
    s.into()
}

/// Um número já com casas fixas (`{:.4}`), com a marca decimal da língua.
pub fn dx(s: String) -> Valor<'static> {
    utsim_idiomas::marca_decimal(&s).into()
}

/// Um texto como está.
pub fn sx(s: String) -> Valor<'static> {
    s.into()
}
