//! Picture of the "Modelo" tab of UTSim Segurança do Trabalho drawn without a window: the list of
//! nodes, the results of the chosen node and its tab fields. With `UTSIM_CAPTURAS=<pasta>`
//! the PNG is saved there for review.

use std::time::{Duration, Instant};

use eframe::egui::{Context, vec2};
use utsim_seguranca_app::{App, Tela};
use utsim_ui_tema::captura::Captura;

#[test]
fn modelo_picture() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    assert!(
        app.sessao
            .criar(&dir.path().join("unidade"), "Fábrica", "Ana")
    );
    let m = dir.path().join("modelo.ron");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/cruzamento_empilhadeira.ron"
        ),
        &m,
    )
    .unwrap();
    app.tela = Tela::Modelo;
    app.modelo.abrir(&m, "Ana").unwrap();
    app.modelo.selecionar("prensa");
    let mut cap = Captura::new(Context::default(), vec2(1280.0, 860.0));
    let limite = Instant::now() + Duration::from_secs(120);
    loop {
        let _ = cap.imagem(1, |ui| app.draw(ui));
        if !app.modelo.trabalho.ocupado() {
            break;
        }
        assert!(Instant::now() < limite, "o resumo não terminou a tempo");
        std::thread::sleep(Duration::from_millis(50));
    }
    let img = cap.imagem(3, |ui| app.draw(ui));
    let mut cores = std::collections::BTreeSet::new();
    for y in (0..img.height).step_by(9) {
        for x in (0..img.width).step_by(9) {
            cores.insert(img.pixel(x, y));
        }
    }
    assert!(cores.len() > 20, "a tela quase não pintou nada");
    if let Some(d) = std::env::var_os("UTSIM_CAPTURAS") {
        let d = std::path::PathBuf::from(d);
        std::fs::create_dir_all(&d).unwrap();
        img.salvar_png(&d.join("seguranca_modelo.png")).unwrap();
    }
    app.modelo.fechar();
}
