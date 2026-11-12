//! A interface nas cinco línguas: cada tela desenha em cada uma, os textos mudam de fato, toda
//! chave pedida existe nos arquivos e a língua escolhida fica na configuração. Um binário de
//! teste próprio, porque a língua vale para o programa todo.

use eframe::egui;
use utsim_idiomas::Idioma;
use utsim_seguranca_app::texto::{TEXTOS, tx};
use utsim_seguranca_app::{App, Tela};

fn quadros(app: &mut App, ctx: &egui::Context, n: usize) {
    for _ in 0..n {
        let mut saida = ctx.run_ui(egui::RawInput::default(), |ui| app.draw(ui));
        saida.textures_delta.clear();
    }
}

#[test]
fn todas_as_telas_nas_cinco_linguas() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    let mut nomes = Vec::new();
    for i in Idioma::TODOS {
        utsim_idiomas::definir(i);
        let mut app = App::new(Some(dir.path().join("cfg").join("qs.json")));
        quadros(&mut app, &ctx, 2);
        assert!(
            app.sessao
                .criar(&dir.path().join(format!("u_{}", i.codigo())), "F", "Ana")
        );
        for t in [
            Tela::Postos,
            Tela::Inventario,
            Tela::Matriz,
            Tela::Exposicao,
            Tela::Ergonomia,
            Tela::Checklist,
            Tela::Epi,
            Tela::Incidentes,
            Tela::Relatorio,
            Tela::Inicial,
        ] {
            app.tela = t;
            quadros(&mut app, &ctx, 2);
        }
        nomes.push(tx("app-matriz_de_risco"));
    }
    nomes.dedup();
    assert_eq!(nomes.len(), Idioma::TODOS.len(), "{nomes:?}");
    assert!(TEXTOS.faltas().is_empty(), "{:?}", TEXTOS.faltas());

    // A língua guardada volta ao abrir.
    let cfg = dir.path().join("cfg").join("qs.json");
    let mut c = utsim_posto_ui::Config::ler(&cfg);
    c.idioma = Some("es".into());
    c.gravar(&cfg).unwrap();
    utsim_idiomas::definir(Idioma::PtBr);
    let _app = App::new(Some(cfg));
    assert_eq!(utsim_idiomas::atual(), Idioma::Es);
    utsim_idiomas::definir(Idioma::PtBr);
}
