//! Aba "Modelo" do UTSim Segurança do Trabalho sem janela: abre, desenha e resume a
//! exposição por nó.

use std::time::{Duration, Instant};

use eframe::egui;
use utsim_seguranca_app::{App, Tela};

fn quadros(app: &mut App, ctx: &egui::Context, n: usize) {
    for _ in 0..n {
        let mut saida = ctx.run_ui(egui::RawInput::default(), |ui| app.draw(ui));
        saida.textures_delta.clear();
    }
}

#[test]
fn aba_modelo_abre_e_resume_a_exposicao() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    assert!(
        app.sessao
            .criar(&dir.path().join("unidade"), "Fábrica", "Bruno")
    );
    let m = dir.path().join("cruzamento.ron");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/cruzamento_empilhadeira.ron"
        ),
        &m,
    )
    .unwrap();

    app.tela = Tela::Modelo;
    quadros(&mut app, &ctx, 2);
    app.modelo.abrir(&m, "Bruno").unwrap();
    assert!(app.modelo.pode_editar());
    let primeiro = app.modelo.objetos().into_iter().next().unwrap();
    app.modelo.selecionar(&primeiro);
    quadros(&mut app, &ctx, 3);

    let limite = Instant::now() + Duration::from_secs(120);
    while app.modelo.trabalho.ocupado() {
        assert!(Instant::now() < limite, "o resumo não terminou a tempo");
        quadros(&mut app, &ctx, 1);
        std::thread::sleep(Duration::from_millis(50));
    }
    let r = app
        .modelo
        .trabalho
        .ok()
        .unwrap_or_else(|| panic!("erro: {:?}", app.modelo.trabalho.erro()));
    let geral = r.get("").expect("linhas do modelo inteiro");
    assert!(geral.iter().any(|(k, v)| k == "Origem" && v == "Simulado"));
    app.modelo.fechar();
}
