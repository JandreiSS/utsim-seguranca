//! Telas do UTSim Segurança do Trabalho sem janela: contexto egui sem renderizador.

use std::time::{Duration, Instant};

use eframe::egui;
use utsim_posto_ui::Sessao;
use utsim_seguranca_app::{AVISO_LEGAL, App, Tela};
use utsim_seguranca_core::inventario::{ACOES, Acao, Inventario, PERIGOS, Perigo};

/// Roda um quadro e devolve todo o texto desenhado.
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

fn aviso_presente(texto: &str) -> bool {
    // O rótulo pode quebrar em linhas: compara sem espaços em branco.
    let sem = |s: &str| s.split_whitespace().collect::<String>();
    sem(texto).contains(&sem(AVISO_LEGAL))
}

const MODELO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/cruzamento_empilhadeira.ron"
);

#[test]
fn inventario_exposicao_e_relatorio() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    assert!(
        aviso_presente(&quadro(&mut app, &ctx)),
        "aviso na tela inicial"
    );

    assert!(
        app.sessao
            .criar(&dir.path().join("unidade"), "Fábrica", "Bia")
    );
    app.postos.importar = MODELO.to_string();
    app.postos.sugerir(&mut app.sessao);
    assert!(!app.postos.sugestoes.is_empty());
    for p in app.postos.sugestoes.clone() {
        app.sessao
            .unidade
            .as_mut()
            .unwrap()
            .gravar_posto(&p)
            .unwrap();
    }
    let posto = app.sessao.unidade.as_ref().unwrap().postos()[0].clone();

    // Todas as telas mostram o aviso legal.
    for t in [
        Tela::Postos,
        Tela::Inventario,
        Tela::Matriz,
        Tela::Exposicao,
        Tela::Relatorio,
    ] {
        app.tela = t;
        quadro(&mut app, &ctx);
        let texto = quadro(&mut app, &ctx);
        assert!(aviso_presente(&texto), "aviso ausente em {t:?}");
    }

    // Inventário: um perigo com nível alto sem ação, depois uma ação atrasada.
    let u = app.sessao.unidade.as_mut().unwrap();
    let pid = u
        .gravar(
            PERIGOS,
            "",
            &Perigo {
                posto: posto.id.clone(),
                descricao: "Prensagem de dedos".into(),
                probabilidade: 3,
                severidade: 4,
                ..Perigo::default()
            },
        )
        .unwrap();
    let inv = Inventario::ler(u);
    assert_eq!(inv.sem_acao().len(), 1);
    u.gravar(
        ACOES,
        "",
        &Acao {
            perigo: pid,
            descricao: "Instalar bimanual".into(),
            prazo: "2020-01-01".into(),
            ..Acao::default()
        },
    )
    .unwrap();
    assert_eq!(Inventario::ler(u).atrasadas("2026-11-06").len(), 1);
    app.tela = Tela::Inventario;
    for etapa in 0..5 {
        app.inventario.etapa = etapa;
        app.inventario.posto = posto.id.clone();
        quadro(&mut app, &ctx);
        let t = quadro(&mut app, &ctx);
        if etapa == 1 {
            assert!(t.contains("Prensagem de dedos"));
        }
    }

    // Matriz: o CSV não monótono é recusado e o erro aparece.
    let csv = dir.path().join("m.csv");
    std::fs::write(&csv, "niveis;Baixo;Alto\nseveridade;S1;S2;S3\nP1;Alto;Baixo;Baixo\nP2;Alto;Alto;Alto\nP3;Alto;Alto;Alto\n").unwrap();
    app.matriz.arquivo_csv = csv.display().to_string();
    app.tela = Tela::Matriz;
    quadro(&mut app, &ctx);
    app.matriz.importar();
    assert!(
        !app.matriz.erros.is_empty(),
        "matriz não monótona deve ser recusada"
    );
    quadro(&mut app, &ctx);

    // Exposição no fio de trabalho.
    app.tela = Tela::Exposicao;
    app.exposicao.modelo = MODELO.to_string();
    app.exposicao.rodar(&mut app.sessao);
    assert!(app.exposicao.trabalho.ocupado());
    let limite = Instant::now() + Duration::from_secs(180);
    while app.exposicao.trabalho.ocupado() {
        assert!(Instant::now() < limite, "a exposição não terminou a tempo");
        quadro(&mut app, &ctx);
        std::thread::sleep(Duration::from_millis(50));
    }
    let r = app
        .exposicao
        .trabalho
        .ok()
        .unwrap_or_else(|| panic!("erro: {:?}", app.exposicao.trabalho.erro()));
    assert!(!r.replicacoes.is_empty());
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("Quase-conflitos simulados"));
    app.exposicao.sugerir(&app.sessao);

    // Relatório HTML com o aviso legal.
    let html = utsim_seguranca_app::relatorio::TelaRelatorio::gerar(&app.sessao, &[]).unwrap();
    assert!(html.contains(&AVISO_LEGAL[..40]));
    assert!(html.contains("Prensagem de dedos"));
    let _: &Sessao = &app.sessao;
}
