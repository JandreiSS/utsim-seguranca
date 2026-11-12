//! Tela do inventário de riscos por posto, nas cinco etapas de `ETAPAS`.

use crate::texto::{fx, tx, vx};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::SystemTime;

use eframe::egui::{self, Color32, RichText};
use utsim_edit::Document;
use utsim_edit::abas;
use utsim_model::Model;
use utsim_model::safety::HazardDef;
use utsim_posto::{Posto, Unidade};
use utsim_posto_ui::{Sessao, TelaModelo, util};
use utsim_seguranca_core::inventario::{
    ACOES, Acao, ETAPAS, Grupo, Hierarquia, Inventario, ORIGEM_NO, PERIGOS, Perigo, REAVALIACOES,
    Reavaliacao, Situacao, passar_ao_no,
};
use utsim_seguranca_core::matriz::{Degrau, Matriz};

const VERMELHO: Color32 = Color32::from_rgb(230, 90, 80);

/// Operação pedida pela tela, aplicada depois de desenhar.
enum Cmd {
    GravarPerigo(Perigo),
    RemoverPerigo(String),
    GravarAcao(Acao),
    RemoverAcao(String),
    GravarReav(Reavaliacao),
}

#[derive(Default)]
pub struct TelaInventario {
    /// Id do posto escolhido.
    pub posto: String,
    /// Etapa de 0 a 4 (`ETAPAS`).
    pub etapa: usize,
    perigos: BTreeMap<String, (Perigo, Perigo)>,
    acoes: BTreeMap<String, (Acao, Acao)>,
    pub novo_perigo: Option<Perigo>,
    /// Ação nova por id de perigo.
    novas_acoes: BTreeMap<String, Acao>,
    /// Reavaliação nova por id de perigo.
    novas_reav: BTreeMap<String, Reavaliacao>,
    /// Os modelos ligados aos postos, para os perigos que moram no nó.
    modelos: Modelos,
    /// O objeto do modelo que o posto escolhido é (para o aviso dos perigos do nó).
    objeto_do_posto: String,
    /// O arquivo do modelo ligado ao posto escolhido, quando há modelo e objeto.
    modelo_do_posto: Option<PathBuf>,
    /// Pedido ao modelo (abrir o nó, levar um perigo), atendido pelo app com a tela Modelo.
    pub pedido: Option<PedidoNo>,
}

/// O que o inventário pede ao modelo ligado ao posto.
#[derive(Clone, Debug, PartialEq)]
pub enum PedidoNo {
    /// Abrir o nó na tela Modelo, para editar os perigos dele na aba Segurança.
    Abrir { modelo: PathBuf, objeto: String },
    /// Levar um perigo da unidade para o nó (`safety.hazards`).
    Levar {
        modelo: PathBuf,
        objeto: String,
        perigo: Box<Perigo>,
    },
}

/// Grava o perigo `h` como um perigo novo do nó `objeto` (liga o bloco de segurança se
/// preciso) e devolve o id que o nó deu a ele.
pub fn gravar_no_no(doc: &mut Document, objeto: &str, h: &HazardDef) -> Result<String, String> {
    let tem_bloco = doc
        .model()
        .objects
        .iter()
        .find(|o| o.name == objeto)
        .ok_or_else(|| fx("inventario-no_fora_do_modelo", &[("objeto", vx(&objeto))]))?
        .safety
        .is_some();
    if !tem_bloco {
        doc.apply(abas::editar(doc.model(), objeto, "safety", "sim")?)?;
    }
    doc.apply(abas::adicionar(doc.model(), objeto, "safety.hazards")?)?;
    let hs = &doc
        .model()
        .objects
        .iter()
        .find(|o| o.name == objeto)
        .expect("nó")
        .safety;
    let i = hs.as_ref().map_or(0, |s| s.hazards.len()) - 1;
    let id = hs.as_ref().expect("bloco").hazards[i].id.clone();
    let base = format!("safety.hazards[{i}]");
    for (campo, valor) in [
        ("group", format!("{:?}", h.group)),
        ("description", h.description.clone()),
        ("source", h.source.clone()),
        ("circumstance", h.circumstance.clone()),
        ("injuries", h.injuries.clone()),
        ("controls", h.controls.clone()),
        ("probability", h.probability.to_string()),
        ("severity", h.severity.to_string()),
    ] {
        let c = abas::editar(doc.model(), objeto, &format!("{base}.{campo}"), &valor)?;
        doc.apply(c)?;
    }
    Ok(id)
}

/// Atende o pedido do inventário com a tela Modelo do app. Devolve `true` quando o app
/// deve ir para a tela Modelo.
pub fn atender_pedido(p: PedidoNo, s: &mut Sessao, modelo: &mut TelaModelo) -> bool {
    let autor = s.config.autor.clone();
    match p {
        PedidoNo::Abrir {
            modelo: caminho,
            objeto,
        } => {
            if modelo.arquivo() == Some(caminho.as_path()) {
                modelo.selecionar(&objeto);
                true
            } else {
                match modelo.abrir_em_no(&caminho, Some(&objeto), &autor) {
                    Ok(()) => true,
                    Err(e) => {
                        s.falhar(e);
                        false
                    }
                }
            }
        }
        PedidoNo::Levar {
            modelo: caminho,
            objeto,
            perigo,
        } => {
            let h = perigo.para_o_no();
            match modelo.editar_arquivo(&caminho, &autor, |d| gravar_no_no(d, &objeto, &h)) {
                Ok(hid) => {
                    let novo = format!("{ORIGEM_NO}:{}:{hid}", perigo.posto);
                    if let Some(u) = s.unidade.as_mut() {
                        let r = passar_ao_no(u, &perigo.id, &novo);
                        let ok = fx("inventario-perigo_levado_ao_no", &[("objeto", vx(&objeto))]);
                        s.resultado(r, &ok);
                    }
                }
                Err(e) => s.falhar(fx("inventario-nao_levou_ao_no", &[("erro", vx(&e))])),
            }
            false
        }
    }
}

/// Os modelos dos postos, relidos quando o arquivo muda no disco.
#[derive(Default)]
struct Modelos(BTreeMap<PathBuf, (Option<SystemTime>, Option<Model>)>);

impl Modelos {
    fn atualizar(&mut self, u: &Unidade, postos: &[Posto]) {
        for p in postos {
            let Some(m) = &p.modelo else { continue };
            let c = u.caminho_modelo(m);
            let t = std::fs::metadata(&c).and_then(|m| m.modified()).ok();
            if self.0.get(&c).is_none_or(|(x, _)| *x != t) {
                let lido = Model::load(&c).ok();
                self.0.insert(c, (t, lido));
            }
        }
    }

    fn de(&self, u: &Unidade, p: &Posto) -> Option<&Model> {
        let m = p.modelo.as_ref()?;
        self.0.get(&u.caminho_modelo(m))?.1.as_ref()
    }
}

/// Um perigo do nó: só leitura aqui, com o aviso de onde editar.
fn perigo_do_no(ui: &mut egui::Ui, p: &Perigo, objeto: &str) {
    ui.weak(fx("inventario-perigo_no_no", &[("objeto", vx(&objeto))]));
    egui::Grid::new(("perigo_no", &p.id))
        .num_columns(2)
        .show(ui, |ui| {
            for (r, v) in [
                (tx("inventario-grupo"), p.grupo.nome().to_string()),
                (tx("inventario-fonte"), p.fonte.clone()),
                (tx("inventario-circunstancia"), p.circunstancia.clone()),
                (tx("inventario-possiveis_lesoes"), p.lesoes.clone()),
                (tx("inventario-controles_existentes"), p.controles.clone()),
            ] {
                if !v.is_empty() {
                    ui.label(r);
                    ui.label(v);
                    ui.end_row();
                }
            }
        });
}

fn combo_degrau(
    ui: &mut egui::Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    valor: &mut usize,
    d: &[Degrau],
) {
    let texto = d.get(*valor).map_or_else(
        || "?".to_string(),
        |x| format!("{} ({})", x.nome, *valor + 1),
    );
    egui::ComboBox::from_id_salt(id)
        .selected_text(texto)
        .show_ui(ui, |ui| {
            for (i, x) in d.iter().enumerate() {
                ui.selectable_value(valor, i, format!("{} ({})", x.nome, i + 1))
                    .on_hover_text(&x.criterio);
            }
        });
}

/// Nome do nível com a cor dele.
fn selo(ui: &mut egui::Ui, m: &Matriz, p: usize, s: usize) {
    let n = m.nivel_de(p, s);
    let cor = util::cor(&n.cor);
    ui.label(
        RichText::new(format!(" {} ", n.nome))
            .background_color(cor)
            .color(util::texto_sobre(cor)),
    )
    .on_hover_text(&n.decisao);
}

fn campo(ui: &mut egui::Ui, rotulo: &str, texto: &mut String) {
    ui.horizontal(|ui| {
        ui.label(rotulo);
        ui.add(egui::TextEdit::singleline(texto).desired_width(f32::INFINITY));
    });
}

fn combo_grupo(ui: &mut egui::Ui, id: impl std::hash::Hash + std::fmt::Debug, g: &mut Grupo) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(g.nome())
        .show_ui(ui, |ui| {
            for x in Grupo::TODOS {
                ui.selectable_value(g, x, x.nome());
            }
        });
}

fn editor_acao(ui: &mut egui::Ui, a: &mut Acao, atrasada: bool) {
    ui.horizontal(|ui| {
        ui.label(tx("inventario-hierarquia"));
        egui::ComboBox::from_id_salt(("hier", &a.id, &a.perigo))
            .selected_text(a.hierarquia.nome())
            .show_ui(ui, |ui| {
                for h in Hierarquia::TODAS {
                    ui.selectable_value(&mut a.hierarquia, h, h.nome());
                }
            });
        ui.label(tx("inventario-situacao"));
        egui::ComboBox::from_id_salt(("sit", &a.id, &a.perigo))
            .selected_text(a.situacao.nome())
            .show_ui(ui, |ui| {
                for x in Situacao::TODAS {
                    ui.selectable_value(&mut a.situacao, x, x.nome());
                }
            });
        ui.label(tx("inventario-eficaz"));
        let t = match a.eficaz {
            None => tx("inventario-nao_verificada"),
            Some(true) => tx("inventario-sim"),
            Some(false) => tx("inventario-nao"),
        };
        egui::ComboBox::from_id_salt(("efi", &a.id, &a.perigo))
            .selected_text(t)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut a.eficaz, None, tx("inventario-nao_verificada"));
                ui.selectable_value(&mut a.eficaz, Some(true), tx("inventario-sim"));
                ui.selectable_value(&mut a.eficaz, Some(false), tx("inventario-nao"));
            });
    });
    campo(ui, tx("inventario-descricao"), &mut a.descricao);
    ui.horizontal(|ui| {
        ui.label(tx("inventario-responsavel"));
        ui.add(egui::TextEdit::singleline(&mut a.responsavel).desired_width(200.0));
        ui.label(tx("inventario-prazo_aaaa_mm_dd"));
        ui.add(egui::TextEdit::singleline(&mut a.prazo).desired_width(100.0));
        if atrasada {
            ui.colored_label(VERMELHO, tx("inventario-atrasada"));
        }
    });
}

impl TelaInventario {
    pub fn ui(&mut self, ui: &mut egui::Ui, s: &mut Sessao) {
        let Some(u) = s.unidade.as_ref() else {
            ui.label(tx("inventario-abra_uma_unidade_na_tela"));
            return;
        };
        let postos = u.postos();
        self.modelos.atualizar(u, &postos);
        let modelos = &self.modelos;
        let inv = Inventario::ler(u).com_nos(|p| modelos.de(u, p));
        ui.heading(tx("inventario-inventario_de_riscos_por_posto"));
        if inv.postos.is_empty() {
            ui.label(tx("inventario-a_unidade_ainda_nao_tem"));
            return;
        }
        if !inv.postos.iter().any(|p| p.id == self.posto) {
            self.posto = inv.postos[0].id.clone();
        }
        self.etapa = self.etapa.min(ETAPAS.len() - 1);
        let hoje = util::hoje();

        ui.horizontal_wrapped(|ui| {
            for (i, e) in ETAPAS.iter().enumerate() {
                ui.selectable_value(&mut self.etapa, i, *e);
            }
        });
        let nome_posto = inv
            .postos
            .iter()
            .find(|p| p.id == self.posto)
            .map(|p| p.nome.clone())
            .unwrap_or_default();
        let posto_escolhido = inv.postos.iter().find(|p| p.id == self.posto);
        self.objeto_do_posto = posto_escolhido
            .and_then(|p| p.objeto.clone())
            .unwrap_or_default();
        self.modelo_do_posto = posto_escolhido
            .filter(|p| p.objeto.as_deref().is_some_and(|o| !o.is_empty()))
            .and_then(|p| p.modelo.as_ref())
            .map(|m| u.caminho_modelo(m));
        ui.horizontal(|ui| {
            ui.label(tx("inventario-posto"));
            egui::ComboBox::from_id_salt("posto_inventario")
                .selected_text(&nome_posto)
                .show_ui(ui, |ui| {
                    for p in &inv.postos {
                        ui.selectable_value(&mut self.posto, p.id.clone(), &p.nome);
                    }
                });
        });

        let meus: Vec<Perigo> = inv
            .perigos_do_posto(&self.posto)
            .into_iter()
            .cloned()
            .collect();
        let ids: Vec<&str> = meus.iter().map(|p| p.id.as_str()).collect();
        let sem_acao: Vec<&Perigo> = inv
            .sem_acao()
            .into_iter()
            .filter(|p| ids.contains(&p.id.as_str()))
            .collect();
        let atrasadas: Vec<&Acao> = inv
            .atrasadas(&hoje)
            .into_iter()
            .filter(|a| ids.contains(&a.perigo.as_str()))
            .collect();
        if !sem_acao.is_empty() {
            ui.colored_label(
                VERMELHO,
                fx(
                    "inventario-perigos_acima_do_nivel_mais",
                    &[
                        ("sem_acao", vx(&sem_acao.len())),
                        (
                            "descricao",
                            vx(&sem_acao
                                .iter()
                                .map(|p| p.descricao.as_str())
                                .collect::<Vec<_>>()
                                .join("; ")),
                        ),
                    ],
                ),
            );
        }
        if !atrasadas.is_empty() {
            ui.colored_label(
                VERMELHO,
                fx(
                    "inventario-acoes_atrasadas_em",
                    &[("atrasadas", vx(&atrasadas.len())), ("hoje", vx(&hoje))],
                ),
            );
        }
        ui.separator();

        let mut cmds: Vec<Cmd> = Vec::new();
        match self.etapa {
            0 => self.etapa_identificar(ui, &inv),
            1 => self.etapa_perigos(ui, &meus, &mut cmds),
            2 => self.etapa_avaliar(ui, &inv, &meus, &mut cmds),
            3 => self.etapa_acoes(ui, &inv, &meus, &hoje, &mut cmds),
            _ => self.etapa_acompanhar(ui, &inv, &meus, &hoje, &mut cmds),
        }
        for c in cmds {
            self.aplicar(s, c);
        }
    }

    fn etapa_identificar(&self, ui: &mut egui::Ui, inv: &Inventario) {
        let Some(p) = inv.postos.iter().find(|p| p.id == self.posto) else {
            return;
        };
        ui.strong(&p.nome);
        egui::Grid::new("identificar")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label(tx("inventario-area"));
                ui.label(&p.area);
                ui.end_row();
                ui.label(tx("inventario-descricao"));
                ui.label(&p.descricao);
                ui.end_row();
                ui.label(tx("inventario-pessoas_expostas_por_turno"));
                ui.label(p.pessoas_expostas.to_string());
                ui.end_row();
                ui.label(tx("inventario-epi_exigidos"));
                ui.label(p.epi.join(", "));
                ui.end_row();
                ui.label(tx("inventario-atividades"));
                ui.label(p.atividades.join("; "));
                ui.end_row();
            });
        ui.weak(tx("inventario-para_mudar_estes_dados_use"));
    }

    fn etapa_perigos(&mut self, ui: &mut egui::Ui, meus: &[Perigo], cmds: &mut Vec<Cmd>) {
        let objeto = self.objeto_do_posto.clone();
        for p in meus {
            if p.eh_do_no() {
                egui::CollapsingHeader::new(format!(
                    "[{}] {} ({})",
                    p.grupo.nome(),
                    p.descricao,
                    tx("inventario-no_do_modelo")
                ))
                .id_salt(("perigo1", &p.id))
                .show(ui, |ui| {
                    perigo_do_no(ui, p, &objeto);
                    if let Some(m) = &self.modelo_do_posto
                        && ui.button(tx("inventario-editar_no_no")).clicked()
                    {
                        self.pedido = Some(PedidoNo::Abrir {
                            modelo: m.clone(),
                            objeto: objeto.clone(),
                        });
                    }
                });
                continue;
            }
            let d = rascunho(&mut self.perigos, &p.id, p);
            let sujo = d != p;
            let titulo = format!(
                "[{}] {}{}",
                d.grupo.nome(),
                d.descricao,
                if sujo { " *" } else { "" }
            );
            egui::CollapsingHeader::new(titulo)
                .id_salt(("perigo1", &p.id))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(tx("inventario-grupo"));
                        combo_grupo(ui, ("grupo", &p.id), &mut d.grupo);
                        if !d.origem.is_empty() {
                            ui.weak(fx("inventario-origem", &[("origem", vx(&d.origem))]));
                        }
                    });
                    campo(ui, tx("inventario-descricao"), &mut d.descricao);
                    campo(ui, tx("inventario-fonte"), &mut d.fonte);
                    campo(ui, tx("inventario-circunstancia"), &mut d.circunstancia);
                    campo(ui, tx("inventario-possiveis_lesoes"), &mut d.lesoes);
                    botoes_perigo(ui, d, p, cmds);
                    if let Some(m) = &self.modelo_do_posto
                        && ui
                            .add_enabled(!sujo, egui::Button::new(tx("inventario-levar_ao_no")))
                            .on_hover_text(fx(
                                "inventario-levar_ao_no_dica",
                                &[("objeto", vx(&objeto))],
                            ))
                            .clicked()
                    {
                        self.pedido = Some(PedidoNo::Levar {
                            modelo: m.clone(),
                            objeto: objeto.clone(),
                            perigo: Box::new(p.clone()),
                        });
                    }
                });
        }
        ui.separator();
        match &mut self.novo_perigo {
            None => {
                if ui.button(tx("inventario-novo_perigo")).clicked() {
                    self.novo_perigo = Some(Perigo {
                        posto: self.posto.clone(),
                        origem: "manual".into(),
                        ..Perigo::default()
                    });
                }
            }
            Some(n) => {
                ui.strong(tx("inventario-novo_perigo"));
                ui.horizontal(|ui| {
                    ui.label(tx("inventario-grupo"));
                    combo_grupo(ui, "grupo_novo", &mut n.grupo);
                });
                campo(ui, tx("inventario-descricao"), &mut n.descricao);
                campo(ui, tx("inventario-fonte"), &mut n.fonte);
                campo(ui, tx("inventario-circunstancia"), &mut n.circunstancia);
                campo(ui, tx("inventario-possiveis_lesoes"), &mut n.lesoes);
                let mut fechar = false;
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            !n.descricao.trim().is_empty(),
                            egui::Button::new(tx("inventario-gravar_perigo")),
                        )
                        .clicked()
                    {
                        cmds.push(Cmd::GravarPerigo(n.clone()));
                        fechar = true;
                    }
                    if ui.button(tx("inventario-cancelar")).clicked() {
                        fechar = true;
                    }
                });
                if fechar {
                    self.novo_perigo = None;
                }
            }
        }
    }

    fn etapa_avaliar(
        &mut self,
        ui: &mut egui::Ui,
        inv: &Inventario,
        meus: &[Perigo],
        cmds: &mut Vec<Cmd>,
    ) {
        let m = &inv.matriz;
        if meus.is_empty() {
            ui.label(tx("inventario-sem_perigos_neste_posto_levante"));
        }
        let objeto = self.objeto_do_posto.clone();
        for p in meus {
            if p.eh_do_no() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(&p.descricao);
                        selo(ui, m, p.probabilidade, p.severidade);
                    });
                    perigo_do_no(ui, p, &objeto);
                });
                continue;
            }
            let d = rascunho(&mut self.perigos, &p.id, p);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.strong(&d.descricao);
                    selo(ui, m, d.probabilidade, d.severidade);
                });
                campo(ui, tx("inventario-controles_existentes"), &mut d.controles);
                ui.horizontal(|ui| {
                    ui.label(tx("inventario-probabilidade"));
                    combo_degrau(ui, ("prob", &p.id), &mut d.probabilidade, &m.probabilidade);
                    ui.label(tx("inventario-severidade"));
                    combo_degrau(ui, ("sev", &p.id), &mut d.severidade, &m.severidade);
                });
                botoes_perigo(ui, d, p, cmds);
            });
        }
    }

    fn etapa_acoes(
        &mut self,
        ui: &mut egui::Ui,
        inv: &Inventario,
        meus: &[Perigo],
        hoje: &str,
        cmds: &mut Vec<Cmd>,
    ) {
        let m = &inv.matriz;
        let sem: Vec<&str> = inv.sem_acao().iter().map(|p| p.id.as_str()).collect();
        for p in meus {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.strong(&p.descricao);
                    selo(ui, m, p.probabilidade, p.severidade);
                    if sem.contains(&p.id.as_str()) {
                        ui.colored_label(VERMELHO, tx("inventario-sem_acao"));
                    }
                });
                for a in inv.acoes_do_perigo(&p.id) {
                    let atrasada = inv.atrasadas(hoje).iter().any(|x| x.id == a.id);
                    let d = rascunho(&mut self.acoes, &a.id, a);
                    let sujo = d != a;
                    ui.separator();
                    editor_acao(ui, d, atrasada);
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(sujo, egui::Button::new(tx("inventario-gravar_acao")))
                            .clicked()
                        {
                            cmds.push(Cmd::GravarAcao(d.clone()));
                        }
                        if sujo && ui.button(tx("inventario-descartar_mudancas")).clicked() {
                            *d = a.clone();
                        }
                        if ui.button(tx("inventario-remover_acao")).clicked() {
                            cmds.push(Cmd::RemoverAcao(a.id.clone()));
                        }
                    });
                }
                match self.novas_acoes.get_mut(&p.id) {
                    None => {
                        if ui.button(tx("inventario-nova_acao")).clicked() {
                            self.novas_acoes.insert(
                                p.id.clone(),
                                Acao {
                                    perigo: p.id.clone(),
                                    ..Acao::default()
                                },
                            );
                        }
                    }
                    Some(n) => {
                        ui.separator();
                        ui.strong(tx("inventario-nova_acao"));
                        editor_acao(ui, n, false);
                        let mut fechar = false;
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(
                                    !n.descricao.trim().is_empty(),
                                    egui::Button::new(tx("inventario-gravar_nova_acao")),
                                )
                                .clicked()
                            {
                                cmds.push(Cmd::GravarAcao(n.clone()));
                                fechar = true;
                            }
                            if ui.button(tx("inventario-cancelar")).clicked() {
                                fechar = true;
                            }
                        });
                        if fechar {
                            self.novas_acoes.remove(&p.id);
                        }
                    }
                }
            });
        }
    }

    fn etapa_acompanhar(
        &mut self,
        ui: &mut egui::Ui,
        inv: &Inventario,
        meus: &[Perigo],
        hoje: &str,
        cmds: &mut Vec<Cmd>,
    ) {
        let m = &inv.matriz;
        for p in meus {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.strong(&p.descricao);
                    ui.label(tx("inventario-avaliacao_inicial"));
                    selo(ui, m, p.probabilidade, p.severidade);
                    ui.label(tx("inventario-nivel_atual"));
                    let atual = inv.nivel_atual(p);
                    let n = &m.niveis[atual.min(m.niveis.len() - 1)];
                    let cor = util::cor(&n.cor);
                    ui.label(
                        RichText::new(format!(" {} ", n.nome))
                            .background_color(cor)
                            .color(util::texto_sobre(cor)),
                    );
                });
                let mut hist: Vec<&Reavaliacao> = inv
                    .reavaliacoes
                    .iter()
                    .filter(|r| r.perigo == p.id)
                    .collect();
                hist.sort_by(|a, b| a.data.cmp(&b.data));
                for r in hist {
                    ui.horizontal(|ui| {
                        ui.label(&r.data);
                        selo(ui, m, r.probabilidade, r.severidade);
                        ui.label(&r.nota);
                    });
                }
                let n = self
                    .novas_reav
                    .entry(p.id.clone())
                    .or_insert_with(|| Reavaliacao {
                        perigo: p.id.clone(),
                        id: String::new(),
                        data: hoje.to_string(),
                        probabilidade: p.probabilidade,
                        severidade: p.severidade,
                        nota: String::new(),
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(tx("inventario-data"));
                    ui.add(egui::TextEdit::singleline(&mut n.data).desired_width(90.0));
                    ui.label(tx("inventario-p"));
                    combo_degrau(ui, ("rp", &p.id), &mut n.probabilidade, &m.probabilidade);
                    ui.label(tx("inventario-s"));
                    combo_degrau(ui, ("rs", &p.id), &mut n.severidade, &m.severidade);
                });
                campo(ui, tx("inventario-nota"), &mut n.nota);
                if ui.button(tx("inventario-registrar_reavaliacao")).clicked() {
                    cmds.push(Cmd::GravarReav(n.clone()));
                }
            });
        }
    }

    fn aplicar(&mut self, s: &mut Sessao, c: Cmd) {
        let Some(u) = s.unidade.as_mut() else {
            return;
        };
        match c {
            Cmd::GravarPerigo(p) => {
                let r = u.gravar(PERIGOS, &p.id, &p);
                if let Some(id) = s.resultado(r, tx("inventario-perigo_gravado")) {
                    self.perigos.remove(&id);
                }
            }
            Cmd::RemoverPerigo(id) => {
                let r = u.remover(PERIGOS, &id);
                if s.resultado(r, tx("inventario-perigo_removido")).is_some() {
                    self.perigos.remove(&id);
                }
            }
            Cmd::GravarAcao(a) => {
                let r = u.gravar(ACOES, &a.id, &a);
                if let Some(id) = s.resultado(r, tx("inventario-acao_gravada")) {
                    self.acoes.remove(&id);
                }
            }
            Cmd::RemoverAcao(id) => {
                let r = u.remover(ACOES, &id);
                if s.resultado(r, tx("inventario-acao_removida")).is_some() {
                    self.acoes.remove(&id);
                }
            }
            Cmd::GravarReav(r0) => {
                let r = u.gravar(REAVALIACOES, "", &r0);
                if s.resultado(r, tx("inventario-reavaliacao_registrada"))
                    .is_some()
                {
                    self.novas_reav.remove(&r0.perigo);
                }
            }
        }
    }
}

/// Rascunho de um registro: guarda a base lida e a cópia editada. Se a base mudou na pasta
/// e a pessoa não tinha editado nada, a cópia acompanha; se tinha editado, as edições ficam.
fn rascunho<'a, T: Clone + PartialEq>(
    m: &'a mut BTreeMap<String, (T, T)>,
    id: &str,
    atual: &T,
) -> &'a mut T {
    let e = m
        .entry(id.to_string())
        .or_insert_with(|| (atual.clone(), atual.clone()));
    if e.0 != *atual {
        if e.0 == e.1 {
            e.1 = atual.clone();
        }
        e.0 = atual.clone();
    }
    &mut e.1
}

fn botoes_perigo(ui: &mut egui::Ui, d: &mut Perigo, original: &Perigo, cmds: &mut Vec<Cmd>) {
    let sujo = d != original;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(sujo, egui::Button::new(tx("inventario-gravar_perigo")))
            .clicked()
        {
            cmds.push(Cmd::GravarPerigo(d.clone()));
        }
        if sujo && ui.button(tx("inventario-descartar_mudancas")).clicked() {
            *d = original.clone();
        }
        if ui.button(tx("inventario-remover_perigo")).clicked() {
            cmds.push(Cmd::RemoverPerigo(original.id.clone()));
        }
    });
}
