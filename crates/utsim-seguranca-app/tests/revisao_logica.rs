//! Itens de lógica da revisão de telas: a pasta restrita ainda não criada não é erro, a
//! inspeção exige a máquina e as horas-homem sugerem o mês atual.

use eframe::egui;
use utsim_posto::Posto;
use utsim_seguranca_app::incidentes::Aba;
use utsim_seguranca_app::{App, Tela};

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

fn app_salva(dir: &std::path::Path) -> App {
    utsim_idiomas::definir_nesta_thread(Some(utsim_idiomas::Idioma::PtBr));
    let mut app = App::new(Some(dir.join("cfg.json")));
    assert!(app.sessao.criar(&dir.join("unidade"), "Fábrica", "Bia"));
    app.sessao
        .unidade
        .as_mut()
        .unwrap()
        .gravar_posto(&Posto {
            nome: "Prensa".into(),
            ..Posto::default()
        })
        .unwrap();
    app.sessao.mudou();
    app
}

#[test]
fn pasta_restrita_ainda_nao_criada_e_neutra_e_so_depois_de_criada_pode_faltar_acesso() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    let mut app = app_salva(dir.path());
    app.tela = Tela::Incidentes;
    app.incidentes.aba = Aba::Pessoas;
    assert!(!app.sessao.unidade.as_ref().unwrap().restrita_criada());
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("ainda não foi criada"), "{texto}");
    assert!(texto.contains("Criar pasta restrita"));
    assert!(!texto.contains("não abre para você"), "{texto}");
    assert!(app.incidentes.criar_restrita(&mut app.sessao));
    assert!(app.sessao.unidade.as_ref().unwrap().restrita_criada());
    let texto = quadro(&mut app, &ctx);
    assert!(!texto.contains("ainda não foi criada"), "{texto}");
}

#[test]
fn nova_inspecao_exige_a_maquina_com_o_aviso_ao_lado_do_campo() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    let mut app = app_salva(dir.path());
    app.tela = Tela::Checklist;
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("Informe a máquina."), "{texto}");
    app.checklist.posto = app.sessao.unidade.as_ref().unwrap().postos()[0].id.clone();
    assert!(!app.checklist.nova(&mut app.sessao));
    assert!(app.checklist.inspecao.is_none());
    app.checklist.maquina = "Prensa 2".into();
    let texto = quadro(&mut app, &ctx);
    assert!(!texto.contains("Informe a máquina."), "{texto}");
    assert!(app.checklist.nova(&mut app.sessao));
}

#[test]
fn horas_homem_comecam_com_o_mes_atual() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    let mut app = app_salva(dir.path());
    app.tela = Tela::Incidentes;
    app.incidentes.aba = Aba::Horas;
    assert!(app.incidentes.horas.mes.is_empty());
    quadro(&mut app, &ctx);
    let hoje = utsim_posto_ui::util::hoje();
    assert_eq!(app.incidentes.horas.mes, hoje[..7]);
}
