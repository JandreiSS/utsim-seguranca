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

#[test]
fn inventario_mostra_os_perigos_do_no() {
    use utsim_edit::{Document, abas};
    let dir = tempfile::tempdir().unwrap();
    // Um perigo gravado no nó da prensa, numa cópia do modelo.
    let copia = dir.path().join("cruzamento.ron");
    std::fs::copy(MODELO, &copia).unwrap();
    let mut d = Document::open(&copia).unwrap();
    for f in [
        (|m: &utsim_model::Model| abas::editar(m, "prensa", "safety", "sim"))
            as fn(&utsim_model::Model) -> Result<_, String>,
        |m| abas::adicionar(m, "prensa", "safety.hazards"),
        |m| {
            abas::editar(
                m,
                "prensa",
                "safety.hazards[0].description",
                "Ruído da prensa",
            )
        },
        |m| abas::editar(m, "prensa", "safety.hazards[0].probability", "3"),
        |m| abas::editar(m, "prensa", "safety.hazards[0].severity", "3"),
    ] {
        let c = f(d.model()).unwrap();
        d.apply(c).unwrap();
    }
    d.save().unwrap();

    let ctx = egui::Context::default();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    assert!(
        app.sessao
            .criar(&dir.path().join("unidade"), "Fábrica", "Bia")
    );
    app.postos.importar = copia.display().to_string();
    app.postos.sugerir(&mut app.sessao);
    for p in app.postos.sugestoes.clone() {
        app.sessao
            .unidade
            .as_mut()
            .unwrap()
            .gravar_posto(&p)
            .unwrap();
    }
    let u = app.sessao.unidade.as_ref().unwrap();
    let posto = u
        .postos()
        .into_iter()
        .find(|p| p.objeto.as_deref() == Some("prensa"))
        .unwrap();
    // A unidade não tem perigo nenhum: o do nó chega pelo modelo.
    assert!(Inventario::ler(u).perigos.is_empty());
    let inv = Inventario::ler_com_nos(u);
    assert_eq!(inv.perigos_do_posto(&posto.id).len(), 1);
    assert_eq!(inv.sem_acao().len(), 1);

    app.tela = Tela::Inventario;
    for etapa in [1, 2] {
        app.inventario.etapa = etapa;
        app.inventario.posto = posto.id.clone();
        quadro(&mut app, &ctx);
        let t = quadro(&mut app, &ctx);
        assert!(t.contains("Ruído da prensa"), "etapa {etapa}: {t}");
        if etapa == 1 {
            assert!(t.contains("no nó do modelo"), "{t}");
        }
    }
    // O relatório também leva o perigo do nó.
    let html = utsim_seguranca_app::relatorio::TelaRelatorio::gerar(&app.sessao, &[]).unwrap();
    assert!(html.contains("Ruído da prensa"));
}

#[test]
fn inventario_leva_o_perigo_da_unidade_para_o_no() {
    use std::sync::Arc;
    use utsim_edit::abas::Aba;
    use utsim_seguranca_app::inventario::PedidoNo;
    use utsim_seguranca_core::inventario::{REAVALIACOES, Reavaliacao};
    let dir = tempfile::tempdir().unwrap();
    let copia = dir.path().join("cruzamento.ron");
    std::fs::copy(MODELO, &copia).unwrap();
    let ctx = egui::Context::default();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    assert!(
        app.sessao
            .criar(&dir.path().join("unidade"), "Fábrica", "Bia")
    );
    app.postos.importar = copia.display().to_string();
    app.postos.sugerir(&mut app.sessao);
    let u = app.sessao.unidade.as_mut().unwrap();
    for p in app.postos.sugestoes.clone() {
        u.gravar_posto(&p).unwrap();
    }
    let posto = u
        .postos()
        .into_iter()
        .find(|p| p.objeto.as_deref() == Some("prensa"))
        .unwrap();
    // Um perigo da unidade, com uma ação e uma reavaliação.
    let perigo = Perigo {
        posto: posto.id.clone(),
        grupo: utsim_seguranca_core::inventario::Grupo::Quimico,
        descricao: "Névoa de óleo".into(),
        fonte: "Lubrificação da prensa".into(),
        probabilidade: 3,
        severidade: 2,
        ..Perigo::default()
    };
    let id = u.gravar(PERIGOS, "", &perigo).unwrap();
    let perigo = Perigo {
        id: id.clone(),
        ..perigo
    };
    u.gravar(
        ACOES,
        "",
        &Acao {
            perigo: id.clone(),
            descricao: "Exaustão local".into(),
            ..Acao::default()
        },
    )
    .unwrap();
    u.gravar(
        REAVALIACOES,
        "",
        &Reavaliacao {
            perigo: id.clone(),
            data: "2026-11-01".into(),
            probabilidade: 2,
            severidade: 2,
            ..Reavaliacao::default()
        },
    )
    .unwrap();
    let levar = PedidoNo::Levar {
        modelo: copia.clone(),
        objeto: "prensa".into(),
        perigo: Box::new(perigo.clone()),
    };

    // Outro app edita o modelo: nada muda, e a mensagem diz quem edita.
    let mut outro = utsim_posto_ui::TelaModelo::nova(
        "qualidade",
        Aba::Qualidade,
        Arc::new(|_| Ok(Default::default())),
    );
    outro.abrir(&copia, "Caio").unwrap();
    app.tela = Tela::Inventario;
    app.inventario.posto = posto.id.clone();
    app.inventario.pedido = Some(levar.clone());
    quadro(&mut app, &ctx);
    assert!(
        app.sessao.mensagem.contains("Caio"),
        "{}",
        app.sessao.mensagem
    );
    let u = app.sessao.unidade.as_ref().unwrap();
    assert_eq!(Inventario::ler(u).perigos.len(), 1);
    outro.fechar();

    // Livre: o perigo vai para o nó, e a ação e a reavaliação seguem com ele.
    app.inventario.pedido = Some(levar);
    quadro(&mut app, &ctx);
    assert!(!app.sessao.erro, "{}", app.sessao.mensagem);
    let m = utsim_model::Model::load(&copia).unwrap();
    let hs = &m
        .objects
        .iter()
        .find(|o| o.name == "prensa")
        .unwrap()
        .safety
        .as_ref()
        .unwrap()
        .hazards;
    assert_eq!(hs.len(), 1);
    assert_eq!(hs[0].description, "Névoa de óleo");
    assert_eq!(hs[0].source, "Lubrificação da prensa");
    assert_eq!(hs[0].group, utsim_model::safety::HazardGroup::Chemical);
    assert_eq!((hs[0].probability, hs[0].severity), (3, 2));
    let novo = format!("no:{}:{}", posto.id, hs[0].id);
    let u = app.sessao.unidade.as_ref().unwrap();
    assert!(Inventario::ler(u).perigos.is_empty());
    let inv = Inventario::ler_com_nos(u);
    assert_eq!(inv.acoes_do_perigo(&novo).len(), 1);
    assert_eq!(inv.ultima_reavaliacao(&novo).unwrap().probabilidade, 2);
    // A trava foi solta: o outro app volta a editar.
    outro.abrir(&copia, "Caio").unwrap();
    assert!(outro.pode_editar());
    outro.fechar();

    // "Editar no nó" abre a tela Modelo no nó.
    app.inventario.pedido = Some(PedidoNo::Abrir {
        modelo: copia.clone(),
        objeto: "prensa".into(),
    });
    quadro(&mut app, &ctx);
    assert_eq!(app.tela, Tela::Modelo);
    assert_eq!(app.modelo.arquivo(), Some(copia.as_path()));
}
