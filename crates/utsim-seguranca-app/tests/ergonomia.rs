//! Tela de ergonomia sem janela: contexto egui sem renderizador.

use std::time::{Duration, Instant};

use eframe::egui;
use utsim_seguranca_app::ergonomia::MARCACOES;
use utsim_seguranca_app::{AVISO_LEGAL, App, Tela};
use utsim_seguranca_core::ergonomia::{AVISO_NR17, Marcacao};

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

#[test]
fn ergonomia_marca_e_nota_sobe() {
    let modelo = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/homem_maquina.ron"
    );
    let dir = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    assert!(
        app.sessao
            .criar(&dir.path().join("unidade"), "Fábrica", "Bia")
    );
    app.tela = Tela::Ergonomia;
    app.ergonomia.modelo = modelo.to_string();
    app.ergonomia.rodar(&mut app.sessao);
    assert!(app.ergonomia.trabalho.ocupado());
    let limite = Instant::now() + Duration::from_secs(180);
    while app.ergonomia.trabalho.ocupado() {
        assert!(Instant::now() < limite, "o estudo não terminou a tempo");
        quadro(&mut app, &ctx);
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        app.ergonomia.trabalho.ok().is_some(),
        "erro: {:?}",
        app.ergonomia.trabalho.erro()
    );
    let texto = quadro(&mut app, &ctx);
    assert!(contem(&texto, AVISO_NR17), "aviso NR-17 ausente");
    assert!(contem(&texto, AVISO_LEGAL), "aviso legal ausente");

    let antes = app.ergonomia.notas(&app.sessao);
    assert!(!antes.is_empty(), "o estudo deveria ter um operador");
    let e = antes[0].elementos[0].clone();
    app.ergonomia.marcar(Marcacao {
        ator: e.ator.clone(),
        elemento: e.elemento.clone(),
        tronco: 3,
        bracos: 3,
        pescoco: 3,
        pernas: 3,
        carga_kg: 25.0,
        pega: 2,
        vibracao: 2,
    });
    let depois = app.ergonomia.notas(&app.sessao);
    assert!(depois[0].nota > antes[0].nota, "a nota deveria subir");
    quadro(&mut app, &ctx);

    // Só grava ao clicar em Gravar.
    let u = app.sessao.unidade.as_ref().unwrap();
    assert_eq!(u.estado().conta(MARCACOES), 0);
    assert_eq!(app.ergonomia.gravar_marcacoes(&mut app.sessao), 1);
    let u = app.sessao.unidade.as_ref().unwrap();
    assert_eq!(u.estado().conta(MARCACOES), 1);

    // Método por CSV inválido: o erro aparece e o método não muda.
    let csv = dir.path().join("m.csv");
    std::fs::write(&csv, "peso_carga;-1\n").unwrap();
    app.ergonomia.arquivo_metodo = csv.display().to_string();
    app.ergonomia.importar_metodo(&mut app.sessao);
    assert!(!app.ergonomia.erro_metodo.is_empty());
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("peso"));
}
