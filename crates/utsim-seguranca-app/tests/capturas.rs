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

/// The "Ferramentas" tab with the demonstration study (only the tools of this app) and its
/// Ishikawa chosen.
#[test]
fn ferramentas_ishikawa_picture() {
    use utsim_estudo::{App as AppEstudo, Ferramenta};
    let dir = tempfile::tempdir().unwrap();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    assert!(
        app.sessao
            .criar(&dir.path().join("unidade"), "Fábrica", "Ana")
    );
    let mut e = utsim_estudo::exemplo::estudo_demonstracao();
    e.app = AppEstudo::Seguranca;
    e.aplicacoes
        .retain(|a| Ferramenta::do_app(AppEstudo::Seguranca).contains(&a.ferramenta));
    app.sessao
        .unidade
        .as_mut()
        .unwrap()
        .gravar(utsim_estudo::ESTUDOS, &e.id, &e)
        .unwrap();
    app.sessao.mudou();
    app.tela = Tela::Ferramentas;
    app.ferramentas.sincronizar(&app.sessao);
    app.ferramentas.abrir_estudo(&mut app.sessao, &e.id);
    let ishikawa = e
        .aplicacoes
        .iter()
        .find(|a| a.ferramenta == Ferramenta::Ishikawa)
        .unwrap()
        .id
        .clone();
    app.ferramentas.escolher_aplicacao(&ishikawa);
    let mut cap = Captura::new(Context::default(), vec2(1280.0, 860.0));
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
        img.salvar_png(&d.join("seguranca_ferramentas_ishikawa.png"))
            .unwrap();
    }
}

/// The date fields of "Incidentes" use the common date field: an impossible date and an end
/// before the start get a red border; `DD/MM/AAAA` is accepted.
#[test]
fn datas_dos_incidentes_picture() {
    use utsim_seguranca_app::incidentes::Aba;
    use utsim_seguranca_core::incidentes::Incidente;
    let dir = tempfile::tempdir().unwrap();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    assert!(
        app.sessao
            .criar(&dir.path().join("unidade"), "Fábrica", "Ana")
    );
    app.tela = Tela::Incidentes;
    let mut cap = Captura::new(Context::default(), vec2(1280.0, 520.0));
    let mut fotos = Vec::new();
    app.incidentes.aba = Aba::Registro;
    app.incidentes.novo = Some(Incidente {
        data: "31/02/2026".into(),
        hora: "25:10".into(),
        ..Incidente::default()
    });
    fotos.push(("seguranca_data_invalida", cap.imagem(3, |ui| app.draw(ui))));
    app.incidentes.novo = None;
    app.incidentes.aba = Aba::Indicadores;
    app.incidentes.de = "01/03/2026".into();
    app.incidentes.ate = "2026-02-10".into();
    fotos.push((
        "seguranca_periodo_invertido",
        cap.imagem(3, |ui| app.draw(ui)),
    ));
    if let Some(d) = std::env::var_os("UTSIM_CAPTURAS") {
        let d = std::path::PathBuf::from(d);
        std::fs::create_dir_all(&d).unwrap();
        for (nome, img) in &fotos {
            img.salvar_png(&d.join(format!("{nome}.png"))).unwrap();
        }
    }
}
