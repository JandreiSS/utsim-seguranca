//! Começar sem pasta e salvar depois: a unidade em branco, a pergunta antes de largar o
//! trabalho, o Salvar como, o salvamento automático desligado e a pasta restrita, que só
//! existe numa unidade salva.

use eframe::egui;
use utsim_posto::{Posto, Unidade};
use utsim_posto_ui::Depois;
use utsim_posto_ui::documento::JanelaSalvar;
use utsim_seguranca_app::{App, Tela};
use utsim_ui_tema::documento::Resposta;

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

fn posto(app: &mut App, nome: &str) {
    app.sessao
        .unidade
        .as_mut()
        .unwrap()
        .gravar_posto(&Posto {
            nome: nome.into(),
            ..Posto::default()
        })
        .unwrap();
    app.sessao.mudou();
}

#[test]
fn abre_sem_pasta_pergunta_antes_de_largar_e_salva_numa_pasta_nova() {
    utsim_idiomas::definir(utsim_idiomas::Idioma::PtBr);
    let dir = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    // Abre direto no trabalho, numa unidade sem pasta.
    assert!(app.sessao.eh_rascunho());
    assert_eq!(app.tela, Tela::Postos);
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("Salvamento automático"), "{texto}");
    assert!(texto.contains("Salvar como"));
    assert!(texto.contains("Não salvo"));
    // Em branco, começar outra não pergunta nada.
    assert!(!app.sessao.nao_salvo());
    let temporaria = app.sessao.unidade.as_ref().unwrap().raiz().to_path_buf();
    posto(&mut app, "Prensa");
    assert!(app.sessao.nao_salvo());
    // Com trabalho, "Nova unidade" pergunta; Cancelar mantém tudo.
    app.documento.pedir(&mut app.sessao, Depois::Nova);
    assert!(app.documento.guarda.perguntando());
    app.documento.responder(&mut app.sessao, Resposta::Cancelar);
    assert_eq!(app.sessao.unidade.as_ref().unwrap().postos().len(), 1);

    // A pasta restrita só existe depois de salvar.
    assert!(!app.incidentes.criar_restrita(&mut app.sessao));

    // Salvar abre o "Salvar como"; salvar numa pasta nova leva o trabalho para lá.
    app.documento.salvar(&mut app.sessao);
    assert!(app.documento.janela.is_some());
    let j = JanelaSalvar {
        nome: "Fábrica Norte".into(),
        pasta: dir.path().display().to_string(),
    };
    assert!(app.documento.salvar_em(&mut app.sessao, &j));
    let pasta = j.destino().unwrap();
    assert!(!app.sessao.eh_rascunho());
    assert!(!app.sessao.nao_salvo());
    assert!(!temporaria.exists());
    assert_eq!(Unidade::abrir(&pasta, "bia").unwrap().postos().len(), 1);
    assert!(app.sessao.config.recentes().contains(&pasta));
    assert!(app.sessao.automatico());
    assert!(quadro(&mut app, &ctx).contains("Salvo"));
    assert!(app.incidentes.criar_restrita(&mut app.sessao));

    // Desligado, o que se faz espera o Salvar, e a escolha fica guardada para a unidade.
    app.sessao.definir_automatico(false);
    posto(&mut app, "Solda");
    assert!(app.sessao.nao_salvo());
    assert_eq!(Unidade::abrir(&pasta, "bia").unwrap().postos().len(), 1);
    app.documento.salvar(&mut app.sessao);
    assert!(!app.sessao.nao_salvo());
    assert_eq!(Unidade::abrir(&pasta, "bia").unwrap().postos().len(), 2);
    assert!(
        app.sessao
            .config
            .sem_automatico
            .contains(&pasta.display().to_string())
    );

    // "Não salvar" larga o que esperava e começa em branco.
    posto(&mut app, "Pintura");
    app.documento.pedir(&mut app.sessao, Depois::Nova);
    app.documento
        .responder(&mut app.sessao, Resposta::NaoSalvar);
    assert!(app.sessao.eh_rascunho());
    assert!(app.sessao.unidade.as_ref().unwrap().postos().is_empty());
    assert_eq!(Unidade::abrir(&pasta, "bia").unwrap().postos().len(), 2);

    // A unidade de antes abre como sempre, com o automático desligado lembrado.
    app.documento
        .pedir(&mut app.sessao, Depois::Abrir(pasta.clone()));
    assert!(!app.sessao.eh_rascunho());
    assert!(!app.sessao.automatico());
    assert_eq!(app.sessao.unidade.as_ref().unwrap().postos().len(), 2);
}
