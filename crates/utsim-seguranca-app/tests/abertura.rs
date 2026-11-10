//! A abertura do app desenhada sem janela: a primeira vez (sem autor) e com uma unidade
//! aberta. Com `UTSIM_CAPTURAS=<pasta>` os PNG ficam lá para conferência.

use eframe::egui::{Context, vec2};
use utsim_posto_ui::TelaInicial;
use utsim_seguranca_app::App;
use utsim_ui_tema::captura::{Captura, Imagem};

fn conferir(img: &Imagem, nome: &str) {
    let mut cores = std::collections::BTreeSet::new();
    for y in (0..img.height).step_by(7) {
        for x in (0..img.width).step_by(7) {
            cores.insert(img.pixel(x, y));
        }
    }
    assert!(cores.len() > 12, "{nome}: a tela quase não pintou nada");
    if let Some(d) = std::env::var_os("UTSIM_CAPTURAS") {
        let d = std::path::PathBuf::from(d);
        std::fs::create_dir_all(&d).unwrap();
        img.salvar_png(&d.join(format!("{nome}.png"))).unwrap();
    }
}

#[test]
fn abertura_picture() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    let ctx = Context::default();
    utsim_ui_tema::install(&ctx);
    let mut cap = Captura::new(ctx, vec2(1440.0, 900.0));
    let img = cap.imagem(3, |ui| app.draw(ui));
    conferir(&img, "seguranca_abertura_primeira_vez");
    assert!(
        app.sessao
            .criar(&dir.path().join("Fábrica Norte"), "Fábrica Norte", "Ana")
    );
    app.inicial = TelaInicial::default();
    let img = cap.imagem(3, |ui| app.draw(ui));
    conferir(&img, "seguranca_abertura");
}
