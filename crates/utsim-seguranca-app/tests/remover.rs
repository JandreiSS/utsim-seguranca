//! Remover com confirmação nas listas do Segurança: incidente e inspeção gravada.

use eframe::egui;
use utsim_seguranca_app::{App, Tela};
use utsim_seguranca_core::checklist::{INSPECOES, InspecaoMaquina};
use utsim_seguranca_core::incidentes::{INCIDENTES, Incidente};

fn quadro(app: &mut App, ctx: &egui::Context) -> String {
    let mut saida = ctx.run_ui(egui::RawInput::default(), |ui| app.draw(ui));
    saida.textures_delta.clear();
    let mut texto = String::new();
    for c in &saida.shapes {
        if let egui::epaint::Shape::Text(t) = &c.shape {
            texto.push_str(t.galley.text());
            texto.push('\n');
        }
    }
    texto
}

/// Põe o botão de remover `(tag, id)` no estado "à espera de confirmação", como depois do
/// primeiro clique (o tema guarda esse estado na memória do egui).
fn pedir_confirmacao(ctx: &egui::Context, tag: &str, id: &str) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(("utsim_botao_confirmado", (tag, id))), true));
}

#[test]
fn incidente_e_inspecao_saem_da_unidade_depois_de_confirmar() {
    utsim_idiomas::definir(utsim_idiomas::Idioma::PtBr);
    let dir = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    assert!(
        app.sessao
            .criar(&dir.path().join("unidade"), "Fábrica", "Bia")
    );
    let u = app.sessao.unidade.as_mut().unwrap();
    // A tela Checklist só lista as inspeções numa unidade com postos.
    let p1 = u
        .gravar_posto(&utsim_posto::Posto {
            nome: "Prensas".into(),
            ..utsim_posto::Posto::default()
        })
        .unwrap();
    let inc = u
        .gravar(
            INCIDENTES,
            "",
            &Incidente {
                data: "2026-11-10".into(),
                descricao: "Corte no dedo".into(),
                ..Incidente::default()
            },
        )
        .unwrap();
    let insp = u
        .gravar(
            INSPECOES,
            "",
            &InspecaoMaquina {
                posto: p1.clone(),
                maquina: "Prensa 2".into(),
                modelo: "proprio".into(),
                data: "2026-11-11".into(),
                ..InspecaoMaquina::default()
            },
        )
        .unwrap();
    app.sessao.mudou();

    // Incidentes: o botão Remover aparece; com a remoção pendente, a confirmação.
    app.tela = Tela::Incidentes;
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("Corte no dedo"));
    assert!(texto.contains("Remover"));
    assert!(!texto.contains("Confirmar"));
    pedir_confirmacao(&ctx, "incidente", &inc);
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("Confirmar"), "{texto}");
    app.incidentes.remover(&mut app.sessao, &inc);
    let texto = quadro(&mut app, &ctx);
    assert!(!texto.contains("Corte no dedo"));

    // Checklist: a inspeção gravada sai, e a aberta fecha junto.
    app.tela = Tela::Checklist;
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("Prensa 2"));
    assert!(!texto.contains("Confirmar"));
    pedir_confirmacao(&ctx, "inspecao", &insp);
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("Confirmar"), "{texto}");
    let aberta = utsim_seguranca_app::checklist::TelaChecklist::inspecoes(&app.sessao);
    app.checklist.inspecao = aberta.into_iter().next();
    app.checklist.remover_inspecao(&mut app.sessao, &insp);
    assert!(app.checklist.inspecao.is_none());
    assert!(utsim_seguranca_app::checklist::TelaChecklist::inspecoes(&app.sessao).is_empty());
}
