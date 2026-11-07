//! Tela de incidentes sem janela: contexto egui sem renderizador.

use eframe::egui;
use utsim_seguranca_app::incidentes::{Aba, TelaIncidentes};
use utsim_seguranca_app::{App, Tela};
use utsim_seguranca_core::incidentes::{HorasHomem, Incidente, Tipo};

const MODELO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/cruzamento_empilhadeira.ron"
);
const NOME: &str = "Maria Silvana Souza";

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

#[test]
fn incidentes_pessoas_so_pelo_codigo_e_indicadores_sem_nomes() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    let mut app = App::new(Some(dir.path().join("cfg.json")));
    assert!(
        app.sessao
            .criar(&dir.path().join("unidade"), "Fábrica", "Bia")
    );
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
    let posto = app.sessao.unidade.as_ref().unwrap().postos()[0].clone();
    app.tela = Tela::Incidentes;

    // Sem pasta restrita: a sub-tela oferece criar, e só então cadastra.
    app.incidentes.aba = Aba::Pessoas;
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("Criar pasta restrita"));
    app.incidentes.pessoa.nome = NOME.into();
    assert!(app.incidentes.cadastrar(&mut app.sessao).is_none());
    assert!(app.incidentes.criar_restrita(&mut app.sessao));
    assert!(
        app.incidentes
            .restrita(&app.sessao)
            .unwrap()
            .pode_escrever()
    );
    app.incidentes.pessoa.nome = NOME.into();
    app.incidentes.pessoa.setor = "Prensas".into();
    let codigo = app.incidentes.cadastrar(&mut app.sessao).expect("código");
    assert!(codigo.starts_with("P-"));
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains(NOME), "a sub-tela restrita mostra o nome");

    // Registro: nome no lugar do código é recusado, com mensagem.
    app.incidentes.aba = Aba::Registro;
    app.incidentes.novo = Some(Incidente {
        data: "2026-10-15".into(),
        posto: posto.id.clone(),
        tipo: Tipo::ComAfastamento,
        descricao: "Esmagamento de dedo na prensa".into(),
        causas: vec!["piso molhado".into()],
        pessoas: vec![NOME.into()],
        dias_perdidos: 3.0,
        ..Incidente::default()
    });
    assert!(!app.incidentes.gravar(&mut app.sessao));
    assert!(app.sessao.erro);
    assert!(app.sessao.mensagem.contains("não é um código de pessoa"));
    assert!(TelaIncidentes::incidentes(&app.sessao).is_empty());
    // Com o código, grava.
    app.incidentes.novo.as_mut().unwrap().pessoas = vec![codigo.clone()];
    assert!(app.incidentes.gravar(&mut app.sessao));
    app.incidentes.novo = Some(Incidente {
        data: "2026-09-20".into(),
        posto: posto.id.clone(),
        tipo: Tipo::QuaseAcidente,
        descricao: "Empilhadeira quase atinge pedestre".into(),
        causas: vec!["piso molhado".into(), "pouca luz".into()],
        ..Incidente::default()
    });
    assert!(app.incidentes.gravar(&mut app.sessao));
    assert_eq!(TelaIncidentes::incidentes(&app.sessao).len(), 2);

    // No registro, com a pasta restrita aberta, o nome acompanha o código.
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains(&format!("{codigo} ({NOME})")));
    // Os registros da unidade guardam só o código.
    let gravado = TelaIncidentes::incidentes(&app.sessao);
    assert!(
        gravado
            .iter()
            .all(|i| !i.pessoas.contains(&NOME.to_string()))
    );

    // Horas-homem e indicadores do período.
    app.incidentes.horas = HorasHomem {
        mes: "2026-10".into(),
        horas: 20_000.0,
        ..HorasHomem::default()
    };
    assert!(app.incidentes.gravar_horas(&mut app.sessao));
    app.incidentes.horas.mes = "2026-09".into();
    app.incidentes.horas.horas = 18_000.0;
    assert!(app.incidentes.gravar_horas(&mut app.sessao));
    app.incidentes.horas.mes = "mês ruim".into();
    assert!(!app.incidentes.gravar_horas(&mut app.sessao));
    app.incidentes.de = "2026-01-01".into();
    app.incidentes.ate = "2026-12-31".into();
    let ind = app.incidentes.calcular(&app.sessao);
    assert_eq!(ind.horas, 38_000.0);
    assert!((ind.taxa_frequencia - 1e6 / 38_000.0).abs() < 1e-6);
    assert_eq!(ind.pareto_causas.rows[0].name, "piso molhado");
    assert_eq!(ind.mensal.len(), 2);

    app.incidentes.aba = Aba::Indicadores;
    quadro(&mut app, &ctx);
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("Taxa de frequência"));
    assert!(texto.contains("Pareto das causas"));
    assert!(texto.contains("Lesões por mês"));
    assert!(!texto.contains(NOME), "nome na tela de indicadores");
    assert!(!texto.contains("Maria"), "nome na tela de indicadores");

    // Anonimizar apaga o nome da pasta restrita; confirmação em dois passos.
    app.incidentes.aba = Aba::Pessoas;
    quadro(&mut app, &ctx);
    app.incidentes.confirmar = Some(codigo.clone());
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains("Confirmar: apaga o nome para sempre"));
    assert!(app.incidentes.anonimizar(&mut app.sessao, &codigo));
    app.incidentes.aba = Aba::Registro;
    let texto = quadro(&mut app, &ctx);
    assert!(!texto.contains(NOME));
    assert!(texto.contains("[anonimizado]"));

    // Se a pasta restrita não abre, o registro mostra só o código.
    let restrita = app.sessao.unidade.as_ref().unwrap().caminho_restrita();
    std::fs::remove_dir_all(restrita).unwrap();
    app.sessao.mudou();
    let texto = quadro(&mut app, &ctx);
    assert!(texto.contains(&codigo));
    assert!(texto.contains("Pasta restrita indisponível"));
}
