//! Tela do checklist de máquina sem janela: contexto egui sem renderizador.

use eframe::egui;
use utsim_seguranca_app::{App, Tela};
use utsim_seguranca_core::checklist::{AVISO_NR12, Resposta};
use utsim_seguranca_core::inventario::{PERIGOS, Perigo};

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

fn contem(texto: &str, trecho: &str) -> bool {
    let sem = |s: &str| s.split_whitespace().collect::<String>();
    sem(texto).contains(&sem(trecho))
}

fn app_com_postos(dir: &std::path::Path) -> App {
    let mut app = App::new(Some(dir.join("cfg.json")));
    assert!(app.sessao.criar(&dir.join("unidade"), "Fábrica", "Bia"));
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
    app
}

#[test]
fn checklist_responde_grava_e_leva_nao_conformidades_ao_inventario() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    let mut app = app_com_postos(dir.path());
    app.tela = Tela::Checklist;
    let texto = quadro(&mut app, &ctx);
    assert!(contem(&texto, AVISO_NR12), "o aviso da NR-12 deve aparecer");
    assert!(texto.contains("Checklist próprio do UTSim"));

    // Modelo da empresa por CSV, guardado na unidade.
    let csv = dir.path().join("empresa.csv");
    std::fs::write(
        &csv,
        "tema;item;severidade\nProteção;Grade presente;4\nSinal;Placa legível;2\n",
    )
    .unwrap();
    app.checklist.arquivo_csv = csv.display().to_string();
    app.checklist.nome_modelo = "Roteiro da empresa".into();
    assert!(app.checklist.importar_csv(&mut app.sessao));
    assert_eq!(app.checklist.modelo.itens.len(), 2);
    assert!(app.checklist.guardar_modelo(&mut app.sessao));
    assert_eq!(
        utsim_seguranca_app::checklist::TelaChecklist::guardados(&app.sessao).len(),
        1
    );
    app.checklist.usar_proprio();
    assert_eq!(app.checklist.modelo.itens.len(), 16);

    // Sem posto e máquina não começa; levar antes de gravar é recusado.
    app.checklist.posto.clear();
    assert!(!app.checklist.nova(&mut app.sessao));
    app.checklist.posto = app.sessao.unidade.as_ref().unwrap().postos()[0].id.clone();
    app.checklist.maquina = "Prensa 1".into();
    assert!(app.checklist.nova(&mut app.sessao));
    assert!(app.checklist.levar_ao_inventario(&mut app.sessao).is_none());

    app.checklist.responder("p01", Resposta::NaoConforme);
    app.checklist
        .inspecao
        .as_mut()
        .unwrap()
        .respostas
        .get_mut("p01")
        .unwrap()
        .observacao = "Falta a grade traseira".into();
    app.checklist.responder("p02", Resposta::Conforme);
    app.checklist.responder("p03", Resposta::NaoSeAplica);
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("Inspeção de Prensa 1"));
    assert!(texto.contains("1 não conforme(s)"));

    // Só vai para a unidade ao gravar.
    assert!(utsim_seguranca_app::checklist::TelaChecklist::inspecoes(&app.sessao).is_empty());
    assert!(app.checklist.gravar(&mut app.sessao));
    assert_eq!(
        utsim_seguranca_app::checklist::TelaChecklist::inspecoes(&app.sessao).len(),
        1
    );

    // Leva ao inventário uma vez só.
    assert_eq!(app.checklist.levar_ao_inventario(&mut app.sessao), Some(1));
    assert_eq!(app.checklist.levar_ao_inventario(&mut app.sessao), Some(0));
    let perigos: Vec<Perigo> = app.sessao.unidade.as_ref().unwrap().estado().lista(PERIGOS);
    assert_eq!(perigos.len(), 1);
    assert!(perigos[0].origem.starts_with("checklist:"));
    assert!(perigos[0].descricao.contains("Prensa 1"));
    assert_eq!(perigos[0].posto, app.checklist.posto);

    // Reabre a inspeção gravada.
    let g = utsim_seguranca_app::checklist::TelaChecklist::inspecoes(&app.sessao).remove(0);
    app.checklist.inspecao = None;
    app.checklist.abrir(&app.sessao, g);
    assert_eq!(
        app.checklist.inspecao.as_ref().unwrap().respostas["p01"].resposta,
        Resposta::NaoConforme
    );
}
