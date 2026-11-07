//! Tela do cadastro de EPI sem janela: contexto egui sem renderizador.

use eframe::egui;
use utsim_seguranca_app::epi::TelaEpi;
use utsim_seguranca_app::{App, Tela};
use utsim_seguranca_core::checklist::Epi;

const MODELO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/cruzamento_empilhadeira.ron"
);

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

#[test]
fn epi_cadastra_e_mostra_pendencias() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    assert!(
        app.sessao
            .criar(&dir.path().join("unidade"), "Fábrica", "Bia")
    );
    app.postos.importar = MODELO.to_string();
    app.postos.sugerir(&mut app.sessao);
    for p in app.postos.sugestoes.clone() {
        app.sessao
            .unidade
            .as_mut()
            .unwrap()
            .gravar_posto(&p)
            .unwrap();
    }
    // Só o primeiro posto exige capacete.
    for (i, mut posto) in app
        .sessao
        .unidade
        .as_ref()
        .unwrap()
        .postos()
        .into_iter()
        .enumerate()
    {
        posto.epi = if i == 0 {
            vec!["capacete".into()]
        } else {
            Vec::new()
        };
        app.sessao
            .unidade
            .as_mut()
            .unwrap()
            .gravar_posto(&posto)
            .unwrap();
    }

    app.tela = Tela::Epi;
    let p = TelaEpi::pendencias(&app.sessao);
    assert_eq!(p.len(), 1);
    assert!(p[0].contains("não está no cadastro de EPI"));
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("não está no cadastro de EPI"));

    // Cadastra com o CA vencido: a pendência muda.
    app.epi.edicao = Some(Epi {
        tipo: "capacete".into(),
        ca: "12345".into(),
        validade_ca: "2020-01-01".into(),
        troca_dias: Some(365),
        ..Epi::default()
    });
    assert!(app.epi.gravar(&mut app.sessao));
    assert_eq!(TelaEpi::lista(&app.sessao).len(), 1);
    let p = TelaEpi::pendencias(&app.sessao);
    assert_eq!(p.len(), 1);
    assert!(p[0].contains("CA com validade vencida"));
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("(vencido)"));

    // Sem tipo não grava; renovando o CA a pendência some.
    app.epi.edicao = Some(Epi::default());
    assert!(!app.epi.gravar(&mut app.sessao));
    let mut e = TelaEpi::lista(&app.sessao).remove(0);
    e.validade_ca = "2099-01-01".into();
    app.epi.edicao = Some(e);
    assert!(app.epi.gravar(&mut app.sessao));
    assert!(TelaEpi::pendencias(&app.sessao).is_empty());
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("Nenhuma pendência."));

    // Remove.
    let id = TelaEpi::lista(&app.sessao)[0].id.clone();
    app.epi.remover(&mut app.sessao, &id);
    assert!(TelaEpi::lista(&app.sessao).is_empty());
}
